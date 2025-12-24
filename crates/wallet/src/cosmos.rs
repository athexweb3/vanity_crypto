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

    pub fn search(&self, found_flag: Option<Arc<AtomicU64>>) -> (PrivateKey, Address) {
        let mut csprng = OsRng;
        let p_prefix = &self.prefix;
        let p_suffix = &self.suffix;
        let p_case = self.case_sensitive;
        let hrp = Hrp::parse(&self.hrp).unwrap_or(Hrp::parse("cosmos").unwrap());

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
            // Note: bech32 crate automatically handles 8-bit to 5-bit conversion in `encode`
            let address_str = bech32::encode::<Bech32>(hrp, &address_bytes).unwrap();

            // 5. Check Match
            let mut is_match = true;

            // TODO check data part only optimization?
            // For now simple string matching on the full address "cosmos1..."

            if !p_prefix.is_empty() {
                // Address format: hrp + "1" + data
                // User prefix search usually implies searching *after* the "1" separator?
                // Or just standard string starts_with?
                // Standard convention: user types "alice", expects "cosmos1alice..."
                // So we check if address_str starts with (hrp + "1" + prefix) if prefix doesn't contain "1"

                // Let's stick to standard string matching logic for now:
                // If user wants "cosmos1test...", they pass prefix "test" or "cosmos1test"?
                // Usually tools allow specifying the part AFTER the separator.
                // But our generic interface takes a "prefix".
                // If I type prefix "A", do I match "cosmos1A..."? Yes.

                // But Bech32 data part is base32 (qn... or similar charset).
                // "A" might not be valid if it's not lowercase (Bech32 checks case).
                // Bech32 is usually all lowercase or all uppercase.

                let check_str = if p_case {
                    address_str.clone()
                } else {
                    address_str.to_lowercase()
                };

                // Helper: construct target prefix
                // If user input "foo", we look for "cosmos1foo..."
                let target_start = format!("{}1{}", self.hrp, p_prefix);

                // If case sensitive, exact match.
                // But wait, bech32 is case insensitive in spec (canonical is lowercase).
                // We always generate lowercase string from encode().

                if !check_str.starts_with(&target_start.to_lowercase()) {
                    is_match = false;
                }
            }

            if is_match && !p_suffix.is_empty() {
                // Suffix matching is simpler, just ends_with
                let check_str = if p_case {
                    address_str.clone()
                } else {
                    address_str.to_lowercase()
                };

                if !check_str.ends_with(p_suffix) {
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
