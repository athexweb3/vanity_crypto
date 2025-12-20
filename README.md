# Vanity Crypto

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Version](https://img.shields.io/github/v/release/athexweb3/vanity_crypto?label=version)
[![Best Practices](https://bestpractices.coreinfrastructure.org/projects/8157/badge)](https://bestpractices.coreinfrastructure.org/projects/8157)
![EIP-55](https://img.shields.io/badge/EIP--55-Compliant-success)

![Linux CI](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-linux.yml/badge.svg)
![macOS CI](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-macos.yml/badge.svg)
![Windows CI](https://github.com/athexweb3/vanity_crypto/actions/workflows/ci-windows.yml/badge.svg)
![Security Audit](https://github.com/athexweb3/vanity_crypto/actions/workflows/security.yml/badge.svg)

![Docker Build](https://github.com/athexweb3/vanity_crypto/actions/workflows/docker-publish.yml/badge.svg)
![Benchmark](https://github.com/athexweb3/vanity_crypto/actions/workflows/benchmark.yml/badge.svg)

## Overview

Vanity Crypto is a high-performance, cryptographically secure Crypto vanity address generator implementation in Rust. It leverages parallel processing architectures (`rayon`) to maximize address generation throughput while adhering to strict security standards.

The application includes an independent verification suite rooted in the Python `eth-account` library, ensuring that all generated cryptographic material is mathematically valid and compliant with the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf) specifications.

## Key Features

*   **Multi-Threaded Performance**: Utilizes all available CPU cores for search operations.
*   **Cryptographic Soundness**: Built upon audited `k256` (ECDSA) and `sha3` (Keccak-256) libraries from the RustCrypto ecosystem.
*   **Automated Verification**: Integrated Python verification pipeline that cross-references Rust-generated keys with standard Ethereum implementations.
*   **Terminal User Interface (TUI)**: Real-time telemetry dashboard visualizing hash rate, probabilities, and search progress.
*   **Cross-Platform**: Fully supported on Linux, macOS, and Windows.

## Installation

### Prerequisites
*   Rust Toolchain (latest stable)
*   Python 3.10+ (for verification suite)

### Build from Source
```bash
git clone https://github.com/athexweb3/vanity_crypto.git
cd vanity_crypto
cargo build --release
```

## Usage

### Interactive Mode
Launch the TUI dashboard:
```bash
cargo run --release -- --prefix 0xABC
```

### Headless CLI Mode
For server environments or scripted automation:
```bash
cargo run --release -- --prefix 0xABC --no-tui
```

### Batch Generation
For fuzzing and large-scale key generation:
```bash
cargo run --release -- --generate-batch 1000
```

## Security & Verification

This project adheres to a "Verify, Do Not Trust" methodology.

### independent Verification
Every release includes a verification suite located in `tests/verify_validate/`. We recommend users perform independent audits of the binary's output.

To verify the generator's correctness against the `eth-account` reference implementation:
```bash
python3 tests/verify_validate/fuzz_test.py 1000
```

### Supply Chain Security
*   **Dependencies**: All cryptographic dependencies are pinned and come from the `RustCrypto` organization.
*   **Audits**: Continuous integration pipelines perform daily vulnerability scans using `cargo audit`.

## Contributing

We welcome contributions that align with our strict security and code quality standards. Please refer to `CONTRIBUTING.md` for the development workflow and `CODE_OF_CONDUCT.md` for community guidelines.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.
