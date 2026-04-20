# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Update zkVM dependency to r0vm `v3.0.5`.

### ⚡️ Features

- Add R0VM-optimized `modexp` precompile for accelerated modular exponentiation (up to 4096-bit).
- Add R0VM-optimized `secp256r1_verify_signature` precompile for accelerated P-256 ECDSA verification (EIP-7951).
- Add R0VM-optimized `secp256k1_ecrecover` precompile.

### ⚙️ Miscellaneous

- Replace the inline `risc0-bigint2` glue in `zeth-core/src/crypto/` (BN254 / P-256 / modexp / ark-based host fallback) with [`risc0-crypto`](https://github.com/Wollac/risc0-crypto) primitives. Removes `num-bigint`, `risc0-bigint2`, and `ark-bn254` / `ark-ec` / `ark-ff` / `ark-secp256r1` as direct dependencies of `zeth-core`. The `r0vm` cargo feature is dropped; `R0vmCrypto` and `install_r0vm_crypto()` are always exported (no-op on non-zkvm targets).
- Add Hoodi support and remove Holešky.

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
