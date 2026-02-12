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

pub(super) mod bn254;
pub(super) mod p256;

use super::{
    field::{modadd_256, modadd_384, modmul_256, modmul_384, unchecked},
    limbs_into_be_bytes,
};
use crate::crypto::{LIMB_BYTES, be_bytes_to_limbs, is_less};

#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
use super::host_impl::ec::*;
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
use risc0_bigint2::ec::*;

/// Extends [`Curve`] with short Weierstrass curve parameters and point validation.
trait CurveExt<const WIDTH: usize>: Curve<WIDTH>
where
    [u32; WIDTH]: ModOps,
{
    /// Base field modulus p
    const PRIME: [u32; WIDTH];
    /// Curve coefficient a
    const A: [u32; WIDTH];
    /// Curve coefficient b
    const B: [u32; WIDTH];

    /// Decodes an affine point from big-endian encoded input.
    ///
    /// Panics if `input` length is not exactly `2 * WIDTH * LIMB_BYTES`.
    #[inline]
    fn bytes_to_affine(input: &[u8]) -> Option<AffinePoint<WIDTH, Self>>
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

    /// Check if point `(x,y)` satisfies the curve equation `y^2 = x^3 + ax + b (mod p)`.
    #[inline]
    fn satisfies_curve_equation(x: &[u32; WIDTH], y: &[u32; WIDTH]) -> bool {
        let mut t1 = [0u32; WIDTH];
        let mut t2 = [0u32; WIDTH];

        // unchecked: intermediate results that feed the checked final operations
        // t1 <- x^2
        <[u32; WIDTH]>::modmul_unchecked(x, x, &Self::PRIME, &mut t1);

        // When a=0 (e.g. BN254), the compiler eliminates the unused branch,
        // saving one modadd syscall by computing x^3 + b directly.
        if Self::A == [0u32; WIDTH] {
            // t2 <- x^3
            <[u32; WIDTH]>::modmul_unchecked(&t1, x, &Self::PRIME, &mut t2);
            // t1 <- x^3 + b [RHS]
            <[u32; WIDTH]>::modadd(&t2, &Self::B, &Self::PRIME, &mut t1);
            // t2 <- y^2 [LHS]
            <[u32; WIDTH]>::modmul(y, y, &Self::PRIME, &mut t2);
        } else {
            // t2 <- x^2 + a
            <[u32; WIDTH]>::modadd_unchecked(&t1, &Self::A, &Self::PRIME, &mut t2);
            // t1 <- x(x^2 + a)
            <[u32; WIDTH]>::modmul_unchecked(&t2, x, &Self::PRIME, &mut t1);
            // t2 <- (x^3 + ax) + b [RHS]
            <[u32; WIDTH]>::modadd(&t1, &Self::B, &Self::PRIME, &mut t2);
            // t1 <- y^2 [LHS]
            <[u32; WIDTH]>::modmul(y, y, &Self::PRIME, &mut t1);
        }

        t1 == t2
    }
}

/// Modular addition and multiplication dispatched by limb-array width.
trait ModOps {
    fn modadd(a: &Self, b: &Self, m: &Self, res: &mut Self);
    fn modadd_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self);
    fn modmul(a: &Self, b: &Self, m: &Self, res: &mut Self);
    fn modmul_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self);
}

impl ModOps for [u32; EC_256_WIDTH_WORDS] {
    fn modadd(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        modadd_256(a, b, m, res);
    }
    fn modadd_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modadd_256(a, b, m, res);
    }
    fn modmul(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        modmul_256(a, b, m, res);
    }
    fn modmul_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modmul_256(a, b, m, res);
    }
}

impl ModOps for [u32; EC_384_WIDTH_WORDS] {
    fn modadd(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        modadd_384(a, b, m, res);
    }
    fn modadd_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modadd_384(a, b, m, res);
    }
    fn modmul(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        modmul_384(a, b, m, res);
    }
    fn modmul_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modmul_384(a, b, m, res);
    }
}

/// Constructs an [`AffinePoint`] in const context.
///
/// Workaround: `AffinePoint::new_unchecked` is not const in `risc0_bigint2`.
const fn const_new_affine_256<C: Curve<EC_256_WIDTH_WORDS>>(
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

/// Encodes an affine point as big-endian bytes.
fn affine_to_bytes<const WIDTH: usize, C: Curve<WIDTH>>(
    point: AffinePoint<WIDTH, C>,
    output: &mut [u8],
) {
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
