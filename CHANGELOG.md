# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

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
