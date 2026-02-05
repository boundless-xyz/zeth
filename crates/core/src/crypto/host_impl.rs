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

//! BigUint-based mock implementation of modmul for host-side testing.

use super::biguint_to_limbs;
use num_bigint::BigUint;

// On host, unchecked and checked are identical; re-export to match risc0_bigint2 structure.
pub(super) mod unchecked {
    pub(crate) use super::*;
}

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

pub(crate) fn modadd_256(a: &[u32; 8], b: &[u32; 8], m: &[u32; 8], res: &mut [u32; 8]) {
    modadd(a, b, m, res)
}

pub(crate) fn modinv_256(a: &[u32; 8], m: &[u32; 8], res: &mut [u32; 8]) {
    modinv(a, m, res)
}

pub(crate) fn modmul_256(a: &[u32; 8], b: &[u32; 8], m: &[u32; 8], res: &mut [u32; 8]) {
    modmul(a, b, m, res)
}

pub(crate) fn modmul_384(a: &[u32; 12], b: &[u32; 12], m: &[u32; 12], res: &mut [u32; 12]) {
    modmul(a, b, m, res)
}

pub(crate) fn modmul_4096(a: &[u32; 128], b: &[u32; 128], m: &[u32; 128], res: &mut [u32; 128]) {
    modmul(a, b, m, res)
}
