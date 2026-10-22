# Dhap (ধাপ) — Project Roadmap

**Mission:** Make the steps clear.  
**Execution Cadence:** 12-week solo milestone followed by community-driven iterations.

---

## 1. Release Milestone Map

```mermaid
timeline
    title Dhap Engineering & Release Roadmap
    section Stage 1: Foundation
      Week 1-2 : Environment audit : Architecture selection : Repo & CI setup : Core Rust crates
    section Stage 2: Product Core
      Week 3-4 : Project model (.dhap) : Media probe & frame extract : Atomic save/open : Golden tests
    section Stage 3: Workflow Slice
      Week 5-6 : Vector annotations : Destructive redactions : Markdown & HTML exports : Preflight checks
    section Stage 4: UI & Automation
      Week 7-8 : Switchable layouts : Theme system : CLI tool : Mock AI provider
    section Stage 5: Reliability & Matrix
      Week 9-10 : Cross-platform builds : Installer packaging : Website & Cloudflare deployment
    section Stage 6: Public Showcase
      Week 11-12 : v1.0.0 release : Demo video : Documentation site : Launch
```

---

## 2. Milestone Details

### Stage 1: Architecture & Foundation (Weeks 1–2) — *Current*
- [x] Environment audit and toolchain verification.
- [x] Architecture Decision Records (ADR-0001 through ADR-0004).
- [x] GitHub repository setup (`nhprince/dhap`) with clean commit history, templates, and permissions.
- [x] Rust workspace creation (`dhap-core`, `dhap-project`, `dhap-media`, `dhap-annotation`, `dhap-export`, `dhap-ai`, `dhap-cli`).
- [x] Baseline CI workflow with pinned actions, format, clippy, and unit tests.

### Stage 2: Media Processing and Project Persistence (Weeks 3–4)
- [x] `.dhap` ZIP bundle reader, writer, and atomic replacement file operations.
- [x] Schema migration framework with fixture tests.
- [x] Video probing (resolution, frame rate, duration) and frame extraction via FFmpeg.
- [x] Keyframe proposal heuristic based on scene difference.

### Stage 3: Annotations, Irreversible Redaction, and Multi-Format Exports (Weeks 5–6)
- [x] Vector annotation models (callouts, arrows, boxes, text).
- [x] Irreversible destructive pixel redaction (solid opaque masks, pixelation, blur).
- [x] GitHub-flavored Markdown generator with relative asset paths.
- [x] Standalone, responsive HTML guide exporter.
- [x] Export preflight safety validation and integrity tests.

### Stage 4: User Experience and Automation (Weeks 7–8)
- [x] Design tokens: typography, ink/warm slate color palettes, light and dark themes.
- [x] Headless CLI (`dhap`) with JSON output and exit codes.
- [x] Optional `AiProvider` trait and deterministic mock provider for testing.
- [ ] Direct screen/window capture integration via OpenScreen adapter.

### Stage 5: Automation, Cross-Platform Packaging, and Site (Weeks 9–10)
- [x] Pinned GitHub Actions workflows: CI (`ci.yml`), Release (`release.yml`), Site Deploy (`deploy-site.yml`).
- [x] Static showcase website and documentation portal.
- [ ] Multi-platform installer bundles (Windows MSI/EXE, macOS DMG, Linux AppImage/tar).

### Stage 6: v1.0 Launch & Community (Weeks 11–12)
- [ ] Public v1.0.0 release tag with signed checksums and release notes.
- [ ] Product walkthrough video and interactive demo project.
- [ ] Onboarding of initial user cohort and feedback tracking.
