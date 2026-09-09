# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added — Phase 7
- **Biquad Parametric EQ** (`mix-engine/src/eq.rs`): real Type-II Transposed DF2 peaking filter replacing Phase 2 passthrough stub. RBJ Audio EQ Cookbook coefficients, SAMPLE_RATE=48000, stereo biquad state inline (no heap), all 4 bands independent.
- **BiquadCoeffs** struct: `identity()` + `peaking(frequency_hz, gain_db, q)` — coefficients recomputed on `set_band`.
- **BiquadState** struct: stereo delay lines `w1_l/w2_l/w1_r/w2_r`, `tick()` for per-sample processing.
- **RMS Compressor** (`mix-engine/src/compressor.rs`): stereo-linked RMS detector + smoothed gain reduction replacing Phase 2 passthrough stub.
- Compressor mutators: `set_threshold`, `set_ratio`, `set_attack_ms`, `set_release_ms` — all bump revision and recompute coefficients.
- **Docker Compose** dev environment (`docker-compose.yml`): api-server, musician-ui, engineer-ui services with health checks.
- **Musician Guide** (`docs/guides/MUSICIANS-GUIDE.md`): 14-section guide in pt-BR covering Linux requirements, server setup, JWT key generation, login, mix controls, WebRTC status, permissions, diagnostics, LAN, security, known limitations.
- **PHASE-6-REVIEW.md** and **PHASE-7-REVIEW.md** added to `docs/reviews/`.
- `cargo fmt --all` applied across workspace.

### Changed — Phase 7
- `ParametricEq::process` signature changed from `&self` to `&mut self` (required for biquad state mutation).
- mix-engine test count: 57 → 74 (+17 new DSP tests for biquad EQ and compressor).
- Total workspace tests: 135 → 152.

### Added — Phase 6
- Real trickle-ICE candidate injection via `Candidate::from_sdp_string` + `Rtc::add_remote_candidate` in streaming crate (SIMULATED on VPS)
- Named constants replace magic numbers in streaming crate (`MAX_SDP_BYTES`, `MAX_CANDIDATE_BYTES`, `MAX_USER_ID_BYTES`)
- HTTP integration test suite for api-server (19 tests: auth, RBAC, CSRF, audio routes, channel controls)
- Test fixture role casing corrected (`"MUSICIAN"`, `"ADMIN"`, `"ENGINEER"` — SCREAMING_SNAKE_CASE)
- `axum-test` pinned to `"21"`, `jsonwebtoken` gains `rust_crypto` feature
- Project directory structure (all planned directories)
- 11 project-specific agent skills in `.agents/skills/`
- Documentation structure (`docs/` with all subdirectories)
- ADR baseline (ADR-001 through ADR-008)
- `docs/SPEC-AUDIT.md` — specification audit with 14 FRs, 10 NFRs, 10 missing requirements, 10 architecture gaps
- `docs/ARCHITECTURE-GAPS.md` — 10 identified gaps with phase dependencies
- `docs/SKILLS.md` — skills registry and orchestration guide
- `docs/TODO.md` — prioritized backlog
- `docs/DEVELOPMENT-LOG.md` — development history
- `docs/DEVELOPMENT-ENVIRONMENT.md` — environment audit
- GitHub Actions CI foundation (Rust lint/test/build, ARM64 cross-build, frontend, skill validation)
- `scripts/validate-skills.sh` — skill validation script
- Symlinks: `.hermes/skills/` and `.claude/skills/` → `.agents/skills/`
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `LICENSE` (Apache 2.0), `.gitignore`

## [0.0.1] - 2026-09-07

### Added
- Initial repository creation
- Minimal README
