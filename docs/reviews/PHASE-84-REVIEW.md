# Phase 84 Review — Release reproducibility

**Date:** 2026-09-13
**Status:** PASS WITH CONDITIONS — workflow correction implemented; duplicate CI build and release remain pending.

## Scope

Reduce checksum drift between repeated release builds for the same commit.

## Changes

- Added Cargo `--locked` to release clippy, tests and x86_64/ARM64 builds.
- Derive `SOURCE_DATE_EPOCH` from the checked-out commit.
- Normalize archive ordering, timestamps and ownership for server and web artifacts before checksums; Ed25519 signatures remain limited to server archives.
- This does not prove full reproducibility: runner, Rust/Node toolchains and archive tooling remain floating.

## Verification

- Workflow YAML passed syntax validation after edits.
- Independent review identified archive timestamp/toolchain drift as a MEDIUM reproducibility finding.
- Manual review of changed workflow lines found no added hardcoded secrets, shell injection, eval/exec or unsafe deserialization pattern; static scan command required rerun after shell quoting failure.

## Conditions

- `ubuntu-latest`, Rust stable and the ARM64 linker remain floating; pinning them is a future supply-chain/reproducibility task.
- Two CI builds of the same tag have not been executed.
- Release `v0.3.1`, real installation and Raspberry Pi 5 remain unvalidated.
- Audio and media remain `SIMULATED`.
