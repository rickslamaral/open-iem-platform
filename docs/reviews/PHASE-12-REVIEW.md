# Phase 12 Review — Release Fixes, Dependabot, Self-Delete Protection, SBOM

**Date:** 2026-09-09
**Branch:** feat/phase12-release-fixes
**Status:** PASS — all tests green, CI pending push.

## Delivered

### Bug Fix — Release Artifact Naming
- `.github/workflows/release.yml`: `build-server-x86`, `build-server-arm64`, `build-web` jobs changed from `needs: quality-gate` to `needs: [quality-gate, validate-version]`.
- Root cause: `${{ needs.validate-version.outputs.version }}` resolved to empty string because those jobs did not list `validate-version` in their `needs` chain.
- Fix ensures artifact names like `open-iem-server-0.2.0-x86_64-linux.tar.gz` are correct on next release.

### Dependabot
- `.github/dependabot.yml` created.
- Ecosystems: `cargo` (`/server`, weekly, limit 5, labels: `dependencies rust`), `npm` (`/web/musician` and `/web/engineer`, weekly, limit 3, labels: `dependencies npm`), `github-actions` (`/`, weekly, label: `dependencies github-actions`).

### Admin Self-Delete Protection
- `server/api-server/src/routes/admin.rs` — `delete_user` handler: checks `user_id == claims.user_id` after role check; returns `ApiError::Forbidden("Cannot delete your own account")` (HTTP 403) if true.
- Fail-closed: role check fires first, then self-delete guard before any DB mutation.
- Integration test `admin_cannot_delete_own_account` in `tests/integration.rs`: seeds admin, attempts DELETE on own ID, asserts 403 + `code: "FORBIDDEN"`.

### SBOM Generation
- `build-server-x86` job: new step `Generate SBOM (best-effort)` with `continue-on-error: true`.
- Installs `cargo-sbom --locked`; on success runs `cargo sbom --manifest-path server/Cargo.toml`.
- Output: `open-iem-server-${VERSION}-sbom.json`.
- Upload step includes `open-iem-server-*-sbom.json` glob.
- GitHub Release `files:` includes `dist/*.json`.
- Non-blocking: install or run failure prints WARNING but does not fail the release.

## Verification

- `cargo fmt --all`: PASS
- `cargo clippy --all-targets -- -D warnings`: PASS
- `cargo test --all`: PASS — 172 tests, 0 failures (↑1 from `admin_cannot_delete_own_account`)
- Static security scan: no hardcoded secrets, no SQL injection, no shell injection.
- Self-delete guard is fail-closed (returns 403, no DB call on match).
- SBOM step is non-blocking (`continue-on-error: true`).

## Not Yet Validated

- SBOM generation on actual CI runner (requires `cargo-sbom` install to succeed in Actions).
- ARM64 binary on real Raspberry Pi 5 hardware.
- Tag v0.3.0 end-to-end pipeline run.
