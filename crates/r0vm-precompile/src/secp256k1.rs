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

//! secp256k1 public key recovery.

use alloy_primitives::Address;
use risc0_crypto::{
    curves::secp256k1,
    ecdsa::{RecoverableSignature, RecoveryId, Signature},
    BigInt,
};

/// Recovers the Ethereum [`Address`] from an ECDSA signature.
///
/// Returns `None` for `r=0`, `s=0`, `r≥n`, `s≥n`, `recid > 3`, or when no
/// point exists at the derived `x` coordinate. Applies EIP-2 low-S
/// normalization, so both low-S and high-S inputs recover the same address.
#[inline]
pub fn secp256k1_ecrecover(sig: &[u8; 64], recid: u8, msg: &[u8; 32]) -> Option<Address> {
    // Signature (r, s) — both must be canonical scalars in [1, n).
    let r = secp256k1::Fr::from_bigint(BigInt::<8>::from_be_bytes(&sig[..32]))?;
    let s = secp256k1::Fr::from_bigint(BigInt::<8>::from_be_bytes(&sig[32..]))?;
    let signature = Signature::<secp256k1::Config, 8>::new(r, s)?;

    let recovery_id = RecoveryId::from_byte(recid)?;
    let rsig = RecoverableSignature::new(signature, recovery_id);

    // EIP-2: accept either low-S or high-S; recover the canonical address.
    let rsig = rsig.normalized_s();

    let pubkey = rsig.recover(msg)?;
    let (x, y) = pubkey.xy()?;

    let mut xy = [0u8; 64];
    x.as_bigint().write_be_bytes(&mut xy[..32]);
    y.as_bigint().write_be_bytes(&mut xy[32..]);
    Some(Address::from_raw_public_key(&xy))
}
