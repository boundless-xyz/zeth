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
//! Implements [EIP-7212](https://eips.ethereum.org/EIPS/eip-7212).

use super::{be_bytes_to_limbs, is_less, modadd_256, modmul_256, unchecked};
use risc0_bigint2::ec::{AffinePoint, Curve, EC_256_WIDTH_WORDS, WeierstrassCurve};

/// Number of 32-bit limbs for 256-bit values.
const N_LIMBS_256: usize = EC_256_WIDTH_WORDS;

/// The zero 256-bit value.
const ZERO: [u32; N_LIMBS_256] = [0; N_LIMBS_256];

/// The secp256r1 (P-256) curve.
enum Secp256r1 {}

impl Secp256r1 {
    /// Base field modulus
    const PRIME: [u32; N_LIMBS_256] = [
        0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000, 0x00000001,
        0xFFFFFFFF,
    ];
    /// Curve coefficient a
    const A: [u32; N_LIMBS_256] = [
        0xFFFFFFFC, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000, 0x00000000, 0x00000000, 0x00000001,
        0xFFFFFFFF,
    ];
    /// Curve coefficient b
    const B: [u32; N_LIMBS_256] = [
        0x27D2604B, 0x3BCE3C3E, 0xCC53B0F6, 0x651D06B0, 0x769886BC, 0xB3EBBD55, 0xAA3A93E7,
        0x5AC635D8,
    ];
    /// Curve order
    const N: [u32; N_LIMBS_256] = [
        0xFC632551, 0xF3B9CAC2, 0xA7179E84, 0xBCE6FAAD, 0xFFFFFFFF, 0xFFFFFFFF, 0x00000000,
        0xFFFFFFFF,
    ];
    /// Base point
    const G: AffinePoint<N_LIMBS_256, Self> = const_affine_point([
        [
            0xd898c296, 0xf4a13945, 0x2deb33a0, 0x77037d81, 0x63a440f2, 0xf8bce6e5, 0xe12c4247,
            0x6b17d1f2,
        ],
        [
            0x37bf51f5, 0xcbb64068, 0x6b315ece, 0x2bce3357, 0x7c0f9e16, 0x8ee7eb4a, 0xfe1a7f9b,
            0x4fe342e2,
        ],
    ]);

    /// Check if point `(x,y)` satisfies the curve equation `y^2 = x^3 + ax + b (mod p)`.
    fn satisfies_curve_equation(x: &[u32; N_LIMBS_256], y: &[u32; N_LIMBS_256]) -> bool {
        let mut t1 = [0u32; N_LIMBS_256];
        let mut t2 = [0u32; N_LIMBS_256];

        // t1 <- x^2
        unchecked::modmul_256(x, x, &Self::PRIME, &mut t1);
        // t2 <- x^2 + a
        unchecked::modadd_256(&t1, &Self::A, &Self::PRIME, &mut t2);
        // t1 <- x(x^2 + a)
        unchecked::modmul_256(&t2, x, &Self::PRIME, &mut t1);
        // t2 <- (x^3 + ax) + b [RHS]
        modadd_256(&t1, &Self::B, &Self::PRIME, &mut t2);
        // t1 <- y^2 [LHS]
        modmul_256(y, y, &Self::PRIME, &mut t1);

        t1 == t2
    }
}

impl Curve<N_LIMBS_256> for Secp256r1 {
    const CURVE: &'static WeierstrassCurve<N_LIMBS_256> =
        &WeierstrassCurve::<N_LIMBS_256>::new(Self::PRIME, Self::A, Self::B);
}

/// Verifies an ECDSA signature over the secp256r1 curve.
pub(super) fn verify_signature(msg_hash: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> bool {
    // Message Hash (h)
    let h = be_bytes_to_limbs(msg_hash);

    // Signature (r, s)
    let r = be_bytes_to_limbs(&sig[0..32]);
    let s = be_bytes_to_limbs(&sig[32..64]);
    // Validate: 0 < r < n and 0 < s < n
    if !(r != ZERO && is_less(&r, &Secp256r1::N)) || !(s != ZERO && is_less(&s, &Secp256r1::N)) {
        return false;
    }

    // Public Key (x, y)
    let qx = be_bytes_to_limbs(&pk[0..32]);
    let qy = be_bytes_to_limbs(&pk[32..64]);
    // Validate: 0 <= qx < p and 0 <= qy < p
    if !is_less(&qx, &Secp256r1::PRIME) || !is_less(&qy, &Secp256r1::PRIME) {
        return false;
    }
    // Validate: (qx, qy) satisfies the curve equation; this implies (qx, qy) != (0, 0)
    if !Secp256r1::satisfies_curve_equation(&qx, &qy) {
        return false;
    }

    let q_pt = AffinePoint::new_unchecked(qx, qy);

    // s_inv = s^(-1) (mod n)
    let mut s_inv = [0u32; N_LIMBS_256];
    unchecked::modinv_256(&s, &Secp256r1::N, &mut s_inv);

    let mut t = [0u32; N_LIMBS_256];

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

    // Compare: r' ≡ r (mod n)
    modadd_256(r_prime, &ZERO, &Secp256r1::N, &mut t);
    t == r
}

/// Constructs an [`AffinePoint`] in const context.
///
/// Workaround: `AffinePoint::new_unchecked` is not const in `risc0_bigint2`.
const fn const_affine_point<C>(coords: [[u32; N_LIMBS_256]; 2]) -> AffinePoint<N_LIMBS_256, C> {
    #[allow(unused)]
    struct Raw {
        buffer: [[u32; N_LIMBS_256]; 2],
        identity: bool,
    }
    // SAFETY: AffinePoint has same layout as Raw; PhantomData<C> is ZST
    unsafe { std::mem::transmute(Raw { buffer: coords, identity: false }) }
}

#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
mod host_impl {
    use crate::crypto::{LIMB_BYTES, be_bytes_to_limbs, limbs_to_be_bytes};
    use ark_ec::CurveGroup;
    use ark_ff::{AdditiveGroup, BigInteger, PrimeField};
    use ark_secp256r1::{Affine, Fq, Projective};

    fn limbs_to_fq(limbs: &[u32; 8]) -> Fq {
        let bytes = limbs_to_be_bytes(limbs, 8 * LIMB_BYTES);
        Fq::from_be_bytes_mod_order(&bytes)
    }

    fn fq_to_limbs(f: Fq) -> [u32; 8] {
        let bytes = f.into_bigint().to_bytes_be();
        be_bytes_to_limbs(&bytes)
    }

    fn ec_add(
        a: &[[u32; 8]; 2],
        b: &[[u32; 8]; 2],
        _curve: &[[u32; 8]; 3], // ignored, using ark_secp256r1
        res: &mut [[u32; 8]; 2],
    ) {
        let a = Affine::new(limbs_to_fq(&a[0]), limbs_to_fq(&a[1]));
        let b = Affine::new(limbs_to_fq(&b[0]), limbs_to_fq(&b[1]));
        let sum = (Projective::from(a) + Projective::from(b)).into_affine();
        res[0] = fq_to_limbs(sum.x);
        res[1] = fq_to_limbs(sum.y);
    }

    fn ec_double(a: &[[u32; 8]; 2], _curve: &[[u32; 8]; 3], res: &mut [[u32; 8]; 2]) {
        let a = Affine::new(limbs_to_fq(&a[0]), limbs_to_fq(&a[1]));
        let double = Projective::from(a).double().into_affine();
        res[0] = fq_to_limbs(double.x);
        res[1] = fq_to_limbs(double.y);
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
