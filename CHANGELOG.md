# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Update core dependency to Reth `v2.0.0` and Rust toolchain to `v1.94`.
- Update zkVM dependency to r0vm `v3.0.5`.

### ⚡️ Features

- Add R0VM-optimized `modexp` precompile using `risc0-bigint2` for accelerated modular exponentiation (up to 4096-bit).
- Add R0VM-optimized `secp256r1_verify_signature` precompile using `risc0-bigint2` for accelerated P-256 ECDSA verification (EIP-7951).
- Add R0VM-optimized `BN254_ADD` and `BN254_MUL` precompiles using `risc0-bigint2`.
- Use `risc0_zkp` accelerated SHA-256.
- Add `get_input_cached` method for processing blocks from cache without RPC.
- Add support for running against an Anvil devnet when using the RPC proxy.

### ⚙️ Miscellaneous

- Add Hoodi support and remove Holešky.
- Increase default keep-alive on `zeth-rpc-proxy`.
- Add Wycheproof tests for P-256 ECDSA verification.

## [0.3.0](https://github.com/boundless-xyz/zeth/releases/tag/v0.3.0) - 2025-12-03

- Update core dependency to Reth `v1.9.3`, which includes support for the upcoming Osaka hardfork.
- Update zkVM dependency to r0vm `v3.0.4` and rust `v1.91.1`.

### ⚡️ Features

- New feature `unsafe-pre-merge` must be enabled to prove pre-merge blocks.
- Implement versioned caching for input files. This ensures backward compatibility with older cache files by converting them to the current schema on the fly.

## [0.2.1](https://github.com/boundless-xyz/zeth/releases/tag/v0.2.1) - 2025-08-05

- Fix wrong journal decoding in `cli`

## [0.2.0](https://github.com/boundless-xyz/zeth/releases/tag/v0.2.0) - 2025-07-30

- Initial release based on reth `v1.6.0` and risc0-zkvm `v2.3.1`.
