use hex;
use std::fmt;

/// Represents a blockchain address with chain-specific strict types.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Address {
    Ethereum([u8; 20]),
    Bitcoin(String), // Compliant with BIP-58 (Base58Check), BIP-173 (Bech32), or BIP-350 (Bech32m)
    Solana(String),  // Base58 encoded string
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Ethereum(bytes) => write!(f, "Ethereum(0x{})", hex::encode(bytes)),
            Address::Bitcoin(addr) => write!(f, "Bitcoin({})", addr),
            Address::Solana(addr) => write!(f, "Solana({})", addr),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Ethereum(bytes) => {
                // Implement EIP-55 checksum validation
                use sha3::{Digest, Keccak256};
                let addr_hex = hex::encode(bytes);
                let mut hasher = Keccak256::new();
                hasher.update(addr_hex.as_bytes());
                let checksum_hash = hasher.finalize();
                let checksum_hex = hex::encode(checksum_hash);

                let mut checksummed = String::with_capacity(42);
                checksummed.push_str("0x");
                for (c, hash_char) in addr_hex.chars().zip(checksum_hex.chars()) {
                    if hash_char >= '8' {
                        checksummed.push(c.to_ascii_uppercase());
                    } else {
                        checksummed.push(c);
                    }
                }
                write!(f, "{}", checksummed)
            }
            Address::Bitcoin(addr) => {
                // Bitcoin addresses are pre-validated upon creation (via bitcoin crate)
                // Legacy: Case-sensitive Base58Check (BIP-58)
                // SegWit: Case-insensitive Bech32 (BIP-173)
                // Taproot: Case-insensitive Bech32m (BIP-350)
                write!(f, "{}", addr)
            }
            Address::Solana(addr) => write!(f, "{}", addr),
        }
    }
}

impl Address {
    /// Returns a simple hex string for pattern matching without checksum calculation
    /// This is used in hot loops where checksumming is unnecessary overhead
    pub fn to_match_string(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Address::Ethereum(bytes) => std::borrow::Cow::Owned(hex::encode(bytes)),
            Address::Bitcoin(s) => std::borrow::Cow::Borrowed(s),
            Address::Solana(s) => std::borrow::Cow::Borrowed(s),
        }
    }
}

/// Represents a 32-byte Private Key.
/// derived with Debug that redacts the actual key for safety logs,
/// but Display shows it (assuming user intends to see it).
#[derive(Clone, PartialEq, Eq)]
pub enum PrivateKey {
    Ethereum([u8; 32]),
    Bitcoin(String),
    Solana([u8; 64]), // Solana keypairs are 64 bytes (32 private + 32 public)
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrivateKey(REDACTED)")
    }
}

impl fmt::Display for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrivateKey::Ethereum(bytes) => write!(f, "0x{}", hex::encode(bytes)),
            PrivateKey::Bitcoin(wif) => write!(f, "{}", wif),
            PrivateKey::Solana(bytes) => write!(f, "{}", bs58::encode(&bytes[..]).into_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_display() {
        // Ethereum
        let bytes = hex::decode("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed").unwrap();
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        let addr = Address::Ethereum(arr);
        assert_eq!(
            format!("{}", addr),
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        );

        // Bitcoin (Bech32)
        let btc_addr = Address::Bitcoin("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string());
        assert_eq!(
            format!("{}", btc_addr),
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        );
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_eip55_invariant(bytes in proptest::array::uniform20(0u8..255)) {
            // EIP-55 checksum invariant: formatting and re-parsing should preserve bytes
            let address = Address::Ethereum(bytes);
            let formatted = format!("{}", address);

            // Verify it starts with 0x
            assert!(formatted.starts_with("0x"));

            // Verify the hex is exactly 40 characters (20 bytes)
            assert_eq!(formatted.len(), 42); // "0x" + 40 hex chars

            // Verify the checksumming is consistent
            let formatted_again = format!("{}", address);
            assert_eq!(formatted, formatted_again);

            // Verify EIP-55 checksum is correct:
            // For each hex character, if the corresponding hash character >= '8', it should be uppercase
            let addr_hex = hex::encode(bytes);
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(addr_hex.as_bytes());
            let checksum_hash = hasher.finalize();
            let checksum_hex = hex::encode(checksum_hash);

            // Skip "0x" prefix when checking
            let formatted_without_prefix = &formatted[2..];
            for (i, ((addr_char, formatted_char), hash_char)) in addr_hex
                .chars()
                .zip(formatted_without_prefix.chars())
                .zip(checksum_hex.chars())
                .enumerate()
            {
                if hash_char >= '8' {
                    assert_eq!(
                        formatted_char,
                        addr_char.to_ascii_uppercase(),
                        "Checksum mismatch at position {}: expected uppercase for hash_char={}",
                        i,
                        hash_char
                    );
                } else {
                    assert_eq!(
                        formatted_char,
                        addr_char.to_ascii_lowercase(),
                        "Checksum mismatch at position {}: expected lowercase for hash_char={}",
                        i,
                        hash_char
                    );
                }
            }
        }
    }

    #[test]
    fn test_solana_key_format() {
        // Test that Solana PrivateKey formats to full 64-byte Base58 string
        let mut keypair_bytes = [1u8; 64]; // Fill with dummy data
                                           // Make the first 32 bytes (seed) different from the last 32 (pubkey)
        for (i, byte) in keypair_bytes.iter_mut().enumerate().take(32) {
            *byte = i as u8;
        }
        for (i, byte) in keypair_bytes.iter_mut().enumerate().skip(32) {
            *byte = (64 - i) as u8;
        }

        let pk = PrivateKey::Solana(keypair_bytes);
        let display_str = format!("{}", pk);

        // Verify length
        // 64 bytes in Base58 is approx 87-88 chars
        assert!(
            display_str.len() > 80,
            "Solana private key string too short: {}",
            display_str.len()
        );

        // Verify round trip (decode base58 -> 64 bytes)
        let decoded = bs58::decode(&display_str).into_vec().unwrap();
        assert_eq!(decoded.len(), 64, "Decoded Base58 length mismatch");
        assert_eq!(decoded, keypair_bytes.to_vec(), "Decoded content mismatch");
    }
}
