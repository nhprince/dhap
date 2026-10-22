# ADR-0002: Project Bundle Format and Data Model

- **Status:** Accepted
- **Deciders:** Dhap Core Team / NH Prince Pradhan
- **Date:** 2026-10-22

---

## 1. Context and Problem Statement

A Dhap project must tie together:
1. Source media (video files or screenshot collections).
2. Extracted keyframes and their precise millisecond timestamps.
3. Ordered steps with title, markdown instructions, expected results, notes, and callout associations.
4. Nondestructive vector annotations and redaction regions.
5. Export profiles and project metadata.

The format must be human-inspectable, version-migratable, crash-resilient, and safe against path traversal.

---

## 2. Decision: `.dhap` ZIP Archive with JSON Manifest

A `.dhap` file is a standard ZIP archive containing:
```text
my-project.dhap
├── manifest.json         # Archive metadata, schema_version, generator, file inventory
├── project.json          # Complete project document tree
├── media/                # Optional bundled source media
│   └── source-001.mp4
├── frames/               # Extracted PNG/WebP keyframes referenced by ID
│   ├── frame_01H...png
│   └── frame_01H...png
└── exports/              # Generated export caches (optional)
```

### Key Data Integrity Invariants:
1. **ULID / UUID Entity Identifiers:** All steps, frames, annotations, and media sources use stable string identifiers, never zero-based array indices.
2. **Integer Milliseconds:** All timeline points and durations are stored as unsigned 64-bit integers (`u64`) representing milliseconds, preventing floating-point precision drift.
3. **Atomic File Writes:** When saving, Dhap writes to a sibling temporary file (e.g. `filename.dhap.tmp.<uuid>`), flushes buffers to disk, and executes an atomic file rename. If an interruption occurs, the existing `.dhap` file remains uncorrupted.
4. **Path Traversal Defenses:** All archive extraction logic strictly rejects paths containing `..`, absolute paths (leading `/` or Windows drive letters `C:`), and non-normalized path components.
5. **Schema Versioning:** The `schema_version` is explicitly stored in both `manifest.json` and `project.json`. Forward migrations are handled by pure transformation functions with pre-migration backups.
