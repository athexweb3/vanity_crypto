use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use vanity_core::{Address, PrivateKey, TonWalletVersion, VanityGenerator};

// --- CONSTANTS FOR WALLET V4R2 ---

// Header: Descriptors(0051) + SeqNo(0) + WalletID(696840746 = 0x29a9a317)
const DATA_HEAD: [u8; 10] = [0x00, 0x51, 0x00, 0x00, 0x00, 0x00, 0x29, 0xa9, 0xa3, 0x17];
// Tail: Padding for 321 bits (1 bit 0 completion)
const DATA_TAIL: [u8; 1] = [0x40];

// Header: Descriptors(0201) + DataBits(34) + CodeDepth(7) + DataDepth(0)
const STATE_INIT_HEAD: [u8; 7] = [0x02, 0x01, 0x34, 0x00, 0x07, 0x00, 0x00];

// Code Hash (Constant for V4R2)
const CODE_HASH_V4R2: [u8; 32] = [
    0xfe, 0xb5, 0xff, 0x68, 0x20, 0xe2, 0xff, 0x0d, 0x94, 0x83, 0xe7, 0xe0, 0xd6, 0x2c, 0x81, 0x7d,
    0x84, 0x67, 0x89, 0xfb, 0x4a, 0xe5, 0x80, 0xc8, 0x78, 0x86, 0x6d, 0x95, 0x9d, 0xab, 0xd5, 0xc0,
];

// --- CONSTANTS FOR WALLET V5R1 ---

// Code Hash V5R1
const CODE_HASH_V5R1: [u8; 32] = [
    0x20, 0x83, 0x4b, 0x7b, 0x72, 0xb1, 0x12, 0x14, 0x7e, 0x1b, 0x2f, 0xb4, 0x57, 0xb8, 0x4e, 0x74,
    0xd1, 0xa3, 0x0f, 0x04, 0xf7, 0x37, 0xd4, 0xf6, 0x2a, 0x66, 0x8e, 0x95, 0x52, 0xd2, 0xb7, 0x2f,
];

// Descriptor + Prefix
const DATA_HEAD_V5R1: [u8; 10] = [0x00, 0x51, 0x80, 0x00, 0x00, 0x00, 0x3F, 0xFF, 0xFF, 0x88];
// Code Depth: 6. Data Depth: 0.
const STATE_INIT_HEAD_V5R1: [u8; 7] = [0x02, 0x01, 0x34, 0x00, 0x06, 0x00, 0x00];

pub struct TonVanityGenerator {
    prefix: String,
    suffix: String,
    case_sensitive: bool,
    version: TonWalletVersion,
}

impl TonVanityGenerator {
    pub fn new(
        prefix: &str,
        suffix: &str,
        case_sensitive: bool,
        version: TonWalletVersion,
    ) -> Self {
        Self {
            prefix: prefix.to_string(),
            suffix: suffix.to_string(),
            case_sensitive,
            version,
        }
    }

    pub fn search(&self, found_flag: Option<Arc<AtomicU64>>) -> (PrivateKey, Address) {
        let mut csprng = OsRng;
        let p_prefix = &self.prefix;
        let p_suffix = &self.suffix;
        let p_case = self.case_sensitive;

        loop {
            // Update stats
            if let Some(ref attempts) = found_flag {
                attempts.fetch_add(1, Ordering::Relaxed);
            }

            // 1. Generate Keypair
            let (signing_key, verifying_key) = generate_ed25519(&mut csprng);
            let pubkey_bytes = verifying_key.as_bytes();

            // 2. Compute Data Hash
            let mut hasher = Sha256::new();
            if self.version == TonWalletVersion::V5R1 {
                hasher.update(DATA_HEAD_V5R1);
                // Byte 8: Last bit of WalletID (1) + 7 bits of PubKey
                hasher.update([0x80 | (pubkey_bytes[0] >> 1)]);

                // Middle bytes: Shifted PubKey
                for i in 0..31 {
                    hasher.update([(pubkey_bytes[i] << 7) | (pubkey_bytes[i + 1] >> 1)]);
                }

                // Last byte: Last bit of PubKey + Padding (0x20)
                hasher.update([(pubkey_bytes[31] << 7) | 0x20]);
            } else {
                hasher.update(DATA_HEAD);
                hasher.update(pubkey_bytes);
                hasher.update(DATA_TAIL);
            }
            let data_hash = hasher.finalize();

            // 3. Compute StateInit Hash
            let mut hasher = Sha256::new();
            if self.version == TonWalletVersion::V5R1 {
                hasher.update(STATE_INIT_HEAD_V5R1);
                hasher.update(CODE_HASH_V5R1);
            } else {
                hasher.update(STATE_INIT_HEAD);
                hasher.update(CODE_HASH_V4R2);
            }
            hasher.update(data_hash);
            let state_init_hash = hasher.finalize();

            // 4. Encode Address (Base64 URL Safe) to check match
            let tag = if !p_prefix.is_empty() && p_prefix.starts_with('E') {
                0x11
            } else {
                0x51
            };
            let address_str = encode_ton_address(&state_init_hash, tag);

            // 5. Check Match
            let mut is_match = true;
            if !p_prefix.is_empty() {
                if p_case {
                    if !address_str.starts_with(p_prefix) {
                        is_match = false;
                    }
                } else if address_str.len() < p_prefix.len()
                    || !address_str[..p_prefix.len()].eq_ignore_ascii_case(p_prefix)
                {
                    is_match = false;
                }
            }

            if is_match && !p_suffix.is_empty() {
                if p_case {
                    if !address_str.ends_with(p_suffix) {
                        is_match = false;
                    }
                } else if address_str.len() < p_suffix.len()
                    || !address_str[address_str.len() - p_suffix.len()..]
                        .eq_ignore_ascii_case(p_suffix)
                {
                    is_match = false;
                }
            }

            if is_match {
                // Return result
                let secret_bytes = signing_key.to_bytes();
                // Store as 64-byte keypair buffer (standard Ed25519 persistence)
                return (PrivateKey::Ton(secret_bytes), Address::Ton(address_str));
            }
        }
    }
}

