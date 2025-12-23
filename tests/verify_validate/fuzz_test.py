
import sys
import json
import subprocess
import os
import argparse

# Import verification logic
# Append current directory to path
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
try:
    from verifier.chains.bitcoin import get_bitcoin_address
    from verifier.chains.ethereum import Account
    from verifier.chains.solana import get_solana_address
except ImportError as e:
    print(f"[ERROR] Could not import verifier logic: {e}")
    sys.exit(1)

def run_fuzz_test(count, chain, network, btc_type):
    print(f"\n[INFO] Starting Fuzz Test with {count} keys...")
    print(f"       Chain: {chain}, Network: {network}", end="")
    if chain == 'bitcoin':
        print(f", Type: {btc_type}")
    else:
        print("")
    
    # Path to binary
    project_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    binary_name = "vc"
    if os.name == 'nt': binary_name += ".exe"
    binary_path = os.path.join(project_root, "target", "debug", binary_name)
    
    if not os.path.exists(binary_path):
        print(f"[ERROR] Binary not found at: {binary_path}")
        print("Run 'cargo build' first.")
        sys.exit(1)

    # Build command
    cmd = [binary_path, "--generate-batch", str(count)]
    cmd.extend(["--chain", chain])
    cmd.extend(["--network", network])
    if chain == 'bitcoin':
        cmd.extend(["--btc-type", btc_type])

    print("   Generating batch...")
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
    except subprocess.CalledProcessError as e:
        print(f"❌ Rust binary failed: {e}")
        print(e.stderr)
        sys.exit(1)

    lines = result.stdout.strip().split('\n')
    valid_lines = [l for l in lines if l.strip()]
    print(f"   Received {len(valid_lines)} keys. Verifying...")

    passed = 0
    failed = 0

    for i, line in enumerate(valid_lines):
        try:
            data = json.loads(line)
            rust_pk = data['pk'].strip()
            rust_addr = data['addr'].strip()

            if chain == 'ethereum':
                if not rust_pk.startswith("0x"): rust_pk = "0x" + rust_pk
                account = Account.from_key(rust_pk)
                py_addr = account.address.lower()
                rust_addr_norm = rust_addr.lower()
                
                if py_addr == rust_addr_norm:
                    passed += 1
                else:
                    failed += 1
                    print(f"FAIL Mismatch: Rust({rust_addr}) vs Py({account.address})")

            elif chain == 'bitcoin':
                # rust_pk is WIF
                matches = get_bitcoin_address(rust_pk)
                if not matches:
                    print(f"FAIL Invalid WIF: {rust_pk}")
                    failed += 1
                    continue
                
                # Check against specific type
                expected_addr = None
                if btc_type == 'legacy': expected_addr = matches['legacy']
                elif btc_type == 'segwit': expected_addr = matches['segwit']
                elif btc_type == 'taproot': expected_addr = matches['taproot']
                
                if expected_addr == rust_addr:
                    passed += 1
                else:
                    failed += 1
                    print(f"[FAIL] Mismatch at index {i}:")
                    print(f"   Rust Addr: {rust_addr}")
                    print(f"   Py Addr:   {expected_addr}")

                    print(f"   All Derivations: {matches}")
                    print("")
            
            elif chain == 'solana':
                py_addr = get_solana_address(rust_pk)
                if not py_addr:
                    print(f"FAIL Invalid Solana Key: {rust_pk}")
                    failed += 1
                    continue
                
                if py_addr == rust_addr:
                    passed += 1
                else:
                    failed += 1
                    print(f"[FAIL] Mismatch at index {i}:")
                    print(f"   Rust Addr: {rust_addr}")
                    print(f"   Py Addr:   {py_addr}")
                    print("")

        except Exception as e:
            failed += 1
            print(f"[ERROR] Processing line '{line}': {e}")

    print("-" * 40)
    print(f"PASSED: {passed}")
    print(f"FAILED: {failed}")
    print("-" * 40)

    if failed == 0 and passed > 0:
        print("[SUCCESS] Fuzz Test Passed: All keys match.")
        sys.exit(0)
    else:
        print("[FAILED] Fuzz Test Failed.")
        sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Vanity Crypto Fuzz Test")
    parser.add_argument("--count", type=int, default=100, help="Number of keys to generate")
    parser.add_argument("--chain", type=str, default="ethereum", choices=["ethereum", "bitcoin", "solana"])
    parser.add_argument("--network", type=str, default="mainnet", choices=["mainnet", "testnet", "regtest"])
    parser.add_argument("--btc-type", type=str, default="segwit", choices=["legacy", "segwit", "taproot"])
    
    args = parser.parse_args()
    
    run_fuzz_test(args.count, args.chain, args.network, args.btc_type)
