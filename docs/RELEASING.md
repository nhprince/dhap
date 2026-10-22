# Dhap (ধাপ) — Releasing and Distribution Guide

---

## 1. Release Philosophy

1. **Automated & Reproducible:** Releases are produced exclusively by GitHub Actions on tagged commits (`v*.*.*`), never manually compiled or uploaded from local workstations.
2. **Deterministic Checksums:** Every binary artifact is accompanied by a SHA-256 checksum file (`SHA256SUMS.txt`).
3. **Dry-Run Capable:** The release pipeline supports a `dry_run: true` workflow dispatch parameter that tests compilation, artifact packaging, and checksum verification without publishing a public GitHub Release.
4. **Draft First:** Releases are initially staged in `draft` mode to enable release manager verification before public promotion.

---

## 2. Release Steps

### Step 1: Version Bumping and Changelog
1. Update version numbers across Cargo workspace:
   - Root `Cargo.toml` or individual crate `Cargo.toml` files.
2. Update `CHANGELOG.md` with features, bug fixes, and upgrade notes.
3. Commit changes:
   ```bash
   git commit -am "chore(release): prepare v0.1.0"
   ```

### Step 2: Tag Creation and Push
```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin main --tags
```

### Step 3: Automated Workflow Execution
The `.github/workflows/release.yml` pipeline triggers:
- Verifies tag matches crate versions.
- Runs complete test suite across platforms.
- Builds release binaries (`dhap-cli`).
- Calculates SHA-256 hashes.
- Generates automated release notes.
- Publishes a draft GitHub Release.

### Step 4: Verification and Promotion
1. Verify installer and binary integrity.
2. Promote GitHub Release from draft to public.