fn generate_ed25519(csprng: &mut OsRng) -> (SigningKey, VerifyingKey) {
    let mut secret_bytes = [0u8; 32];
    csprng.fill_bytes(&mut secret_bytes);
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

fn encode_ton_address(hash: &[u8], tag: u8) -> String {
    let mut bytes = Vec::with_capacity(36);
    bytes.push(tag);
    bytes.push(0x00); // Workchain 0
    bytes.extend_from_slice(hash);

    let crc = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
    let checksum = crc.checksum(&bytes);

    bytes.push((checksum >> 8) as u8);
    bytes.push((checksum & 0xFF) as u8);

    general_purpose::URL_SAFE.encode(&bytes)
}

impl VanityGenerator for TonVanityGenerator {
    fn generate(&self) -> (PrivateKey, Address) {
        let mut csprng = OsRng;
        let (signing_key, verifying_key) = generate_ed25519(&mut csprng);
        let pubkey_bytes = verifying_key.as_bytes();

        let mut hasher = Sha256::new();
        if self.version == TonWalletVersion::V5R1 {
            hasher.update(DATA_HEAD_V5R1);
            hasher.update([0x80 | (pubkey_bytes[0] >> 1)]);
            for i in 0..31 {
                hasher.update([(pubkey_bytes[i] << 7) | (pubkey_bytes[i + 1] >> 1)]);
            }
            hasher.update([(pubkey_bytes[31] << 7) | 0x20]);
        } else {
            hasher.update(DATA_HEAD);
            hasher.update(pubkey_bytes);
            hasher.update(DATA_TAIL);
        }
        let data_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        if self.version == TonWalletVersion::V5R1 {
            hasher.update(STATE_INIT_HEAD_V5R1);
            hasher.update(CODE_HASH_V5R1);
        } else {
            hasher.update(STATE_INIT_HEAD);
            hasher.update(CODE_HASH_V4R2);
        }
        hasher.update(data_hash);
        let state_init_hash = hasher.finalize();

        let address = encode_ton_address(&state_init_hash, 0x51);
        let secret_bytes = signing_key.to_bytes();
        // Return 32-byte seed
        (PrivateKey::Ton(secret_bytes), Address::Ton(address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_v5r1_address_generation() {
        let seed_hex = "2da54880fb610e9423fc7852d52d27ddadb2c3cc8517114e87d346f42948e14f";
        let target_addr = "UQDm1CtfFIspmCdbM5JylWCQmUArTv8J4FHXcKv9txA-NxZW";

        let mut seed = [0u8; 32];
        hex::decode_to_slice(seed_hex, &mut seed).unwrap();

        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let pubkey_bytes = verifying_key.as_bytes();

        let mut hasher = Sha256::new();
        hasher.update(DATA_HEAD_V5R1);
        hasher.update([0x80 | (pubkey_bytes[0] >> 1)]);
        for i in 0..31 {
            hasher.update([(pubkey_bytes[i] << 7) | (pubkey_bytes[i + 1] >> 1)]);
        }
        hasher.update([(pubkey_bytes[31] << 7) | 0x20]);
        let data_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(STATE_INIT_HEAD_V5R1);
        hasher.update(CODE_HASH_V5R1);
        hasher.update(data_hash);
        let state_init_hash = hasher.finalize();

        let address = encode_ton_address(&state_init_hash, 0x51);
        assert_eq!(address, target_addr, "V5R1 Address Mismatch");
    }

    #[test]
    fn test_v4r2_vector_verification() {
        // Vector from Step 3574
        let seed_hex = "a9b90f710d18b8da16b4f700f05b324daa3dbc57795a780e4c746e98fba242d4";
        let target_v4r2 = "EQCuZJii74lgIKZKX_1zz1aFK_zA4y30EZSzguuTXtEuJp-r";

        let mut seed = [0u8; 32];
        hex::decode_to_slice(seed_hex, &mut seed).unwrap();

        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let pubkey_bytes = verifying_key.as_bytes();

        // Regenerate V4R2 manually
        let mut hasher = Sha256::new();
        hasher.update(DATA_HEAD);
        hasher.update(pubkey_bytes);
        hasher.update(DATA_TAIL);
        let data_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(STATE_INIT_HEAD);
        hasher.update(CODE_HASH_V4R2);
        hasher.update(data_hash);
        let state_init_hash = hasher.finalize();

        // V4R2 traditionally used EQ (0x11)
        let address = encode_ton_address(&state_init_hash, 0x11);
        assert_eq!(address, target_v4r2, "V4R2 Address Mismatch");
    }

    #[test]
    fn test_v5r1_vector_modes() {
        // Vector from Step 3574
        let seed_hex = "a9b90f710d18b8da16b4f700f05b324daa3dbc57795a780e4c746e98fba242d4";
        let target_uq = "UQAf25uVNlQUbtYFPintHesYHmC_GDRUa63bqAwtMp8McURe";
        let target_eq = "EQAf25uVNlQUbtYFPintHesYHmC_GDRUa63bqAwtMp8McRmb";

        let mut seed = [0u8; 32];
        hex::decode_to_slice(seed_hex, &mut seed).unwrap();
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        let pubkey_bytes = verifying_key.as_bytes();

        // Calculate StateInit for V5R1
        let mut hasher = Sha256::new();
        hasher.update(DATA_HEAD_V5R1);
        hasher.update([0x80 | (pubkey_bytes[0] >> 1)]);
        for i in 0..31 {
            hasher.update([(pubkey_bytes[i] << 7) | (pubkey_bytes[i + 1] >> 1)]);
        }
        hasher.update([(pubkey_bytes[31] << 7) | 0x20]);
        let data_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(STATE_INIT_HEAD_V5R1);
        hasher.update(CODE_HASH_V5R1);
        hasher.update(data_hash);
        let state_init_hash = hasher.finalize();

        // Verify UQ
        let addr_uq = encode_ton_address(&state_init_hash, 0x51);
        assert_eq!(addr_uq, target_uq, "V5R1 UQ Mismatch");

        // Verify EQ
        let addr_eq = encode_ton_address(&state_init_hash, 0x11);
        assert_eq!(addr_eq, target_eq, "V5R1 EQ Mismatch");
    }

    #[test]
    fn test_ton_vanity_search() {
        let gen = TonVanityGenerator::new("UQA", "", true, TonWalletVersion::V4R2);
        let (__pk, addr) = gen.search(None);
        let addr_str = addr.to_string();

        assert!(addr_str.starts_with("UQA"));
    }

    #[test]
    fn test_ton_vanity_search_bounceable() {
        let gen = TonVanityGenerator::new("EQA", "", true, TonWalletVersion::V4R2);
        let (__pk, addr) = gen.search(None);
        let addr_str = addr.to_string();

        assert!(addr_str.starts_with("EQA"));
    }

    #[test]
    fn test_ton_suffix_search() {
        // Search for a suffix, e.g., "A" (very fast)
        // With case sensitivity FALSE
        let gen = TonVanityGenerator::new("", "A", false, TonWalletVersion::V4R2);
        let (__pk, addr) = gen.search(None);
        let addr_str = addr.to_string();

        assert!(addr_str.to_uppercase().ends_with('A'));
    }

    #[test]
    fn test_ton_case_sensitivity() {
        // Base64 is case sensitive. "UQa" is impossible for Workchain 0 (UQ + 0000xx -> A/B/C/D).
        // Search for a suffix match which is valid and tests case sensitivity.
        // Searching for "x" (lowercase) at the end.
        let gen = TonVanityGenerator::new("UQ", "x", true, TonWalletVersion::V5R1);
        let (__pk, addr) = gen.search(None);
        let addr_str = addr.to_string();

        assert!(addr_str.ends_with('x'));
    }

    #[test]
    fn test_v5r1_data_hashing() {
        let pubkey_hex = "d2073d7a52073c61b01c58ffe568aa2db57718a7838e43c92fa6ab56d6514ae2";
        let target_hash_hex = "14caaf3e738985957c1f389434735a0728432151029ca5f1c458a3bbc5c72c1b";
        let mut pubkey = [0u8; 32];
        hex::decode_to_slice(pubkey_hex, &mut pubkey).unwrap();
        let mut data = Vec::with_capacity(43);

        data.push(0x00);
        data.push(0x51);

        data.extend_from_slice(&[0x80, 0x00, 0x00, 0x00, 0x3F, 0xFF, 0xFF, 0x88]);

        data.push(0x80 | (pubkey[0] >> 1));

        for i in 0..31 {
            data.push((pubkey[i] << 7) | (pubkey[i + 1] >> 1));
        }

        data.push((pubkey[31] << 7) | 0x20);

        let calculated_hash = Sha256::digest(&data);
        assert_eq!(
            hex::encode(calculated_hash),
            target_hash_hex,
            "V5R1 Hash Mismatch"
        );
    }
}
