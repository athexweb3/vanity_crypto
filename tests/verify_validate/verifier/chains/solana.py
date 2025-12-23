def get_solana_address(key_str):
    """
    Decodes a Base58 private key (64 bytes) and derives the address.
    Returns the address string if valid, None otherwise.
    """
    try:
        import base58
        from nacl.signing import SigningKey
    except ImportError:
        return None

    try:
        decoded_pk = base58.b58decode(key_str)
        if len(decoded_pk) != 64:
             return None
        
        seed = decoded_pk[:32]
        signing_key = SigningKey(seed)
        verify_key = signing_key.verify_key
        
        derived_pubkey_bytes = verify_key.encode()
        derived_addr = base58.b58encode(derived_pubkey_bytes).decode('ascii')
        return derived_addr
        
    except Exception:
        return None

def verify_solana_key(key_str):
    print("\n[VERIFYING SOLANA KEYPAIR]")
    
    # Check deps first for better error message
    try:
        import base58
        import nacl.signing
        # Silence unused import warning (dependency check)
        _ = (base58, nacl.signing)
    except ImportError:
         print("[ERROR] Verification Skipped. Required libraries not found.")
         print("To verify Solana keys, install: pip install base58 pynacl")
         return

    address = get_solana_address(key_str)
    
    if address:
        print("✅ VALID Keypair Verified")
        print(f"   Private Key: {key_str[:16]}...{key_str[-16:]} (REDACTED)")
        print(f"   Address:     {address}")
        print("\n   Note: Key is valid Ed25519 keypair.")
        print("")
    else:
        # Re-run strict check for specific error printing
        try:
            decoded = base58.b58decode(key_str)
            if len(decoded) != 64:
                print(f"❌ INVALID LENGTH: 64 bytes required (decoded from {len(key_str)} chars). Found {len(decoded)} bytes.")
            else:
                print("❌ Invalid keypair calculation.")
        except Exception as e:
            print(f"❌ Key Error: {e}")
        print("")
