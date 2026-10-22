# Licensing Policy and Provenance

## 1. Project License

Dhap is released as free and open-source software under the **MIT License**.

- You are free to inspect, run, modify, redistribute, and build upon Dhap for private, educational, non-profit, or commercial purposes.
- The full license terms are in the root [LICENSE](file:///d:/Workspace/Dhap/LICENSE) file.

---

## 2. Upstream Architecture & Provenance

### OpenScreen (`getopenscreen/openscreen`)
- **Status:** Evaluated reference and upstream interoperability target.
- **License:** MIT License.
- **Relationship:** Dhap is an independent project with sovereign `.dhap` project formatting and documentation capabilities. Dhap maintains an adapter layer for importing OpenScreen recordings without hard-coupling internal structures.
- **Attribution:** OpenScreen copyright remains with its respective authors and contributors.

---

## 3. Media Tooling & FFmpeg Boundary

- **Integration Mode:** Subprocess execution / Dynamic CLI IPC.
- **Licensing Model:** FFmpeg is invoked as an external system binary. It is neither statically linked into Dhap binaries nor distributed inside the core Rust crates.
- **Legal Implication:** This dynamic subprocess boundary preserves the standalone MIT licensing of the Dhap Rust engine while allowing users to utilize their system-installed FFmpeg builds (LGPL/GPL).

---

## 4. Fonts and Artistic Assets

- **Fonts:** All web and export templates use standard system fonts or open-source typefaces (Inter, Outfit, Fira Code) licensed under SIL Open Font License (OFL).
- **Icons & Branding:** Dhap wordmark, logo, and step icons are original works created specifically for Dhap, licensed under MIT.
