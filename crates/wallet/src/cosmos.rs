use bech32::{Bech32, Hrp};
use k256::ecdsa::SigningKey;
use k256::elliptic_curve::rand_core::OsRng;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use vanity_core::{Address, PrivateKey, VanityGenerator};

pub struct CosmosVanityGenerator {
    hrp: String,
    prefix: String,
    suffix: String,
    case_sensitive: bool,
}

impl CosmosVanityGenerator {
    pub fn new(hrp: &str, prefix: &str, suffix: &str, case_sensitive: bool) -> Self {
        Self {
            hrp: hrp.to_string(),
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
            case_sensitive,
        }
    }

    // ... (imports remain same)

    pub fn search(&self, found_flag: Option<Arc<AtomicU64>>) -> (PrivateKey, Address) {
        let mut csprng = OsRng;
        let p_prefix = &self.prefix;
        let p_suffix = &self.suffix;
        let p_case = self.case_sensitive;
        // Code Review: Fallback to "cosmos" is active, but we should potentially warn/error in new() if invalid.
        // For search loop, we must have a valid HRP.
        let hrp = Hrp::parse(&self.hrp).unwrap_or(Hrp::parse("cosmos").expect("valid default"));

        loop {
            if let Some(ref attempts) = found_flag {
                attempts.fetch_add(1, Ordering::Relaxed);
            }

            // 1. Generate Keypair (secp256k1 compressed)
            let signing_key = SigningKey::random(&mut csprng);
            let verifying_key = signing_key.verifying_key();
            let pubkey_bytes = verifying_key.to_sec1_bytes(); // 33 bytes compressed

            // 2. SHA256(pubkey)
            let sha256_hash = Sha256::digest(&pubkey_bytes);

            // 3. RIPEMD160(sha256_hash)
            let mut ripemd_hasher = Ripemd160::new();
            ripemd_hasher.update(sha256_hash);
            let address_bytes = ripemd_hasher.finalize();

            // 4. Encode Bech32
            // Code Review: Handle potential encoding errors gracefully or expect success for valid inputs.
            let address_str =
                bech32::encode::<Bech32>(hrp, &address_bytes).expect("bech32 encoding failed");

            // 5. Check Match
            let mut is_match = true;

            if !p_prefix.is_empty() {
                // Simplified Logic from Code Review
                let target_start = format!("{}1{}", self.hrp, p_prefix);

                let matches = if p_case {
                    address_str.starts_with(&target_start)
                } else {
                    address_str.starts_with(&target_start.to_lowercase())
                };

                if !matches {
                    is_match = false;
                }
            }

            if is_match && !p_suffix.is_empty() {
                let matches = if p_case {
                    address_str.ends_with(p_suffix)
                } else {
                    address_str.ends_with(&p_suffix.to_lowercase())
                };

                if !matches {
                    is_match = false;
                }
            }

            if is_match {
                let secret_bytes = signing_key.to_bytes();
                return (
                    PrivateKey::Cosmos(secret_bytes.into()),
                    Address::Cosmos(address_str),
                );
            }
        }
    }
}

impl VanityGenerator for CosmosVanityGenerator {
    fn generate(&self) -> (PrivateKey, Address) {
        let mut csprng = OsRng;
        let signing_key = SigningKey::random(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let pubkey_bytes = verifying_key.to_sec1_bytes();

        let sha256_hash = Sha256::digest(&pubkey_bytes);
        let mut ripemd_hasher = Ripemd160::new();
        ripemd_hasher.update(sha256_hash);
        let address_bytes = ripemd_hasher.finalize();

        let hrp = Hrp::parse(&self.hrp).unwrap_or(Hrp::parse("cosmos").unwrap());
        let address_str = bech32::encode::<Bech32>(hrp, &address_bytes).unwrap();

        let secret_bytes = signing_key.to_bytes();
        (
            PrivateKey::Cosmos(secret_bytes.into()),
            Address::Cosmos(address_str),
        )
    }
}
