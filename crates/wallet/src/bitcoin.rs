use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey, XOnlyPublicKey};
use rayon::prelude::*;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use vanity_core::{Address as CoreAddress, PrivateKey, VanityGenerator};

#[derive(Clone, Copy, Debug)]
pub enum BitcoinAddressType {
    Legacy,
    SegWit,
    Taproot,
}

pub struct BitcoinVanityGenerator {
    prefix: String,
    suffix: String,
    case_sensitive: bool,
    network: Network,
    addr_type: BitcoinAddressType,
    secp: Secp256k1<All>,
}

impl BitcoinVanityGenerator {
    pub fn new(
        prefix: &str,
        suffix: &str,
        case_sensitive: bool,
        network: Network,
        addr_type: BitcoinAddressType,
    ) -> Self {
        // Preprocess prefix and suffix based on case sensitivity
        let (prefix_processed, suffix_processed) = if case_sensitive {
            (prefix.to_string(), suffix.to_string())
        } else {
            (prefix.to_lowercase(), suffix.to_lowercase())
        };

        Self {
            prefix: prefix_processed,
            suffix: suffix_processed,
            case_sensitive,
            network,
            addr_type,
            secp: Secp256k1::new(),
        }
    }

    #[inline(always)]
    fn matches(&self, addr_str: &str) -> bool {
        // Prefix and suffix are already preprocessed in new()
        if self.case_sensitive {
            let p_match = self.prefix.is_empty() || addr_str.starts_with(&self.prefix);
            if !p_match {
                return false;
            }
            self.suffix.is_empty() || addr_str.ends_with(&self.suffix)
        } else {
            let lower_addr = addr_str.to_lowercase();
            let p_match = self.prefix.is_empty() || lower_addr.starts_with(&self.prefix);
            if !p_match {
                return false;
            }
            self.suffix.is_empty() || lower_addr.ends_with(&self.suffix)
        }
    }

    // New helper function
    pub fn derive_address(
        secp: &Secp256k1<All>,
        network: Network,
        addr_type: BitcoinAddressType,
        secret_key: bitcoin::secp256k1::SecretKey,
        public_key: bitcoin::secp256k1::PublicKey,
    ) -> String {
        match addr_type {
            BitcoinAddressType::Legacy => {
                let pubkey = PublicKey::from(CompressedPublicKey(public_key));
                Address::p2pkh(pubkey, network).to_string()
            }
            BitcoinAddressType::SegWit => {
                let pubkey = CompressedPublicKey(public_key);
                Address::p2wpkh(&pubkey, network).to_string()
            }
            BitcoinAddressType::Taproot => {
                let keypair = bitcoin::secp256k1::Keypair::from_secret_key(secp, &secret_key);
                let (x_only, _parity) = XOnlyPublicKey::from_keypair(&keypair);
                Address::p2tr(secp, x_only, None, network).to_string()
            }
        }
    }

    pub fn search(
        &self,
        progress: Option<Arc<std::sync::atomic::AtomicU64>>,
    ) -> (PrivateKey, CoreAddress) {
        // Use cached Secp256k1 context (thread-safe)

        rayon::iter::repeat(())
            .map(|_| {
                if let Some(p) = &progress {
                    p.fetch_add(1, Ordering::Relaxed);
                }

                // Generate key using cached context
                let (secret_key, public_key) = self.secp.generate_keypair(&mut rand::thread_rng());

                // Create bitcoin::PrivateKey for WIF
                let bitcoin_private_key = bitcoin::PrivateKey::new(secret_key, self.network);
                let wif = bitcoin_private_key.to_string();

                let addr_str = Self::derive_address(
                    &self.secp,
                    self.network,
                    self.addr_type,
                    secret_key,
                    public_key,
                );

                let pk = PrivateKey::Bitcoin(wif);
                let address = CoreAddress::Bitcoin(addr_str);

                (pk, address)
            })
            .find_any(|(_pk, addr)| match addr {
                CoreAddress::Bitcoin(s) => self.matches(s),
                _ => false,
            })
            .expect("Infinite iterator execution")
    }
}

