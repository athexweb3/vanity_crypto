use bs58;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use vanity_core::{Address, PrivateKey, VanityGenerator};

pub struct SolanaVanityGenerator {
    prefix: String,
    suffix: String,
    case_sensitive: bool,
}

impl SolanaVanityGenerator {
    pub fn new(prefix: &str, suffix: &str, case_sensitive: bool) -> Self {
        Self {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
            case_sensitive,
        }
    }

    pub fn search(&self, found_flag: Option<Arc<AtomicU64>>) -> (PrivateKey, Address) {
        let mut csprng = OsRng;

        // Convert input prefix/suffix ref for loop to avoid borrow checker issues
        let p_prefix = &self.prefix;
        let p_suffix = &self.suffix;
        let p_case = self.case_sensitive;

        loop {
            // Update attempts counter if provided
            if let Some(ref attempts) = found_flag {
                // Relaxed ordering is sufficient for stats
                attempts.fetch_add(1, Ordering::Relaxed);
            }

            // 1. Generate Keypair
            // Workaround for `SigningKey::generate` feature issues: generate 32 random bytes
            let mut secret_bytes = [0u8; 32];
            csprng.fill_bytes(&mut secret_bytes);
            let signing_key = SigningKey::from_bytes(&secret_bytes);
            let verifying_key: VerifyingKey = signing_key.verifying_key();
            let address = bs58::encode(verifying_key.as_bytes()).into_string();

            // 2. Check Match
            let mut is_match = true;

            if !p_prefix.is_empty() {
                if p_case {
                    if !address.starts_with(p_prefix) {
                        is_match = false;
                    }
                } else {
                    // Optimized lowercase check for prefix
                    if address.len() < p_prefix.len()
                        || !address[..p_prefix.len()].eq_ignore_ascii_case(p_prefix)
                    {
                        is_match = false;
                    }
                }
            }

            if is_match && !p_suffix.is_empty() {
                if p_case {
                    if !address.ends_with(p_suffix) {
                        is_match = false;
                    }
                } else {
                    // Optimized lowercase check for suffix
                    if address.len() < p_suffix.len()
                        || !address[address.len() - p_suffix.len()..].eq_ignore_ascii_case(p_suffix)
                    {
                        is_match = false;
                    }
                }
            }

            if is_match {
                // Get 32-byte seed
                let secret_bytes = signing_key.to_bytes();

                // Construct standard 64-byte Solana private key format (seed + pubkey)
                let mut full_keypair = [0u8; 64];
                full_keypair[..32].copy_from_slice(&secret_bytes);
                full_keypair[32..].copy_from_slice(verifying_key.as_bytes());

                return (PrivateKey::Solana(full_keypair), Address::Solana(address));
            }
        }
    }
}

impl VanityGenerator for SolanaVanityGenerator {
    fn generate(&self) -> (PrivateKey, Address) {
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        csprng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let address = bs58::encode(verifying_key.as_bytes()).into_string();

        let mut full_keypair = [0u8; 64];
        full_keypair[..32].copy_from_slice(&secret_bytes);
        full_keypair[32..].copy_from_slice(verifying_key.as_bytes());

        (PrivateKey::Solana(full_keypair), Address::Solana(address))
    }
}

/// Generates a random Solana keypair without checking for vanity patterns.
/// Returns raw bytes of the seed and the address string.
pub fn generate_random_address() -> (Vec<u8>, String) {
    let mut csprng = OsRng;
    let mut secret_bytes = [0u8; 32];
    csprng.fill_bytes(&mut secret_bytes);
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    let address = bs58::encode(verifying_key.as_bytes()).into_string();
    (signing_key.to_bytes().to_vec(), address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_address_format() {
        let (_, address) = generate_random_address();
        assert!(address.len() >= 32 && address.len() <= 44);
        let decoded = bs58::decode(&address).into_vec();
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().len(), 32);
    }

    #[test]
    fn test_vanity_search() {
        let gen = SolanaVanityGenerator::new("A", "", true);
        let (_, addr) = gen.search(None);

        match addr {
            Address::Solana(s) => assert!(s.starts_with("A")),
            _ => panic!("Wrong address type"),
        }
    }
}
