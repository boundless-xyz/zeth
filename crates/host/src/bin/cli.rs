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

use alloy::{eips::BlockId, primitives::B256, providers::ProviderBuilder};
use anyhow::{Context, ensure};
use clap::{Parser, Subcommand};
use humansize::{DECIMAL, format_size};
use std::{fs, path::PathBuf};
use tracing::{Instrument, debug_span, info};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
use zeth_host::{BlockProcessor, to_zkvm_input_bytes};

/// Simple CLI to create Ethereum block execution proofs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Ethereum RPC endpoint URL.
    #[arg(long, env)]
    eth_rpc_url: String,

    /// Block to execute: number (e.g., 21000000), tag ("latest"), or hash ("0xabcd...").
    #[arg(long, global = true, default_value = "latest")]
    block: BlockId,

    /// Directory for caching block inputs.
    #[arg(long, global = true, default_value = "./cache")]
    cache_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
enum Commands {
    /// Validate the block and generate a RISC Zero proof.
    Prove(ProveCommand),

    /// Validate the block on the host machine, without proving.
    Validate,
}

#[derive(Parser, Debug, PartialEq, Eq)]
struct ProveCommand {
    /// Segment size as a power of 2 (e.g., 20 = 1M cycles per segment).
    #[arg(long, env)]
    segment_po2: Option<u32>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    // This is a hack to ensure that `blst` gets linked into this binary.
    let _ = unsafe { blst::blst_p1_sizeof() };

    let cli = Cli::parse();

    // ensure the cache directory exists
    fs::create_dir_all(&cli.cache_dir).context("failed to create cache directory")?;

    // set up the provider and processor
    let provider =
        ProviderBuilder::new().connect(&cli.eth_rpc_url).await.context("RPC connection failed")?;
    let processor = BlockProcessor::new(provider).await?;

    info!(chain = %processor.chain(), "Initialized block processor");

    let input = processor
        .get_input_with_cache(cli.block, &cli.cache_dir)
        .instrument(debug_span!("retrieve_input"))
        .await?;
    let block_hash = input.block.hash_slow();

    info!(
        block_number = input.block.number,
        %block_hash,
        size = %format_size(to_zkvm_input_bytes(&input).len(), DECIMAL),
        "Retrieved input for block",
    );

    // always validate
    {
        let _guard = debug_span!("validate").entered();
        processor.validate(input.clone()).context("host validation failed")?;
    }
    info!("Host validation successful");

    // create proof if requested
    if let Commands::Prove(ProveCommand { segment_po2 }) = cli.command {
        let (receipt, image_id) = processor
            .prove(input, segment_po2)
            .instrument(debug_span!("prove"))
            .await
            .context("proving failed")?;

        receipt.verify(image_id).context("proof verification failed")?;

        let proven_hash =
            B256::try_from(receipt.journal.as_ref()).context("failed to decode journal")?;
        ensure!(proven_hash == block_hash, "journal output mismatch");
        info!(%block_hash, "Proving successful");
    }

    Ok(())
}

fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}
