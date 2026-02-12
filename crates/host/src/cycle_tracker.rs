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

use alloy::consensus::Header;
use reth_chainspec::ChainSpec;
use risc0_zkvm::ExecutorEnvBuilder;
use std::sync::Arc;

#[cfg(feature = "cycle-tracker")]
pub use implementation::*;

#[cfg(not(feature = "cycle-tracker"))]
pub use noop::*;

#[cfg(feature = "cycle-tracker")]
mod implementation {
    use super::*;
    use alloy::primitives::map::AddressMap;
    use flate2::{Compression, write::GzEncoder};
    use reth_evm::{
        ConfigureEvm,
        revm::{
            bytecode::OpCode,
            precompile::{PrecompileSpecId, Precompiles},
        },
    };
    use std::{collections::HashMap, env, fs::File, io::BufWriter};
    use zeth_core::{
        EthEvmConfig,
        cycle_tracker::{CYCLE_TRACKER_FD, TraceCollector, TraceEvent, TraceId},
    };

    const TRACE_FILE_ENV: &str = "TRACE_FILE";
    const DEFAULT_TRACE_FILE: &str = "trace.json.gz";

    pub struct HostCycleTracker {
        metrics: HashMap<String, Vec<(u64, u64)>>,
        precompiles: AddressMap<&'static str>,
    }

    /// Manages the collection and saving of execution cycle traces from the zkVM guest.
    ///
    /// This struct runs on the host and listens to trace events streamed from the guest via a
    /// dedicated file descriptor. It aggregates these events and saves them to a file for later
    /// analysis.
    impl HostCycleTracker {
        const UNKNOWN_LABEL: &'static str = "unknown";

        pub fn new(chain_spec: Arc<ChainSpec>, header: &Header) -> Self {
            // initialize precompiles map for decoding addresses
            let spec_id = EthEvmConfig::new(chain_spec).evm_env(header).unwrap().cfg_env.spec;
            let precompiles = Precompiles::new(PrecompileSpecId::from_spec_id(spec_id)).inner();

            Self {
                metrics: HashMap::default(),
                precompiles: precompiles.iter().map(|(a, p)| (*a, p.id().name())).collect(),
            }
        }

        /// Attaches the cycle tracker to the zkVM executor environment.
        pub fn attach<'a>(&'a mut self, env_builder: &mut ExecutorEnvBuilder<'a>) {
            let collector = TraceCollector::new(move |event| {
                self.process(event);
            });
            env_builder.write_fd(CYCLE_TRACKER_FD, collector);
        }

        fn process(&mut self, event: TraceEvent) {
            let key = match event.id {
                TraceId::Custom(str) => str.into_owned(),
                TraceId::Opcode(op) => match OpCode::new(op) {
                    Some(opcode) => opcode.to_string(),
                    None => Self::UNKNOWN_LABEL.to_string(),
                },
                TraceId::Precompile(addr) => match self.precompiles.get(&addr) {
                    Some(precompile) => precompile.to_string(),
                    None => Self::UNKNOWN_LABEL.to_string(),
                },
            };

            self.metrics.entry(key).or_default().push((event.cycles, event.gas));
        }

        /// Saves the collected trace metrics to a Gzip-compressed JSON file.
        ///
        /// The output path is determined by the `TRACE_FILE` environment variable, defaulting to
        /// `trace.json.gz` if unset.
        pub fn save(self) -> anyhow::Result<()> {
            let path = env::var_os(TRACE_FILE_ENV).unwrap_or(DEFAULT_TRACE_FILE.into());
            let file = File::create(path)?;
            let encoder = GzEncoder::new(BufWriter::new(file), Compression::fast());
            serde_json::to_writer(encoder, &self.metrics)?;

            Ok(())
        }
    }
}

#[cfg(not(feature = "cycle-tracker"))]
mod noop {
    use super::*;

    pub struct HostCycleTracker;

    impl HostCycleTracker {
        pub fn new(_: Arc<ChainSpec>, _: &Header) -> Self {
            Self
        }

        pub fn attach(&mut self, _: &mut ExecutorEnvBuilder) {}

        pub fn save(self) -> anyhow::Result<()> {
            Ok(())
        }
    }
}
