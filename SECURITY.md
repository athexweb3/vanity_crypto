# Security Policy

## 🛡️ "Don't Trust, Verify"

We adhere to the highest standards of cryptographic transparency. As a tool that generates private keys, trust is earned through verification, not promises.

### 1. Independent Verification
We provide a built-in, isolated verification suite that allows you to mathematically prove the validity of generated keys without relying on the Rust binary alone.

*   **Location**: `tests/verify_validate/main.py`
*   **Method**: Uses standard Python libraries to independently derive addresses:
    *   **Ethereum**: `eth-account` (Reference implementation)
    *   **Bitcoin**: `base58` (BIP-58), `bech32` (BIP-173, BIP-350), and pure Python Elliptic Curve math.
*   **Usage**:
    ```bash
    # Run the fuzzing suite
    python3 tests/verify_validate/fuzz_test.py --chain bitcoin --btc-type taproot
    ```

### 2. Supply Chain Security
*   **Dependencies**: We strictly limit our dependency tree. All cryptographic primitives come from audited sources:
    *   **Ethereum**: [RustCrypto](https://github.com/RustCrypto) (`k256`, `sha3`)
    *   **Bitcoin**: [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin) (`bitcoin`, `secp256k1`)
*   **Lockfile**: `Cargo.lock` is committed to ensure reproducible builds.
*   **No Network**: This tool is designed to be **offline-first**. It makes zero network requests. You should run it on an air-gapped machine for maximum security.

### 3. Reporting Vulnerabilities

If you discover a security vulnerability (e.g., weak entropy, side-channel attack, panic behavior), please report it responsibly.

*   **Email**: `athexweb3@gmail.com`
*   **GPG Key**: Run `gpg --locate-keys athexweb3@gmail.com` to fetch the latest signing key.
*   **Policy**: We pledge to acknowledge reports within 48 hours and provide a timeline for fixes.

**DO NOT** open public GitHub issues for critical security vulnerabilities that could affect users' funds.

## Disclaimer
This software is provided "as is", without warranty of any kind. You are solely responsible for the safe custody of your private keys.
