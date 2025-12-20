# Vanity Crypto 🛡️

A high-performance, secure, and beautiful Ethereum vanity address generator written in Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Status](https://img.shields.io/badge/status-stable-green.svg)

## ✨ Features

- **🚀 High Performance**: Multi-threaded generation utilizing all CPU cores (built on `rayon`).
- **🖥️ Interactive TUI**: Beautiful terminal dashboard to track progress, speed, and probability (built on `ratatui`).
- **🛡️ Cryptographically Secure**: Uses audited `k256` (ECDSA) and `sha3` (Keccak-256) libraries.
- **✅ Verifiable**: Includes automated Python verification scripts to mathematically prove key validity.
- **✨ Copy-Friendly**: Smart output formatting for easy, safe private key copying.

## 📦 Installation

### Prerequisites
- [Rust Toolchain](https://rustup.rs/) (cargo)
- Python 3 (optional, for verification)

### Build from Source
```bash
git clone https://github.com/athexweb3/vanity_crypto.git
cd vanity_crypto
cargo build --release
```

## 🚀 Usage

### Interactive Mode (Recommended)
Run the tool without arguments or with a starting prefix to enter the interactive dashboard:

```bash
cargo run --release -- --prefix 0xABC
```
*Press `q` to quit or stop searching.*

### Headless CLI Mode
For scripting or servers, use the `--no-tui` flag:

```bash
cargo run --release -- --prefix 0xABC --no-tui
```

### Batch Generation (Fuzzing/Testing)
Generate raw JSON keys for testing:
```bash
cargo run --release -- --generate-batch 5
```

## 🔒 Security & Verification

Security is our #1 priority.

1.  **Audited Libraries**: We do not roll our own crypto. We use `RustCrypto/elliptic-curves` and `RustCrypto/hashes`.
2.  **Automated Verification**: Every generated key is automatically verified against the standard `eth-account` Python library if installed. [See SECURITY.md](SECURITY.md) for full audit details.
3.  **Correctness Proof**:
    Run our fuzzing suite to verify thousands of keys:
    ```bash
    python3 tests/verify_validate/fuzz_test.py 1000
    ```

## ⚠️ Security Warning

**Treat your private keys with extreme caution.**
- We recommend running this tool on an **air-gapped (offline)** machine for significant assets.
- Always verify the generated key with a small transaction before sending large amounts.
- **NEVER** share your private key or screenshot the output.

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
