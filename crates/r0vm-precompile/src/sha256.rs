/// SHA-256 — wraps `risc0-zkp`'s zkVM-accelerated implementation.
#[inline(always)]
pub fn sha256(input: &[u8]) -> [u8; 32] {
    use risc0_zkp::core::hash::sha::{Impl, Sha256};
    (*Impl::hash_bytes(input)).into()
}
