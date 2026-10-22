# ADR-0001: Architecture Selection and Upstream Strategy

- **Status:** Accepted
- **Deciders:** Dhap Core Team / NH Prince Pradhan
- **Date:** 2026-10-22

---

## 1. Context and Problem Statement

Dhap (ধাপ) is an open-source, local-first desktop application that turns screen recordings and visual workflows into editable documentation and media outputs. The core promise is: **"Record once. Create the video and guide together."**

The engineering challenge requires:
1. Native screen/window capture and media ingestion across Windows, macOS, and Linux.
2. High-performance video frame extraction, thumbnailing, and scene change / keyframe candidate heuristics.
3. Vector screenshot annotation and mathematically irreversible pixel redaction.
4. Export orchestration for Markdown, self-contained interactive HTML guides, and PDF documents.
5. Versioned local project bundle (`.dhap`) with atomic file writes and crash resilience.
6. Responsive, accessible, multi-panel desktop UI.
7. Long-term maintainability for an open-source solo maintainer and contributors.

---

## 2. Options Considered

### Option A: Direct Fork and Deep Modification of OpenScreen
Fork `getopenscreen/openscreen` (MIT license, Electron + React UI + Rust GPU renderer + native capture). Add documentation editing directly into OpenScreen's existing codebase.

- **Pros:** Reuses native screen capture and GPU rendering directly out of the box; retains existing MP4/GIF export pipeline.
- **Cons:** OpenScreen's internal project format (`.openscreen`) and CLI are in active flux and not designed as a structured documentation model. Hard-forking creates massive upstream merge conflicts, high maintenance burden, and drags in large Electron dependencies even when executing CLI or batch workflows.

### Option B: Modular Architecture — Pure Rust Core Engine & CLI + React/TypeScript UI Application Shell
Build Dhap as a modular, layered system:
- **Core Engine & CLI:** Written in idiomatic, safe Rust (`crates/dhap-core`, `crates/dhap-project`, `crates/dhap-media`, `crates/dhap-annotation`, `crates/dhap-export`, `crates/dhap-ai`, `crates/dhap-cli`). Handles all schema validation, media probing, frame extraction via FFmpeg/native codecs, annotation geometry, pixel redaction baking, and export generation.
- **Desktop Application Shell:** High-performance web-tech UI (React/TypeScript) communicating over a typed IPC bridge to the Rust backend.
- **OpenScreen Interoperability:** An adapter module that can import recordings or project metadata from OpenScreen without coupling Dhap's internal guide format to private upstream internals.

- **Pros:**
  - Absolute separation of concerns: domain logic, project persistence, and exports are 100% testable in headless Rust CI without a browser or GUI runner.
  - CLI and future MCP agents reuse the exact same Rust engine crates without loading a desktop window.
  - Blazing-fast frame extraction, image manipulation, and redaction in native compiled Rust.
  - Maximum UI flexibility with React and strict TypeScript for complex multi-panel layouts (Guide editor, Timeline, Preview, Annotator).
  - Clean upgrade path: if upstream OpenScreen changes, Dhap's core documentation engine remains completely unaffected.
- **Cons:** Requires maintaining the typed IPC boundary between the UI and Rust core.

### Option C: Pure Native Rust UI (Slint / Iced)
Build the entire application, including the UI, in Rust using Slint or Iced.

- **Pros:** Ultra-lightweight binary, low memory footprint, single-language stack.
- **Cons:** Canvas-level screenshot annotation, rich text editing, responsive multi-panel layout switching, and web previewing are significantly slower to develop and maintain in contemporary Rust GUI frameworks. Would dramatically delay shipping the v1.0 milestone.

---

## 3. Evaluation & Scoring Matrix

| Criterion (Weight) | Option A: Fork OpenScreen | Option B: Modular Rust Engine + TS UI (Selected) | Option C: Pure Native Rust (Slint/Iced) |
|---|:---:|:---:|:---:|
| Reuse of recording/rendering (15%) | 9 / 10 | 8 / 10 (via adapter) | 4 / 10 |
| Ease of building guide editor (20%) | 6 / 10 | **9 / 10** | 4 / 10 |
| Memory and execution speed (15%) | 6 / 10 | **8 / 10** | 10 / 10 |
| Solo maintainer velocity (15%) | 6 / 10 | **9 / 10** | 5 / 10 |
| Cross-platform reliability (10%) | 8 / 10 | **9 / 10** | 6 / 10 |
| Headless CLI & MCP automation (10%) | 4 / 10 | **10 / 10** | 8 / 10 |
| Upstream decoupling & independence (10%) | 3 / 10 | **9 / 10** | 10 / 10 |
| Automated testing ergonomics (5%) | 5 / 10 | **10 / 10** | 7 / 10 |
| **Weighted Total** | **6.15 / 10** | **8.85 / 10** | **6.40 / 10** |

---

## 4. Decision Outcome

**Selected: Option B (Modular Architecture with Safe Rust Core Crates and TypeScript/React UI).**

### Specific Architectural Mandates:
1. **Domain Isolation:** All project models, `.dhap` archive packaging, frame extraction, annotation rasterization, irreversible redaction, and Markdown/HTML/PDF exports live in pure Rust crates.
2. **Deterministic Testability:** The Rust core must have 100% test coverage for project save/load, schema migration, corrupt file handling, and export golden matching without requiring an Electron or Tauri runtime.
3. **OpenScreen Integration:** Support importing OpenScreen recordings and projects via a dedicated adapter crate, preserving full upstream MIT license attribution and keeping Dhap's internal data model sovereign.
4. **Desktop UI:** Provide a rich, responsive interface with switchable layouts (Guide, Timeline, Preview) and instant visual feedback.
5. **No AI Hard-Locks:** The Rust core and UI must operate completely offline with zero network calls required for all core features.
