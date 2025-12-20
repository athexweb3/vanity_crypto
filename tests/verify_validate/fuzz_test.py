
import sys
import json
import subprocess
import os

try:
    from eth_account import Account
except ImportError:
    # Try to auto-detect venv relative to this script
    script_dir = os.path.dirname(os.path.abspath(__file__))
    # Go up 2 levels: tests/verify_validate -> tests -> root
    project_root = os.path.dirname(os.path.dirname(script_dir))
    venv_python = os.path.join(project_root, ".venv", "bin", "python")
    
    if os.path.exists(venv_python) and sys.executable != venv_python:
        print(f"[INFO] 'eth_account' missing in {sys.executable}.")
        print(f"[INFO] Relaunching with venv python: {venv_python}")
        os.execv(venv_python, [venv_python] + sys.argv)
    
    print("[ERROR] 'eth_account' is not installed.")
    print("Please install requirements: pip install -r requirements.txt")
    sys.exit(1)

def run_fuzz_test(count=100):
    print(f"[INFO] Starting Fuzz Test with {count} keys...")
    
    # Path to binary (assuming debug build for dev)
    # Move up two levels from 'tests/verify_validate' to project root
    project_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    
    binary_name = "vc"
    if os.name == 'nt':
        binary_name += ".exe"

    binary_path = os.path.join(project_root, "target", "debug", binary_name)
    
    if not os.path.exists(binary_path):
        print(f"[ERROR] Binary not found at: {binary_path}")
        print("Run 'cargo build' first.")
        # Try to list directory to help debugging
        debug_dir = os.path.join(project_root, "target", "debug")
        if os.path.exists(debug_dir):
            print(f"Contents of {debug_dir}:")
            try:
                print(os.listdir(debug_dir))
            except:
                pass
        sys.exit(1)

    print(f"   Using binary: {binary_path}")
    print("   Generating batch...")

    try:
        # Run Rust binary in batch mode
        result = subprocess.run(
            [binary_path, "--generate-batch", str(count)], 
            capture_output=True, 
            text=True,
            check=True
        )
    except subprocess.CalledProcessError as e:
        print(f"❌ Rust binary failed: {e}")
        print(e.stderr)
        sys.exit(1)

    # Parse output lines
    lines = result.stdout.strip().split('\n')
    print(f"   Received {len(lines)} keys. Verifying...")
    print("")

    passed = 0
    failed = 0

    for i, line in enumerate(lines):
        if not line.strip(): continue
        try:
            data = json.loads(line)
            rust_pk = data['pk'].strip()
            rust_addr = data['addr'].strip()

            # Verify with Python eth_account
            # Ensure 0x prefix for eth_account
            if not rust_pk.startswith("0x"):
                pk_input = "0x" + rust_pk
            else:
                pk_input = rust_pk

            account = Account.from_key(pk_input)
            py_addr = account.address.lower()
            
            # Normalize rust address
            rust_addr_norm = rust_addr.lower()

            if py_addr == rust_addr_norm:
                passed += 1
            else:
                failed += 1
                print(f"[FAIL] Mismatch at index {i}:")
                print(f"   Rust PK:   {rust_pk}")
                print(f"   Rust Addr: {rust_addr}")
                print(f"   Py Addr:   {account.address}")
                print("")

        except Exception as e:
            failed += 1
            print(f"[ERROR] Processing line '{line}': {e}")

    print("-" * 40)
    print(f"PASSED: {passed}")
    print(f"FAILED: {failed}")
    print("-" * 40)
    print("")

    if failed == 0 and passed > 0:
        print("[SUCCESS] Fuzz Test Passed: All keys match.")
        sys.exit(0)
    else:
        print("[FAILED] Fuzz Test Failed.")
        sys.exit(1)

if __name__ == "__main__":
    count = 100
    if len(sys.argv) > 1:
        try:
            count = int(sys.argv[1])
        except:
            pass
    run_fuzz_test(count)
