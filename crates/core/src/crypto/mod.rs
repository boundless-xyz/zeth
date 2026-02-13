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

/// Converts a hex literal into a little-endian limb array at compile time.
macro_rules! hex_limbs {
    ($hex:literal) => {{
        const BYTES: &[u8] = &alloy_primitives::hex!($hex);
        const { assert!(BYTES.len() % 4 == 0, "Hex string must be a multiple of 4 bytes") };
        let mut limbs = [0u32; BYTES.len() / 4];
        let mut i = 0;
        while i < BYTES.len() / 4 {
            let j = BYTES.len() - (i + 1) * 4;
            limbs[i] = u32::from_be_bytes([BYTES[j], BYTES[j + 1], BYTES[j + 2], BYTES[j + 3]]);
            i += 1;
        }
        limbs
    }};
}

mod ec;
#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
mod host_impl;
mod modexp;

use num_bigint::BigUint;
use reth_evm::revm::precompile::{Crypto, DefaultCrypto, PrecompileError, install_crypto};

#[cfg(not(all(target_os = "zkvm", target_vendor = "risc0")))]
use host_impl::*;
#[cfg(all(target_os = "zkvm", target_vendor = "risc0"))]
use risc0_bigint2::*;

/// Bytes per limb.
const LIMB_BYTES: usize = size_of::<u32>();
/// Bits per limb.
const LIMB_BITS: usize = u32::BITS as usize;

/// R0VM-optimized [`Crypto`] provider for REVM precompiles.
///
/// Accelerates:
/// - `modexp` (EIP-198)
/// - `bn254_g1_add` / `bn254_g1_mul` (EIP-196)
/// - `secp256r1_verify_signature` (EIP-7951)
#[derive(Debug, Clone, Default)]
pub struct R0vmCrypto;

/// Installs the R0VM crypto provider globally.
///
/// Returns `true` if installed, `false` if a provider was already set.
#[inline]
pub fn install_r0vm_crypto() -> bool {
    install_crypto(R0vmCrypto)
}

impl Crypto for R0vmCrypto {
    #[inline]
    fn sha256(&self, input: &[u8]) -> [u8; 32] {
        use risc0_zkp::core::hash::sha::{Impl, Sha256};

        (*Impl::hash_bytes(input)).into()
    }

