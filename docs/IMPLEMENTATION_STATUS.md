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
| **dhap-core** | Entities (`Project`, `GuideStep`, `Frame`), step reordering API | :white_check_mark: Built & Tested | Unit tests in `crates/dhap-core` |
| **dhap-project** | `.dhap` ZIP packaging, path traversal check, atomic save | :white_check_mark: Built & Tested | Round-trip tests in `crates/dhap-project` |
| **dhap-media** | FFmpeg probe, frame extraction, ScreenRecorder, synthetic test frames | :white_check_mark: Built & Tested | Integration & recorder tests in `crates/dhap-media` |
| **dhap-annotation** | Vector callouts, solid & pixelate redactions (image buffer) | :white_check_mark: Built & Tested | Pixel-baking tests in `crates/dhap-annotation` |
| **dhap-export** | Multi-template Markdown & HTML (SOP, Classroom, GitHub, Standard), secret scanner | :white_check_mark: Built & Tested | Golden fixture & scanner tests in `crates/dhap-export` |
| **dhap-ai** | `AiProvider` trait, `MockAiProvider`, consent disclosure | :white_check_mark: Built & Tested | Async tests in `crates/dhap-ai` |
| **dhap-cli** | CLI tool (`record`, `probe`, `validate`, `export --template`, `mcp`, `version`) | :white_check_mark: Built & Tested | End-to-end CLI & recording tests in `crates/dhap-cli` |
| **Automation CI/CD** | Pinned GitHub Actions (`ci.yml`, `release.yml`, `deploy-site.yml`) | :white_check_mark: Verified Green | Run 39769328309 passed across macOS, Windows, Linux & Desktop UI |
| **Documentation & Showcase** | Static website & interactive documentation portal | :white_check_mark: Verified Built | `deploy-site.yml` run 39760622929 passed |
| **Desktop Studio UI** | React 19, TypeScript studio with step reordering, templates & secret preflight | :white_check_mark: Built & Verified | Verified in CI with recorder modal, annotator & save/load |
| **Desktop Capture Integration** | Cross-platform screen capture (`gdigrab`/`avfoundation`/`x11grab`) & UI recorder | :white_check_mark: Built & Verified | ScreenRecorder in `dhap-media`, CLI `record`, and studio modal |
| **MCP Agent Automation** | Model Context Protocol server (`dhap mcp`) over stdio | :white_check_mark: Built & Tested | JSON-RPC 2.0 stdio server + integration test `mcp_protocol.rs` |
| **Interactive Keyboard Stepper** | Accessible client-side stepper with hotkeys (Arrow/j/k, x) and progress tracking | :white_check_mark: Built & Verified | Integrated in responsive HTML exporter |
| **Project Snapshots & History** | Named project snapshots with timestamped step title arrays | :white_check_mark: Built & Tested | `ProjectSnapshot` in `dhap-core` with backward-compatible Serde |

| **Installer Packages** | Multi-platform MSI, DMG, AppImage packaging | :hourglass_flowing_sand: Planned (Stage 5) | Release workflow target |
| **Browser Extension** | Web DOM event recorder | :calendar: Deferred (v2.x) | Future milestone |

---

## 2. Quality Gates Checklist

- [x] Zero unreviewed unsafe code blocks in Rust crates.
- [x] All GitHub Actions pinned to immutable full-length commit SHAs with version comments.
- [x] Path traversal protections asserted via test fixtures containing `../evil.txt`.
- [x] Irreversible redaction verified: target pixels in output buffer are confirmed overwritten.
- [x] Full offline independence: complete suite runs with zero network access and no API keys.
