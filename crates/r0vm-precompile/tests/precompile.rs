//! Wrapper edge-case and real-world happy-path tests.
//!
//! These test the byte-marshalling and dispatch behaviour of this crate's
//! functions. Primitive correctness (BN254 / P-256 / modexp arithmetic) is
//! covered by `risc0-crypto`'s own tests.
//!
//! Run on the zkVM target:
//! ```text
//! cargo risczero guest test --manifest-path crates/r0vm-precompile/Cargo.toml
//! ```
//!
//! The file is target-gated because the primitives panic on host (they call
//! `risc0-bigint2` syscalls that only the guest runtime provides). Keeping
//! tests empty on host lets `cargo test --workspace` run without exclusions.

#![cfg(all(target_os = "zkvm", target_vendor = "risc0"))]

use alloy_primitives::{address, hex, Bytes};
use num_bigint::BigUint;
use rstest::rstest;
use zeth_r0vm_precompile::{
    bn254_g1_add, bn254_g1_mul, modexp, secp256k1_ecrecover, secp256r1_verify,
};

#[rstest]
#[case::zero_exp("2a", "00", "03e8", 1)]
#[case::modulus_one("2a", "0a", "01", 0)]
#[case::zero_modulus("0a", "02", "00", 0)]
#[case::base_gt_mod("0400", "01", "03e8", 24)]
fn modexp_wrapper_edges(
    #[case] b: Bytes,
    #[case] e: Bytes,
    #[case] m: Bytes,
    #[case] expected: u64,
) {
    let result = modexp(&b, &e, &m).expect("wrapper should produce a result for these inputs");
    assert_eq!(BigUint::from_bytes_be(&result), BigUint::from(expected));
}

/// `base.len() > 32` at the 256-bit dispatch → `None` so the caller can fall
/// through to its default impl.
#[test]
fn modexp_base_overflows_limbs_returns_none() {
    let b = hex!("010000000000000000000000000000000000000000000000000000000000000000");
    let e = hex!("01");
    let m = hex!("03");
    assert_eq!(modexp(&b, &e, &m), None);
}

/// `modulus.len() > 512` exceeds the 4096-bit backend → `None`.
#[test]
fn modexp_modulus_over_4096_bits_returns_none() {
    let mut m = vec![0u8; 513];
    m[0] = 0x80; // ensure it's non-zero
    assert_eq!(modexp(&[0x02], &[0x02], &m), None);
}

/// Happy path per size dispatch. Each modulus is a real prime; expected bytes
/// were pre-computed against a known-good reference.
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
fn modexp_happy(#[case] b: Bytes, #[case] e: Bytes, #[case] m: Bytes, #[case] expected: Bytes) {
    assert_eq!(modexp(&b, &e, &m), Some(expected.to_vec()));
}

#[rstest]
#[rustfmt::skip]
// (0,0) + (0,0) — identity + identity → identity.
#[case::identity_both(
    "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    Some([0u8; 64]),
)]
// (0,0) + G → G. Asymmetric case exercises the LHS-identity branch.
#[case::identity_plus_g(
    "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
    Some(hex!("00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002")),
)]
// x = p (≥ p) → None.
#[case::x_eq_p(
    "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd470000000000000000000000000000000000000000000000000000000000000002",
    "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
    None,
)]
// (1, 1) ∉ y² = x³ + 3 → None.
#[case::not_on_curve(
    "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001",
    "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
    None,
)]
fn bn254_g1_add_wrapper_edges(
    #[case] p1: Bytes,
    #[case] p2: Bytes,
    #[case] expected: Option<[u8; 64]>,
) {
    assert_eq!(bn254_g1_add(&p1, &p2), expected);
}

/// Happy path: real on-chain call to the BN256Add precompile (0x06) at mainnet
/// tx `0xc8be04ad28b53466327e23b1f3273f9b90e94517df065b652c6d4bb3007963cb`.
/// Sourced via Dune `ethereum.traces`.
#[test]
fn bn254_g1_add_happy_real_world() {
    let p1 = hex!(
        "196013309322dc5ce901c20b2d8079da2a3d68cd8b98c943785298ead0b2ba4d"
        "1d08f60c613bfe68af690cecb6f2232c28b7eaa8fc582f76cd7112a164211890"
    );
    let p2 = hex!(
        "2c457dd809d9232408da339685a1523f079f0c89ea2d7fed45630b304f84d716"
        "10c6b33db48524934cff28b3f15378af480f7b6db12c22908cfb4767edca9dbf"
    );
    let expected = hex!(
        "05b1d7acfc557b31f7526bd463577a91749af5e394539970e744c01e17cf68d5"
        "27a561e9ff8babe2e667e6b28a7b6cac65e3b350922c813eb13a7ee24256a8a2"
    );
    assert_eq!(bn254_g1_add(&p1, &p2).unwrap(), expected);
}

#[rstest]
#[rustfmt::skip]
// (0,0) * k — identity * scalar → identity.
#[case::identity_times_scalar(
    "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000003",
    Some([0u8; 64]),
)]
// x = p (≥ p) → None.
#[case::x_eq_p(
    "30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd470000000000000000000000000000000000000000000000000000000000000002",
    "0000000000000000000000000000000000000000000000000000000000000003",
    None,
)]
// (1, 1) off-curve → None.
#[case::not_on_curve(
    "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001",
    "0000000000000000000000000000000000000000000000000000000000000003",
    None,
)]
// G * 0 → identity.
#[case::zero_scalar(
    "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
    "0000000000000000000000000000000000000000000000000000000000000000",
    Some([0u8; 64]),
)]
fn bn254_g1_mul_wrapper_edges(
    #[case] p: Bytes,
    #[case] scalar: Bytes,
    #[case] expected: Option<[u8; 64]>,
) {
    assert_eq!(bn254_g1_mul(&p, &scalar), expected);
}

