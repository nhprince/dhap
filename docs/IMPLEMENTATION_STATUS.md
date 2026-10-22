# Dhap (ধাপ) — Implementation Status

**Last Updated:** 2026-10-22  
**Active Milestone:** v0.1.0 Foundation & Core Workflow Slice  

---

## 1. Feature Status Matrix

| Subsystem | Requirement | Status | Verification Method |
|---|---|:---:|---|
| **Environment & Tooling** | Rust 1.99, Node 24, FFmpeg 9.0, Git, gh authenticated | :white_check_mark: Verified | `docs/ENVIRONMENT_AUDIT.md` executed & documented |
| **Architecture Decision** | ADR-0001 (Modular Rust Engine + UI Shell) | :white_check_mark: Accepted | `docs/DECISIONS/ADR-0001-architecture-selection.md` |
| **Data Model & Bundle** | ADR-0002 (.dhap ZIP archive, atomic write, schema v1) | :white_check_mark: Accepted | `docs/DECISIONS/ADR-0002-project-bundle-and-data-model.md` |
| **Irreversible Redaction** | ADR-0003 (Pixel-level destructive baking in memory) | :white_check_mark: Accepted | `docs/DECISIONS/ADR-0003-redaction-and-export-safety.md` |
| **AI Privacy Abstraction** | ADR-0004 (Offline-by-default, Mock/Ollama/Cloud traits) | :white_check_mark: Accepted | `docs/DECISIONS/ADR-0004-optional-ai-and-privacy.md` |
| **dhap-core** | Entities (`Project`, `GuideStep`, `Frame`, `Annotation`) | :white_check_mark: Built & Tested | Unit tests in `crates/dhap-core` |
| **dhap-project** | `.dhap` ZIP packaging, path traversal check, atomic save | :white_check_mark: Built & Tested | Round-trip tests in `crates/dhap-project` |
| **dhap-media** | FFmpeg probe, frame extraction, synthetic test frames | :white_check_mark: Built & Tested | Integration tests in `crates/dhap-media` |
| **dhap-annotation** | Vector callouts, solid & pixelate redactions (image buffer) | :white_check_mark: Built & Tested | Pixel-baking tests in `crates/dhap-annotation` |
| **dhap-export** | Markdown with local assets, standalone HTML with stepper | :white_check_mark: Built & Tested | Golden fixture tests in `crates/dhap-export` |
| **dhap-ai** | `AiProvider` trait, `MockAiProvider`, consent disclosure | :white_check_mark: Built & Tested | Async tests in `crates/dhap-ai` |
| **dhap-cli** | CLI tool (`probe`, `validate`, `export`, `version`) | :white_check_mark: Built & Tested | End-to-end CLI tests in `crates/dhap-cli` |
| **Automation CI/CD** | Pinned GitHub Actions (`ci.yml`, `release.yml`, `deploy-site.yml`) | :white_check_mark: Verified Green | Run 39760981884 passed across Windows, macOS, Linux |
| **Documentation & Showcase** | Static website & interactive documentation portal | :white_check_mark: Verified Built | `deploy-site.yml` run 39760622929 passed |
| **Desktop Capture Integration** | Native display capture via OpenScreen adapter | :hourglass_flowing_sand: Planned (Stage 4) | Roadmap item |
| **Installer Packages** | Multi-platform MSI, DMG, AppImage packaging | :hourglass_flowing_sand: Planned (Stage 5) | Release workflow target |
| **Browser Extension** | Web DOM event recorder | :calendar: Deferred (v2.x) | Future milestone |

---

## 2. Quality Gates Checklist

- [x] Zero unreviewed unsafe code blocks in Rust crates.
- [x] All GitHub Actions pinned to immutable full-length commit SHAs with version comments.
- [x] Path traversal protections asserted via test fixtures containing `../evil.txt`.
- [x] Irreversible redaction verified: target pixels in output buffer are confirmed overwritten.
- [x] Full offline independence: complete suite runs with zero network access and no API keys.
