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
use tokio::time::Instant;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use std::{
    cmp::PartialEq,
    fs::{self},
    path::PathBuf,
};
use zeth_host::{BlockProcessor, to_zkvm_input_bytes};

/// Simple CLI to create Ethereum block execution proofs.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// URL of the Ethereum RPC endpoint to connect to.
    #[arg(long, env)]
    eth_rpc_url: String,

    /// Block number, tag, or hash (e.g., "latest", "0x1565483") to execute.
    #[arg(long, global = true, default_value = "latest")]
    block: BlockId,

    /// Cache folder for input files.
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
    /// Optional segment limit po2
    #[arg(long, env)]
    segment_po2: Option<u32>,
}

/// Configure the tracing library.
fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
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
    let provider = ProviderBuilder::new().connect(&cli.eth_rpc_url).await?;
    let processor = BlockProcessor::new(provider).await?;

    tracing::info!(chain = %processor.chain(), "Initialized block processor");
    let retrieve_input_start = Instant::now();

    let input = processor.get_input_with_cache(cli.block, &cli.cache_dir).await?;
    let block_hash = input.block.hash_slow();

    tracing::info!(
        block_number=input.block.number,
        %block_hash,
        size=format!("{:.3} MB", to_zkvm_input_bytes(&input)?.len() as f64 / 1e6),
        elapsed=?retrieve_input_start.elapsed(),
        "Retrieved input for block",
    );

    // always validate
    let validate_start = Instant::now();
    processor.validate(input.clone()).context("host validation failed")?;
    tracing::info!(elapsed=?validate_start.elapsed(), "Host validation successful");

    // create proof if requested
    if let Commands::Prove(ProveCommand { segment_po2 }) = cli.command {
        let proving_start = Instant::now();

        let (receipt, image_id) =
            processor.prove(input, segment_po2).await.context("proving failed")?;

        tracing::info!(elapsed=?proving_start.elapsed(), "Proving completed");
        receipt.verify(image_id).context("proof verification failed")?;

        let proven_hash =
            B256::try_from(receipt.journal.as_ref()).context("failed to decode journal")?;
        ensure!(proven_hash == block_hash, "journal output mismatch");
    }

    Ok(())
}
