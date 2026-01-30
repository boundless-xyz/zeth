// Copyright 2026 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! End-to-end test using Anvil local testnet.
//!
//! This test:
//! 1. Spins up Anvil with chain ID 1337 (Dev) and Prague hardfork
//! 2. Deploys a simple counter contract
//! 3. Executes transactions interacting with the contract
//! 4. Starts the zeth-rpc-proxy to provide `debug_executionWitness`
//! 5. Uses BlockProcessor to create input and validate the block

use alloy::{
    eips::BlockId,
    network::EthereumWallet,
    node_bindings::Anvil,
    primitives::{B256, U256},
    providers::{Provider, ProviderBuilder, ext::AnvilApi},
    signers::local::PrivateKeySigner,
    sol,
};
use risc0_zkvm::ProverOpts;
use std::{
    process::{Child, Command, Stdio},
    time::Duration,
};
use test_log::test;
use zeth_host::BlockProcessor;

// Simple counter contract using alloy's sol! macro
sol! {
    // docker run -i ethereum/solc:0.8.30 - --optimize --evm-version prague --via-ir --bin
    #[sol(rpc, bytecode="6080806040523460145760fa90816100198239f35b5f80fdfe60808060405260043610156011575f80fd5b5f3560e01c90816306661abd1460ad575080632baeceb714608f578063d09de08a14605d5763d14e62b8146043575f80fd5b3460595760203660031901126059576004355f55005b5f80fd5b346059575f3660031901126059575f5460018101809111607b575f55005b634e487b7160e01b5f52601160045260245ffd5b346059575f3660031901126059575f545f198101908111607b575f55005b346059575f3660031901126059576020905f548152f3fea26469706673582212209992735b7366762054a5f1d624675de9e429b8861009ad165dfb588901a77d6564736f6c634300081e0033")]
    contract Counter {
        uint256 public count;

        function increment() public {
            count += 1;
        }

        function decrement() public {
            count -= 1;
        }

        function setCount(uint256 _count) public {
            count = _count;
        }
    }
}

/// Helper struct to manage the rpc-proxy subprocess
struct RpcProxyHandle {
    child: Child,
}

