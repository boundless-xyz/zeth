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

use alloy::{
    eips::BlockId,
    primitives::{B256, BlockHash},
    providers::{Provider, ext::DebugApi},
    rpc::types::debug::ExecutionWitness,
};
use anyhow::{Context, Result, bail};
use guests::{DEV_ELF, HOODI_ELF, MAINNET_ELF, SEPOLIA_ELF};
use reth_chainspec::{ChainSpec, EthChainSpec, NamedChain};
use reth_ethereum_primitives::{Block, TransactionSigned};
use reth_stateless::UncompressedPublicKey;
use risc0_zkvm::{
    Digest, ExecutorEnvBuilder, ProverOpts, Receipt, compute_image_id, default_prover,
};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
    sync::Arc,
};
use zeth_core::Input;

/// Processes Ethereum blocks, including creating inputs, validating, and proving.
pub struct BlockProcessor<P> {
    /// The provider for fetching data from the Ethereum network.
    provider: Arc<P>,
    /// The chain specification.
    chain_spec: Arc<ChainSpec>,
}

impl<P> Clone for BlockProcessor<P> {
    fn clone(&self) -> Self {
        Self { provider: Arc::clone(&self.provider), chain_spec: Arc::clone(&self.chain_spec) }
    }
}

impl<P: Provider + DebugApi> BlockProcessor<P> {
    /// Creates a new BlockProcessor.
    ///
    /// This will make a network call to determine the chain ID and select the appropriate chain
    /// specification.
    pub async fn new(provider: P) -> Result<Self> {
        let chain_id = provider.get_chain_id().await.context("eth_chainId failed")?;
        let chain = chain_id.try_into().context("invalid chain ID")?;
        let chain_spec = match chain {
            NamedChain::Mainnet => reth_chainspec::MAINNET.clone(),
            NamedChain::Sepolia => reth_chainspec::SEPOLIA.clone(),
            NamedChain::Hoodi => reth_chainspec::HOODI.clone(),
            NamedChain::AnvilHardhat => reth_chainspec::DEV.clone(),
            chain => bail!("unsupported chain: {chain}"),
        };

        Ok(Self { provider: provider.into(), chain_spec })
    }

    /// Returns the underlying provider.
    pub fn provider(&self) -> &P {
        &self.provider
    }

    /// Returns the named chain identifier.
    pub fn chain(&self) -> NamedChain {
        // This unwrap is safe because the constructor ensures a valid named chain.
        self.chain_spec.chain().named().unwrap()
    }

    /// Returns the guest program ELF and its corresponding image ID for the current chain.
    pub fn elf(&self) -> Result<(&'static [u8], Digest)> {
        let elf = match self.chain() {
            NamedChain::Mainnet => MAINNET_ELF,
            NamedChain::Sepolia => SEPOLIA_ELF,
            NamedChain::Hoodi => HOODI_ELF,
            NamedChain::AnvilHardhat => DEV_ELF,
            chain => bail!("unsupported chain for proving: {chain}"),
        };
        let image_id = compute_image_id(elf).context("failed to compute image id")?;

        Ok((elf, image_id))
    }

    /// Fetches the necessary data from the RPC endpoint to create the input.
    pub async fn create_input(&self, block: impl Into<BlockId>) -> Result<(Input, B256)> {
        let block_id = block.into();
        let rpc_block = self
            .provider
            .get_block(block_id)
            .full()
            .await?
            .with_context(|| format!("block {block_id} not found"))?;
        let witness = self.provider.debug_execution_witness(rpc_block.number().into()).await?;
        let block_hash = rpc_block.header.hash_slow();
        let block = reth_ethereum_primitives::Block::from(rpc_block);
        let signers = recover_signers(block.body.transactions())?;

        Ok((
            Input {
                block,
                signers,
                witness: ExecutionWitness {
                    state: witness.state,
                    codes: witness.codes,
                    keys: vec![], // keys are not used
                    headers: witness.headers,
                },
            },
            block_hash,
        ))
    }

    /// Validates the block execution on the host machine.
    pub fn validate(&self, input: Input) -> Result<B256> {
        let config = zeth_core::EthEvmConfig::new(self.chain_spec.clone());
        let hash = zeth_core::validate_block(input, config)?;

        Ok(hash)
    }

    /// Generates a RISC Zero proof of block execution.
    ///
    /// This method is computationally intensive and is run on a blocking thread.
    pub async fn prove(&self, input: Input, po2: Option<u32>) -> Result<(Receipt, Digest)> {
        self.prove_with_opts(input, po2, ProverOpts::default()).await
    }

    /// Generates a RISC Zero proof of block execution using the specified [ProverOpts].
    ///
    /// This method is computationally intensive and is run on a blocking thread.
    pub async fn prove_with_opts(
        &self,
        input: Input,
        po2: Option<u32>,
        opts: ProverOpts,
    ) -> Result<(Receipt, Digest)> {
        let (elf, image_id) = self.elf()?;

        // prove in a blocking thread using the default prover
        let info = tokio::task::spawn_blocking(move || {
            let mut env_builder = ExecutorEnvBuilder::default();
            if let Some(po2) = po2 {
                env_builder.segment_limit_po2(po2);
            }
            let env = env_builder.write(&input)?.build()?;
            default_prover().prove_with_opts(env, elf, &opts)
        })
        .await
        .context("prover task panicked")??;

        Ok((info.receipt, image_id))
    }

