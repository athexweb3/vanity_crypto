#!/usr/bin/env python3
import sys
from verifier.chains.ethereum import verify_ethereum_key
from verifier.chains.bitcoin import verify_bitcoin_key
from verifier.chains.solana import verify_solana_key

def main():
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
    
    # Detect input type (WIF vs Hex vs Base58)
    # Ethereum: 64 hex chars (32 bytes) or 0x prefix
    if input_key.startswith("0x") or (len(input_key) == 64 and all(c in '0123456789abcdefABCDEF' for c in input_key)):
        verify_ethereum_key(input_key)
    # Solana: Base58 string of ~87-88 chars (64 bytes)
    elif len(input_key) > 80:
        verify_solana_key(input_key)
    # Bitcoin: WIF (Base58)
    else:
        verify_bitcoin_key(input_key)

if __name__ == "__main__":
    main()
