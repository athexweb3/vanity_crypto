# Security Policy

## 🛡️ "Don't Trust, Verify"

We adhere to the highest standards of cryptographic transparency. As a tool that generates private keys, trust is earned through verification, not promises.

### 1. Independent Verification
We provide a built-in, isolated verification suite that allows you to mathematically prove the validity of generated keys without relying on the Rust binary alone.

*   **Location**: `tests/verify_validate/main.py`
*   **Method**: Uses the standard Python `eth-account` library to independently derive the address from the private key.
*   **Usage**:
    ```bash
    cargo run --release -- --prefix 0xABC --no-tui
    # The tool automatically runs the python verifier on exit.
    ```

### 2. Supply Chain Security
*   **Dependencies**: We strictly limit our dependency tree. All cryptographic primitives come from the audited [RustCrypto](https://github.com/RustCrypto) project (`k256`, `sha3`).
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
