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

//! Versioned cache format for [`Input`] with automatic migration from legacy formats.
//!
//! Cache files are named `input_{block_hash}.{version}.json` and stored in a configurable
//! directory. When a legacy file is loaded, it is automatically re-saved in the current format.
//!
//! | Version | File pattern              | Format                                            |
//! |---------|---------------------------|---------------------------------------------------|
//! | v3      | `input_{hash}.v3.json`    | Default `Block` serde (camelCase, hex quantities) |
//! | v2      | `input_{hash}.v2.json`    | `serde_bincode_compat` block, with signers        |
//! | v1      | `input_{hash}.json`       | `serde_bincode_compat` block, no signers          |

use alloy::primitives::BlockHash;
use alloy_rlp::{Decodable, Encodable};
use anyhow::{Context, Result};
use stateless::{ExecutionWitness, UncompressedPublicKey};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};
use zeth_core::Input;

use crate::recover_signers;

/// Ordered list of legacy formats to try when the current format is not found.
const LEGACY_FORMATS: &[&dyn CacheFormat] = &[&LegacyV2, &LegacyV1];

/// Loads an [`Input`] from the cache directory, trying the current format first and then
/// falling back to legacy formats. Returns `None` on cache miss.
///
/// When a legacy file is found, the input is automatically re-saved in the current format.
pub(crate) fn load(block_hash: BlockHash, cache_dir: &Path) -> Result<Option<Input>> {
    if let Some(input) = Current.load_from_dir(block_hash, cache_dir)? {
        return Ok(Some(input));
    }
    for format in LEGACY_FORMATS {
        if let Some(input) = format.load_from_dir(block_hash, cache_dir)? {
            if let Err(err) = save(&input, cache_dir) {
                tracing::warn!("Failed to save migrated cache: {}", err);
            }
            return Ok(Some(input));
        }
    }
    Ok(None)
}

/// Atomically writes an [`Input`] to the cache directory in the current format.
pub(crate) fn save(input: &Input, cache_dir: &Path) -> Result<()> {
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
        format!("input_{hash}.v3.json")
    }

    fn load(&self, reader: BufReader<File>) -> Result<Input> {
        Ok(serde_json::from_reader(reader)?)
    }
}

/// `serde_bincode_compat` block type from reth v1 used to deserialize the v1 and v2 cache formats.
type RethV1Block<'a> = reth_primitives_traits_v1::serde_bincode_compat::Block<
    'a,
    reth_ethereum_primitives_v1::TransactionSigned,
    reth_primitives_traits_v1::Header,
>;

/// Convert a legacy v1-typed block into the workspace block type by RLP round-tripping.
/// Ethereum block RLP is a stable protocol-level format, so this works across the alloy 1.x
/// → 2.x type boundary.
fn v1_to_workspace(
    v1_block: reth_ethereum_primitives_v1::Block,
) -> Result<reth_ethereum_primitives::Block> {
    let mut buf = Vec::with_capacity(v1_block.length());
    v1_block.encode(&mut buf);
    reth_ethereum_primitives::Block::decode(&mut buf.as_slice())
        .context("failed to re-decode legacy block as workspace block")
}

struct LegacyV2;

impl CacheFormat for LegacyV2 {
    fn file_name(&self, hash: BlockHash) -> String {
        format!("input_{hash}.v2.json")
    }

    fn load(&self, reader: BufReader<File>) -> Result<Input> {
        #[serde_with::serde_as]
        #[derive(serde::Deserialize)]
        struct InputV2 {
            #[serde_as(as = "RethV1Block<'_>")]
            block: reth_ethereum_primitives_v1::Block,
            signers: Vec<UncompressedPublicKey>,
            witness: ExecutionWitness,
        }

        let v2: InputV2 = serde_json::from_reader(reader).context("failed to deserialize V2")?;
        let block = v1_to_workspace(v2.block)?;
        Ok(Input { block, signers: v2.signers, witness: v2.witness })
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
            #[serde_as(as = "RethV1Block<'_>")]
            block: reth_ethereum_primitives_v1::Block,
            witness: ExecutionWitness,
        }

        let v1: InputV1 = serde_json::from_reader(reader).context("failed to deserialize V1")?;
        let block = v1_to_workspace(v1.block)?;
        let signers = recover_signers(block.body.transactions())?;
        Ok(Input { block, signers, witness: v1.witness })
    }
}
