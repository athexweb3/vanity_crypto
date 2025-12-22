use bitcoin::secp256k1::Secp256k1;
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
}

impl BitcoinVanityGenerator {
    pub fn new(
        prefix: &str,
        suffix: &str,
        case_sensitive: bool,
        network: Network,
        addr_type: BitcoinAddressType,
    ) -> Self {
        Self {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
            case_sensitive,
            network,
            addr_type,
        }
    }

    #[inline(always)]
    fn matches(&self, addr_str: &str) -> bool {
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
            target.starts_with(&p)
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

    pub fn search(
        &self,
        progress: Option<Arc<std::sync::atomic::AtomicU64>>,
    ) -> (PrivateKey, CoreAddress) {
        // Create Secp256k1 context once (expensive operation)
        // Secp256k1 is Send + Sync, so it can be safely shared across threads
        let secp = Secp256k1::new();

        rayon::iter::repeat(())
            .map(|_| {
                if let Some(p) = &progress {
                    p.fetch_add(1, Ordering::Relaxed);
                }

                let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());

                // Create bitcoin::PrivateKey for WIF
                let bitcoin_private_key = bitcoin::PrivateKey::new(secret_key, self.network);
                let wif = bitcoin_private_key.to_string();

                let addr_str = match self.addr_type {
                    BitcoinAddressType::Legacy => {
                        let pubkey = PublicKey::new(public_key);
                        Address::p2pkh(pubkey, self.network).to_string()
                    }
                    BitcoinAddressType::SegWit => {
                        let pubkey = CompressedPublicKey(public_key);
                        Address::p2wpkh(&pubkey, self.network).to_string()
                    }
                    BitcoinAddressType::Taproot => {
                        let keypair =
                            bitcoin::secp256k1::Keypair::from_secret_key(&secp, &secret_key);
                        let (x_only, _parity) = XOnlyPublicKey::from_keypair(&keypair);
                        Address::p2tr(&secp, x_only, None, self.network).to_string()
                    }
                };

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
        self.search(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::Hash;
    use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
    use std::str::FromStr;

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

        let verify_addr = |addr_type: BitcoinAddressType, expected: &str| {
            let address_str = match addr_type {
                BitcoinAddressType::Legacy => {
                    let public_key =
                        bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
                    let pubkey = PublicKey::from(CompressedPublicKey(public_key));
                    Address::p2pkh(pubkey, Network::Bitcoin).to_string()
                }
                BitcoinAddressType::SegWit => {
                    let public_key =
                        bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
                    let pubkey = CompressedPublicKey(public_key);
                    Address::p2wpkh(&pubkey, Network::Bitcoin).to_string()
                }
                BitcoinAddressType::Taproot => {
                    let keypair = bitcoin::secp256k1::Keypair::from_secret_key(&secp, &secret_key);
                    let (xonly, _) = XOnlyPublicKey::from_keypair(&keypair);
                    Address::p2tr(&secp, xonly, None, Network::Bitcoin).to_string()
                }
            };
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
