use hex;
use std::fmt;

/// Represents an Ethereum address (20 bytes).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Address(pub [u8; 20]);

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_checksum_display())
    }
}

impl Address {
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn to_checksum_display(&self) -> String {
        use sha3::{Digest, Keccak256};

        // 1. Get lowercase hex without 0x
        let addr_hex = hex::encode(self.0);

        // 2. Hash the lowercase hex string
        let mut hasher = Keccak256::new();
        hasher.update(addr_hex.as_bytes());
        let hash = hasher.finalize();

        // 3. Convert hash to hex string to easily check nibbles (0-f)
        let hash_hex = hex::encode(hash);

        // 4. Build checksummed string
        let mut checksummed = String::with_capacity(42);
        checksummed.push_str("0x");

        for (i, c) in addr_hex.chars().enumerate() {
            // Check the ith char of the hash (nibble)
            // If >= 8, uppercase the address char
            let hash_char = hash_hex.chars().nth(i).unwrap();
            let should_upper = hash_char >= '8';

            if should_upper {
                checksummed.push(c.to_ascii_uppercase());
            } else {
                checksummed.push(c);
            }
        }

        checksummed
    }
}

/// Represents a 32-byte Private Key.
/// derived with Debug that redacts the actual key for safety logs,
/// but Display shows it (assuming user intends to see it).
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateKey(pub [u8; 32]);

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrivateKey(REDACTED)")
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(self.0))
    }
}

impl PrivateKey {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eip55_checksum() {
        // Test Vector from EIP-55
        // 0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed
        let hex_raw = "5aaeb6053f3e94c9b9a09f33669435e7ef1beaed";
        let bytes = hex::decode(hex_raw).unwrap();
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        let addr = Address(arr);

        assert_eq!(
            addr.to_checksum_display(),
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        );
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_eip55_invariant(bytes in proptest::array::uniform20(0u8..255)) {
            let addr = Address(bytes);
            let checksummed = addr.to_checksum_display();

            // 1. Length Check
            assert_eq!(checksummed.len(), 42);
            assert!(checksummed.starts_with("0x"));

            // 2. Case Invariance
            assert_eq!(
                checksummed.to_lowercase(),
                format!("0x{}", hex::encode(bytes))
            );

            // 3. Logic Check (Self-Consistent)
            let lower_hex = hex::encode(bytes);
            let mut hasher = sha3::Keccak256::new();
            use sha3::Digest;
            hasher.update(lower_hex.as_bytes());
            let hash = hasher.finalize();
            let hash_hex = hex::encode(hash);

            for (i, c) in lower_hex.chars().enumerate() {
                let hash_nibble = hash_hex.chars().nth(i).unwrap();
                let output_char = checksummed.chars().nth(i + 2).unwrap(); // +2 for 0x

                if hash_nibble >= '8' {
                    assert!(output_char.is_uppercase() || !c.is_alphabetic(),
                        "Char at {} should be uppercase. Hash: {}, Addr: {}", i, hash_nibble, c);
                } else {
                    assert!(output_char.is_lowercase() || !c.is_alphabetic(),
                        "Char at {} should be lowercase. Hash: {}, Addr: {}", i, hash_nibble, c);
                }
            }
        }
    }
}
