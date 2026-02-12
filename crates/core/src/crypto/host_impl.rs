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

//! BigUint-based host implementation of the `risc0-bigint2` API.

#![allow(unused, unreachable_pub)]

pub(super) mod field {
    use crate::crypto::biguint_to_limbs;
    use num_bigint::BigUint;

    fn modadd<const N: usize>(a: &[u32; N], b: &[u32; N], m: &[u32; N], res: &mut [u32; N]) {
        let a = BigUint::from_slice(a);
        let b = BigUint::from_slice(b);
        let m = BigUint::from_slice(m);
        *res = biguint_to_limbs(&((a + b) % m));
    }

    fn modinv<const N: usize>(a: &[u32; N], m: &[u32; N], res: &mut [u32; N]) {
        let a = BigUint::from_slice(a);
        let m = BigUint::from_slice(m);
        *res = biguint_to_limbs(&a.modinv(&m).unwrap());
    }

    fn modmul<const N: usize>(a: &[u32; N], b: &[u32; N], m: &[u32; N], res: &mut [u32; N]) {
        let a = BigUint::from_slice(a);
        let b = BigUint::from_slice(b);
        let m = BigUint::from_slice(m);
        *res = biguint_to_limbs(&((a * b) % m));
    }

    pub fn modadd_256(a: &[u32; 8], b: &[u32; 8], m: &[u32; 8], res: &mut [u32; 8]) {
        modadd(a, b, m, res)
    }

    pub fn modadd_384(a: &[u32; 12], b: &[u32; 12], m: &[u32; 12], res: &mut [u32; 12]) {
        modadd(a, b, m, res)
    }

    pub fn modinv_256(a: &[u32; 8], m: &[u32; 8], res: &mut [u32; 8]) {
        modinv(a, m, res)
    }

    pub fn modinv_384(a: &[u32; 12], m: &[u32; 12], res: &mut [u32; 12]) {
        modinv(a, m, res)
    }

    pub fn modmul_256(a: &[u32; 8], b: &[u32; 8], m: &[u32; 8], res: &mut [u32; 8]) {
        modmul(a, b, m, res)
    }

    pub fn modmul_384(a: &[u32; 12], b: &[u32; 12], m: &[u32; 12], res: &mut [u32; 12]) {
        modmul(a, b, m, res)
    }

    pub fn modmul_4096(a: &[u32; 128], b: &[u32; 128], m: &[u32; 128], res: &mut [u32; 128]) {
        modmul(a, b, m, res)
    }

    // On host, unchecked and checked are identical; re-export to match risc0_bigint2 structure.
    pub mod unchecked {
        pub use super::*;
    }
}

pub(super) mod ec {
    use ark_ec::{AffineRepr, CurveGroup, short_weierstrass as sw};
    use ark_ff::{BigInteger, PrimeField};

    pub use risc0_bigint2::ec::{EC_256_WIDTH_WORDS, EC_384_WIDTH_WORDS};

    pub trait Curve<const WIDTH: usize> {
        const CURVE: &'static WeierstrassCurve<WIDTH>;
    }

    /// Maps a curve marker type to its arkworks short-Weierstrass config.
    pub trait ArkSW: Curve<EC_256_WIDTH_WORDS> {
        type Config: sw::SWCurveConfig<BaseField: PrimeField>;
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct WeierstrassCurve<const WIDTH: usize>(risc0_bigint2::ec::WeierstrassCurve<WIDTH>);

    impl<const WIDTH: usize> WeierstrassCurve<WIDTH> {
        pub const fn new(prime: [u32; WIDTH], a: [u32; WIDTH], b: [u32; WIDTH]) -> Self {
            Self(risc0_bigint2::ec::WeierstrassCurve::new(prime, a, b))
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub struct AffinePoint<const WIDTH: usize, C>(risc0_bigint2::ec::AffinePoint<WIDTH, C>);

    // Manual clone and copy implementations to not require C to be Copy/Clone
    impl<const WIDTH: usize, C> Clone for AffinePoint<WIDTH, C> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<const WIDTH: usize, C> Copy for AffinePoint<WIDTH, C> {}

    impl<const WIDTH: usize, C> AffinePoint<WIDTH, C> {
        pub const IDENTITY: Self = Self(risc0_bigint2::ec::AffinePoint::IDENTITY);

        pub fn new_unchecked(x: [u32; WIDTH], y: [u32; WIDTH]) -> Self {
            Self(risc0_bigint2::ec::AffinePoint::new_unchecked(x, y))
        }
        pub fn as_u32s(&self) -> Option<&[[u32; WIDTH]; 2]> {
            self.0.as_u32s()
        }
        pub fn is_identity(&self) -> bool {
            self.0.is_identity()
        }
    }

    impl<C: ArkSW> AffinePoint<EC_256_WIDTH_WORDS, C> {
        pub fn mul(&self, scalar: &[u32; EC_256_WIDTH_WORDS], result: &mut Self) {
            let scalar_u64 = limbs_to_u64s(scalar);
            let prod = self.to_ark().mul_bigint(&scalar_u64);
            *result = Self::from_ark(prod.into_affine());
        }

        pub fn double(&self, result: &mut Self) {
            self.add(self, result);
        }

        pub fn add(&self, rhs: &Self, result: &mut Self) {
            let sum = self.to_ark().into_group() + rhs.to_ark().into_group();
            *result = Self::from_ark(sum.into_affine());
        }

        fn to_ark(&self) -> sw::Affine<C::Config> {
            match self.0.as_u32s() {
                None => sw::Affine::identity(),
                Some([x, y]) => sw::Affine::new_unchecked(limbs_to_field(x), limbs_to_field(y)),
            }
        }

        fn from_ark(p: sw::Affine<C::Config>) -> Self {
            if p.is_zero() {
                Self::IDENTITY
            } else {
                Self::new_unchecked(field_to_limbs(&p.x), field_to_limbs(&p.y))
            }
        }
    }

    fn limbs_to_field<const N: usize, F: PrimeField>(limbs: &[u32; N]) -> F {
        let bytes: Vec<u8> = limbs.iter().flat_map(|l| l.to_le_bytes()).collect();
        F::from_le_bytes_mod_order(&bytes)
    }

    fn field_to_limbs<const N: usize, F: PrimeField>(fp: &F) -> [u32; N] {
        assert_eq!(F::BigInt::NUM_LIMBS * 2, N);
        let bigint = fp.into_bigint();
        let u64s = bigint.as_ref();
        let mut result = [0u32; N];
        for i in 0..N / 2 {
            result[2 * i] = u64s[i] as u32;
            result[2 * i + 1] = (u64s[i] >> 32) as u32;
        }
        result
    }

    fn limbs_to_u64s<const N: usize>(limbs: &[u32; N]) -> Vec<u64> {
        limbs.chunks_exact(2).map(|c| (c[0] as u64) | ((c[1] as u64) << 32)).collect()
    }
}