/// Scalar reduction: `G * (n + 1) ≡ G * 1 = G`. A wrapper bug that passes the
/// raw scalar straight to the curve op would diverge for `scalar >= n`.
#[test]
fn bn254_g1_mul_scalar_gt_n_reduces() {
    let g = hex!(
        "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002"
    );
    // n + 1 for the BN254 scalar field.
    let n_plus_one = hex!("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000002");
    assert_eq!(bn254_g1_mul(&g, &n_plus_one).unwrap(), g);
}

/// Happy path: real on-chain call to the BN256ScalarMul precompile (0x07) at
/// mainnet tx `0xbd92117c6c4583915be479a694617c021dff583dd63aac6fbeada0de97afc320`.
/// Sourced via Dune `ethereum.traces`.
#[test]
fn bn254_g1_mul_happy_real_world() {
    let point = hex!(
        "20c36247444990e19dc5face987ad8453a86f4dfa7757f5874e0326852fd018a"
        "05c1f08f6764ecfa331b41613ec449651a1b75e6b582c6242965d091e5b9f9db"
    );
    let scalar = hex!("1382dc8a36d53180c2312461327c41d331c2f7366f35667ca4788765a79d0b96");
    let expected = hex!(
        "1ab33ab69e8b325080703793ae200d85f5676c2f83bd24c0f07aea6a2e82379b"
        "0528638d89e9dcf66972ac64ee509f59ce774a6246187fabca414536661415ca"
    );
    assert_eq!(bn254_g1_mul(&point, &scalar).unwrap(), expected);
}

#[rstest]
#[rustfmt::skip]
// daimo-eth `fail_invalid_sig` — r and s patched to all-ones (≥ n).
#[case::invalid_sig("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff4aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e")]
// daimo-eth `fail_invalid_pubkey` — pk mostly zero, (qx, qy) not on curve.
#[case::invalid_pubkey("4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d6000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")]
fn secp256r1_verify_wrapper_rejects(#[case] input: Bytes) {
    let msg = input[..32].try_into().unwrap();
    let sig = input[32..96].try_into().unwrap();
    let pk = input[96..160].try_into().unwrap();
    assert!(!secp256r1_verify(msg, sig, pk));
}

/// Happy path: daimo-eth P-256 verifier `ok_1` test vector.
#[test]
fn secp256r1_verify_happy() {
    let input = hex!(
        "4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4d"
        "a73bd4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac"
        "36dbcd03009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d60"
        "4aebd3099c618202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff3"
        "7618b065f9832de4ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e"
    );
    let msg = input[..32].try_into().unwrap();
    let sig = input[32..96].try_into().unwrap();
    let pk = input[96..160].try_into().unwrap();
    assert!(secp256r1_verify(msg, sig, pk));
}

/// Happy path: real on-chain call to the ecRecover precompile (0x01) at
/// mainnet tx `0x06b26ab19bddd92dbc1780fd191fedfec1384acd7e0a5ca20693e071d466b509`
/// (v = 28 → recid = 1). Sourced via Dune `ethereum.traces`.
#[test]
fn secp256k1_ecrecover_happy_real_world() {
    let msg = hex!("b7c3591439b59be2c5c2e75c0de9ab943f515b874636002c14a2bb4d4516d1be");
    let sig = hex!(
        "0658d6b7ad447b1d6bf3ac622354b19b506fa006a68e8d569e94c7193db95b32"
        "2eb3830ced56085dedb3775134fccf853a37fd331bad60a14eeeb53722f1f292"
    );
    let expected = address!("0x6969de628c1fee2c34ac8e80cea725f695556b94");
    assert_eq!(secp256k1_ecrecover(&sig, 1, &msg), Some(expected));
}

#[rstest]
#[rustfmt::skip]
// r = 0 → None (Signature::new rejects zero components).
#[case::r_zero(
    "00000000000000000000000000000000000000000000000000000000000000002eb3830ced56085dedb3775134fccf853a37fd331bad60a14eeeb53722f1f292",
    1,
)]
// s = 0 → None.
#[case::s_zero(
    "0658d6b7ad447b1d6bf3ac622354b19b506fa006a68e8d569e94c7193db95b320000000000000000000000000000000000000000000000000000000000000000",
    1,
)]
// r = n → None (`Fr::from_bigint` returns None for values ≥ modulus).
#[case::r_ge_n(
    "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd03641412eb3830ced56085dedb3775134fccf853a37fd331bad60a14eeeb53722f1f292",
    1,
)]
// s = n → None.
#[case::s_ge_n(
    "0658d6b7ad447b1d6bf3ac622354b19b506fa006a68e8d569e94c7193db95b32fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
    1,
)]
fn secp256k1_ecrecover_wrapper_rejects(#[case] sig_bytes: Bytes, #[case] recid: u8) {
    let msg = hex!("b7c3591439b59be2c5c2e75c0de9ab943f515b874636002c14a2bb4d4516d1be");
    let sig: &[u8; 64] = sig_bytes.as_ref().try_into().unwrap();
    assert_eq!(secp256k1_ecrecover(sig, recid, &msg), None);
}
