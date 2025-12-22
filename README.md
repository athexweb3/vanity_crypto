# Vanity Crypto

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/github/v/release/athexweb3/vanity_crypto?label=version)](https://github.com/athexweb3/vanity_crypto/releases)

[![CI - Linux](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-linux.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-linux.yml)
[![CI - macOS](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-macos.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-macos.yml)
[![CI - Windows](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-windows.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-windows.yml)
[![Benchmark](https://github.com/athexweb3/vanity_crypto/actions/workflows/benchmark.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/benchmark.yml)
[![Security Audit](https://github.com/athexweb3/vanity_crypto/actions/workflows/security.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/security.yml)
[![CodeQL](https://github.com/athexweb3/vanity_crypto/actions/workflows/codeql/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/codeql)

[![NPM](https://img.shields.io/badge/npm-supported-CB3837.svg?logo=npm&logoColor=white)](https://www.npmjs.com/package/vanity_crypto)
[![Homebrew](https://img.shields.io/badge/homebrew-supported-FBB040.svg?logo=homebrew&logoColor=white)](https://github.com/athexweb3/vanity_crypto/tree/main/Formula)
[![Scoop](https://img.shields.io/badge/scoop-supported-4084D0.svg?logo=windows&logoColor=white)](https://github.com/athexweb3/vanity_crypto/tree/main/scoop)

[![CI - Linux](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-linux.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-linux.yml)
[![CI - macOS](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-macos.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-macos.yml)
[![CI - Windows](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-windows.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-windows.yml)
[![Security Audit](https://github.com/athexweb3/vanity_crypto/actions/workflows/security.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/security.yml)
[![Benchmark](https://github.com/athexweb3/vanity_crypto/actions/workflows/benchmark.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/benchmark.yml)

**Vanity Crypto** is a high-performance, cryptographically secure vanity address generator for **Ethereum** and **Bitcoin**. It is engineered to provide the highest possible search throughput on consumer hardware while maintaining strict distinctness of duties between key generation and verification.

The library strictly adheres to the following standards:

### Ethereum
*   **[EIP-55](https://eips.ethereum.org/EIPS/eip-55)**: Mixed-case checksum address encoding.
*   **[NIST FIPS 202](https://csrc.nist.gov/publications/detail/fips/202/final)**: SHA-3 Standard (Keccak-256).

### Bitcoin
*   **[BIP-173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki)**: SegWit Bech32 address format.
*   **[BIP-350](https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki)**: Taproot Bech32m address format.
*   **[BIP-58](https://github.com/bitcoin/bips/blob/master/bip-0058.mediawiki)**: Base58Check encoding for Legacy addresses.
*   **[SEC 1](https://www.secg.org/sec1-v2.pdf)**: Elliptic Curve Cryptography (secp256k1).

## Architecture

The project employs a specific **Verify-after-Generate** architecture to eliminate single points of failure in the cryptographic logic.

1.  **Entropy & Generation (Rust)**:
    Using the `rand::OsRng` system entropy source, a 256-bit private key is generated. The corresponding public key and address are derived via RustCrypto or libsecp256k1. This process is parallelized across all logical CPU cores using a work-stealing scheduler (`rayon`).

2.  **Cross-Verification (Python)**:
    Upon identifying a candidate address matching the user's constraints, the key material is passed to an isolated subprocess. This process invokes reference Python implementations (`eth_account` for Ethereum, `base58`/`bech32` for Bitcoin) to independently re-derive the address from the private key.

3.  **Validation**:
    The result is presented to the user **if and only if** the Rust-derived address and the Python-derived address are bitwise identical.

## Installation

### NPM (Node.js)
The suggested installation method for most users. This wrapper automatically downloads the correct architecture-specific binary for your system.

```bash
npm install -g vanity_crypto
```

### Homebrew (macOS / Linux)
Distributed via a focused Tap to distinguish it from unverified tools.

```bash
brew tap athexweb3/vanity_crypto https://github.com/athexweb3/vanity_crypto
brew install vanity_crypto
```

### Scoop (Windows)
Distributed via a dedicated Bucket.

```bash
scoop bucket add vanity_crypto https://github.com/athexweb3/vanity_crypto
scoop install vanity_crypto
```

### Building from Source

To build from source, a standard Rust toolchain (stable) is required. To run the verification suite, Python 3.10+ is also required.

```bash
git clone https://github.com/athexweb3/vanity_crypto.git
cd vanity_crypto
cargo build --release
```

## Usage

### Interactive Mode (TUI)
The binary launches into an interactive Terminal User Interface by default, providing real-time telemetry on hash rate and probability.

```bash
vc
```
*Shortcuts:* 
- `Ctrl+Enter` / `Cmd+Enter`: Start search
- `q` / `Esc`: Quit/Exit

### CLI Arguments
For integration into automated pipelines, the CLI accepts arguments to bypass the TUI.

```bash
# Ethereum (default)
vc --chain ethereum --prefix dead

# Bitcoin (Legacy)
vc --chain bitcoin --btc-type legacy --prefix 1bad

# Bitcoin (SegWit)
vc --chain bitcoin --btc-type segwit --prefix bc1q

# Bitcoin (Taproot)
vc --chain bitcoin --btc-type taproot --prefix bc1p
```

| Argument | Description |
| :--- | :--- |
| `--chain <ethereum\|bitcoin>` | Select the blockchain network (Default: ethereum). |
| `--prefix <STRING>` | The case-insensitive string to search for. |
| `--btc-type <legacy\|segwit\|taproot>` | **[Bitcoin]** The address type to generate. |
| `--case-sensitive` | Strictly enforce casing (e.g. `DeaD` vs `dead`). |
| `--threads <N>` | Override thread count (Default: logical core count). |
| `--no-tui` | Disable the TUI and output only the final result JSON. |

## Independent Verification

Trust in cryptographic tools must be earned through verification. We provide a fuzzing suite that compares thousands of iterations of the Rust generator against the Python reference implementation.

To run the audit:

```bash
# Requires Python 3.10+
# Install deps: pip install -r tests/verify_validate/requirements.txt

# Audit Ethereum
python3 tests/verify_validate/fuzz_test.py --chain ethereum

# Audit Bitcoin (Legacy)
python3 tests/verify_validate/fuzz_test.py --chain bitcoin --btc-type legacy

# Audit Bitcoin (Taproot/Schnorr)
python3 tests/verify_validate/fuzz_test.py --chain bitcoin --btc-type taproot
```

## License

Copyright (c) 2025 Athex Web3.
This project is licensed under the **MIT License**.

See [LICENSE](LICENSE) for more information.
