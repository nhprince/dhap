# Dhap (ধাপ) — Development Guide

---

## 1. Prerequisites

- **Rust:** Stable 1.85+ (tested on `rustc 1.99.1`) with `cargo` and `rustfmt`.
- **Node.js:** v20+ (tested on `v24.20.0`) and `npm 10+` (for website/UI components).
- **FFmpeg:** 6.0+ (tested on `FFmpeg 9.0.1`) present in system `PATH`.
- **Git & GitHub CLI:** Git 2.40+ and `gh` 2.50+.

---

## 2. Repository Structure

```text
dhap/
├── Cargo.toml            # Rust workspace definition
├── Cargo.lock            # Pinned dependency graph
├── crates/
│   ├── dhap-core/        # Data entities, identifiers, common errors
│   ├── dhap-project/     # .dhap ZIP packaging, schema, atomic writes
│   ├── dhap-media/       # FFmpeg wrapper, frame extraction, scene detection
│   ├── dhap-annotation/  # Vector annotations, pixel redactions (image crate)
│   ├── dhap-export/      # Markdown & standalone HTML exporters
│   ├── dhap-ai/          # Optional AI provider trait and mocks
│   └── dhap-cli/         # Headless CLI executable (dhap)
├── apps/
│   └── site/             # Static showcase website and documentation
├── fixtures/             # Deterministic synthetic test media & projects
├── docs/                 # Architecture, specifications, decisions, security
└── .github/
    └── workflows/        # Pinned GitHub Actions (CI, Release, Deploy)
```

---

## 3. Common Development Commands

### Building the Workspace
```powershell
cargo build --workspace
```

### Running All Unit and Integration Tests
```powershell
cargo test --workspace -- --nocapture
```

### Checking Formatting and Lints
```powershell
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Running the Dhap CLI Locally
```powershell
cargo run -p dhap-cli -- --help
cargo run -p dhap-cli -- version
```

### Testing Project Validation and Export via CLI
```powershell
cargo run -p dhap-cli -- validate fixtures/sample_project.dhap
cargo run -p dhap-cli -- export fixtures/sample_project.dhap --format markdown,html --output-dir target/demo_export
```