impl VanityGenerator for BitcoinVanityGenerator {
    fn generate(&self) -> (PrivateKey, CoreAddress) {
        // Single-threaded generation for efficient batch processing
        // avoids rayon overhead when we just want one key
        let (secret_key, public_key) = self.secp.generate_keypair(&mut rand::thread_rng());

        // Create bitcoin::PrivateKey for WIF
        let bitcoin_private_key = bitcoin::PrivateKey::new(secret_key, self.network);
        let wif = bitcoin_private_key.to_string();

        let addr_str = Self::derive_address(
            &self.secp,
            self.network,
            self.addr_type,
            secret_key,
            public_key,
        );

        (PrivateKey::Bitcoin(wif), CoreAddress::Bitcoin(addr_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::Hash;
    use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
    use std::str::FromStr;

    // ... (test_bitcoin_key_usability remains the same)
    #[test]
    fn test_bitcoin_key_usability() {
        let secp = Secp256k1::new();
        let types = [
            BitcoinAddressType::Legacy,
            BitcoinAddressType::SegWit,
            BitcoinAddressType::Taproot,
        ];

        for addr_type in types {
            let generator = BitcoinVanityGenerator::new("", "", false, Network::Bitcoin, addr_type);
            let (pk, _addr) = generator.generate();

            let wif = match pk {
                PrivateKey::Bitcoin(w) => w,
                _ => panic!("Expected Bitcoin private key"),
            };

            // 1. Recover Secret Key from WIF
            let btc_priv_key = bitcoin::PrivateKey::from_str(&wif).expect("valid WIF");
            let secret_key = btc_priv_key.inner;

            // 2. Sign and Verify Message
            let msg_bytes = b"Is this a valid Bitcoin key?";
            let msg_hash = bitcoin::hashes::sha256::Hash::hash(msg_bytes);
            let message = Message::from_digest(msg_hash.to_byte_array());

            match addr_type {
                BitcoinAddressType::Taproot => {
                    // Taproot uses Schnorr signatures
                    let keypair = bitcoin::secp256k1::Keypair::from_secret_key(&secp, &secret_key);
                    // Using with_rng to avoid randomness issues if default sign_schnorr is missing
                    let sig =
                        secp.sign_schnorr_with_rng(&message, &keypair, &mut rand::thread_rng());
                    let (x_only, _) = XOnlyPublicKey::from_keypair(&keypair);

                    assert!(
                        secp.verify_schnorr(&sig, &message, &x_only).is_ok(),
                        "Schnorr verification failed"
                    );
                }
                _ => {
                    // Legacy and SegWit use ECDSA
                    let sig = secp.sign_ecdsa(&message, &secret_key);
                    let pubkey = PublicKey::from_private_key(&secp, &btc_priv_key);

                    // argument order: message, sig, pubkey
                    assert!(
                        secp.verify_ecdsa(&message, &sig, &pubkey.inner).is_ok(),
                        "ECDSA verification failed"
                    );
                }
            }
        }
    }

    #[test]
    fn test_bip_compliance_vectors() {
        let secp = Secp256k1::new();

        // 1. Private Key (Simpler known key for validation)
        // 1. Private Key (Scalar 1)
        // Private Key: 0x0000000000000000000000000000000000000000000000000000000000000001
        // Corresponding Public Key (Compressed): 0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798
        let mut secret_bytes = [0u8; 32];
        secret_bytes[31] = 1;
        let secret_key = SecretKey::from_slice(&secret_bytes).unwrap();
        // Pre-compute public key to emulate real usage
        let public_key = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &secret_key);

        let verify_addr = |addr_type: BitcoinAddressType, expected: &str| {
            // Use the shared production logic:
            let address_str = BitcoinVanityGenerator::derive_address(
                &secp,
                Network::Bitcoin,
                addr_type,
                secret_key,
                public_key,
            );
            assert_eq!(address_str, expected, "Mismatch for {:?}", addr_type);
        };

        // Standard Test Vectors for PrivKey 0101...01

        // P2PKH (Legacy) - BIP-58
        // 1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH
        verify_addr(
            BitcoinAddressType::Legacy,
            "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH",
        );

        // P2WPKH (SegWit) - BIP-173
        // bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
        verify_addr(
            BitcoinAddressType::SegWit,
            "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
        );

        // P2TR (Taproot) - BIP-350 / BIP-86
        // bc1pmfr3p9j00pfxjh0zmgp99y8zftmd3s5pmedqhyptwy6lm87hf5sspknck9
        verify_addr(
            BitcoinAddressType::Taproot,
            "bc1pmfr3p9j00pfxjh0zmgp99y8zftmd3s5pmedqhyptwy6lm87hf5sspknck9",
        );
    }
}
