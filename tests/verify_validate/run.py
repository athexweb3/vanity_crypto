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
    else:
        # Try to decode as Base58 to check for Solana or Bitcoin
        try:
            import base58
            decoded = base58.b58decode(input_key)
            # Solana keypairs are 64 bytes
            if len(decoded) == 64:
                verify_solana_key(input_key)
            else:
                # Assume Bitcoin WIF otherwise
                verify_bitcoin_key(input_key)
        except Exception:
            # If base58 decoding fails, it's likely not a valid Solana or Bitcoin key,
            # but we can let the Bitcoin verifier give the final error.
            verify_bitcoin_key(input_key)

if __name__ == "__main__":
    main()
