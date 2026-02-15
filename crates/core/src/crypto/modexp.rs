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

use super::{
    LIMB_BITS, LIMB_BYTES, be_bytes_to_limbs, biguint_to_limbs, field::unchecked, is_less,
    limbs_into_be_bytes,
};
use num_bigint::BigUint;

/// Bit-level access to an integer value.
pub(super) trait BitAccess {
    /// Returns the fewest number of bits necessary to represent this value.
    fn bits(&self) -> usize;
    /// Returns the `i`-th bit (0 = LSB). Returns `false` for out-of-range bits.
    fn bit(&self, i: usize) -> bool;
}

/// Bit-level access for big-endian byte slices.
impl BitAccess for [u8] {
    fn bits(&self) -> usize {
        self.iter()
            .position(|&b| b != 0)
            .map_or(0, |i| (self.len() - i) * 8 - self[i].leading_zeros() as usize)
    }

    fn bit(&self, i: usize) -> bool {
        let byte_offset = i / 8;
        if byte_offset >= self.len() {
            return false;
        }
        self[self.len() - 1 - byte_offset] & (1 << (i % 8)) != 0
    }
}

/// Bit-level access for little-endian limb arrays.
impl<const N: usize> BitAccess for [u32; N] {
    fn bits(&self) -> usize {
        self.iter()
            .rposition(|&l| l != 0)
            .map_or(0, |i| i * LIMB_BITS + (LIMB_BITS - self[i].leading_zeros() as usize))
    }

    fn bit(&self, i: usize) -> bool {
        let limb = i / LIMB_BITS;
        limb < N && self[limb] & (1 << (i % LIMB_BITS)) != 0
    }
}

/// Modular multiplication dispatched by limb-array width.
pub(super) trait ModMul {
    fn modmul_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self);
}

impl ModMul for [u32; 8] {
    fn modmul_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modmul_256(a, b, m, res)
    }
}

impl ModMul for [u32; 12] {
    fn modmul_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modmul_384(a, b, m, res)
    }
}

impl ModMul for [u32; 128] {
    fn modmul_unchecked(a: &Self, b: &Self, m: &Self, res: &mut Self) {
        unchecked::modmul_4096(a, b, m, res)
    }
}

/// Computes `base^exp mod modulus` using square-and-multiply.
///
/// Operates on `[u32; N]` little-endian limb arrays. The exponent can be
/// any type implementing `BitAccess` (e.g. `[u32; N]` or `[u8]`).
///
/// The modulus must be non-zero; behaviour is undefined otherwise.
/// Panics if the result is not canonical (dishonest prover).
pub(super) fn modexp<const N: usize, E>(base: &[u32; N], exp: &E, modulus: &[u32; N]) -> [u32; N]
where
    E: BitAccess + ?Sized,
    [u32; N]: ModMul,
{
    // Double buffering to avoid mem copy
    let mut t1 = [0u32; N];
    let mut t2 = [0u32; N];
    let mut curr = &mut t1;
    let mut next = &mut t2;

    // Initialize result to 1
    curr[0] = 1;

    // Exponentiation by squaring (left-to-right)
    for i in (0..exp.bits()).rev() {
        // next <- curr^2
        <[u32; N]>::modmul_unchecked(curr, curr, modulus, next);
        if exp.bit(i) {
            // curr <- next * base
            <[u32; N]>::modmul_unchecked(next, base, modulus, curr);
        } else {
            // curr <- next
            std::mem::swap(&mut curr, &mut next);
        }
    }

    // Verify result is canonical (honest prover check)
    assert!(is_less(curr, modulus));

    *curr
}

/// Like [`modexp`], but accepts a big-endian byte base and exponent.
/// Returns minimal big-endian bytes (no leading zeros).
pub(super) fn modexp_bytes<const N: usize>(base: &[u8], exp: &[u8], modulus: &[u32; N]) -> Vec<u8>
where
    [u32; N]: ModMul,
{
    // If the modulus is zero, the result is empty
    if modulus.iter().all(|&l| l == 0) {
        return vec![];
    }

    // Fast path: base fits inside the limb array
    let base_arr = if base.len() <= N * LIMB_BYTES {
        be_bytes_to_limbs(base)
    } else {
        // Slow path: Reduction required
        let mut base_bn = BigUint::from_bytes_be(base);
        base_bn %= BigUint::from_slice(modulus);
        biguint_to_limbs(&base_bn)
    };

    let result = modexp(&base_arr, exp, modulus);

    let mut output = vec![0u8; N * LIMB_BYTES];
    limbs_into_be_bytes(&result, &mut output);
    let start = output.iter().position(|&b| b != 0).unwrap_or(output.len());
    output.drain(..start);
    output
}
