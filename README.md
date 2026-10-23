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
> **Record once. Create the video, guide, presentation, and animated walkthrough together.**

From a single project file (`.dhap`), you can simultaneously generate:
- Clean, GitHub-ready **Markdown** with local assets and code syntax blocks.
- **Standalone Interactive HTML 2.0 Super-Studio** with live search, presentation slide deck mode, Web Speech API narration, and checklist progress persistence.
- **Animated Walkthrough GIFs and WebPs** generated with pure Rust + FFmpeg palettegen.
- **Machine-readable JSON archives** (`guide.json`) for CI/CD automation and tooling.
- Print-optimized **PDF** documentation with dedicated CSS `@media print` stylesheets.

---

## 2. Core Principles

- **Local-First & Offline:** Your recordings, screenshots, transcripts, and guides remain strictly on your machine.
- **Free Core, No Arbitrary Caps:** No watermarks, export quotas, guide-count limits, or artificial paywalls.
- **AI is Optional & Offline-First:** Out-of-the-box heuristic action classification and prerequisite detection, with native zero-telemetry local Ollama support.
- **Irreversible Redactions:** Redacted pixels are physically overwritten in memory before export. Sensitive tokens or emails cannot be extracted from HTML or image layers.
- **Open Artifacts:** No proprietary hosted page required. You own portable Markdown and HTML bundles ready for static hosting.

---

## 3. Architecture Overview

```text
dhap/
├── crates/
│   ├── dhap-core/        # Domain entities (Project, Step, Frame, ActionKind, Diff, Stats)
│   ├── dhap-project/     # .dhap ZIP archive, atomic save/load, schema validation
│   ├── dhap-media/       # FFmpeg probe, frame extraction, ScreenRecorder, Animated GIF/WebP Exporter
│   ├── dhap-annotation/  # Vector callouts & destructive pixel redaction
│   ├── dhap-export/      # Markdown, HTML 2.0, JSON & Walkthrough GIF exporters
│   ├── dhap-ai/          # OllamaAiProvider, ActionExtractor heuristics, GuideSummarizer
│   └── dhap-cli/         # Multi-format CLI & Model Context Protocol (MCP) server
├── apps/
│   ├── desktop/          # React 19 + TypeScript studio (Editor, Slide Deck, Search, History)
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

### Validate and Export Documentation
```bash
# Validate project schema and integrity
cargo run -p dhap-cli -- validate my_guide.dhap

# Export Markdown, HTML 2.0, JSON, and Animated Walkthrough GIF
cargo run -p dhap-cli -- export my_guide.dhap --format markdown,html,json,gif --output-dir ./dist_docs
```

### Project Statistics & Version Diffing
```bash
# Print project statistics (step count, actions, code blocks, secrets warnings)
cargo run -p dhap-cli -- stats my_guide.dhap

# Compare two project versions to see added, removed, and modified steps
cargo run -p dhap-cli -- diff v1_guide.dhap v2_guide.dhap

# Generate executive summary and detected prerequisites (Docker, Rust, Node, Git, AWS)
cargo run -p dhap-cli -- summarize my_guide.dhap
```

### Animated GIF Walkthrough Generation
```bash
# Create an animated GIF looping through all guide steps
cargo run -p dhap-cli -- gif my_guide.dhap --output ./walkthrough.gif --fps 1 --scale 800
```

### Model Context Protocol (MCP) Agent Server
Dhap exposes 7 standard tools over JSON-RPC 2.0 stdio for integration with AI assistants (Claude, Antigravity, Cursor, etc.):
```bash
cargo run -p dhap-cli -- mcp
```
- `dhap_validate_project` — Check project integrity and schema conformance.
- `dhap_export_project` — Export to Markdown, HTML, JSON, and GIF.
- `dhap_scan_secrets` — Privacy pre-flight scan for leaked API keys, tokens, or private keys.
- `dhap_project_stats` — Compute project metrics, total steps, and action distributions.
- `dhap_diff_projects` — Compute semantic diff between two `.dhap` project versions.
- `dhap_summarize_project` — Generate executive summary and detect system prerequisites.
- `dhap_generate_gif` — Render an animated walkthrough GIF from project keyframes.

---

## 5. Development and Testing

```bash
# Run all workspace unit and integration tests (22 of 22 passing)
cargo test --workspace

# Run code formatting check
cargo fmt --check

# Run Clippy lints with all warnings treated as errors
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build the desktop studio
cd apps/desktop
npm install
npm run build
```

---

## 6. Security and Privacy

Dhap is built from the ground up to prevent secret leakage. For threat models, vulnerability disclosures, and redaction guarantees, see [docs/SECURITY.md](docs/SECURITY.md).

---

## 7. License

Dhap is free and open-source software licensed under the [MIT License](LICENSE).  
Bengali brand: ধাপ (Dhap).
