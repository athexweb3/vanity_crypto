import hashlib
import bech32
import ecdsa

def get_cosmos_address(private_key_hex, hrp="cosmos"):
    """
    Derives a Cosmos address from a private key hex string.
    """
    try:
        # 1. Decode Private Key
        priv_key_bytes = bytes.fromhex(private_key_hex)
        
        # 2. Get Public Key (Compressed)
        sk = ecdsa.SigningKey.from_string(priv_key_bytes, curve=ecdsa.SECP256k1)
        vk = sk.verifying_key
        pub_key_bytes = vk.to_string("compressed")
        
        # 3. SHA256(pubkey)
        sha256_hash = hashlib.sha256(pub_key_bytes).digest()
        
        # 4. RIPEMD160(sha256_hash)
        ripemd160 = hashlib.new('ripemd160')
        ripemd160.update(sha256_hash)
        raw_address = ripemd160.digest()
        
        # 5. Bech32 Encode (convert 8-bit to 5-bit first)
        five_bit_data = bech32.convertbits(raw_address, 8, 5)
        address = bech32.bech32_encode(hrp, five_bit_data)
        
        return address
        return address
    except Exception as e:
        print(f"Error deriving Cosmos address: {e}")
        return None

def verify_cosmos_key(input_key, hrp="cosmos"):
    print("\n[VERIFYING COSMOS KEY]")
    
    # Strip 0x if present
    if input_key.startswith("0x"):
        input_key = input_key[2:]
        
    if len(input_key) != 64:
        print("❌ INVALID LENGTH: 64 hex chars required (32 bytes).")
        return

    try:
        address = get_cosmos_address(input_key, hrp)
        if address:
            print("✅ VALID Private Key Verified")
            print(f"   Private Key: {input_key}")
            print(f"   Address:     {address}")
            print("\n   Note: Address depends on HRP (default 'cosmos').")
            print("")
        else:
            print("❌ Failed to derive address.")
    except Exception as e:
        print(f"❌ Key Error: {e}")

