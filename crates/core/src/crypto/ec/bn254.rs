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

//! BN254 (alt_bn128) G1 point addition and scalar multiplication optimized for R0VM.
//!
//! Implements [EIP-196](https://eips.ethereum.org/EIPS/eip-196).

use super::{AffinePoint, Curve, CurveExt, WeierstrassCurve, affine_to_bytes};
use crate::crypto::{LIMB_BITS, be_bytes_to_limbs};

/// Limbs needed to represent the BN254 curve.
const EC_LIMBS: usize = 256 / LIMB_BITS;

/// The BN254 (alt_bn128) curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bn254 {}

impl CurveExt<EC_LIMBS> for Bn254 {
    const PRIME: [u32; EC_LIMBS] =
        hex_limbs!("0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47");
    const A: [u32; EC_LIMBS] =
        hex_limbs!("0x0000000000000000000000000000000000000000000000000000000000000000");
    const B: [u32; EC_LIMBS] =
        hex_limbs!("0x0000000000000000000000000000000000000000000000000000000000000003");
}

impl Curve<EC_LIMBS> for Bn254 {
    const CURVE: &'static WeierstrassCurve<EC_LIMBS> =
        &WeierstrassCurve::new(Self::PRIME, Self::A, Self::B);
}

/// BN254 point addition. Returns `None` if either input is not a valid G1 point.
///
/// Panics if `a` or `b` is not exactly 64 bytes.
pub(crate) fn add(a: &[u8], b: &[u8]) -> Option<[u8; 64]> {
    let a = Bn254::bytes_to_affine(a)?;
    let b = Bn254::bytes_to_affine(b)?;

    let mut sum = AffinePoint::IDENTITY;
    a.add(&b, &mut sum);

    let mut result = [0u8; 64];
    affine_to_bytes(sum, &mut result);
    Some(result)
}

/// BN254 scalar multiplication. Returns `None` if the input is not a valid G1 point.
///
/// Panics if `p` is not exactly 64 bytes or `scalar` is longer than 32 bytes.
pub(crate) fn mul(p: &[u8], scalar: &[u8]) -> Option<[u8; 64]> {
    let p = Bn254::bytes_to_affine(p)?;
    let scalar = be_bytes_to_limbs::<EC_LIMBS>(scalar);

    let mut prod = AffinePoint::IDENTITY;
    p.mul(&scalar, &mut prod);

    let mut result = [0u8; 64];
    affine_to_bytes(prod, &mut result);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ec::const_new_affine_256;

    // non-reduced (1,2)
    const G: AffinePoint<EC_LIMBS, Bn254> = const_new_affine_256([
        hex_limbs!("0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd48"),
        hex_limbs!("0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd49"),
    ]);

    #[test]
    fn satisfies_curve_equation_valid() {
        let [gx, gy] = G.as_u32s().unwrap();
        assert!(Bn254::satisfies_curve_equation(gx, gy));
    }

    #[test]
    fn satisfies_curve_equation_invalid() {
        let [gx, gy] = G.as_u32s().unwrap();
        let mut bad_gy = *gy;
        bad_gy[0] ^= 1; // flip one bit
        assert!(!Bn254::satisfies_curve_equation(gx, &bad_gy));
    }
}
