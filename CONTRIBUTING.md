# Contributing

We welcome contributions from the cypherpunk and open-source community! 🏴

Because this tool handles sensitive cryptographic material, we have strict standards for contributions. Please read this carefully.

## 🚨 The Golden Rule: Security First

**Performance is secondary to Correctness.**
Any PR that optimizes speed but introduces even a theoretical risk to key generation entropy or correctness will be rejected.

## 🛠️ Development Workflow

### 1. Verification is Mandatory
We have a "Giant Testing System" for a reason. Before pushing, you **MUST** run the full fuzzing suite.

```bash
# 1. Run Standard Tests
cargo test

# 2. Run Fuzzing Suite (Validate 1000 keys against Python)
# This proves your changes didn't break address derivation.
python3 tests/verify_validate/fuzz_test.py 1000
```

### 2. Dependency Policy
*   **Minimalism**: Do not add new dependencies unless absolutely necessary.
*   **Auditability**: Use pure Rust libraries where possible. Avoid C-bindings that complicate the build chain.
*   **Crypto**: Only use `RustCrypto` or similarly audited libraries. Rolling your own crypto primitives is strictly forbidden.

### 3. Code Signing (Optional but Recommended)
We encourage signing your commits with GPG to verify authorship.
```bash
git commit -S -m "feat: secure implementation"
```

## 🐛 Bug Reports

When reporting bugs, please include:
1.  **Reproduction**: A command or script to trigger the issue.
2.  **Environment**: OS (Mac/Linux/Windows) and CPU architecture.
3.  **Logs**: Output from the terminal.

**Note**: If the bug is a security vulnerability, see [SECURITY.md](SECURITY.md).

## 📝 Style Guide

*   **Rust**: Run `cargo fmt` and `cargo clippy` before submitting.
*   **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) (e.g., `feat:`, `fix:`, `docs:`, `security:`).

## ⚖️ License
By contributing, you agree that your code will be licensed under the MIT License.
