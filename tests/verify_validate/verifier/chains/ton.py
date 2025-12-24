try:
    from tonsdk.contract.wallet import WalletV4ContractR2
    from nacl.signing import SigningKey
    import hashlib
    import base64
except ImportError as e:
    print(f"Skipping TON verification: {e}")
    WalletV4ContractR2 = None

def crc16(data: bytes) -> bytes:
    crc = 0x0000
    for byte in data:
        crc ^= (byte << 8)
        for _ in range(8):
            if (crc & 0x8000):
                crc = (crc << 1) ^ 0x1021
            else:
                crc = crc << 1
            crc &= 0xFFFF
    return (crc).to_bytes(2, "big")

def get_ton_address(seed_hex):
    """
    Returns a dictionary of derived addresses for the given seed:
    {
        'v4r2_eq': 'EQ...',
        'v5r1_eq': 'EQ...',
        'v5r1_uq': 'UQ...'
    }
    """
    if WalletV4ContractR2 is None:
        return None

    try:
        # 1. Parse Seed
        if seed_hex.startswith("0x"):
            seed_hex = seed_hex[2:]
        seed_bytes = bytes.fromhex(seed_hex)
        
        if len(seed_bytes) != 32:
            return None

        # 2. Derive Keypair (Ed25519)
        signing_key = SigningKey(seed_bytes)
        pub_key = signing_key.verify_key.encode()

        results = {}

        # Helper: Encode Address
        def encode_addr(hash_bytes, tag):
            payload = bytearray()
            payload.append(tag)
            payload.append(0x00) 
            payload.extend(hash_bytes)
            
            crc = crc16(payload)
            payload.extend(crc)
            
            return base64.urlsafe_b64encode(payload).decode('utf-8')

        # --- V4R2 (Manual) ---
        # Data Cell V4R2: 00 51 00 00 00 00 29 a9 a3 17 + pubkey + 40
        data_v4 = bytearray()
        data_v4.extend([0x00, 0x51, 0x00, 0x00, 0x00, 0x00, 0x29, 0xa9, 0xa3, 0x17])
        data_v4.extend(pub_key)
        data_v4.append(0x40)
        
        data_hash_v4 = hashlib.sha256(data_v4).digest()
        
        # StateInit V4R2
        state_init_head_v4 = bytes([0x02, 0x01, 0x34, 0x00, 0x07, 0x00, 0x00])
        code_hash_v4r2 = bytes([
            0xfe, 0xb5, 0xff, 0x68, 0x20, 0xe2, 0xff, 0x0d, 0x94, 0x83, 0xe7, 0xe0, 0xd6, 0x2c, 0x81, 0x7d,
            0x84, 0x67, 0x89, 0xfb, 0x4a, 0xe5, 0x80, 0xc8, 0x78, 0x86, 0x6d, 0x95, 0x9d, 0xab, 0xd5, 0xc0,
        ])
        
        hasher_v4 = hashlib.sha256()
        hasher_v4.update(state_init_head_v4)
        hasher_v4.update(code_hash_v4r2)
        hasher_v4.update(data_hash_v4)
        v4r2_hash = hasher_v4.digest()
        
        results['v4r2_eq'] = encode_addr(v4r2_hash, 0x11)
        results['v4r2_uq'] = encode_addr(v4r2_hash, 0x51)

        # --- V5R1 (Manual) ---
        data = bytearray()
        data.extend([0x00, 0x51]) 
        data.extend([0x80, 0x00, 0x00, 0x00, 0x3F, 0xFF, 0xFF, 0x88]) 
        data.append(0x80 | (pub_key[0] >> 1))
        for i in range(31):
            data.append((pub_key[i] << 7 & 0xFF) | (pub_key[i + 1] >> 1))
        data.append((pub_key[31] << 7 & 0xFF) | 0x20)
        
        data_hash = hashlib.sha256(data).digest()

        state_init_head = bytes([0x02, 0x01, 0x34, 0x00, 0x06, 0x00, 0x00])
        code_hash_v5r1 = bytes([
            0x20, 0x83, 0x4b, 0x7b, 0x72, 0xb1, 0x12, 0x14, 0x7e, 0x1b, 0x2f, 0xb4, 0x57, 0xb8, 0x4e, 0x74,
            0xd1, 0xa3, 0x0f, 0x04, 0xf7, 0x37, 0xd4, 0xf6, 0x2a, 0x66, 0x8e, 0x95, 0x52, 0xd2, 0xb7, 0x2f,
        ])
        
        hasher = hashlib.sha256()
        hasher.update(state_init_head)
        hasher.update(code_hash_v5r1)
        hasher.update(data_hash)
        state_init_hash = hasher.digest()

        results['v5r1_uq'] = encode_addr(state_init_hash, 0x51)
        results['v5r1_eq'] = encode_addr(state_init_hash, 0x11)
        
        return results

    except Exception as e:
        print(f"Error deriving TON: {e}")
        return None

def verify_ton_key(input_key):
    print("\n[VERIFYING TON KEY]")
    
    # Strip 0x if present
    if input_key.startswith("0x"):
        input_key = input_key[2:]
        
    if len(input_key) != 64:
        print("❌ INVALID LENGTH: 64 hex chars required (32 bytes).")
        return

    try:
        addresses = get_ton_address(input_key)
        if addresses:
            print("✅ VALID Private Key Verified")
            print(f"   Private Key: {input_key}")
            print(f"   V4R2 (EQ):   {addresses.get('v4r2_eq')}")
            print(f"   V5R1 (UQ):   {addresses.get('v5r1_uq')}")
            print("")
        else:
            print("❌ Failed to derive address.")
    except Exception as e:
        print(f"❌ Key Error: {e}")

