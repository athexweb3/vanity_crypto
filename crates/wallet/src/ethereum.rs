use hex;
use k256::ecdsa::{SigningKey, VerifyingKey};

use rayon::prelude::*;
use sha3::{Digest, Keccak256};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use vanity_core::{Address, PrivateKey, VanityGenerator};

pub struct EthereumVanityGenerator {
    prefix: String,
    suffix: String,
    case_sensitive: bool,
}

impl EthereumVanityGenerator {
    pub fn new(prefix: &str, suffix: &str, case_sensitive: bool) -> Self {
        // Validation happens before this, usually in CLI parsing
        Self {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
            case_sensitive,
        }
    }

    /// Checks if a given address string matches the criteria
    #[inline(always)]
    fn matches(&self, addr_str: &str) -> bool {
        // addr_str is typically lower case hex without 0x if we use hex::encode
        // But let's assume we compare normalized strings.

        let target = if self.case_sensitive {
            addr_str.to_string()
        } else {
            addr_str.to_lowercase()
        };

        let p_match = if !self.prefix.is_empty() {
            let p = if self.case_sensitive {
                self.prefix.clone()
            } else {
                self.prefix.to_lowercase()
            };
            // Strip 0x from prefix if present
            let p = p.trim_start_matches("0x");
            target.starts_with(p)
        } else {
            true
        };

        let s_match = if !self.suffix.is_empty() {
            let s = if self.case_sensitive {
                self.suffix.clone()
            } else {
                self.suffix.to_lowercase()
            };
            target.ends_with(&s)
        } else {
            true
        };

        p_match && s_match
    }

    /// Run the search using multiple threads.
    /// This uses rayon to parallelize.
    /// Takes an optional progress counter to track attempts.
    pub fn search(
        &self,
        progress: Option<Arc<std::sync::atomic::AtomicU64>>,
    ) -> (PrivateKey, Address) {
        // Rayon's infinite iterator
        rayon::iter::repeat(())
            .map(|_| {
                // Determine if we should check progress or just execute
                // We check progress per batch or just allow relaxed atomic increment

                if let Some(p) = &progress {
                    p.fetch_add(1, Ordering::Relaxed);
                }

                // Thread-local generator
                let bytes: [u8; 32] = rand::random();

                let signing_key =
                    SigningKey::from_bytes(&bytes.into()).expect("valid key from random");
                let verifying_key = VerifyingKey::from(&signing_key);
                let encoded_point = verifying_key.to_encoded_point(false);
                let encoded = encoded_point.as_bytes();
                // encoded[0] is 0x04 for uncompressed
                // we want the rest
                let public_key_bytes = &encoded[1..];

                let mut hasher = Keccak256::new();
                hasher.update(public_key_bytes);
                let hash = hasher.finalize();

                let mut address_bytes = [0u8; 20];
                address_bytes.copy_from_slice(&hash[12..]);

                let address = Address(address_bytes);
                let pk = PrivateKey(bytes);

                (pk, address)
            })
            .find_any(|(_pk, addr)| {
                // hex::encode returns lowercase
                let s = hex::encode(addr.as_bytes());
                self.matches(&s)
            })
            .expect("Infinite iterator execution")
    }
}

impl VanityGenerator for EthereumVanityGenerator {
    fn generate(&self) -> (PrivateKey, Address) {
        self.search(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_derivation_vector_1() {
        // Known vector:
        // PK: 1
        // Addr: 0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf

        let pk_bytes = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ];

        // We need to manually invoke the inner logic or expose a helper
        // Since search uses random, we need a deterministic derivation function for testing.
        // Let's copy the derivation logic here to test it specifically.

        let signing_key = SigningKey::from_bytes(&pk_bytes.into()).expect("valid key");
        let verifying_key = VerifyingKey::from(&signing_key);
        let encoded_point = verifying_key.to_encoded_point(false);
        let encoded = encoded_point.as_bytes();
        let public_key_bytes = &encoded[1..];

        let mut hasher = Keccak256::new();
        hasher.update(public_key_bytes);
        let hash = hasher.finalize();

        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash[12..]);

        let address_str = hex::encode(address_bytes);
        assert_eq!(
            address_str.to_lowercase(),
            "7e5f4552091a69125d5dfcb7b8c2659029395bdf".to_lowercase()
        );
    }

    #[test]
    fn test_address_derivation_hardhat_0() {
        // Known vector: Hardhat Account 0
        // PK: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
        // Addr: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

        let pk_hex = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let pk_bytes = hex::decode(pk_hex).expect("decode");

        let signing_key = SigningKey::from_bytes(pk_bytes.as_slice().into()).expect("valid key");
        let verifying_key = VerifyingKey::from(&signing_key);
        let encoded_point = verifying_key.to_encoded_point(false);
        let encoded = encoded_point.as_bytes();
        let public_key_bytes = &encoded[1..];

        let mut hasher = Keccak256::new();
        hasher.update(public_key_bytes);
        let hash = hasher.finalize();

        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&hash[12..]);

        let address_str = hex::encode(address_bytes);
        assert_eq!(
            address_str.to_lowercase(),
            "f39fd6e51aad88f6f4ce6ab8827279cfffb92266".to_lowercase()
        );
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_pk_length_invariant(bytes in proptest::array::uniform32(0u8..255)) {
            // Ensure any 32-byte array (valid scalar) produces a valid 20-byte address
            // Note: Not all 32-byte arrays are valid secp256k1 scalars (curve order),
            // but k256 handles checks. We mostly want to ensure NO PANICS on random inputs.

            if let Ok(signing_key) = SigningKey::from_bytes(&bytes.into()) {
                let verifying_key = VerifyingKey::from(&signing_key);
                let encoded_point = verifying_key.to_encoded_point(false);
                let public_key_bytes = &encoded_point.as_bytes()[1..];

                let mut hasher = Keccak256::new();
                hasher.update(public_key_bytes);
                let hash = hasher.finalize();

                // Invariant: Hash is always 32 bytes
                assert_eq!(hash.len(), 32);

                // Invariant: Address is always last 20 bytes
                let address_bytes = &hash[12..];
                assert_eq!(address_bytes.len(), 20);
            }
        }
    }
}
