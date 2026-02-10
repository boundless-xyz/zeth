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
    use crate::crypto::biguint_to_limbs;
    use ec_generic::{EllipticCurve, Point};
    use num_bigint::BigUint;

    pub use risc0_bigint2::ec::{EC_256_WIDTH_WORDS, EC_384_WIDTH_WORDS};

    pub trait Curve<const WIDTH: usize> {
        const CURVE: &'static WeierstrassCurve<WIDTH>;
    }

    #[derive(PartialEq, Clone, Debug)]
    pub struct WeierstrassCurve<const WIDTH: usize> {
        prime: [u32; WIDTH],
        a: [u32; WIDTH],
        b: [u32; WIDTH],
    }

    impl<const WIDTH: usize> WeierstrassCurve<WIDTH> {
        pub const fn new(prime: [u32; WIDTH], a: [u32; WIDTH], b: [u32; WIDTH]) -> Self {
            Self { prime, a, b }
        }

        fn to_curve(&self) -> EllipticCurve {
            EllipticCurve {
                a: BigUint::from_slice(&self.a),
                b: BigUint::from_slice(&self.b),
                p: BigUint::from_slice(&self.prime),
            }
        }
    }

    #[derive(PartialEq, Clone, Debug)]
    pub struct AffinePoint<const WIDTH: usize, C: Curve<WIDTH>>(
        risc0_bigint2::ec::AffinePoint<WIDTH, C>,
    );

    impl<const WIDTH: usize, C: Curve<WIDTH>> AffinePoint<WIDTH, C> {
        pub const IDENTITY: Self = Self(risc0_bigint2::ec::AffinePoint::IDENTITY);

        pub fn new_unchecked(x: [u32; WIDTH], y: [u32; WIDTH]) -> Self {
            Self(risc0_bigint2::ec::AffinePoint::new_unchecked(x, y))
        }
        fn from_point(point: &Point) -> Self {
            match point {
                Point::Coor(x, y) => Self(risc0_bigint2::ec::AffinePoint::new_unchecked(
                    biguint_to_limbs(x),
                    biguint_to_limbs(y),
                )),
                Point::Identity => Self::IDENTITY,
            }
        }

        pub fn as_u32s(&self) -> Option<&[[u32; WIDTH]; 2]> {
            self.0.as_u32s()
        }
        pub fn is_identity(&self) -> bool {
            self.0.is_identity()
        }
        pub fn mul(&self, scalar: &[u32; WIDTH], result: &mut Self) {
            let scalar = BigUint::from_slice(scalar);
            if scalar.bits() == 0 {
                *result = Self::IDENTITY;
            } else {
                let mul = C::CURVE.to_curve().scalar_mul(&self.to_point(), &scalar).unwrap();
                *result = Self::from_point(&mul)
            }
        }
        pub fn double(&self, result: &mut Self) {
            *result = Self::from_point(&C::CURVE.to_curve().double(&self.to_point()).unwrap())
        }
        pub fn add(&self, rhs: &Self, result: &mut Self) {
            let sum = C::CURVE.to_curve().add(&self.to_point(), &rhs.to_point()).unwrap();
            *result = Self::from_point(&sum)
        }

        fn to_point(&self) -> Point {
            match self.0.as_u32s() {
                None => Point::Identity,
                Some([x, y]) => Point::Coor(BigUint::from_slice(x), BigUint::from_slice(y)),
            }
        }
    }
}
