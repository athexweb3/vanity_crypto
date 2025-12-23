# Optional Dependencies
Account = None
keys = None

try:
    from eth_account import Account
    from eth_keys import keys
except ImportError:
    pass

def verify_ethereum_key(hex_key):
    # Standalone CLI verification logic
    print("\n[VERIFYING ETHEREUM KEY]")
    
    if Account is None or keys is None:
        print("[ERROR] Verification Skipped. Required libraries not found.")
        print("To verify Ethereum keys, install: pip install eth-account eth-keys")
        return

    hex_key = hex_key.strip().replace("0x", "")
    if len(hex_key) != 64:
        print("❌ INVALID LENGTH: 64 hex chars required.")
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
