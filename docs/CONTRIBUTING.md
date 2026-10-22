# Contributing to Dhap (ধাপ)

Thank you for your interest in contributing to Dhap! Dhap is an open-source project dedicated to making workflow documentation clear, beautiful, and accessible to everyone.

---

## 1. Code of Conduct

We are committed to providing a welcoming, inclusive, and harassment-free experience for everyone. Please be respectful, constructive, and considerate in all discussions, pull requests, and issue reports.

---

## 2. Getting Started

1. Fork the repository on GitHub: `https://github.com/nhprince/dhap`.
2. Clone your fork locally.
3. Verify your environment meets the prerequisites:
   - Rust 1.85+ (`cargo`)
   - FFmpeg 6.0+
   - Node.js 20+
4. Run tests to confirm a healthy baseline:
   ```bash
   cargo test --workspace
   ```

---

## 3. Pull Request Guidelines

- **Atomic Commits:** Make focused commits using conventional commit syntax:
  - `feat: add scene detection threshold parameter`
  - `fix: prevent path traversal in bundle extractor`
  - `docs: update export checklist documentation`
  - `test: add golden fixture for markdown tables`
- **Tests Required:** Any new domain feature or bug fix must include automated unit or integration tests.
- **Code Quality:** Ensure all checks pass prior to submitting a PR:
  ```bash
  cargo fmt --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace
  ```
- **License Integrity:** All contributions are submitted under the project's MIT License. Do not introduce dependencies with incompatible copyleft or commercial restriction licenses without prior discussion.
