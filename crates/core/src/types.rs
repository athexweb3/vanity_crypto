use hex;
use std::fmt;

/// Represents a blockchain address with chain-specific strict types.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Address {
    Ethereum([u8; 20]),
    Bitcoin(String), // Compliant with BIP-58 (Base58Check), BIP-173 (Bech32), or BIP-350 (Bech32m)
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Ethereum(bytes) => write!(f, "Ethereum(0x{})", hex::encode(bytes)),
            Address::Bitcoin(addr) => write!(f, "Bitcoin({})", addr),
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
                for (i, c) in addr_hex.chars().enumerate() {
                    let hash_char = checksum_hex.chars().nth(i).unwrap();
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
        }
    }
}

impl Address {
    /// Returns the string representation useful for pattern matching
    /// For Ethereum, returns lowercase hex (no 0x prefix) for efficient searching?
    /// Or returns full string?
    /// The matching logic usually strips prefixes/suffixes.
    /// Returns standard string representation
    pub fn as_str(&self) -> std::borrow::Cow<'_, str> {
        match self {
            // Use raw bytes for internal operations
            // BUT, `as_str` is usually expected to be "the address string".
            Address::Ethereum(_) => std::borrow::Cow::Owned(self.to_string()),
            Address::Bitcoin(s) => std::borrow::Cow::Borrowed(s),
        }
    }
}

/// Represents a 32-byte Private Key.
/// derived with Debug that redacts the actual key for safety logs,
/// but Display shows it (assuming user intends to see it).
/// Represents a Private Key.
/// derived with Debug that redacts the actual key for safety logs,
/// but Display shows it (assuming user intends to see it).
#[derive(Clone, PartialEq, Eq)]
pub enum PrivateKey {
    Ethereum([u8; 32]),
    Bitcoin(String),
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
        }
    }
}

impl PrivateKey {
    /// Returns reference to Ethereum bytes if applicable
    pub fn as_ethereum_bytes(&self) -> Option<&[u8; 32]> {
        match self {
            PrivateKey::Ethereum(bytes) => Some(bytes),
            _ => None,
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

        // Bitcoin
        let btc_str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let addr_btc = Address::Bitcoin(btc_str.to_string());
        assert_eq!(format!("{}", addr_btc), btc_str);
    }
}
