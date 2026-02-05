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

use super::{LIMB_BYTES, be_bytes_to_limbs, biguint_to_limbs, is_less, limbs_to_be_bytes};
use num_bigint::BigUint;

trait BitAccess {
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

/// Computes `base^exp mod modulus` using square-and-multiply with N-limb arithmetic.
///
/// `modmul_fn` is expected to be unchecked; final `is_less` ensures canonical result.
pub(super) fn modexp_generic<const N: usize, F>(
    base: &[u8],
    exp: &[u8],
    modulus: &[u8],
    modmul_fn: F,
) -> Vec<u8>
where
    F: Fn(&[u32; N], &[u32; N], &[u32; N], &mut [u32; N]),
{
    assert!(modulus.len() <= N * LIMB_BYTES);
    let mod_arr = be_bytes_to_limbs(modulus);

    // EIP-198: if the modulus is zero, the result is empty
    if mod_arr.iter().all(|&b| b == 0) {
        return vec![];
    }

    // Fast path: base fits inside the limb array
    let base_arr = if base.len() <= N * LIMB_BYTES {
        be_bytes_to_limbs(base)
    } else {
        // Slow path: Reduction required
        let mut base_bn = BigUint::from_bytes_be(base);
        let mod_bn = BigUint::from_bytes_be(modulus);
        base_bn %= mod_bn;
        biguint_to_limbs(&base_bn)
    };

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
        modmul_fn(curr, curr, &mod_arr, next);
        if exp.bit(i) {
            // curr <- next * base
            modmul_fn(next, &base_arr, &mod_arr, curr);
        } else {
            // curr <- next
            std::mem::swap(&mut curr, &mut next);
        }
    }

    // Verify result is canonical (honest prover check)
    assert!(is_less(curr, &mod_arr));

    limbs_to_be_bytes(curr, modulus.len())
}
