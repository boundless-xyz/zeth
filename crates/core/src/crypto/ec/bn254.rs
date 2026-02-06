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

use super::{super::be_bytes_to_limbs, Curve, encode_point};
use reth_evm::revm::precompile::PrecompileError;
use risc0_bigint2::ec::{AffinePoint, Curve as R0vmCurve, EC_256_WIDTH_WORDS, WeierstrassCurve};

/// The BN254 (alt_bn128) curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bn254 {}

impl Curve<EC_256_WIDTH_WORDS> for Bn254 {
    const PRIME: [u32; EC_256_WIDTH_WORDS] =
        hex_limbs!("0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47");
    const A: [u32; EC_256_WIDTH_WORDS] =
        hex_limbs!("0x0000000000000000000000000000000000000000000000000000000000000000");
    const B: [u32; EC_256_WIDTH_WORDS] =
        hex_limbs!("0x0000000000000000000000000000000000000000000000000000000000000003");
}

impl R0vmCurve<EC_256_WIDTH_WORDS> for Bn254 {
    const CURVE: &'static WeierstrassCurve<EC_256_WIDTH_WORDS> =
        &WeierstrassCurve::new(Self::PRIME, Self::A, Self::B);
}

/// BN254 point addition.
pub(crate) fn add(a: &[u8], b: &[u8]) -> Result<[u8; 64], PrecompileError> {
    let a = Bn254::decode_point(a).ok_or(PrecompileError::Bn254AffineGFailedToCreate)?;
    let b = Bn254::decode_point(b).ok_or(PrecompileError::Bn254AffineGFailedToCreate)?;

    let mut sum = AffinePoint::IDENTITY;
    a.add(&b, &mut sum);

    let mut result = [0u8; 64];
    encode_point(sum, &mut result);
    Ok(result)
}

/// BN254 scalar multiplication.
pub(crate) fn mul(p: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileError> {
    let p = Bn254::decode_point(p).ok_or(PrecompileError::Bn254AffineGFailedToCreate)?;
    let scalar = be_bytes_to_limbs::<EC_256_WIDTH_WORDS>(scalar);

    let mut prod = AffinePoint::IDENTITY;
    p.mul(&scalar, &mut prod);

    let mut result = [0u8; 64];
    encode_point(prod, &mut result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ec::const_affine_point_256;

    // non-reduced (1,2)
    const G: AffinePoint<EC_256_WIDTH_WORDS, Bn254> = const_affine_point_256([
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
