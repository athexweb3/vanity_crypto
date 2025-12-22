
import sys
import binascii
import hashlib

# --- MAIN LOGIC ---

try:
    from eth_account import Account
    from eth_keys import keys
    import base58
    import bech32
    # Use ecdsa library for Bitcoin ECC operations
    from ecdsa import SECP256k1, VerifyingKey
    from ecdsa.util import sigdecode_der
except ImportError:
    import os
    print("[ERROR] Required libraries not found.")
    sys.exit(1)

def hash160(data):
    sha = hashlib.sha256(data).digest()
    return hashlib.new('ripemd160', sha).digest()

# --- EXPORTED VERIFICATION FUNCTIONS ---

def get_bitcoin_address(wif_key):
    """
    Decodes WIF and returns a dictionary of derived addresses:
    { 'p2pkh': ..., 'p2wpkh': ..., 'p2tr': ... }
    Returns None if WIF is invalid.
    """
    wif_key = wif_key.strip()
    try:
        # 1. Base58 Decode
        decoded = base58.b58decode_check(wif_key)
        
        version = decoded[0]
        if len(decoded) == 33:
            key_bytes = decoded[1:33]
            is_compressed = False
        elif len(decoded) == 34:
            key_bytes = decoded[1:33]
            compression_flag = decoded[33]
            if compression_flag != 0x01: return None
            is_compressed = True
        else:
            return None
            
        if version == 0x80: 
            p2pkh_ver = 0x00
            hrp = "bc"
        elif version == 0xef:
            p2pkh_ver = 0x6f
            hrp = "tb"
        else:
            # Fallback
            p2pkh_ver = 0x00
            hrp = "bc"

        # 3. Derive PubKey Operations
        priv_int = int.from_bytes(key_bytes, 'big')
        pub_point = point_mul(priv_int, (G_X, G_Y))
        
        # Format Keys
        x_bytes = pub_point[0].to_bytes(32, 'big')
        y_bytes = pub_point[1].to_bytes(32, 'big')
        
        uncompressed_pub = b'\x04' + x_bytes + y_bytes
        
        prefix = b'\x02' if pub_point[1] % 2 == 0 else b'\x03'
        compressed_pub = prefix + x_bytes

        # --- LEGACY (P2PKH) ---
        pub_to_hash = compressed_pub if is_compressed else uncompressed_pub
        pub_hash = hash160(pub_to_hash)
        p2pkh_addr = base58.b58encode_check(bytes([p2pkh_ver]) + pub_hash).decode()

        # --- SEGWIT (P2WPKH) ---
        p2wpkh_addr = None
        if is_compressed:
            witness_prog = hash160(compressed_pub) 
            witness_prog_5bit = bech32.convertbits(witness_prog, 8, 5)
            p2wpkh_addr = bech32.bech32_encode(hrp, [0] + witness_prog_5bit)
            
        # --- TAPROOT (P2TR) ---
        p2tr_addr = None
        if pub_point[1] % 2 != 0:
            internal_pub = (pub_point[0], P - pub_point[1])
        else:
            internal_pub = pub_point
            
        internal_x_bytes = internal_pub[0].to_bytes(32, 'big')
        tweak_bytes = tagged_hash("TapTweak", internal_x_bytes)
        tweak_int = int.from_bytes(tweak_bytes, 'big')
        tweak_point = point_mul(tweak_int, (G_X, G_Y))
        Q = point_add(internal_pub, tweak_point)
        qx_bytes = Q[0].to_bytes(32, 'big')
        witness_prog_5bit = bech32.convertbits(qx_bytes, 8, 5)
        p2tr_addr = bech32m_encode(hrp, [1] + witness_prog_5bit)
        
        return {
            'legacy': p2pkh_addr,
            'segwit': p2wpkh_addr,
            'taproot': p2tr_addr
        }

    except Exception:
        return None

def verify_ethereum_key(hex_key):
    # Standalone CLI verification logic
    print("\n[VERIFYING ETHEREUM KEY]")
    hex_key = hex_key.strip().replace("0x", "")
    if len(hex_key) != 64:
        print(f"❌ INVALID LENGTH: 64 hex chars required.")
        return
    try:
        pk_str = "0x" + hex_key
        account = Account.from_key(pk_str)
        print("✅ VALID Private Key Verified")
        print(f"   Private Key: {pk_str}")
        print(f"   Address:     {account.address}")
        print("\n   Note: Key is mathematically valid and importable.")
        print("")
    except Exception as e:
        print(f"❌ Key Error: {e}")
        print("")

def verify_bitcoin_key(wif_key):
    print("\n[VERIFYING BITCOIN WIF]")
    addresses = get_bitcoin_address(wif_key)
    
    if addresses:
        print(f"✅ VALID WIF Generated")
        print(f"   Legacy (P2PKH):  {addresses['legacy']}")
        print(f"   SegWit (P2WPKH): {addresses['segwit'] or 'N/A'}")
        print(f"   Taproot (P2TR):  {addresses['taproot']}")
        print("")
    else:
        print(f"❌ Invalid WIF or calculation error.")
        print("")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Argument provided
        print("\n--- Vanity Crypto Key Verifier ---")
        input_key = sys.argv[1].strip()
    elif not sys.stdin.isatty():
        # Piped input
        print("\n--- Vanity Crypto Key Verifier ---")
        input_key = sys.stdin.read().strip()
    else:
        # Interactive mode
        print("\n--- Vanity Crypto Key Verifier ---")
        input_key = input("Enter Private Key (Hex or WIF): ").strip()
    
    # Detect input type (WIF vs Hex)
    
    if input_key.startswith("0x") or (len(input_key) == 64 and all(c in '0123456789abcdefABCDEF' for c in input_key)):
        verify_ethereum_key(input_key)
    else:
        verify_bitcoin_key(input_key)

