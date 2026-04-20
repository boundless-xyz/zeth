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

//! R0VM-accelerated EVM precompile primitives.
//!
//! Plain functions — no revm dependency, no `Crypto` trait impl. Consumers
//! write a ~50-line adapter that implements whichever `revm_precompile::Crypto`
//! version they're on by delegating to these functions, so this crate never
//! blocks a revm upgrade.
//!
//! # Targets
//!
//! This crate assumes it is built for the R0VM zkVM target
//! (`riscv32im-risc0-zkvm-elf`). The primitives call through to
//! `risc0-bigint2` syscalls that only the guest runtime defines, so linking
//! fails on other targets. Consumers should target-gate the dependency:
//!
//! ```toml
//! [target.'cfg(all(target_os = "zkvm", target_vendor = "risc0"))'.dependencies]
//! zeth-r0vm-precompile = "…"
//! ```
//!
//! and place their `impl Crypto for R0vmCrypto { … }` block under the same
//! cfg gate.

#![no_std]

extern crate alloc;

mod bn254;
mod modexp;
mod secp256k1;
mod secp256r1;
mod sha256;

pub use bn254::{bn254_g1_add, bn254_g1_mul};
pub use modexp::modexp;
pub use secp256k1::secp256k1_ecrecover;
pub use secp256r1::secp256r1_verify;
pub use sha256::sha256;
