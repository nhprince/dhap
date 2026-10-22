<div align="center">

# Dhap (ধাপ)

**Make the steps clear.**

*A local-first open-source studio for turning screen recordings and visual workflows into editable, illustrated documentation and media.*

[![CI](https://github.com/nhprince/dhap/actions/workflows/ci.yml/badge.svg)](https://github.com/nhprince/dhap/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

[Product Blueprint](docs/reference/DHAP_Product_Blueprint.md) • [Architecture](docs/ARCHITECTURE.md) • [Roadmap](docs/ROADMAP.md) • [Security](docs/SECURITY.md) • [Contributing](docs/CONTRIBUTING.md)

</div>

---

## 1. What is Dhap?

**Dhap** (Bengali: **ধাপ**, meaning a *step*, *stage*, or *phase* in a journey) reduces workflow documentation fragmentation. Instead of juggling separate tools for recording, screenshot extraction, image annotation, redaction, and document authoring, Dhap provides **one unified, editable source of truth**.

### The Core Promise:
> **Record once. Create the video and guide together.**

From a single project file (`.dhap`), you can simultaneously generate:
- Clean, GitHub-ready **Markdown** with local assets.
- Standalone, responsive **HTML guides** with interactive step navigation.
- Print-optimized **PDF** documentation.
- Polished **video clips and GIFs** for demos and tutorials.

---

## 2. Core Principles

- **Local-First & Offline:** Your recordings, screenshots, transcripts, and guides remain strictly on your machine.
- **Free Core, No Arbitrary Caps:** No watermarks, export quotas, guide-count limits, or artificial paywalls.
- **AI is Optional:** Everything works without an internet connection or an AI provider. When enabled, AI suggestions are treated as drafts for human review.
- **Irreversible Redactions:** Redacted pixels are physically overwritten in memory before export. Sensitive tokens or emails cannot be extracted from HTML or image layers.
- **Open Artifacts:** No proprietary hosted page required. You own portable Markdown and HTML bundles ready for static hosting.

---

## 3. Architecture Overview

```text
dhap/
├── crates/
│   ├── dhap-core/        # Domain entities (Project, Step, Frame, Annotation)
│   ├── dhap-project/     # .dhap ZIP archive, atomic save/load, schema validation
│   ├── dhap-media/       # FFmpeg probe, frame extraction, scene heuristics
│   ├── dhap-annotation/  # Vector callouts & destructive pixel redaction
│   ├── dhap-export/      # Markdown & standalone HTML documentation generators
│   ├── dhap-ai/          # Optional AiProvider trait, Mock provider for CI
│   └── dhap-cli/         # Headless CLI for automation and CI/CD pipelines
├── apps/
│   └── site/             # Static product showcase and documentation website
└── fixtures/             # Deterministic synthetic test media & projects
```

---

## 4. Quick Start (CLI)

### Installation from Source
Ensure Rust (1.85+) and FFmpeg (6.0+) are installed.

```bash
git clone https://github.com/nhprince/dhap.git
cd dhap
cargo build --release
```

### Inspect Media Container
```bash
cargo run -p dhap-cli -- probe path/to/recording.mp4
```

### Extract Keyframes from Recording
```bash
cargo run -p dhap-cli -- extract-frames path/to/recording.mp4 --interval-ms 2000 --output-dir ./extracted_frames
```

### Validate and Export a Dhap Project
```bash
# Validate project schema and integrity
cargo run -p dhap-cli -- validate my_guide.dhap

# Export Markdown and HTML guides
cargo run -p dhap-cli -- export my_guide.dhap --format markdown,html --output-dir ./dist_docs
```

---

## 5. Development and Testing

```bash
# Run all workspace unit and integration tests
cargo test --workspace

# Run code formatting check
cargo fmt --check

# Run Clippy lints with all warnings treated as errors
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

## 6. Security and Privacy

Dhap is built from the ground up to prevent secret leakage. For threat models, vulnerability disclosures, and redaction guarantees, see [docs/SECURITY.md](docs/SECURITY.md).

---

## 7. License

Dhap is free and open-source software licensed under the [MIT License](LICENSE).  
Bengali brand: ধাপ (Dhap).
