
import sys
import binascii
import hashlib

# --- MAIN LOGIC ---

# Optional Imports
Account = None
keys = None
base58 = None
bech32 = None
SECP256k1 = None
SigningKey = None

# 1. Try Ethereum Dependencies
try:
    from eth_account import Account
    from eth_keys import keys
except ImportError:
    pass # Handled in verify_ethereum_key

# 2. Try Bitcoin Dependencies
try:
    import base58
    import bech32
    from ecdsa import SECP256k1, SigningKey
except ImportError:
    pass # Handled in verify_bitcoin_key

def hash160(data):
    sha = hashlib.sha256(data).digest()
    return hashlib.new('ripemd160', sha).digest()

# --- Bech32m Implementation (BIP-350) ---
# Included to ensure rigorous correctness independent of installed library versions.

BECH32_CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
BECH32_CONST = 1
BECH32M_CONST = 0x2bc830a3

class Encoding:
    BECH32 = 1
    BECH32M = 2

def bech32_polymod(values):
    GEN = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3]
    chk = 1
    for v in values:
        b = chk >> 25
        chk = (chk & 0x1ffffff) << 5 ^ v
        for i in range(5):
            chk ^= GEN[i] if ((b >> i) & 1) else 0
    return chk

def bech32_hrp_expand(hrp):
    return [ord(x) >> 5 for x in hrp] + [0] + [ord(x) & 31 for x in hrp]

def bech32_create_checksum(hrp, data, spec):
    values = bech32_hrp_expand(hrp) + data
    const = BECH32M_CONST if spec == Encoding.BECH32M else BECH32_CONST
    polymod = bech32_polymod(values + [0, 0, 0, 0, 0, 0]) ^ const
    return [(polymod >> 5 * (5 - i)) & 31 for i in range(6)]

def local_bech32_encode(hrp, data, spec):
    combined = data + bech32_create_checksum(hrp, data, spec)
    return hrp + '1' + ''.join([BECH32_CHARSET[d] for d in combined])

# --- EXPORTED VERIFICATION FUNCTIONS ---

def get_bitcoin_address(wif_key):
    """
    Decodes WIF and returns a dictionary of derived addresses:
    { 'legacy': ..., 'segwit': ..., 'taproot': ... }
    Returns None if WIF is invalid.
    Uses audited ecdsa library for all cryptographic operations.
    """
    wif_key = wif_key.strip()
    try:
        # 1. Base58 Decode WIF
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

        # 2. Use ecdsa library for public key derivation
        sk = SigningKey.from_string(key_bytes, curve=SECP256k1)
        vk = sk.get_verifying_key()
        
        # Get uncompressed and compressed public keys
        point = vk.pubkey.point
        x_bytes = point.x().to_bytes(32, 'big')
        y_bytes = point.y().to_bytes(32, 'big')
        
        uncompressed_pub = b'\x04' + x_bytes + y_bytes
        prefix = b'\x02' if point.y() % 2 == 0 else b'\x03'
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
            # Use local bech32 encode with v0 spec (BECH32)
            p2wpkh_addr = local_bech32_encode(hrp, [0] + witness_prog_5bit, Encoding.BECH32)
            
        # --- TAPROOT (P2TR) ---
        p2tr_addr = None
        if is_compressed:
            # BIP340: use x-only pubkey
            # But for correct address derivation, we MUST tweak the key
            # Q = P + hash(P||TapTweak)G
            
            # 1. Get internal key point P and ensure even Y (BIP-340)
            G = SECP256k1.generator
            P_point = vk.pubkey.point

            if P_point.y() % 2 != 0:
                # Negate Y to ensure P has even coordinate
                from ecdsa.ellipticcurve import Point
                curve = SECP256k1.curve
                p = curve.p()
                new_y = p - P_point.y()
                P_point = Point(curve, P_point.x(), new_y)

            # 2. Get x-coordinate of the even-Y point
            internal_x = P_point.x().to_bytes(32, 'big')

            # 3. Calculate Tweak Hash (BIP-341)
            tag = "TapTweak"
            tag_hash = hashlib.sha256(tag.encode()).digest()
            tweak_hash = hashlib.sha256(tag_hash + tag_hash + internal_x).digest()
            tweak_int = int.from_bytes(tweak_hash, 'big')

            # 4. Apply Tweak: Q = P + tweak * G
            Q_point = P_point + (G * tweak_int)
            
            # 4. Get tweaked x-coordinate
            output_x = Q_point.x().to_bytes(32, 'big')


            witness_prog_5bit = bech32.convertbits(output_x, 8, 5)
            # Use segwit v1 (Taproot) with bech32m encoding (BECH32M)
            p2tr_addr = local_bech32_encode(hrp, [1] + witness_prog_5bit, Encoding.BECH32M)
        
        return {
            'legacy': p2pkh_addr,
            'segwit': p2wpkh_addr,
            'taproot': p2tr_addr
        }

    except Exception as e:
        import traceback
        traceback.print_exc()
        return None

def verify_ethereum_key(hex_key):
    # Standalone CLI verification logic
    print("\n[VERIFYING ETHEREUM KEY]")
    
    if Account is None or keys is None:
        print("[ERROR] Verification Skipped. Required libraries not found.")
        print("To verify Ethereum keys, install: pip install eth-account eth-keys")
        return

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
    
    if base58 is None or bech32 is None or SECP256k1 is None:
        print("[ERROR] Verification Skipped. Required libraries not found.")
        print("To verify Bitcoin keys, install: pip install base58 bech32 ecdsa")
        return

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

