# Phase 11 Review — Versioned Release Pipeline

**Date:** 2026-09-09
**Branch:** feat/phase11-release-pipeline
**Status:** PASS — local verification, CI gate pending push.

## Delivered

- `.github/workflows/release.yml`: semver tag (`v*.*.*`) triggers full release pipeline.
- **Jobs:**
  1. `validate-version`: tag == `server/Cargo.toml` workspace version (fail-closed).
  2. `quality-gate`: fmt + clippy + tests + cargo-audit (all pass before builds start).
  3. `build-server-x86`: Linux x86_64 release binaries; `open-iem-server-<ver>-x86_64-linux.tar.gz` + SHA-256.
  4. `build-server-arm64`: cross-compile to `aarch64-unknown-linux-gnu` for Raspberry Pi 5; `open-iem-server-<ver>-aarch64-linux.tar.gz` + SHA-256. **SIMULATED** — not tested on real Pi hardware.
  5. `build-web`: Musician PWA + Engineer UI dist bundles; separate tarballs + SHA-256.
  6. `github-release`: publishes all artefacts to GitHub Release with CHANGELOG excerpt.
- `server/.cargo/config.toml`: ARM64 linker config (`aarch64-linux-gnu-gcc`).
- `server/Cargo.toml` workspace version bumped `0.1.0` → `0.2.0`.
- `server/admin-cli/Cargo.toml` aligned to workspace (`version.workspace = true`).
- `CHANGELOG.md`: `[0.2.0]` section added.
- `README.md`: release pipeline section added.

## Verification

- `cargo fmt --manifest-path server/Cargo.toml --all -- --check`: PASS
- `cargo clippy --manifest-path server/Cargo.toml --all-targets -- -D warnings`: PASS
- `cargo test --manifest-path server/Cargo.toml --all`: PASS (171 tests, 0 failures)
- Static security scan (secrets, shell injection, eval, pickle, SQL injection): CLEAN
- Independent reviewer subagent: `passed=true`, no security concerns, no logic errors
- VPS has no PipeWire hardware: audio remains SIMULATED.

## Limitations / SIMULATED

- ARM64 cross-compile: CI will verify build succeeds; runtime on Pi 5 is NOT validated.
- Windows/macOS: NOT targeted (toolchain dependencies prevent clean cross). BLOCKED/UNSUPPORTED per project policy.
- PipeWire/Opus: SIMULATED until Raspberry Pi 5 hardware is available.

## Follow-up

- [ ] Tag v0.2.0 and verify full pipeline run on GitHub Actions.
- [ ] Validate ARM64 binary boot on real Raspberry Pi 5 hardware.
- [ ] Add Dependabot for Cargo + npm dependency updates.
- [ ] SBOM generation (cyclonedx or cargo-sbom) for future release.
