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

use num_bigint::BigUint;

const LIMB_BYTES: usize = size_of::<u32>();

#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
pub(super) use host_impl::{modmul_256, modmul_384, modmul_4096};
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
pub(super) use risc0_bigint2::field::unchecked::{modmul_256, modmul_384, modmul_4096};

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
    let mod_arr = bytes_to_limbs(modulus);

    // EIP-198: if the modulus is zero, the result is empty
    if mod_arr.iter().all(|&b| b == 0) {
        return vec![];
    }

    // Fast path: base fits inside the limp array
    let base_arr = if base.len() <= N * LIMB_BYTES {
        bytes_to_limbs(base)
    } else {
        // Slow path: Reduction required
        let mut base_bn = BigUint::from_bytes_be(base);
        let mod_bn = BigUint::from_bytes_be(modulus);
        base_bn %= mod_bn;
        biguint_to_limbs(&base_bn)
    };

    // Double buffering to avoid mem copy
    let mut buf_a = [0u32; N];
    let mut buf_b = [0u32; N];
    let mut curr = &mut buf_a;
    let mut next = &mut buf_b;

    // Initialize result to 1
    curr[0] = 1;

    // Exponentiation by squaring (left-to-right)
    for i in (0..exp.bits()).rev() {
        // Square: next = curr * curr
        modmul_fn(curr, curr, &mod_arr, next);
        if exp.bit(i) {
            // Multiply: curr = next * base
            modmul_fn(next, &base_arr, &mod_arr, curr);
        } else {
            // Swap: curr = next
            std::mem::swap(&mut curr, &mut next);
        }
    }

    // Verify result is canonical (honest prover check)
    assert!(is_less(curr, &mod_arr));

    limbs_to_be_bytes(curr, modulus.len())
}

/// Converts a big-endian byte slice into a fixed-size little-endian limb array.
fn bytes_to_limbs<const N: usize>(bytes: &[u8]) -> [u32; N] {
    let mut arr = [0u32; N];
    for (dst, chunk) in arr.iter_mut().zip(bytes.rchunks(LIMB_BYTES)) {
        *dst = match chunk {
            // Hot path: Full 4-byte chunk
            [a, b, c, d] => u32::from_be_bytes([*a, *b, *c, *d]),
            // Tail paths: 1-3 bytes remaining at the start of the slice
            [a, b, c] => u32::from_be_bytes([0, *a, *b, *c]),
            [a, b] => u32::from_be_bytes([0, 0, *a, *b]),
            [a] => *a as u32,
            _ => unreachable!(),
        };
    }
    arr
}

/// Converts a BigUint to a fixed-size little-endian limb array.
fn biguint_to_limbs<const N: usize>(bn: &BigUint) -> [u32; N] {
    let mut arr = [0u32; N];
    for (dst, src) in arr.iter_mut().zip(bn.iter_u32_digits()) {
        *dst = src;
    }
    arr
}

/// Converts a little-endian limb array to big-endian bytes of specified length.
fn limbs_to_be_bytes<const N: usize>(arr: &[u32; N], len: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(len);
    if len > 0 {
        let idx = (len - 1) / LIMB_BYTES;
        let skip = (idx + 1) * LIMB_BYTES - len;

        bytes.extend_from_slice(&arr[idx].to_be_bytes()[skip..]);
        for i in (0..idx).rev() {
            bytes.extend_from_slice(&arr[i].to_be_bytes());
        }
    }
    bytes
}

/// Returns true if lhs < rhs (little-endian limb arrays).
fn is_less<const N: usize>(lhs: &[u32; N], rhs: &[u32; N]) -> bool {
    lhs.iter().rev().cmp(rhs.iter().rev()) == std::cmp::Ordering::Less
}

#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
#[allow(unreachable_pub)]
mod host_impl {
    //! BigUint-based mock implementation of modmul for host-side testing.
    use super::*;

    fn modmul<const N: usize>(a: &[u32; N], b: &[u32; N], m: &[u32; N], res: &mut [u32; N]) {
        let a = BigUint::from_slice(a);
        let b = BigUint::from_slice(b);
        let m = BigUint::from_slice(m);
        *res = biguint_to_limbs(&((a * b) % m));
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
}
