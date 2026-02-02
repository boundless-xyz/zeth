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

//! R0VM-optimized crypto provider for REVM precompiles.

mod modexp;

use reth_evm::revm::precompile::{Crypto, DefaultCrypto, PrecompileError, install_crypto};

use self::modexp::{modexp_generic, modmul_256, modmul_384, modmul_4096};

/// R0VM-optimized crypto provider for REVM precompiles.
#[derive(Debug, Clone, Default)]
pub struct R0vmCrypto;

/// Installs the custom R0VM crypto provider globally.
#[inline]
pub fn install_r0vm_crypto() -> bool {
    install_crypto(R0vmCrypto)
}

impl Crypto for R0vmCrypto {
    #[inline]
    fn modexp(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileError> {
        let len = modulus.len();
        if len <= 32 {
            return Ok(modexp_generic(base, exp, modulus, modmul_256));
        } else if len <= 48 {
            return Ok(modexp_generic(base, exp, modulus, modmul_384));
        } else if len <= 512 {
            return Ok(modexp_generic(base, exp, modulus, modmul_4096));
        }

        // Fallback for > 4096 bits
        DefaultCrypto.modexp(base, exp, modulus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;
    use num_bigint::BigUint;

    #[test]
    fn modexp_basic() {
        // 3^256 mod 1000 = 521 (multi-byte exponent)
        let base = vec![3u8];
        let exp = vec![0x01, 0x00]; // 256
        let modulus = vec![0x03, 0xe8]; // 1000

        let result = R0vmCrypto.modexp(&base, &exp, &modulus).unwrap();
        assert_eq!(BigUint::from_bytes_be(&result), BigUint::from(521u32));
    }

    #[test]
    fn modexp_zero_modulus() {
        let base = vec![10u8];
        let exp = vec![2u8];
        let modulus = vec![0u8; 32];

        let result = R0vmCrypto.modexp(&base, &exp, &modulus).unwrap();
        assert_eq!(BigUint::from_bytes_be(&result), BigUint::ZERO);
    }

    #[test]
    fn modexp_zero_exp() {
        // x^0 mod m = 1 for any x and m > 1
        let base = vec![42u8];
        let exp = vec![0u8];
        let modulus = vec![0x03, 0xe8]; // 1000

        let result = R0vmCrypto.modexp(&base, &exp, &modulus).unwrap();
        assert_eq!(BigUint::from_bytes_be(&result), BigUint::from(1u32));
    }

    #[test]
    fn modexp_large_base() {
        // Base exceeds N*32 bits, gets reduced mod m first
        // base = 2^256, exp = 1, mod = 3 => (2^256 mod 3)^1 mod 3 = 1
        let mut base = vec![0u8; 33];
        base[0] = 1;
        let exp = vec![1u8];
        let modulus = vec![3u8];

        let result = R0vmCrypto.modexp(&base, &exp, &modulus).unwrap();
        assert_eq!(BigUint::from_bytes_be(&result), BigUint::from(1u32));
    }

    #[test]
    fn modexp_modulus_one() {
        // x^n mod 1 = 0 for any x, n
        let base = vec![42u8];
        let exp = vec![10u8];
        let modulus = vec![1u8];

        let result = R0vmCrypto.modexp(&base, &exp, &modulus).unwrap();
        assert_eq!(BigUint::from_bytes_be(&result), BigUint::ZERO);
    }

    #[test]
    fn modexp_base_greater_than_modulus() {
        // base = secp256k1 prime, modulus = 1000, exp = 3
        let base = hex!("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        let exp = vec![1u8];
        let modulus = vec![0x03, 0xe8]; // 1000

        // secp256k1_prime mod 1000 = 247
        let result = R0vmCrypto.modexp(&base, &exp, &modulus).unwrap();
        assert_eq!(BigUint::from_bytes_be(&result), BigUint::from(663u32));
    }

    #[test]
    fn modexp_256_arbitrary() {
        // 256-bit values with secp256k1 prime modulus
        let base = hex!("deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe");
        let exp = hex!("0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20");
        let modulus = hex!("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        let expected =
            hex!("3342da48c80689c7249cfa42e35acf017d363d4440ea2f81e34e8f40b23d8ea6").to_vec();

        assert_eq!(R0vmCrypto.modexp(&base, &exp, &modulus), Ok(expected));
    }

    #[test]
    fn modexp_384_arbitrary() {
        // 384-bit values with P-384 prime modulus
        let base = hex!(
            "deadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabedeadbeefcafebabe"
        );
        let exp = hex!(
            "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30"
        );
        let modulus = hex!(
            "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff"
        );
        let expected = hex!("947f3da78206e34313d74488225577ee4135d89717a7c2f831fa3f9fe992be73c69d443a4d983956925e5dbceb1b2264").to_vec();

        assert_eq!(R0vmCrypto.modexp(&base, &exp, &modulus), Ok(expected));
    }

    #[test]
    fn modexp_4096_arbitrary() {
        // 4096-bit modulus: 2^4095 + 0x0123456789abcdef
        let base = hex!("deadbeef");
        let exp = hex!("cafebabe");
        let mut modulus = vec![0u8; 512];
        modulus[0] = 0x80;
        modulus[504..512].copy_from_slice(&hex!("0123456789abcdef"));
        let expected = hex!(
            "28e7c3395b2e53ae75df885245d8d249b917062726e674fb6c2ab1c1c368cedfee2c86c654431577ed4c950ad1e2b1262bd68f86730d519f5a00d415c853c0ab0b052d667808a4e9fdc073a356522e42f9541016438e44f6451836ad51f885626cdb9390404752185022fef38480a0ba71bd07ddf6d023c8a4dc66fc296bd828f705ed65f460a7cf517f576daf8fa769d10abd6255b6a2d4abda9aae5c8ba2cab0f66560a9bcd3653a48fe379d4afd238d18fd951f83b1412910d52d1c49f4e043dc117a205f3587d4f6148cebc09e5e0f0139a59fee47bbe499a3863abdc27f89bc3ff5e85edd2134d1b25d9034f69129e034dc047ec64fc5f89343f79d0882c04daca53a5cb050c83f3495128756fe8e79c2cf288595b31f39361528e385b9d26626e846ae2041263fd314bdf26af2d3b8aa6c906b49cd5b515ad79d9c001eb4766b2edb5d3cc2c798554bdb5855c6f8ed32ba63a084f84cc91e800dd1a7dc74a9c3e3b881f1d3205622d9fb800fd1bb20386f27f4008d4ecb04ee975f5f14443b0ecef565cff6dc2f2aa205521bbd78f35099db646f078810a877db4435d61d66bd276fb3bcf9ac8f9b360b767e4dd16c7c605c0972a91edce445d4b695e191f76d825bd05626c65deab5a85d15368581f521cbc73552518911946e79709cccd939b239667e95e5bce6ef187bfa76a05c6ca1d0b356baf5b6821268c96f7a"
        ).to_vec();

        assert_eq!(R0vmCrypto.modexp(&base, &exp, &modulus), Ok(expected));
    }
}
