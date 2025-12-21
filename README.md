# Vanity Crypto

![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)
![Language](https://img.shields.io/badge/language-Rust-orange.svg?style=flat-square)
[![NPM Version](https://img.shields.io/npm/v/vanity_crypto.svg?style=flat-square)](https://www.npmjs.com/package/vanity_crypto)
![CI - Linux](https://img.shields.io/github/actions/workflow/status/athexweb3/vanity_crypto/ci-linux.yml?label=Linux&style=flat-square)
![CI - macOS](https://img.shields.io/github/actions/workflow/status/athexweb3/vanity_crypto/ci-macos.yml?label=macOS&style=flat-square)
![CI - Windows](https://img.shields.io/github/actions/workflow/status/athexweb3/vanity_crypto/ci-windows.yml?label=Windows&style=flat-square)

**Vanity Crypto** is a high-performance, cryptographically secure Ethereum vanity address generator. It is designed to be the fastest and safest tool of its kind, leveraging Rust's safety guarantees and Rayon's parallelism.

## Design Goals

-   **Performance**: Utilizes all available CPU cores to maximize hash rate (MH/s).
-   **Security**: "Verify, Don't Trust." Every generated key is mathematically verifiable against standard Ethereum implementations.
-   **Experience**: A beautiful, real-time Terminal User Interface (TUI) that provides feedback without clutter.
-   **Portability**: First-class support for macOS (Apple Silicon/Intel), Linux, and Windows.

---

## Installation

### 1. NPM (Recommended)
The quickest way to install for Node.js users.
```bash
npm install -g vanity_crypto
```
*Or run purely without installation:*
```bash
npx vanity_crypto
```

### 2. Homebrew (macOS / Linux)
Install via our official tap.
```bash
brew tap athexweb3/vanity_crypto https://github.com/athexweb3/vanity_crypto
brew install vanity_crypto
```

### 3. Scoop (Windows)
Install via our official bucket.
```bash
scoop bucket add vanity_crypto https://github.com/athexweb3/vanity_crypto
scoop install vanity_crypto
```

### 4. Build from Source
Required for running the Python Verification Suite.
```bash
git clone https://github.com/athexweb3/vanity_crypto.git
cd vanity_crypto
cargo build --release
```

---

## Usage

### Interactive Mode
Simply run `vc` to launch the interactive TUI.
```bash
vc
```

### Command Line Arguments
*(Coming in v1.0)*

---

## Security & Verification

We believe cryptographic tools should be audible.

**Generation**: Keys are generated using strict adherence to `k256` (ECDSA) and `sha3` (Keccak-256) standards.

**Verification**:
To perform an independent audit of the generator's integrity, check out the source code and run the verification suite. This uses the Python `eth-account` library to cross-validate results.

```bash
# Requires Python 3.10+
python3 tests/verify_validate/fuzz_test.py 1000
```
*Note: The binary release does not include the Python test scripts. You must clone the repo to run manual verification.*

---

## License

The code is licensed under the [MIT License](LICENSE).

Copyright (c) 2025 Athex Web3.
