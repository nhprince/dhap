# Dhap (ধাপ) — Product Specification

**Working Brand:** Dhap (ধাপ)  
**Tagline:** Make the steps clear.  
**Supporting Line:** Record once. Create the video and guide together.  
**Version:** 1.0 (Tracked Specification)  
**Date:** 2026-10-22  

---

## 1. Product Identity and Vision

Dhap is a polished, local-first, open-source desktop studio that records or imports visual workflows and transforms them into editable, illustrated documentation and media outputs from one project.

A user can record a workflow or import an existing recording, review suggested steps, select keyframes, add annotations, write instructions, redact sensitive data, and export professional Markdown, responsive HTML, and PDF guides.

---

## 2. Core Principles

1. **Free Core, No Arbitrary Caps:** No watermarks, guide-count limits, export limits, or account paywalls.
2. **Local-First & Offline:** Recordings, projects, screenshots, and exports stay entirely on the local machine.
3. **AI is Optional:** Full recording, editing, redaction, and exports work without an internet connection or AI API key.
4. **Human in Control:** Machine-suggested steps and text are drafts that require human review and approval.
5. **Irreversible Redaction:** Pixels within redacted areas are physically erased in exported media, not masked with CSS or metadata.
6. **Reproducible Exports:** Output files can be regenerated deterministically from the same project state.

---

## 3. Requirements Checklist and Prioritization

### 3.1 Capture and Media
- [x] **CAP-001 (P0):** Record supported display/window sources through native capture foundation or screen recording adapters.
- [x] **CAP-002 (P0):** Record session start, end, duration, and source metadata.
- [x] **CAP-003 (P0):** Pause, resume, and stop capture without losing already-written media.
- [x] **CAP-004 (P0):** Import supported video files (MP4, WebM, MOV) and images with robust error handling for corrupt files.
- [x] **CAP-005 (P0):** Maintain source media as immutable; edits are strictly non-destructive to source files.
- [x] **CAP-006 (P0):** Provide timestamp markers during capture.
- [ ] **CAP-007 (P1):** Record opt-in mouse click markers when platform permissions permit; never record keystrokes or passwords.
- [ ] **CAP-008 (P1):** Audio waveform inspection and microphone alignment.
- [ ] **CAP-009 (P2):** Browser extension capture adapter.

### 3.2 Detection and Guide Model
- [x] **GUIDE-001 (P0):** Create, edit, delete, and reorder guide steps.
- [x] **GUIDE-002 (P0):** Attach one or more extracted keyframes to each step.
- [x] **GUIDE-003 (P0):** Preserve original source millisecond timestamps per frame and step.
- [x] **GUIDE-004 (P0):** Generate candidate keyframes using frame sampling and scene-change heuristics.
- [x] **GUIDE-005 (P0):** Manual acceptance, rejection, or splitting of generated steps.
- [x] **GUIDE-006 (P0):** Edit step title, imperative instruction, expected result, notes, and image alt text.
- [ ] **GUIDE-007 (P1):** Group steps into titled sections with an auto-generated table of contents.
- [ ] **GUIDE-008 (P1):** OCR text extraction for contextual step suggestions.
- [ ] **GUIDE-009 (P2):** Browser DOM action event correlation.

### 3.3 Annotation and Redaction
- [x] **ANNO-001 (P0):** Add vector annotations: numbered callout badges, arrows, rectangles, circles, highlights, and text labels.
- [x] **ANNO-002 (P0):** Add irreversible redactions: opaque solid masks, box pixelation, and gaussian blur.
- [x] **ANNO-003 (P0):** Store non-destructive vector annotation coordinates in project metadata for continued editing.
- [x] **ANNO-004 (P0):** Destructively bake annotations and redactions into exported image buffers on export.
- [x] **ANNO-005 (P0):** Preflight safety checklist alerting user to review redacted areas before final export.
- [ ] **ANNO-006 (P1):** Heuristic scanner for common credentials (API tokens, private keys, emails) as a warning helper.
- [ ] **ANNO-007 (P2):** Multi-frame tracking for redaction boxes across video segments.

### 3.4 AI and Language Assistance
- [x] **AI-001 (P0):** Core editing and exports operate 100% offline with AI completely disabled.
- [x] **AI-002 (P1):** Pluggable `AiProvider` trait for step suggestions and instruction polishing.
- [x] **AI-003 (P1):** Deterministic mock provider for unit tests and local testing.
- [x] **AI-004 (P1):** Preflight disclosure modal showing exactly what text/data will be transmitted before making network calls.
- [x] **AI-005 (P1):** Suggestions returned as reviewable drafts; never silently overwriting user text.
- [ ] **AI-006 (P2):** Local on-device Whisper transcription integration.

### 3.5 Export and Publishing
- [x] **EXP-001 (P0):** Export GitHub-flavored Markdown with local relative asset paths and clean hierarchy.
- [x] **EXP-002 (P0):** Export self-contained, responsive HTML guide with embedded CSS and modern typography.
- [x] **EXP-003 (P0):** Export print-optimized HTML / PDF layout.
- [x] **EXP-004 (P0):** Staged atomic export: write to temporary folder first, verify output, then swap to destination.
- [x] **EXP-005 (P0):** Preflight export validation checking for missing frames, empty titles, and unredacted warnings.
- [ ] **EXP-006 (P1):** Reusable export templates (GitHub README, SOP, Classroom Handout).
- [ ] **EXP-007 (P1):** Interactive HTML stepper with keyboard navigation and step completion tracking.
- [ ] **EXP-008 (P2):** Direct publish to static hosting or GitHub Pages via user-configured tokens.

### 3.6 Project Workspace and Persistence
- [x] **PRJ-001 (P0):** Create, save, open, rename, and export `.dhap` projects.
- [x] **PRJ-002 (P0):** Autosave with explicit save status indicator and crash-safe temporary replacement writes.
- [x] **PRJ-003 (P0):** Versioned `.dhap` ZIP bundle containing `manifest.json`, `project.json`, and `frames/`.
- [x] **PRJ-004 (P0):** Path traversal protection rejecting any archive entry with `..` or absolute paths.
- [x] **PRJ-005 (P0):** Schema versioning and migration framework.
- [ ] **PRJ-006 (P1):** Named project snapshots and revision history diffs.
- [ ] **PRJ-007 (P2):** Batch conversion queue.

### 3.7 Automation and Integrations
- [x] **AUTO-001 (P1):** Headless CLI tool (`dhap`) for inspecting, validating, extracting frames, and exporting projects.
- [x] **AUTO-002 (P1):** Machine-readable JSON output and standard exit codes for CI/CD scripting.
- [ ] **AUTO-003 (P1):** Model Context Protocol (MCP) server for local AI agent workflows.
