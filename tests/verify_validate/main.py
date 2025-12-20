
import sys
import binascii

try:
    from eth_account import Account
except ImportError:
    # Try to auto-detect venv relative to this script
    import os
    script_dir = os.path.dirname(os.path.abspath(__file__))
    # Go up 2 levels: tests/verify_validate -> tests -> root
    project_root = os.path.dirname(os.path.dirname(script_dir))
    venv_python = os.path.join(project_root, ".venv", "bin", "python")
    
    if os.path.exists(venv_python) and sys.executable != venv_python:
        # Re-launch
        os.execv(venv_python, [venv_python] + sys.argv)

    print("[ERROR] 'eth_account' is not installed.")
    print("Please install requirements: pip install -r requirements.txt")
    sys.exit(1)

def verify_private_key(hex_key):
    # Clean the input
    hex_key = hex_key.strip().replace("0x", "")
    
    # Check length
    if len(hex_key) != 64:
        print(f"❌ INVALID LENGTH: Private key must be 64 hex characters (32 bytes). Found {len(hex_key)}.")
        return

    print("")
    try:
        # Add 0x back for eth_account
        pk_str = "0x" + hex_key
        account = Account.from_key(pk_str)
        
        print("[VALID] Private Key Verified")
        print(f"   Private Key: {pk_str}")
        print(f"   Address:     {account.address}")
        print("\n   Note: Key is mathematically valid and importable.")
        print("")
        
    except Exception as e:
        print(f"[INVALID] Key Error: {e}")
        print("")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # Argument provided
        print("\n--- Vanity Crypto Key Verifier ---")
        key = sys.argv[1]
        verify_private_key(key)
    elif not sys.stdin.isatty():
        # Piped input (no print banner/prompt to keep it clean)
        print("\n--- Vanity Crypto Key Verifier ---")
        key = sys.stdin.read().strip()
        verify_private_key(key)
    else:
        # Interactive mode
        print("\n--- Vanity Crypto Key Verifier ---")
        key = input("Enter Private Key to verify: ")
        verify_private_key(key)