    #[inline]
    fn modexp(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileError> {
        let len = modulus.len();
        if len <= 32 {
            return Ok(modexp::modexp_generic(base, exp, modulus, field::unchecked::modmul_256));
        } else if len <= 48 {
            return Ok(modexp::modexp_generic(base, exp, modulus, field::unchecked::modmul_384));
        } else if len <= 512 {
            return Ok(modexp::modexp_generic(base, exp, modulus, field::unchecked::modmul_4096));
        }

        // Fallback for > 4096 bits
        DefaultCrypto.modexp(base, exp, modulus)
    }

    #[inline]
    fn bn254_g1_add(&self, p1: &[u8], p2: &[u8]) -> Result<[u8; 64], PrecompileError> {
        ec::bn254::add(p1, p2).ok_or(PrecompileError::Bn254AffineGFailedToCreate)
    }

    #[inline]
    fn bn254_g1_mul(&self, point: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileError> {
        ec::bn254::mul(point, scalar).ok_or(PrecompileError::Bn254AffineGFailedToCreate)
    }

    #[inline]
    fn secp256r1_verify_signature(&self, msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> bool {
        ec::p256::verify_signature(msg, sig, pk)
    }
}

/// Converts a BigUint to a fixed-size little-endian limb array.
fn biguint_to_limbs<const N: usize>(bn: &BigUint) -> [u32; N] {
    let mut arr = [0u32; N];
    let digits = bn.iter_u32_digits();
    assert!(digits.len() <= N, "BigUint too large for {N} limbs");
    for (dst, src) in arr.iter_mut().zip(digits) {
        *dst = src;
    }
    arr
}

/// Converts a big-endian byte slice into a fixed-size little-endian limb array.
fn be_bytes_to_limbs<const N: usize>(bytes: &[u8]) -> [u32; N] {
    assert!(bytes.len() <= N * LIMB_BYTES, "byte slice too large for {N} limbs");
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

/// Writes a little-endian limb array into a big-endian byte buffer.
fn limbs_into_be_bytes<const N: usize>(arr: &[u32; N], output: &mut [u8]) {
    assert_eq!(output.len(), N * LIMB_BYTES);
    for (dst, src) in output.rchunks_exact_mut(LIMB_BYTES).zip(arr.iter()) {
        dst.copy_from_slice(&src.to_be_bytes())
    }
}

/// Returns true if lhs < rhs as little-endian integers. Unlike Rust's `<`, compares from most
/// significant limb.
fn is_less<const N: usize>(lhs: &[u32; N], rhs: &[u32; N]) -> bool {
    for i in (0..N).rev() {
        if lhs[i] != rhs[i] {
            return lhs[i] < rhs[i];
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Bytes;
    use num_bigint::BigUint;
    use rstest::rstest;

    #[rstest]
    #[case::zero_exp("2a", "00", "03e8", 1)]
    #[case::modulus_one("2a", "0a", "01", 0)]
    #[case::zero_modulus("0a", "02", "00", 0)]
    #[case::base_gt_mod("0400", "01", "03e8", 24)]
    #[case::base_overflows_limbs(
        "010000000000000000000000000000000000000000000000000000000000000000",
        "01",
        "03",
        1
    )]
    fn modexp_edge(#[case] b: Bytes, #[case] e: Bytes, #[case] m: Bytes, #[case] expected: u64) {
        let result = R0vmCrypto.modexp(&b, &e, &m).unwrap();
        assert_eq!(BigUint::from_bytes_be(&result), BigUint::from(expected));
    }

    #[rstest]
    #[case::bits_256(
        "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe",
        "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
        "3342da48c80689c7249cfa42e35acf017d363d4440ea2f81e34e8f40b23d8ea6"
    )]
    #[case::bits_384(
        "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe",
        "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30",
        "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff",
        "947f3da78206e34313d74488225577ee4135d89717a7c2f831fa3f9fe992be73c69d443a4d983956925e5dbceb1b2264"
    )]
    #[case::bits_4096(
        "deadbeef",
        "cafebabe",
        "8000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000123456789abcdef",
        "28e7c3395b2e53ae75df885245d8d249b917062726e674fb6c2ab1c1c368cedfee2c86c654431577ed4c950ad1e2b1262bd68f86730d519f5a00d415c853c0ab0b052d667808a4e9fdc073a356522e42f9541016438e44f6451836ad51f885626cdb9390404752185022fef38480a0ba71bd07ddf6d023c8a4dc66fc296bd828f705ed65f460a7cf517f576daf8fa769d10abd6255b6a2d4abda9aae5c8ba2cab0f66560a9bcd3653a48fe379d4afd238d18fd951f83b1412910d52d1c49f4e043dc117a205f3587d4f6148cebc09e5e0f0139a59fee47bbe499a3863abdc27f89bc3ff5e85edd2134d1b25d9034f69129e034dc047ec64fc5f89343f79d0882c04daca53a5cb050c83f3495128756fe8e79c2cf288595b31f39361528e385b9d26626e846ae2041263fd314bdf26af2d3b8aa6c906b49cd5b515ad79d9c001eb4766b2edb5d3cc2c798554bdb5855c6f8ed32ba63a084f84cc91e800dd1a7dc74a9c3e3b881f1d3205622d9fb800fd1bb20386f27f4008d4ecb04ee975f5f14443b0ecef565cff6dc2f2aa205521bbd78f35099db646f078810a877db4435d61d66bd276fb3bcf9ac8f9b360b767e4dd16c7c605c0972a91edce445d4b695e191f76d825bd05626c65deab5a85d15368581f521cbc73552518911946e79709cccd939b239667e95e5bce6ef187bfa76a05c6ca1d0b356baf5b6821268c96f7a"
    )]
    fn modexp(#[case] b: Bytes, #[case] e: Bytes, #[case] m: Bytes, #[case] expected: Bytes) {
        assert_eq!(R0vmCrypto.modexp(&b, &e, &m), Ok(expected.to_vec()));
    }

    #[rstest]
    #[rustfmt::skip]
    #[case::not_on_curve("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001", "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002")]
    #[case::point_gt_p("30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4830644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd49", "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002")]
    #[case::identity_both("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")]
    #[case::identity_lhs("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002")]
    #[case::identity_rhs("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")]
    #[case::inverse("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "000000000000000000000000000000000000000000000000000000000000000130644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd45")]
    #[case::double("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002")]
    fn bn254_g1_add(#[case] p1: Bytes, #[case] p2: Bytes) {
        let expected = DefaultCrypto.bn254_g1_add(&p1, &p2);
        assert_eq!(R0vmCrypto.bn254_g1_add(&p1, &p2).ok(), expected.ok());
    }

    #[rstest]
    #[rustfmt::skip]
    #[case::not_on_curve("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001", "")]
    #[case::point_gt_p("30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd4830644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd49", "")]
    #[case::identity("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "0000000000000000000000000000000000000000000000000000000000000003")]
    #[case::zero_scalar("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "0000000000000000000000000000000000000000000000000000000000000000")]
    #[case::scalar_one("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "0000000000000000000000000000000000000000000000000000000000000001")]
    #[case::scalar_n("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001")]
    #[case::scalar_n_minus_1("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000")]
    #[case::scalar_gt_n("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000002")]
    #[case::ok("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002", "0000000000000000000000000000000000000000000000000000000000000003")]
    fn bn254_g1_mul(#[case] p: Bytes, #[case] scalar: Bytes) {
        let expected = DefaultCrypto.bn254_g1_mul(&p, &scalar);
        assert_eq!(R0vmCrypto.bn254_g1_mul(&p, &scalar).ok(), expected.ok());
    }

    // Wycheproof test vectors for P-256 ECDSA (IEEE P1363 signature format)
    mod wycheproof {
        use super::*;
        use serde::Deserialize;
        use sha2::Digest;

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Suite {
            test_groups: Vec<Group>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Group {
            public_key: PublicKey,
            tests: Vec<TestCase>,
        }

        #[derive(Deserialize)]
        struct PublicKey {
            uncompressed: Bytes,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TestCase {
            tc_id: u64,
            msg: Bytes,
            sig: Bytes,
            result: String,
        }

        #[test]
        fn ecdsa_secp256r1_sha256_p1363() {
            let suite: Suite = serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/wycheproof/ecdsa_secp256r1_sha256_p1363_test.json"
            )))
            .unwrap();

            for group in &suite.test_groups {
                // Strip the 04 prefix to get the raw 64-byte x||y public key
                let pk: &[u8; 64] = group.public_key.uncompressed[1..].try_into().unwrap();

                for tc in &group.tests {
                    let is_valid = tc.result == "valid";

                    // The API takes a fixed [u8; 64] sig; wrong-length sigs are invalid
                    let Ok(sig): Result<&[u8; 64], _> = tc.sig.as_ref().try_into() else {
                        assert!(!is_valid, "tcId {}: wrong-length sig marked valid", tc.tc_id);
                        continue;
                    };

                    let hash: [u8; 32] = sha2::Sha256::digest(&tc.msg).into();
                    let verified = R0vmCrypto.secp256r1_verify_signature(&hash, sig, pk);
                    assert_eq!(verified, is_valid, "tcId {}: expected {}", tc.tc_id, tc.result);
                }
            }
        }
    }

    // Test vectors from https://github.com/daimo-eth/p256-verifier/tree/master/test-vectors
    #[rstest]
    #[rustfmt::skip]
    #[case::ok_1("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e", true)]
    #[case::ok_2("3fec5769b5cf4e310a7d150508e82fb8e3eda1c2c94c61492d3bd8aea99e06c9e22466e928fdccef0de49e3503d2657d00494a00e764fd437bdafa05f5922b1fbbb77c6817ccf50748419477e843d5bac67e6a70e97dde5a57e0c983b777e1ad31a80482dadf89de6302b1988c82c29544c9c07bb910596158f6062517eb089a2f54c9a0f348752950094d3228d3b940258c75fe2a413cb70baa21dc2e352fc5", true)]
    #[case::ok_3("e775723953ead4a90411a02908fd1a629db584bc600664c609061f221ef6bf7c440066c8626b49daaa7bf2bcc0b74be4f7a1e3dcf0e869f1542fe821498cbf2de73ad398194129f635de4424a07ca715838aefe8fe69d1a391cfa70470795a80dd056866e6e1125aff94413921880c437c9e2570a28ced7267c8beef7e9b2d8d1547d76dfcf4bee592f5fefe10ddfb6aeb0991c5b9dbbee6ec80d11b17c0eb1a", true)]
    #[case::ok_4("b5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da3a81046703fccf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdcef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1", true)]
    #[case::ok_5("858b991cfd78f16537fe6d1f4afd10273384db08bdfc843562a22b0626766686f6aec8247599f40bfe01bec0e0ecf17b4319559022d4d9bf007fe929943004eb4866760dedf31b7c691f5ce665f8aae0bda895c23595c834fecc2390a5bcc203b04afcacbb4280713287a2d0c37e23f7513fab898f2c1fefa00ec09a924c335d9b629f1d4fb71901c3e59611afbfea354d101324e894c788d1c01f00b3c251b2", true)]
    #[case::fail_wrong_msg_1("3cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e", false)]
    #[case::fail_wrong_msg_2("afec5769b5cf4e310a7d150508e82fb8e3eda1c2c94c61492d3bd8aea99e06c9e22466e928fdccef0de49e3503d2657d00494a00e764fd437bdafa05f5922b1fbbb77c6817ccf50748419477e843d5bac67e6a70e97dde5a57e0c983b777e1ad31a80482dadf89de6302b1988c82c29544c9c07bb910596158f6062517eb089a2f54c9a0f348752950094d3228d3b940258c75fe2a413cb70baa21dc2e352fc5", false)]
    #[case::fail_wrong_msg_3("f775723953ead4a90411a02908fd1a629db584bc600664c609061f221ef6bf7c440066c8626b49daaa7bf2bcc0b74be4f7a1e3dcf0e869f1542fe821498cbf2de73ad398194129f635de4424a07ca715838aefe8fe69d1a391cfa70470795a80dd056866e6e1125aff94413921880c437c9e2570a28ced7267c8beef7e9b2d8d1547d76dfcf4bee592f5fefe10ddfb6aeb0991c5b9dbbee6ec80d11b17c0eb1a", false)]
    #[case::fail_wrong_msg_4("c5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da3a81046703fccf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdcef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1", false)]
    #[case::fail_wrong_msg_5("958b991cfd78f16537fe6d1f4afd10273384db08bdfc843562a22b0626766686f6aec8247599f40bfe01bec0e0ecf17b4319559022d4d9bf007fe929943004eb4866760dedf31b7c691f5ce665f8aae0bda895c23595c834fecc2390a5bcc203b04afcacbb4280713287a2d0c37e23f7513fab898f2c1fefa00ec09a924c335d9b629f1d4fb71901c3e59611afbfea354d101324e894c788d1c01f00b3c251b2", false)]
    #[case::fail_invalid_sig("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff4aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e", false)]
    #[case::fail_invalid_pubkey("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d6000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", false)]
    fn secp256r1_verify_signature(#[case] input: Bytes, #[case] expect: bool) {
        let msg = (&input[..32]).try_into().unwrap();
        let sig = (&input[32..96]).try_into().unwrap();
        let pk = (&input[96..160]).try_into().unwrap();
        assert_eq!(R0vmCrypto.secp256r1_verify_signature(&msg, &sig, &pk), expect);
    }
}
