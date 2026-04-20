//! EIP-198 modular exponentiation.

use alloc::vec;
use alloc::vec::Vec;
use risc0_crypto::{
    modexp::{self, BitAccess, ModMul},
    BigInt,
};

/// Computes `base^exp mod modulus`.
///
/// Picks a 256-, 384-, or 4096-bit `modmul` backend based on `modulus.len()`:
/// - `0` → empty output
/// - `1..=32` → 256-bit
/// - `33..=48` → 384-bit
/// - `49..=512` → 4096-bit
///
/// Returns `None` when the inputs don't fit the chosen limb width (e.g.
/// `base.len() > 512`) or when the modulus is wider than 4096 bits. Consumers
/// fall through to their own default implementation on `None`.
///
/// A zero modulus yields an output of length `modulus.len()` filled with zeros.
#[inline]
pub fn modexp(base: &[u8], exp: &[u8], modulus: &[u8]) -> Option<Vec<u8>> {
    match modulus.len() {
        0 => Some(Vec::new()),
        1..=32 => modexp_n::<8>(base, exp, modulus),
        33..=48 => modexp_n::<12>(base, exp, modulus),
        49..=512 => modexp_n::<128>(base, exp, modulus),
        _ => None,
    }
}

fn modexp_n<const N: usize>(base: &[u8], exp: &[u8], modulus: &[u8]) -> Option<Vec<u8>>
where
    [u32; N]: ModMul,
{
    // `BigInt::from_be_bytes` panics on overflow; return None so the caller
    // can fall through to its default implementation.
    if base.len() > N * 4 || modulus.len() > N * 4 {
        return None;
    }
    let base_bi = BigInt::<N>::from_be_bytes(base);
    let mod_bi = BigInt::<N>::from_be_bytes(modulus);

    // EIP-198: zero modulus yields a zero-filled output of modulus length.
    if mod_bi.is_zero() {
        return Some(vec![0u8; modulus.len()]);
    }

    let result = modexp::modexp::<N>(&base_bi, &ByteExp(exp), &mod_bi);

    let mut full = vec![0u8; N * 4];
    result.write_be_bytes(&mut full);
    Some(full[N * 4 - modulus.len()..].to_vec())
}

/// `BitAccess` adapter over a big-endian byte slice, mirroring revm's default
/// modexp behaviour for arbitrarily-long exponents.
struct ByteExp<'a>(&'a [u8]);

impl BitAccess for ByteExp<'_> {
    #[inline]
    fn bits(&self) -> usize {
        self.0
            .iter()
            .position(|&b| b != 0)
            .map_or(0, |i| (self.0.len() - i) * 8 - self.0[i].leading_zeros() as usize)
    }

    #[inline]
    fn bit(&self, i: usize) -> bool {
        let byte_offset = i / 8;
        if byte_offset >= self.0.len() {
            return false;
        }
        self.0[self.0.len() - 1 - byte_offset] & (1 << (i % 8)) != 0
    }
}
