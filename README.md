# Vanity Crypto

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/github/v/release/athexweb3/vanity_crypto?label=version)](https://github.com/athexweb3/vanity_crypto/releases)

[![CI - Linux](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-linux.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-linux.yml)
[![CI - macOS](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-macos.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-macos.yml)
[![CI - Windows](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-windows.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-windows.yml)
[![Benchmark](https://github.com/athexweb3/vanity_crypto/actions/workflows/benchmark.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/benchmark.yml)
[![Security Audit](https://github.com/athexweb3/vanity_crypto/actions/workflows/security.yml/badge.svg)](https://github.com/athexweb3/vanity_crypto/actions/workflows/security.yml)


[![NPM](https://img.shields.io/badge/npm-supported-CB3837.svg?logo=npm&logoColor=white)](https://www.npmjs.com/package/vanity_crypto)
[![Homebrew](https://img.shields.io/badge/homebrew-supported-FBB040.svg?logo=homebrew&logoColor=white)](https://github.com/athexweb3/vanity_crypto/tree/main/Formula)
[![Scoop](https://img.shields.io/badge/scoop-supported-4084D0.svg?logo=windows&logoColor=white)](https://github.com/athexweb3/vanity_crypto/tree/main/scoop)


**Vanity Crypto** is a high-performance, cryptographically secure vanity address generator for **Ethereum**, **Bitcoin**, and **Solana**. It is engineered to provide the highest possible search throughput on consumer hardware while maintaining strict distinctness of duties between key generation and verification.

The library strictly adheres to the following standards:

### Ethereum
*   **[EIP-55](https://eips.ethereum.org/EIPS/eip-55)**: Mixed-case checksum address encoding.
*   **[NIST FIPS 202](https://csrc.nist.gov/publications/detail/fips/202/final)**: SHA-3 Standard (Keccak-256).

### Bitcoin
*   **[BIP-173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki)**: SegWit Bech32 address format.
*   **[BIP-350](https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki)**: Taproot Bech32m address format.
*   **[BIP-58](https://github.com/bitcoin/bips/blob/master/bip-0058.mediawiki)**: Base58Check encoding for Legacy addresses.
*   **[SEC 1](https://www.secg.org/sec1-v2.pdf)**: Elliptic Curve Cryptography (secp256k1).

### Solana
*   **[Ed25519](https://ed25519.cr.yp.to/)**: High-speed Edwards-curve Digital Signature Algorithm.
*   **[Base58](https://learn.bybit.com/blockchain/what-is-base58/)**: Standard Solana address encoding.

### TON (The Open Network)
*   **[V4R2](https://ton.org/docs)**: Standard high-performance wallet contract (Wallet ID `0x29a9a317`).
*   **[V5R1](https://docs.ton.org/v3/guidelines/smart-contracts/wallet-v5)**: Latest W5 standard (Wallet ID `0x7fffff11`), optimized for gasless operations.
*   **Smart Addresses**: Generates non-bounceable (UQ) addresses by default, and automatically switches to bounceable (EQ) if the prefix starts with `E`.

### Cosmos
*   **[Cosmos SDK](https://docs.cosmos.network/)**: Standard `secp256k1` key generation.
*   **[BIP-173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki)**: Bech32 address format.
*   **Interchain Ready**: Supports any Cosmos chain via configurable HRP (e.g. `cosmos`, `osmo`, `juno`).

## Architecture

The project employs a specific **Verify-after-Generate** architecture to eliminate single points of failure in the cryptographic logic.

1.  **Entropy & Generation (Rust)**:
    Using the `rand::OsRng` system entropy source, a 256-bit private key is generated. The corresponding public key and address are derived via RustCrypto or libsecp256k1. This process is parallelized across all logical CPU cores using a work-stealing scheduler (`rayon`).

2.  **Cross-Verification (Python)**:
    Upon identifying a candidate address matching the user's constraints, the key material is passed to an isolated subprocess. This process invokes reference Python implementations (`eth_account` for Ethereum, `base58`/`bech32` for Bitcoin, manual derivation for TON) to independently re-derive the address from the private key.

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

# Solana
vc --chain solana --prefix abc

# TON (V4R2)
vc --chain ton --ton-version v4r2 --prefix EQA

# TON (V5R1)
vc --chain ton --ton-version v5r1 --prefix UQ

# Cosmos (Default: cosmos)
vc --chain cosmos --prefix atom

# Cosmos (Osmosis)
vc --chain cosmos --hrp osmo --prefix wow
```

| Argument | Description |
| :--- | :--- |
| `--chain <ethereum\|bitcoin\|solana\|ton>` | Select the blockchain network (Default: ethereum). |
| `--prefix <STRING>` | The case-insensitive string to search for. |
| `--btc-type <legacy\|segwit\|taproot>` | **[Bitcoin]** The address type to generate. |
| `--ton-version <v4r2\|v5r1>` | **[TON]** The wallet contract version (Default: v4r2). |
| `--hrp <STRING>` | **[Cosmos]** The Human-Readable Part (Default: cosmos). |
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

# Audit TON (V4R2 & V5R1)
python3 tests/verify_validate/fuzz_test.py --chain ton --ton-version v4r2
python3 tests/verify_validate/fuzz_test.py --chain ton --ton-version v5r1

# Audit Cosmos
python3 tests/verify_validate/fuzz_test.py --chain cosmos

```

## License

Copyright (c) 2025 Athex Web3.
This project is licensed under the **MIT License**.

See [LICENSE](LICENSE) for more information.
