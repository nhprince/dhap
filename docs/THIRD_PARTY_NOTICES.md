# Third-Party Notices and Acknowledgments

Dhap incorporates, interfaces with, or depends on several open-source software libraries. We gratefully acknowledge the authors and contributors of these projects.

---

## 1. Upstream Projects

### OpenScreen
- **Homepage:** https://github.com/getopenscreen/openscreen
- **License:** MIT License
- **Copyright:** (c) OpenScreen Contributors

---

## 2. Rust Ecosystem Dependencies

The following Rust libraries are utilized under their respective open-source licenses:

| Crate | Purpose | License |
|---|---|---|
| `serde` & `serde_json` | Serialization and data modeling | MIT OR Apache-2.0 |
| `tokio` | Asynchronous runtime | MIT |
| `image` | Image decoding, pixel manipulation, encoding | MIT OR Apache-2.0 |
| `zip` | `.dhap` archive reading and atomic extraction | MIT |
| `clap` | Command-line argument parsing | MIT OR Apache-2.0 |
| `thiserror` | Structured typed domain errors | MIT OR Apache-2.0 |
| `uuid` | Unique identifier generation | MIT OR Apache-2.0 |
| `pulldown-cmark` | Markdown parsing and rendering | MIT |
| `tempfile` | Crash-safe atomic file writing | MIT OR Apache-2.0 |
| `async-trait` | Trait abstractions for AI providers | MIT OR Apache-2.0 |

---

## 3. External Executables

### FFmpeg
- **Homepage:** https://ffmpeg.org
- **License:** LGPL 2.1+ / GPL 2.0+ (depending on build configuration)
- **Notice:** FFmpeg is invoked externally by `dhap-media` as a separate CLI subprocess to probe video containers and extract keyframes.
