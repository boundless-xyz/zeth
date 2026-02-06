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

use super::{limbs_into_be_bytes, unchecked};
use crate::crypto::{LIMB_BYTES, be_bytes_to_limbs, is_less};
use risc0_bigint2::ec::{AffinePoint, EC_256_WIDTH_WORDS, EC_384_WIDTH_WORDS};

pub(super) mod bn254;
pub(super) mod p256;

trait Curve<const WIDTH: usize>
where
    [u32; WIDTH]: Field,
{
    /// Base field modulus p
    const PRIME: [u32; WIDTH];
    /// Curve coefficient a
    const A: [u32; WIDTH];
    /// Curve coefficient b
    const B: [u32; WIDTH];

    /// Check if point `(x,y)` satisfies the curve equation `y^2 = x^3 + ax + b (mod p)`.
    #[inline]
    fn satisfies_curve_equation(x: &[u32; WIDTH], y: &[u32; WIDTH]) -> bool {
        let mut t1 = [0u32; WIDTH];
        let mut t2 = [0u32; WIDTH];

        // unchecked: final equality is reduction-agnostic
        // t1 <- x^2
        <[u32; WIDTH]>::unchecked_modmul(x, x, &Self::PRIME, &mut t1);
        // t2 <- x^2 + a
        <[u32; WIDTH]>::unchecked_moadd(&t1, &Self::A, &Self::PRIME, &mut t2);
        // t1 <- x(x^2 + a)
        <[u32; WIDTH]>::unchecked_modmul(&t2, x, &Self::PRIME, &mut t1);
        // t2 <- (x^3 + ax) + b [RHS]
        <[u32; WIDTH]>::unchecked_moadd(&t1, &Self::B, &Self::PRIME, &mut t2);
        // t1 <- y^2 [LHS]
        <[u32; WIDTH]>::unchecked_modmul(y, y, &Self::PRIME, &mut t1);

        t1 == t2
    }

    /// Parses a point big-endian encoded input.
    #[inline]
    fn decode_point(input: &[u8]) -> Option<AffinePoint<WIDTH, Self>>
    where
        Self: Sized,
    {
        assert_eq!(input.len(), 2 * WIDTH * LIMB_BYTES);
        let x = be_bytes_to_limbs(&input[..WIDTH * LIMB_BYTES]);
        let y = be_bytes_to_limbs(&input[WIDTH * LIMB_BYTES..]);

        // (0, 0) is the identity point
        if is_zero(&x) && is_zero(&y) {
            return Some(AffinePoint::IDENTITY);
        }

        // Validate coordinates are in the field
        if !is_less(&x, &Self::PRIME) || !is_less(&y, &Self::PRIME) {
            return None;
        }
        // Validate point is on the curve
        if !Self::satisfies_curve_equation(&x, &y) {
            return None;
        }

        Some(AffinePoint::new_unchecked(x, y))
    }
}

trait Field {
    fn unchecked_moadd(a: &Self, b: &Self, m: &Self, res: &mut Self);
    fn unchecked_modmul(a: &Self, b: &Self, m: &Self, res: &mut Self);
}

impl Field for [u32; EC_256_WIDTH_WORDS] {
    fn unchecked_moadd(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modadd_256(a, b, m, res);
    }
    fn unchecked_modmul(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modmul_256(a, b, m, res);
    }
}

impl Field for [u32; EC_384_WIDTH_WORDS] {
    fn unchecked_moadd(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modadd_384(a, b, m, res);
    }
    fn unchecked_modmul(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modmul_384(a, b, m, res);
    }
}

/// Constructs an [`AffinePoint`] in const context.
///
/// Workaround: `AffinePoint::new_unchecked` is not const in `risc0_bigint2`.
const fn const_affine_point_256<C>(
    coords: [[u32; EC_256_WIDTH_WORDS]; 2],
) -> AffinePoint<EC_256_WIDTH_WORDS, C> {
    #[allow(unused)]
    struct Raw {
        buffer: [[u32; EC_256_WIDTH_WORDS]; 2],
        identity: bool,
    }
    // SAFETY: AffinePoint has same layout as Raw; PhantomData<C> is ZST
    unsafe { std::mem::transmute(Raw { buffer: coords, identity: false }) }
}

/// Encodes a point.
#[inline]
fn encode_point<const WIDTH: usize, C>(point: AffinePoint<WIDTH, C>, output: &mut [u8]) {
    assert_eq!(output.len(), 2 * WIDTH * LIMB_BYTES);
    match point.as_u32s() {
        None => output.fill(0),
        Some([x, y]) => {
            limbs_into_be_bytes(x, &mut output[..WIDTH * LIMB_BYTES]);
            limbs_into_be_bytes(y, &mut output[WIDTH * LIMB_BYTES..]);
        }
    }
}

#[inline(always)]
fn is_zero<const WIDTH: usize>(x: &[u32; WIDTH]) -> bool {
    x.iter().all(|&u| u == 0)
}
