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

//! EIP-7951 P-256 signature verification.

use risc0_crypto::{curves::secp256r1, ecdsa::Signature, AffinePoint, BigInt};

/// Verifies a P-256 ECDSA signature.
///
/// Returns `false` for `r=0`, `s=0`, `r≥n`, `s≥n`, public keys with `x≥p` or
/// `y≥p`, and off-curve public keys (including the all-zero `(0, 0)` pubkey).
#[inline]
pub fn secp256r1_verify(msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> bool {
    verify_inner(msg, sig, pk).unwrap_or(false)
}

fn verify_inner(msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> Option<bool> {
    // Signature (r, s) — both must be canonical scalars in [1, n).
    let r = secp256r1::Fr::from_bigint(BigInt::<8>::from_be_bytes(&sig[..32]))?;
    let s = secp256r1::Fr::from_bigint(BigInt::<8>::from_be_bytes(&sig[32..]))?;
    let signature = Signature::<secp256r1::Config, 8>::new(r, s)?;

    // Public key (x, y) — both must be canonical base-field elements on the
    // curve. `AffinePoint::new` rejects off-curve inputs including (0, 0).
    let x = secp256r1::Fq::from_bigint(BigInt::<8>::from_be_bytes(&pk[..32]))?;
    let y = secp256r1::Fq::from_bigint(BigInt::<8>::from_be_bytes(&pk[32..]))?;
    let pubkey = AffinePoint::<secp256r1::Config, 8>::new(x, y)?;

    Some(signature.verify(&pubkey, msg))
}
