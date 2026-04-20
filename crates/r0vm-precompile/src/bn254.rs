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

//! EIP-196 BN254 G1 add and scalar multiplication.

use risc0_crypto::{AffinePoint, BigInt, curves::bn254};

/// Adds two 64-byte G1 points.
///
/// Treats all-zero input as the point at infinity. Returns `None` for wrong
/// input length, `x ≥ p` or `y ≥ p`, or off-curve points.
#[inline]
pub fn bn254_g1_add(p1: &[u8], p2: &[u8]) -> Option<[u8; 64]> {
    let p1 = parse_point(p1)?;
    let p2 = parse_point(p2)?;
    Some(encode_point(&(&p1 + &p2)))
}

/// Multiplies a 64-byte G1 point by a big-endian scalar. The scalar is reduced
/// mod `n` (the group order) before multiplication. Returns `None` for
/// wrong-length input or an invalid point.
#[inline]
pub fn bn254_g1_mul(point: &[u8], scalar: &[u8]) -> Option<[u8; 64]> {
    let p = parse_point(point)?;
    let s = bn254::Fr::from_be_bytes_mod_order(scalar);
    Some(encode_point(&(&p * &s)))
}

fn parse_point(bytes: &[u8]) -> Option<bn254::Affine> {
    if bytes.len() != 64 {
        return None;
    }
    if bytes.iter().all(|&b| b == 0) {
        return Some(AffinePoint::IDENTITY);
    }
    let x = bn254::Fq::from_bigint(BigInt::<8>::from_be_bytes(&bytes[..32]))?;
    let y = bn254::Fq::from_bigint(BigInt::<8>::from_be_bytes(&bytes[32..]))?;
    AffinePoint::new(x, y)
}

fn encode_point(p: &bn254::Affine) -> [u8; 64] {
    let mut out = [0u8; 64];
    if let Some((x, y)) = p.xy() {
        x.as_bigint().write_be_bytes(&mut out[..32]);
        y.as_bigint().write_be_bytes(&mut out[32..]);
    }
    out
}
