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

//! secp256r1 (P-256) ECDSA signature verification optimized for R0VM.
//!
//! Implements [EIP-7951](https://eips.ethereum.org/EIPS/eip-7951).

use super::{
    super::{be_bytes_to_limbs, is_less, modadd_256, modmul_256, unchecked},
    Curve, const_affine_point_256,
};
use risc0_bigint2::ec::{AffinePoint, Curve as R0vmCurve, EC_256_WIDTH_WORDS, WeierstrassCurve};

/// The zero 256-bit value.
const ZERO: [u32; EC_256_WIDTH_WORDS] = [0; EC_256_WIDTH_WORDS];

/// The secp256r1 (P-256) curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Secp256r1 {}

impl Secp256r1 {
    /// Group order n
    const N: [u32; EC_256_WIDTH_WORDS] =
        hex_limbs!("0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551");
    /// Base point G
    const G: AffinePoint<EC_256_WIDTH_WORDS, Self> = const_affine_point_256([
        hex_limbs!("0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296"),
        hex_limbs!("0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5"),
    ]);
}

impl Curve<EC_256_WIDTH_WORDS> for Secp256r1 {
    const PRIME: [u32; EC_256_WIDTH_WORDS] =
        hex_limbs!("0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff");
    const A: [u32; EC_256_WIDTH_WORDS] =
        hex_limbs!("0xffffffff00000001000000000000000000000000fffffffffffffffffffffffc");
    const B: [u32; EC_256_WIDTH_WORDS] =
        hex_limbs!("0x5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b");
}

impl R0vmCurve<EC_256_WIDTH_WORDS> for Secp256r1 {
    const CURVE: &'static WeierstrassCurve<EC_256_WIDTH_WORDS> =
        &WeierstrassCurve::new(Self::PRIME, Self::A, Self::B);
}

