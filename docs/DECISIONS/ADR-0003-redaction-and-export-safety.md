# ADR-0003: Irreversible Redaction and Export Safety Pipeline

- **Status:** Accepted
- **Deciders:** Dhap Core Team / NH Prince Pradhan
- **Date:** 2026-10-22

---

## 1. Context and Problem Statement

Screen recordings frequently contain sensitive information: API credentials, passwords, session tokens, personal email addresses, financial details, or proprietary customer data.

Common failure modes in existing documentation tools include:
- Applying a CSS `filter: blur(...)` in the HTML export that leaves the underlying raw image unaltered and readable via browser DevTools.
- Storing unredacted source images inside the export directory or ZIP archive alongside the redacted ones.
- Relying purely on automated secret detectors that fail to catch unconventional or domain-specific private data.

---

## 2. Decision: Destructive Pixel Baking and Preflight Checklist

### 1. In-Project Nondestructive Editing vs. In-Export Destructive Baking
- **Within the Editor:** Redactions are stored as geometric vector regions (`x, y, width, height, kind: solid | pixelate | blur`). The author can move, resize, adjust, or remove redaction boxes at any time.
- **Within the Exporter:** When generating Markdown, HTML, or PDF exports, Dhap's Rust export pipeline (`dhap-export` / `dhap-annotation`) renders the frame into an in-memory pixel buffer, applies destructive pixel mutations (opaque solid fill, box pixelation, or heavy kernel blur) directly to the pixel data, encodes a fresh PNG/WebP, and outputs only the mutated image.
- **Information Theoretic Safety:** The source pixels within the redacted bounding box are physically erased and cannot be reconstructed from the exported image files or HTML source.

### 2. Export Bundle Hygiene
- Under no circumstances is the unredacted source frame copied to the exported directory or archive.
- Export verification tests in CI must assert that no unredacted source hash appears in the output folder.

### 3. Preflight Export Verification
Before writing export artifacts to disk, Dhap runs an automated preflight check:
1. Validates that every step has a valid title and instructions.
2. Asserts that all referenced frames exist and are decodable.
3. Checks if any step has detected warning patterns (such as potential unredacted keys or auth headers) and alerts the user for manual confirmation.
4. Generates all files into a temporary staging folder first; verifies file size and integrity; then atomically copies/renames into the user's chosen destination.
