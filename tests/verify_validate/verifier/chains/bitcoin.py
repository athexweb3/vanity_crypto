import hashlib
from ..utils import hash160, local_bech32_encode, Encoding


# Optional Dependencies
base58 = None
bech32 = None
SECP256k1 = None
SigningKey = None

try:
    import base58
    import bech32
    from ecdsa import SECP256k1, SigningKey
except ImportError:
    pass

def get_bitcoin_address(wif_key):
    """
    Decodes WIF and returns a dictionary of derived addresses:
    { 'legacy': ..., 'segwit': ..., 'taproot': ... }
    Returns None if WIF is invalid or deps missing.
    """
    if not (base58 and bech32 and SECP256k1 and SigningKey):
        return None

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

    except Exception:
        import traceback
        traceback.print_exc()
        return None

def verify_bitcoin_key(wif_key):
    print("\n[VERIFYING BITCOIN WIF]")
    
    if not (base58 and bech32 and SECP256k1):
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
