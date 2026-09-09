# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added — Phase 6 Engineer Console
- Engineer Console funcional em `web/engineer`: login, refresh por cookie HttpOnly após 401, dashboard autenticado, polling limitado, revisão do estado, sessões WebRTC e atribuição/remoção de mixes.
- Indicador explícito `SIMULATED` para áudio sem PipeWire no VPS.

### Changed — Phase 6
- Assignment usa ID numérico porque catálogo de usuários continua restrito à API Admin.

### Added — Phase 12
- Admin self-delete protection: `DELETE /api/v1/admin/users/{id}` returns 403 if caller matches target user ID.
- Dependabot configuration (`.github/dependabot.yml`): Cargo (weekly, limit 5), npm musician/engineer (weekly, limit 3), GitHub Actions (weekly).
- SBOM generation in release pipeline (`cargo-sbom`, best-effort, non-blocking); output `open-iem-server-<ver>-sbom.json` attached to GitHub Release.

### Fixed — Phase 12
- Release artifact naming bug: `build-server-x86`, `build-server-arm64`, `build-web` jobs now have `needs: [quality-gate, validate-version]` so version string is populated in artifact filenames.

---

## [0.2.0] — 2026-09-09

### Added — Phase 11
- Versioned release pipeline (`release.yml`): semver tag triggers quality gate + multi-target builds + GitHub Release.
- Linux x86_64 server artefact: `api-server` + `open-iem-admin` + README + LICENSE + CHANGELOG, SHA-256 checksum.
- Linux ARM64 (aarch64) server artefact: cross-compiled for Raspberry Pi 5 target; SIMULATED on VPS until real Pi validates PipeWire/Opus.
- Musician PWA and Engineer UI artefacts bundled from `web/musician/dist` and `web/engineer/dist`.
- Version consistency gate: release workflow rejects if tag version ≠ `[workspace.package].version` in `server/Cargo.toml`.
- `server/.cargo/config.toml` linker config for ARM64 cross-compilation.
- `admin-cli` Cargo manifest aligned to workspace semver.

### Changed
- Workspace `version` bumped `0.1.0` → `0.2.0`.

---

### Added — Phase 10
- Persistent SQLite mix assignments with one mix per user.
- Engineer/Admin mix assignment API.
- Musician-owned mix send API for gain, pan, mute, and state reads.
- Numeric user ID (`uid`) in access JWT claims.

### Security
- Musician send mutations require assigned mix ownership; channel master controls remain Engineer+.


### Added — Phase 9
- **Admin API server-side routes** (`server/api-server/src/routes/admin.rs`): 4 endpoints, all require Admin role:
  - `GET /api/v1/admin/users` — list all users (id, username, role)
  - `DELETE /api/v1/admin/users/{id}` — delete user by numeric ID (204 / 404)
  - `GET /api/v1/admin/sessions` — list active (non-revoked, non-expired) refresh-token sessions
  - `DELETE /api/v1/admin/sessions/{id}` — revoke session by numeric ID (204 / 404)
- **DB methods** added to `Db`: `list_users`, `delete_user`, `list_active_sessions`, `revoke_session_by_id` — all parameterized, no format-string SQL.
- **`ApiError::NotFound(String)`** variant added (was `&'static str`); HTTP 404 response.
- **8 new HTTP integration tests** for all admin endpoints (RBAC enforcement, success paths, 404 paths). api-server: 19 → 27 tests.
- **3 new DB unit tests**: `list_users_empty_and_populated`, `delete_user_ok_and_not_found`, `list_and_revoke_sessions`.
- **npm audit job** added to CI (HIGH severity gate, non-blocking until dependencies present).
- **Biquad coefficient validation** vs Python/scipy: max delta ≈ 5×10⁻⁸ (f32 rounding only), algorithm confirmed identical.

### Fixed — Phase 9
- **Admin CLI `user create`** command: `--name` → `--username`, added required `--password` arg. Server body now `{username, password, role}` matching `CreateUserRequest`.
- **Admin CLI `session revoke`** command: `--token: String` → `--id: u64`, dispatches `DELETE /api/v1/admin/sessions/{id}`.

### Changed — Phase 9
- Total workspace tests: 157 → 168 (+11: 8 integration + 3 DB unit).
- `GET /api/v1/admin/users` added alongside existing `POST` (combined route).


- **EQ + Compressor integrated into `Mix::process`** audio chain: Sum → EQ → Compressor → Master Gain → Limiter. Both are disabled by default (passthrough when disabled).
- **`Mix` struct fields** `eq: ParametricEq` and `compressor: Compressor` exposed for per-mix DSP configuration.
- **5 new mix-engine tests** covering EQ boost at frequency, compressor reduction on loud signal, EQ passthrough when disabled, compressor passthrough when disabled, and EQ→Compressor chain order verification.
- **Admin CLI binary** (`server/admin-cli/`): `open-iem-admin` — commands: `user list/create/delete`, `session list/revoke`, `health`. Global `--server`, `--token` (or `OPEN_IEM_ADMIN_TOKEN` env), `--json` flags. Graceful 404 handling (Phase 9 server-side routes).
- **Musician Guide PDF** (`docs/guides/MUSICIANS-GUIDE.pdf`): generated via pandoc+xelatex from `MUSICIANS-GUIDE.md`. 61 KB, PDF 1.5. Emoji glyphs (⚠, ✅, ❌) render as blank in lmroman — cosmetic only.
- **`PHASE-8-REVIEW.md`** in `docs/reviews/`.

### Changed — Phase 8
- mix-engine test count: 74 → 79 (+5 new chain integration tests).
- Total workspace tests: 152 → 157.
- `server/Cargo.toml` workspace members: added `admin-cli`.

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
