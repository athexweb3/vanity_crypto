#!/usr/bin/env python3
import sys
from verifier.chains.ethereum import verify_ethereum_key
from verifier.chains.bitcoin import verify_bitcoin_key
from verifier.chains.solana import verify_solana_key

from verifier.chains.ton import verify_ton_key
from verifier.chains.cosmos import verify_cosmos_key

def main():
    target_chain = None
    if len(sys.argv) > 1:
        # Argument provided
        print("\n--- Vanity Crypto Key Verifier ---")
        input_key = sys.argv[1].strip()
        if len(sys.argv) > 2:
            target_chain = sys.argv[2].strip().lower()
            if target_chain.startswith('"') and target_chain.endswith('"'):
                target_chain = target_chain[1:-1] # Remove quotes if passed by Debug format
    elif not sys.stdin.isatty():
        # Piped input
        print("\n--- Vanity Crypto Key Verifier ---")
        input_key = sys.stdin.read().strip()
    else:
        # Interactive mode
        print("\n--- Vanity Crypto Key Verifier ---")
        input_key = input("Enter Private Key (Hex or WIF): ").strip()
        target_chain = input("Target Chain (optional, e.g. ton, ethereum): ").strip().lower()

    if target_chain == "ton":
        verify_ton_key(input_key)
        return
    elif target_chain == "ethereum":
        verify_ethereum_key(input_key)
        return
    elif target_chain == "solana":
        verify_solana_key(input_key)
        return
    elif target_chain == "bitcoin":
        verify_bitcoin_key(input_key)

    elif target_chain == "cosmos":
        verify_cosmos_key(input_key)
        return

    # Auto-detection fallback
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
