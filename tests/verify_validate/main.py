
import sys
import binascii
import hashlib

# --- SECP256K1 & MATH ---
P = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F
N = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141
G_X = 0x79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
G_Y = 0x483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8

def point_add(p1, p2):
    if p1 is None: return p2
    if p2 is None: return p1
    x1, y1 = p1
    x2, y2 = p2
    if x1 == x2 and y1 != y2: return None
    if x1 == x2:
        lam = (3 * x1 * x1 * pow(2 * y1, P - 2, P)) % P
    else:
        lam = ((y2 - y1) * pow(x2 - x1, P - 2, P)) % P
    x3 = (lam * lam - x1 - x2) % P
    y3 = (lam * (x1 - x3) - y1) % P
    return (x3, y3)

def point_mul(k, p):
    r = None
    while k > 0:
        if k % 2 == 1: r = point_add(r, p)
        p = point_add(p, p)
        k //= 2
    return r

def lift_x(x):
    if x >= P: return None
    y_sq = (pow(x, 3, P) + 7) % P
    y = pow(y_sq, (P + 1) // 4, P)
    if pow(y, 2, P) != y_sq: return None
    return (x, y if y % 2 == 0 else P - y)

def tagged_hash(tag: str, data: bytes) -> bytes:
    tag_hash = hashlib.sha256(tag.encode()).digest()
    return hashlib.sha256(tag_hash + tag_hash + data).digest()

# --- BECH32M ---
BECH32M_CONST = 0x2bc830a3

def bech32_polymod(values):
    GEN = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3]
    chk = 1
    for v in values:
        b = (chk >> 25)
        chk = (chk & 0x1ffffff) << 5 ^ v
        for i in range(5):
            chk ^= GEN[i] if ((b >> i) & 1) else 0
    return chk

def bech32_hrp_expand(hrp):
    return [ord(x) >> 5 for x in hrp] + [0] + [ord(x) & 31 for x in hrp]

def bech32_create_checksum(hrp, data, spec_const):
    values = bech32_hrp_expand(hrp) + data
    polymod = bech32_polymod(values + [0, 0, 0, 0, 0, 0]) ^ spec_const
    return [(polymod >> 5 * (5 - i)) & 31 for i in range(6)]

def bech32m_encode(hrp, data):
    # Data is 5-bit integers
    combined = data + bech32_create_checksum(hrp, data, BECH32M_CONST)
    CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
    return hrp + '1' + ''.join([CHARSET[d] for d in combined])

# --- MAIN LOGIC ---

try:
    from eth_account import Account
    from eth_keys import keys
    import base58
    import bech32
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