    /// Gets the input from the filesystem cache, or returns None.
    /// Handles migration from legacy formats automatically.
    pub fn get_input_cached(&self, block_hash: B256, cache_dir: &Path) -> Result<Option<Input>> {
        // 1. Try current version
        if let Some(input) = Current.load_from_dir(block_hash, cache_dir)? {
            return Ok(Some(input));
        }
        // 2. Try legacy versions
        for format in LEGACY_FORMATS {
            if let Some(input) = format.load_from_dir(block_hash, cache_dir)? {
                // Migration: Save as current version
                if let Err(err) = self.save_to_cache(&input, cache_dir) {
                    tracing::warn!("Failed to save migrated cache: {}", err);
                }

                return Ok(Some(input));
            }
        }
        Ok(None)
    }

    /// Fetches input, using the filesystem cache if available.
    /// Handles migration from legacy formats automatically.
    pub async fn get_input_with_cache(&self, block_id: BlockId, cache_dir: &Path) -> Result<Input> {
        let block_hash = match block_id {
            BlockId::Hash(hash) => hash.block_hash,
            _ => {
                // First, get the block header to determine the canonical hash for caching.
                let header = self
                    .provider()
                    .get_block(block_id)
                    .await?
                    .with_context(|| format!("block {block_id} not found"))?
                    .header;

                header.hash
            }
        };

        if let Some(input) = self.get_input_cached(block_hash, cache_dir)? {
            return Ok(input);
        }

        tracing::info!("Cache miss for block {block_hash}. Fetching from RPC.");
        let (input, _) = self.create_input(block_hash).await?;
        if let Err(e) = self.save_to_cache(&input, cache_dir) {
            tracing::warn!("Failed to save cache: {}", e);
        }

        Ok(input)
    }

    /// Performs an atomic write of the input.
    fn save_to_cache(&self, input: &Input, cache_dir: &Path) -> Result<()> {
        let temp_file =
            tempfile::NamedTempFile::new_in(cache_dir).context("failed to create temp file")?;
        {
            let mut w = BufWriter::new(&temp_file);
            serde_json::to_writer(&mut w, input).context("failed to serialize input")?;
            w.flush()?;
        }

        let hash = input.block.header.hash_slow();
        let cache_path = cache_dir.join(Current.file_name(hash));
        temp_file.persist(cache_path).context("failed to persist cache file")?;

        Ok(())
    }
}

const LEGACY_FORMATS: &[&dyn CacheFormat] = &[&LegacyV1];

trait CacheFormat: Send + Sync {
    fn file_name(&self, hash: BlockHash) -> String;
    fn load(&self, reader: BufReader<File>) -> Result<Input>;

    fn load_from_dir(&self, hash: BlockHash, dir: &Path) -> Result<Option<Input>> {
        let path = dir.join(self.file_name(hash));
        if !path.exists() {
            return Ok(None);
        }

        tracing::info!("Cache hit for block {}. Loading from file: {:?}", hash, path);
        let f = File::open(&path)?;
        let input = self.load(BufReader::new(f)).context("failed to load input from cache file")?;

        Ok(Some(input))
    }
}

struct Current;

impl CacheFormat for Current {
    fn file_name(&self, hash: BlockHash) -> String {
        format!("input_{hash}.v2.json")
    }

    fn load(&self, reader: BufReader<File>) -> Result<Input> {
        Ok(serde_json::from_reader(reader)?)
    }
}

struct LegacyV1;

impl CacheFormat for LegacyV1 {
    fn file_name(&self, hash: BlockHash) -> String {
        format!("input_{hash}.json")
    }

    fn load(&self, reader: BufReader<File>) -> Result<Input> {
        #[serde_with::serde_as]
        #[derive(serde::Deserialize)]
        struct InputV1 {
            #[serde_as(as = "zeth_core::serde_bincode_compat::Block")]
            block: Block,
            witness: ExecutionWitness,
        }

        let legacy: InputV1 =
            serde_json::from_reader(reader).context("failed to deserialize V1")?;
        // v1 input misses the signers
        let signers = recover_signers(legacy.block.body.transactions())?;

        Ok(Input { block: legacy.block, signers, witness: legacy.witness })
    }
}

/// Serializes the input into a byte slice suitable for the RISC Zero ZKVM.
///
/// The ZKVM guest expects aligned words, and this function handles the conversion
/// from a struct to a raw byte vector.
pub fn to_zkvm_input_bytes(input: &Input) -> Vec<u8> {
    let words = risc0_zkvm::serde::to_vec(input).expect("failed to serialize input");
    bytemuck::cast_slice(words.as_slice()).to_vec()
}

/// Recovers the signing [`VerifyingKey`] from each transaction's signature.
pub fn recover_signers<'a, I>(txs: I) -> Result<Vec<UncompressedPublicKey>>
where
    I: IntoIterator<Item = &'a TransactionSigned>,
{
    txs.into_iter()
        .enumerate()
        .map(|(i, tx)| {
            tx.signature()
                .recover_from_prehash(&tx.signature_hash())
                .map(|keys| {
                    UncompressedPublicKey(
                        keys.to_encoded_point(false).as_bytes().try_into().unwrap(),
                    )
                })
                .with_context(|| format!("failed to recover signature for tx #{i}"))
        })
        .collect::<Result<Vec<_>, _>>()
}