/// Verifies an ECDSA signature over the secp256r1 curve.
pub(crate) fn verify_signature(msg_hash: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> bool {
    // Signature (r, s)
    let r = be_bytes_to_limbs(&sig[0..32]);
    let s = be_bytes_to_limbs(&sig[32..64]);
    // Validate: 0 < r < n and 0 < s < n
    if r == ZERO || !is_less(&r, &Secp256r1::N) || s == ZERO || !is_less(&s, &Secp256r1::N) {
        return false;
    }

    // Public Key (x, y)
    let q_pt = match Secp256r1::decode_point(pk) {
        // Validate: 0 <= qx < p and 0 <= qy < p
        // Validate: (qx, qy) satisfies the curve equation
        None => return false,
        // Validate: (qx, qy) != (0, 0)
        Some(AffinePoint::IDENTITY) => return false,
        Some(pt) => pt,
    };

    // Message Hash (h)
    let h = be_bytes_to_limbs(msg_hash);

    let mut s_inv = [0u32; EC_256_WIDTH_WORDS];
    // s_inv <- s^(-1) (mod n)
    // unchecked: feeds checked modmul below
    unchecked::modinv_256(&s, &Secp256r1::N, &mut s_inv);

    let mut t = [0u32; EC_256_WIDTH_WORDS];

    // Recover the random point used during signing:
    // R' = [h * s_inv]G + [r * s_inv]Q
    let r_prime_pt = {
        // t <- h * s_inv (mod n)
        modmul_256(&h, &s_inv, &Secp256r1::N, &mut t);
        // u₁G <- [h * s_inv]G
        let mut u1_g_pt = AffinePoint::IDENTITY;
        Secp256r1::G.mul(&t, &mut u1_g_pt);
        // t <- r * s_inv (mod n)
        modmul_256(&r, &s_inv, &Secp256r1::N, &mut t);
        // u₂Q <- [r * s_inv]Q
        let mut u2_q_pt = AffinePoint::IDENTITY;
        q_pt.mul(&t, &mut u2_q_pt);
        // R' <- u₁G + u₂Q
        let mut r_prime_pt = AffinePoint::IDENTITY;
        u1_g_pt.add(&u2_q_pt, &mut r_prime_pt);
        r_prime_pt
    };

    let r_prime = match r_prime_pt.as_u32s() {
        None => return false, // Check for point at infinity
        Some([x, _]) => x,    // Extract x-coordinate from R'
    };

    // t <- r' (mod n)
    modadd_256(r_prime, &ZERO, &Secp256r1::N, &mut t);
    // Verify: r' ≡ r (mod n)
    t == r
}

#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
mod host_impl {
    use super::*;
    use crate::crypto::{LIMB_BYTES, be_bytes_to_limbs, limbs_into_be_bytes};
    use ark_ec::CurveGroup;
    use ark_ff::{AdditiveGroup, BigInteger, PrimeField};
    use ark_secp256r1::{Affine, Fq, Projective};

    const CURVE_PARAMS: [[u32; 8]; 3] = [Secp256r1::PRIME, Secp256r1::A, Secp256r1::B];

    fn limbs_to_fq(limbs: &[u32; 8]) -> Fq {
        let mut bytes = [0u8; 8 * LIMB_BYTES];
        limbs_into_be_bytes(limbs, &mut bytes);
        Fq::from_be_bytes_mod_order(&bytes)
    }

    fn fq_to_limbs(f: Fq) -> [u32; 8] {
        let bytes = f.into_bigint().to_bytes_be();
        be_bytes_to_limbs(&bytes)
    }

    fn ec_double(a: &[[u32; 8]; 2], curve: &[[u32; 8]; 3], res: &mut [[u32; 8]; 2]) {
        assert_eq!(curve, &CURVE_PARAMS); // this host implementation is only for secp256r1
        let a = Affine::new(limbs_to_fq(&a[0]), limbs_to_fq(&a[1]));
        let double = Projective::from(a).double().into_affine();
        res[0] = fq_to_limbs(double.x);
        res[1] = fq_to_limbs(double.y);
    }

    fn ec_add(
        a: &[[u32; 8]; 2],
        b: &[[u32; 8]; 2],
        curve: &[[u32; 8]; 3],
        res: &mut [[u32; 8]; 2],
    ) {
        assert_eq!(curve, &CURVE_PARAMS); // this host implementation is only for secp256r1
        let a = Affine::new(limbs_to_fq(&a[0]), limbs_to_fq(&a[1]));
        let b = Affine::new(limbs_to_fq(&b[0]), limbs_to_fq(&b[1]));
        let sum = (Projective::from(a) + Projective::from(b)).into_affine();
        res[0] = fq_to_limbs(sum.x);
        res[1] = fq_to_limbs(sum.y);
    }

    #[unsafe(no_mangle)]
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe extern "C" fn sys_bigint2_3(
        _blob_ptr: *const u8,
        a1: *const u32,
        a2: *const u32,
        a3: *const u32,
    ) {
        let res = &mut *a3.cast::<[[u32; 8]; 2]>().cast_mut();
        ec_double(&*a1.cast(), &*a2.cast(), res);
    }

    #[unsafe(no_mangle)]
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe extern "C" fn sys_bigint2_4(
        _blob_ptr: *const u8,
        a1: *const u32,
        a2: *const u32,
        a3: *const u32,
        a4: *const u32,
    ) {
        let res = &mut *a4.cast::<[[u32; 8]; 2]>().cast_mut();
        ec_add(&*a1.cast(), &*a2.cast(), &*a3.cast(), res);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;

    #[test]
    fn g_const_layout() {
        let qx = be_bytes_to_limbs(&hex!(
            "0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296"
        ));
        let qy = be_bytes_to_limbs(&hex!(
            "0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5"
        ));
        assert_eq!(Secp256r1::G, AffinePoint::new_unchecked(qx, qy));
    }

    #[test]
    fn satisfies_curve_equation_valid() {
        let [gx, gy] = Secp256r1::G.as_u32s().unwrap();
        assert!(Secp256r1::satisfies_curve_equation(gx, gy));
    }

    #[test]
    fn satisfies_curve_equation_invalid() {
        let [gx, gy] = Secp256r1::G.as_u32s().unwrap();
        let mut bad_gy = *gy;
        bad_gy[0] ^= 1; // flip one bit
        assert!(!Secp256r1::satisfies_curve_equation(gx, &bad_gy));
    }
}