impl RpcProxyHandle {
    /// Spawns the rpc-proxy pointing to the given upstream URL
    fn spawn(upstream_url: &str, bind_port: u16) -> anyhow::Result<Self> {
        let bind_address = format!("127.0.0.1:{}", bind_port);
        let child = Command::new("cargo")
            .args([
                "run",
                "--package",
                "zeth-rpc-proxy",
                "--",
                "--rpc-url",
                upstream_url,
                "--bind-address",
                &bind_address,
                // Use fewer nibbles for faster startup in tests
                "--preimage-cache-nibbles",
                "0",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self { child })
    }
}

impl Drop for RpcProxyHandle {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// Waits for the RPC proxy to become available
async fn wait_for_rpc(url: &str, timeout: Duration) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        let result = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(r#"{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}"#)
            .send()
            .await;

        if result.is_ok() {
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    anyhow::bail!("RPC proxy did not become available within {:?}", timeout)
}

#[test(tokio::test)]
#[ignore = "requires anvil binary and takes time to compile rpc-proxy"]
async fn prove_anvil_block_with_contract_interaction() -> anyhow::Result<()> {
    // 1. Start Anvil with chain ID 1337 (Dev) and Prague hardfork
    let anvil = Anvil::new()
        .chain_id(1337)
        .arg("--hardfork")
        .arg("prague")
        .block_time(1) // Mine a block every second
        .try_spawn()?;

    let anvil_url = anvil.endpoint();
    tracing::info!("Anvil started at {}", anvil_url);

    // Create a wallet from the first Anvil account
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer.clone());

    // Create provider connected to Anvil
    let anvil_provider =
        ProviderBuilder::new().wallet(wallet.clone()).connect_http(anvil_url.parse()?);

    // Verify we're connected to Anvil with Dev chain ID (1337)
    let chain_id = anvil_provider.get_chain_id().await?;
    assert_eq!(chain_id, 1337, "Expected Dev chain ID (1337)");

    // 2. Deploy the Counter contract
    tracing::info!("Deploying Counter contract...");
    let counter = Counter::deploy(&anvil_provider).await?;
    let contract_address = *counter.address();
    tracing::info!("Counter deployed at {}", contract_address);

    // 3. Interact with the contract - increment a few times
    let counter = Counter::new(contract_address, &anvil_provider);

    tracing::info!("Incrementing counter...");
    let _ = counter.increment().send().await?.get_receipt().await?;
    let _ = counter.increment().send().await?.get_receipt().await?;
    let _ = counter.setCount(U256::from(42)).send().await?.get_receipt().await?;

    // Verify the count
    let count = counter.count().call().await?;
    assert_eq!(count, U256::from(42), "Counter should be 42");
    tracing::info!("Counter value: {}", count);

    // Mine a final block to ensure all transactions are included
    anvil_provider.anvil_mine(Some(1), None).await?;

    // Get the latest block number
    let latest_block = anvil_provider.get_block_number().await?;
    tracing::info!("Latest block: {}", latest_block);

    // 4. Start the RPC proxy
    // Use a random port to avoid conflicts
    let proxy_port = 18545 + (std::process::id() % 1000) as u16;
    tracing::info!("Starting RPC proxy on port {}...", proxy_port);
    let _proxy = RpcProxyHandle::spawn(&anvil_url, proxy_port)?;

    // Wait for proxy to be ready
    wait_for_rpc(&format!("http://127.0.0.1:{}", proxy_port), Duration::from_secs(120)).await?;
    tracing::info!("RPC proxy is ready");

    // 5. Create BlockProcessor connected to the proxy
    let proxy_provider =
        ProviderBuilder::new().connect_http(format!("http://127.0.0.1:{}", proxy_port).parse()?);

    let processor = BlockProcessor::new(proxy_provider).await?;
    assert_eq!(processor.chain(), reth_chainspec::NamedChain::Dev, "Should detect Dev chain");

    // 6. Create input for a block with transactions (not genesis)
    // Use a block that has our contract interactions
    let target_block = latest_block.saturating_sub(1).max(1);
    tracing::info!("Creating input for block {}...", target_block);
    let (input, block_hash) = processor.create_input(BlockId::number(target_block)).await?;

    tracing::info!(
        "Input created for block {} (hash: {}), {} transactions",
        target_block,
        block_hash,
        input.block.body.transactions().count()
    );

    // 7. Validate on host
    tracing::info!("Validating block on host...");
    let validated_hash = processor.validate(input.clone())?;
    assert_eq!(validated_hash, block_hash, "Validation hash mismatch");
    tracing::info!("Block validated successfully");

    // 8. Prove in dev mode
    tracing::info!("Proving block in dev mode...");
    let (receipt, _image_id) =
        processor.prove_with_opts(input, None, ProverOpts::default().with_dev_mode(true)).await?;

    let proven_hash = B256::try_from(receipt.journal.as_ref())?;
    assert_eq!(proven_hash, block_hash, "Proven hash mismatch");
    tracing::info!("Block proven successfully!");

    Ok(())
}

#[test(tokio::test)]
async fn validate_anvil_genesis_block() -> anyhow::Result<()> {
    // This is a simpler test that just validates we can connect to Anvil
    // and the chain spec is correct, without needing the rpc-proxy

    let anvil = Anvil::new().chain_id(1337).arg("--hardfork").arg("prague").try_spawn()?;
    let anvil_url = anvil.endpoint();

    let provider = ProviderBuilder::new().connect_http(anvil_url.parse()?);

    let chain_id = provider.get_chain_id().await?;
    assert_eq!(chain_id, 1337, "Expected Dev chain ID (1337)");

    // Verify the genesis block exists
    let genesis = provider.get_block(BlockId::number(0)).await?.expect("Genesis should exist");
    tracing::info!("Genesis block hash: {}", genesis.header.hash);

    Ok(())
}
