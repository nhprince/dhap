# Security Policy

## 1. Supported Versions

| Version | Supported |
|---|:---:|
| `0.1.x` | :white_check_mark: |

---

## 2. Threat Model and Invariants

Dhap operates under a local-first security model:

1. **Local Privacy:**
   - No user content, screen recordings, extracted frames, transcripts, or project notes are ever sent across the network by default.
   - Core editing and exports function completely offline.

2. **Irreversible Redactions:**
   - Any region marked for redaction undergoes destructive pixel manipulation (solid opaque fill or mosaic destruction) during export rendering.
   - Unredacted source frames are never copied into the export directory.

3. **Archive Path Traversal Defenses:**
   - When unpacking `.dhap` project bundles, every relative path entry is strictly checked.
   - Any entry attempting directory traversal (`..`), leading slashes (`/`), or Windows drive roots (`C:`) is immediately rejected with an error.

4. **Credential Safety:**
   - No API keys, credentials, or personal access tokens are stored in `.dhap` project files or git repositories.
   - AI provider keys are stored in operating system secure keyrings or passed as runtime environment variables.

---

## 3. Reporting a Vulnerability

If you discover a security vulnerability in Dhap, please report it privately:

- **Primary Contact:** NH Prince Pradhan
- **Email:** security@dhap.dev (or via GitHub Private Vulnerability Reporting on `nhprince/dhap`)
- **Response SLA:** Acknowledgment within 48 hours; assessment and patch timeline within 7 days.

Please do not disclose security issues publicly until a fix has been released.
