# Development Log

All significant milestones documented here in reverse chronological order.

---

## 2026-09-09 — Phase 6: Engineer Console dashboard

**Branch:** `main`
**Ambiente:** VPS Linux; áudio SIMULATED

### Implementado

- `web/engineer` ganhou login com token em memória, refresh por cookie HttpOnly após 401, dashboard autenticado, polling de 5 segundos, revision do estado, sessões WebRTC e gestão de assignment dos dois mixes.
- UI usa somente contratos API existentes. Catálogo de usuários permanece Admin-only; assignment recebe ID numérico.
- Erros HTTP/rede têm estado visível. Nenhuma métrica, canal ou mídia real é inventada.
- CSS responsivo próprio e indicador explícito de áudio SIMULATED.

### Verificação

- Frontend: typecheck, 2 testes e build PASS.
- Backend: fmt, 172 testes e clippy PASS.
- Documentação: `scripts/validate-docs.sh` PASS.

### Pendências

Snapshot completo, meters, telemetria e PipeWire/Opus dependem de contrato API e validação em Raspberry Pi 5.

---

## 2026-09-09 — Phase 12: release fixes, Dependabot, self-delete protection, SBOM

**Branch:** `feat/phase12-release-fixes`
**Tests:** 172 passed (↑ from 171), 0 failed

### Implemented

Fixed artifact naming bug in release pipeline (`validate-version` not in build job needs chain). Build jobs `build-server-x86`, `build-server-arm64`, `build-web` now have `needs: [quality-gate, validate-version]` so `${{ needs.validate-version.outputs.version }}` resolves correctly.

Added Dependabot for Cargo, npm (musician/engineer), and GitHub Actions in `.github/dependabot.yml`.

Admin self-delete protection: `DELETE /api/v1/admin/users/{id}` returns 403 Forbidden if caller's `uid` matches the path ID. Integration test `admin_cannot_delete_own_account` added.

SBOM generation added to release pipeline (best-effort, `continue-on-error: true`): installs `cargo-sbom` and generates `open-iem-server-<ver>-sbom.json`, uploaded as artifact and included in GitHub Release.

**Verification:** cargo fmt PASS; cargo clippy PASS; cargo test --all PASS: 172 tests, 0 failures.

---

## 2026-09-09 — Phase 8: DSP Chain Integration + Admin CLI + Musician Guide PDF

**Branch:** `feat/phase8-dsp-chain`
**Tests:** 157 passed (↑ from 152), 0 failed

### Implemented

#### EQ + Compressor wired into Mix::process (`mix-engine/src/mix.rs`)
- `Mix` struct gains `pub eq: ParametricEq` and `pub compressor: Compressor` fields.
- `Mix::new()` initializes both disabled (flat EQ, compressor off).
- `Mix::process()` chain: Sum → `eq.process()` → `compressor.process()` → Master Gain → Limiter.
- Realtime safety preserved: no allocation, no I/O in process path.
- 5 new tests: `test_mix_eq_boosts_at_freq`, `test_mix_compressor_reduces_loud`, `test_mix_eq_passthrough_when_disabled`, `test_mix_compressor_passthrough_when_disabled`, `test_mix_chain_order`.

#### Admin CLI (`server/admin-cli/`)
- Binary crate `open-iem-admin` added to workspace.
- Commands: `user list`, `user create --name --role`, `user delete --id`, `session list`, `session revoke --token`, `health`.
- Auth: `--token <JWT>` or `OPEN_IEM_ADMIN_TOKEN` env var.
- Output: human-readable table (default) or `--json`.
- Graceful 404 (Phase 9 server routes not yet implemented).
- `cargo clippy -D warnings` clean.

#### Musician Guide PDF (`docs/guides/MUSICIANS-GUIDE.pdf`)
- Generated via pandoc 3.1.11.1 + xelatex from `MUSICIANS-GUIDE.md`.
- 61 KB, PDF 1.5, table of contents.
- Emoji characters (⚠, ✅, ❌, 🔴) render as blank in lmroman font — cosmetic only, text content complete.

### Test Results

| Crate | Tests |
|-------|-------|
| mix-engine | 79 |
| api-server (integration) | 19 |
| audio-engine | 20 |
| control-server | 13 |
| control-protocol | 7 |
| doc-tests | 3 |
| **Total** | **157** |

### Not Yet Done (Phase 9 targets)
- Admin API server-side routes (`/api/v1/admin/users`, `/api/v1/admin/sessions`).
- Biquad coefficient validation vs scipy reference.
- ARM64 CI cross-build improvement.

---

## 2026-09-09 — Phase 7: Biquad EQ + RMS Compressor DSP

**Branch:** `feat/phase7-dsp` → squash-merge pending  
**Tests:** 152 passed (↑ from 135), 0 failed

### Implemented

#### Biquad Parametric EQ (`mix-engine/src/eq.rs`)
- Replaced Phase 2 passthrough stub with real Type-II Transposed Direct Form II (TDF2) peaking biquad filter.
- Coefficients follow Audio EQ Cookbook (RBJ): b0/b1/b2/a1/a2 for peaking filter at configured frequency/gain/Q.
- `SAMPLE_RATE = 48_000.0` constant; coefficients recomputed on `set_band`.
- `BiquadCoeffs { b0, b1, b2, a1, a2 }` — `identity()` and `peaking(frequency_hz, gain_db, q)`.
- `BiquadState { w1_l, w2_l, w1_r, w2_r }` — stereo delay lines inline, zero heap allocation.
- `ParametricEq::process(&mut self, l, r) -> (f32, f32)` — applies all enabled bands sequentially.
- API surface maintained: `bands: [EqBand; 4]`, `revision`, `set_band(index, band)`.
- Breaking change: `process` now `&mut self` (state mutation required for biquad).

#### RMS Compressor (`mix-engine/src/compressor.rs`)
- Replaced Phase 2 passthrough stub with stereo-linked RMS detector + smoothed gain reduction.
- Stereo link: detector uses `max(|L|, |R|)`.
- RMS: exp-moving-average of x² using configurable attack/release coefficients.
- Gain reduction: `(1 - 1/ratio) * (threshold_db - rms_db)` when RMS above threshold.
- Smoothed envelope: separate attack/release on gain_reduction_db.
- New mutators: `set_threshold`, `set_ratio`, `set_attack_ms`, `set_release_ms` — all bump revision.
- State inline: `rms_state: f32`, `gain_db: f32` — no heap allocation.
- Disabled: passthrough.

#### Documentation & Infrastructure
- `docker-compose.yml` — dev environment with api-server, musician-ui, engineer-ui.
- `docs/guides/MUSICIANS-GUIDE.md` — 14-section guide in pt-BR (server setup, login, mix control, WebRTC status, permissions, diagnostics, LAN, security, limitations).
- `docs/reviews/PHASE-6-REVIEW.md` — Phase 6 review (was missing).
- `docs/reviews/PHASE-7-REVIEW.md` — Phase 7 review.
- CHANGELOG, TODO, DEVELOPMENT-LOG updated.
- `cargo fmt --all` applied.

### Test Results

| Crate | Tests |
|-------|-------|
| mix-engine | 74 |
| api-server (integration) | 19 |
| audio-engine | 20 |
| control-server | 13 |
| control-protocol | 7 |
| doc-tests | 3 |
| **Total** | **152** |

### Not Yet Done (Phase 8 targets)
- Integrate EQ + Compressor into `Mix::process` audio chain.
- Admin CLI for user management.
- Musician Guide PDF generation.
- EQ coefficient validation vs reference implementation (scipy).

---

## 2026-09-08 — Phase 6: Real trickle-ICE injection + HTTP integration tests

**Branch:** `feat/phase6-trickle-ice`

### Implemented

- **streaming crate:** `add_ice_candidate` now performs real RFC 5245 candidate injection via `Candidate::from_sdp_string` + `Rtc::add_remote_candidate` (str0m). Previous stub only checked session existence. SIMULATED on VPS; no `poll_output` I/O loop runs until Raspberry Pi hardware.
- **streaming crate:** Named constants `MAX_SDP_BYTES`, `MAX_CANDIDATE_BYTES`, `MAX_USER_ID_BYTES` replace inline magic numbers.
- **streaming crate:** 4 new unit tests: `malformed_candidate_rejected_before_session_lookup`, `oversized_candidate_rejected`, `valid_candidate_injected_after_offer`, `candidate_rejected_when_no_session`.
- **api-server:** Full HTTP integration test suite added (`server/api-server/tests/integration.rs`) — 19 tests covering health, auth, RBAC (Musician/Engineer/Admin), CSRF/Origin, audio routes, channel controls.
- **api-server:** Test fixture role casing fixed (`"Musician"` → `"MUSICIAN"`, SCREAMING_SNAKE_CASE); previous fixtures caused 422 instead of testing actual role enforcement.
- **api-server:** `axum-test` pinned to `"21"` (was `"16"`, resolved to 21.1.0). `jsonwebtoken` gains `rust_crypto` feature.

**Verification:** `cargo test --workspace` — 135 tests passed, 0 failed. Independent reviewer: `passed=true`, no security concerns, no logic errors.

---

## 2026-09-08 — Phase 5: Audio transport signaling scaffold

**Branch:** `feat/phase5-audio-transport`

### Implemented

- Accepted ADR-004: WebRTC primary transport; RTP/UDP reserved for dedicated receivers.
- Added `server/streaming` Rust crate using `str0m 0.23`.
- Added bounded SDP offer validation and per-musician WebRTC session registry.
- Added bounded ICE candidate validation and session lifecycle/list operations.
- Defined SIMULATED 48 kHz stereo, 20 ms silence-frame contract for VPS development.
- Added Phase 5 specification and review.

**Verification:** `cargo fmt --all`, `cargo test --workspace` (122 passed), and `cargo clippy --workspace --all-targets -- -D warnings` pass. No Raspberry Pi hardware available on VPS; PipeWire and real Opus media remain pending.

---

## 2026-09-08 — Phase 5: Authenticated audio signaling routes

**Branch:** `main` (working tree delivery; commit pending)

### Implemented

- Wired `server/streaming` into `api-server` application state.
- Added authenticated `POST /api/v1/audio/offer` for SDP negotiation.
- Added authenticated `POST /api/v1/audio/ice-candidate` for bounded ICE signaling.
- Added engineer-only `GET /api/v1/audio/sessions`.
- Added `SessionInfo` JSON serialization and bounded `mix_id` validation.
- Mapped SDP parser failures to generic client errors.

**Verification:** `cargo fmt --all -- --check`, `cargo test --workspace`, and `cargo clippy --workspace --all-targets -- -D warnings` pass. Independent review initially found input/error-boundary gaps; fixes applied and re-review passed. VPS remains **SIMULATED**: no Raspberry Pi, PipeWire, or real Opus path.

---

## 2026-09-08 — Phase 4: Musician PWA and Engineer scaffold

**Branch:** `feat/phase4-frontend-init` → squash-merged to `main` as PR #3
**Commit:** `b8ca865`

### Implemented

#### Musician PWA (`web/musician/`)
- Vite 5 + React 18 + TypeScript strict, mobile-first CSS modules
- `src/api/auth.ts`: login/refresh/logout via fetch with `credentials:'include'`
- `src/hooks/useWebSocket.ts`: WS hook connecting to `/ws/v1?token=<jwt>` (browser WS cannot set headers — known limitation, short TTL + LAN mitigation)
- `src/components/Login.tsx`: form, in-memory token storage (never localStorage)
- `src/components/Channel.tsx`: gain slider (−60 to +6 dBFS, step 0.5), mute toggle
- `src/components/MixControl.tsx`: 8 channels, master volume, logout
- `src/components/ConnectionStatus.tsx`: status dot + revision display
- `src/protocol/types.ts + envelope.ts`: matches server `control-protocol` wire format
- PWA manifest + SVG icons (192px, 512px)
- 25 vitest tests passing, typecheck clean, build: 150 KB JS (48 KB gzip)

#### Engineer Scaffold (`web/engineer/`)
- Phase 6 placeholder page
- 2 vitest tests passing, typecheck clean, build: 143 KB JS

### Security review
- Independent reviewer: **passed** (0 security, 0 logic errors)
- 3 non-blocking suggestions logged (master volume wire-up Phase 6, WS reconnect design, log scrubbing for Pi)

### Post-merge test results
- web/musician: 25/25 tests passing
- web/engineer: 2/2 tests passing

---

## 2026-09-08 — Phase 3: Security boundary hardening — follow-up

**Branch:** `fix/phase3-auth-boundaries`

- Added Origin validation for browser state-changing requests and rejected cookie-authenticated requests without Origin.
- Bounded login and admin user-creation username/password inputs before Argon2 work.
- Refresh rotation now revokes old token and inserts replacement in one SQLite transaction.
- API defaults to loopback HTTP; non-loopback insecure bind fails unless explicit isolated-development override is set.
- WebSocket enforces role checks, 16 KiB messages, 120 messages/minute, and closes at JWT expiry.
- Global HTTP body limit set to 16 KiB.
- TLS reverse-proxy deployment documented in `docs/deployment/TLS.md`.

**Test results:** `cargo fmt --all`: PASS; `cargo test --workspace`: 99 tests passed; `cargo clippy --workspace --all-targets -- -D warnings`: PASS.

---

## 2026-09-08 — Phase 3: Security boundary hardening

**Branch:** `fix/phase3-auth-boundaries`

- Independent security review found unresolved BLOCKER/HIGH gaps: plaintext HTTP, unauthorised WebSocket mutations, missing token expiry/revocation on sockets, non-atomic refresh rotation, missing CSRF/origin validation, and unbounded auth inputs.
- Fixed refresh expiry boundary (`now >= expires`), explicit `OsRng` refresh-token generation, bounded WebSocket message/request IDs, and REST gain range validation.
- Remaining BLOCKER/HIGH findings are tracked in `docs/TODO.md`; no merge is allowed until resolved.

**Test results:** `cargo fmt --all -- --check`: PASS; `cargo test --workspace`: 99 tests passed; `cargo clippy --workspace --all-targets -- -D warnings`: PASS.

---

## 2026-09-08 — Phase 3: Control Server State Dispatcher

**Branch:** `feat/phase3-control-protocol`

- Added `server/control-server/`, a tested control-plane consumer of `control-protocol` and `mix-engine`.
- Implemented revision-aware dispatch for `GetState`, `SetChannelGain`, and `SetChannelMute`.
- Invalid channel indexes and non-finite gains fail without state mutation.
- Accepted ADR-008: Ed25519 JWT access tokens, rotating opaque refresh tokens, Argon2id passwords, HTTPS-only cookies/WebSocket upgrades. Authentication implementation remains pending.

**Test results:** `cargo test --workspace`: 89 tests passed (86 unit + 3 doc tests); `cargo clippy --workspace --all-targets -- -D warnings`: PASS; `cargo fmt --all -- --check`: PASS.

---

## 2026-09-08 — Phase 3: Versioned Control Protocol Foundation

**Branch:** `feat/phase3-control-protocol`

- Added `server/control-protocol/` crate with JSON/Serde envelope, protocol version validation, role catalog, and typed client/server message catalog.
- Added round-trip and unsupported-version tests.
- Control protocol is control-plane only; it does not touch realtime audio path.

**Test results:** `cargo test --workspace`: 82 tests passed (20 audio-engine, 57 mix-engine, 2 control-protocol, 3 doc tests); `cargo clippy --workspace --all-targets -- -D warnings`: PASS.

---

## 2026-09-08 — Phase 0: Bootstrap & Specification Audit

### Status: IN PROGRESS

### Actions

- Read and analyzed `START.md` (2135 lines — full engineering master prompt)
- Verified GitHub remote: `https://github.com/rickslamaral/open-iem-platform`
- Pulled existing commit (initial commit with minimal README)
- Environment audit: Ubuntu 25.10, x86_64, 4 CPUs, 15 GiB RAM
- Installed Rust 1.98.1 via rustup
- Created full project directory structure
- Created 11 project-specific agent skills in `.agents/skills/`
- Created documentation structure (`docs/` with all subdirectories)
- Created ADR baseline (ADR-001 through ADR-008)
- Created `docs/SPEC-AUDIT.md`
- Created `docs/ARCHITECTURE-GAPS.md`
- Created `docs/SKILLS.md`
- Created `docs/TODO.md`
- Created `docs/DEVELOPMENT-ENVIRONMENT.md`
- Created CI/CD foundation (GitHub Actions)
- Created `scripts/validate-skills.sh`
- Created symlinks `.hermes/skills/` and `.claude/skills/` → `.agents/skills/`
- Created root files: README, CONTRIBUTING, SECURITY, LICENSE, CHANGELOG, .gitignore
- Bootstrap commit to main

### Environment Findings

- PipeWire not available on VPS (expected — audio hardware required for Phase 1 validation)
- Rust not pre-installed — installed via rustup
- Node 22 LTS available
- Docker available

### Next

Phase 0 Review → PASS → Phase 1 (Audio Engine POC)

---

## 2026-09-08 — Phase 1: Audio Engine POC (Research + Rust Workspace)

**Agent:** Autonomous Engineering Agent (cron)
**Branch:** main
**Commit:** (pending — see below)

### Completed

#### Research Documents Created

| Document | Gap Closed | Status |
|----------|-----------|--------|
| `docs/audio/LATENCY-BUDGET.md` | GAP-005 | DEFINED |
| `docs/audio/AUDIO-SLA.md` | GAP-006 | DEFINED |
| `docs/research/pipewire-integration.md` | GAP-003 | CLOSED — pipewire-jack selected for Phase 1 |
| `docs/research/realtime-scheduling.md` | GAP-008 | CLOSED — PipeWire+rtkit managed for Phase 1 |
| `docs/research/audio-transport/EVALUATION.md` | GAP-001 (partial) | PRELIMINARY — WebRTC selected as primary candidate |

#### Rust Workspace Initialized

- `server/Cargo.toml` — workspace root, resolver = "2", workspace lints
- `server/mix-engine/Cargo.toml` — crate definition
- `server/mix-engine/src/lib.rs` — public API, constants, `db_to_linear`, `linear_to_db`, `apply_pan`
- `server/mix-engine/src/channel.rs` — `Channel` struct
- `server/mix-engine/src/mix_send.rs` — `MixSend` struct  
- `server/mix-engine/src/limiter.rs` — `Limiter` stub
- `server/mix-engine/src/mix.rs` — `Mix` struct with `process()`
- `server/mix-engine/src/mix_engine.rs` — `MixEngine` top-level coordinator

#### Key Design Decisions

- No heap allocation in audio path (`core::array::from_fn`, fixed-size arrays)
- No I/O in `process_frame()` — enforced by code structure and `#[deny]` directives
- Revision counter (monotonic `u64`) on every mutable type
- Limiter is a stub (hard-clip) — proper lookahead planned for Phase 2
- `GAIN_DB_MIN = -144.0 dBFS` (practical –∞), `GAIN_DB_MAX = +12.0 dBFS`
- Equal-power (sine/cosine law) pan
- Solo logic evaluated at Mix level, not MixSend level

#### Test Results

```
cargo test: 50 unit tests + 3 doc tests = 53 total — ALL PASS
cargo clippy -- -D warnings: 0 warnings
cargo fmt: applied
skills validate: 11/11 PASS
```

### Gap Status After This Session

| Gap | Before | After |
|-----|--------|-------|
| GAP-001 Audio Transport | UNRESOLVED | PARTIALLY CLOSED |
| GAP-003 PipeWire Integration | UNRESOLVED | CLOSED |
| GAP-005 Latency Budget | UNDEFINED | DEFINED |
| GAP-006 XRUN SLA | UNDEFINED | DEFINED |
| GAP-008 RT Scheduling | UNRESOLVED | CLOSED |

### Not Yet Done (Phase 1 continuation)

- Hardware testing (Raspberry Pi 5 + USB audio) — requires hardware
- PipeWire filter node registration — requires PipeWire on target
- Audio I/O integration — Phase 1b
- ADR-004 final update with benchmark data — Phase 5

### 2026-09-08 — Phase 1 audio-engine integration validated

- Validated `server/audio-engine/` simulated backend and feature-gated JACK bridge.
- `cargo test --workspace`: 73 tests passed (20 audio-engine, 50 mix-engine, 3 doc tests).
- `cargo clippy -p audio-engine`: no warnings.
- Real PipeWire/JACK execution remains **SIMULATED** until Raspberry Pi 5 hardware is available.
- Phase 2 limiter replacement remains pending; current limiter is hard-clip by design.

### 2026-09-08 — Phase 2: Mix Engine complete

**Implemented:**

#### Lookahead Brick-wall Limiter (`mix-engine/src/limiter.rs`)
- Replaced Phase 1 hard-clip stub with proper lookahead limiter
- `LOOKAHEAD_FRAMES = 64` (1.33 ms @ 48 kHz — within IEM latency budget)
- Attack coef = `exp(-1/(48000×0.0005))` (0.5 ms), Release = `exp(-1/(48000×0.1))` (100 ms)
- Brick-wall guaranteed: `envelope_gain` never exceeds `target_gain`
- Zero heap allocation — fixed `[f32; LOOKAHEAD_FRAMES*2]` inline buffer
- `Limiter::process` now `&mut self` (state-mutating); `Mix::process` and `MixEngine::process_frame` updated to `&mut self` accordingly
- `Limiter::reset()` clears buffer and envelope (call on engine restart)

#### Parametric EQ stub (`mix-engine/src/eq.rs`)
- `ParametricEq` with `MAX_EQ_BANDS=4`, `EqBand { frequency_hz, gain_db, q, enabled }`
- `process(&self, l, r) → (l, r)` passthrough; biquad DSP deferred to Phase 7
- Revision counter on every mutation

#### Compressor stub (`mix-engine/src/compressor.rs`)
- `Compressor { threshold_db, ratio, attack_ms, release_ms, enabled }`
- `process(&self, l, r) → (l, r)` passthrough; dynamics DSP deferred to Phase 7
- Safe defaults: -18 dBFS threshold, 4:1 ratio, 10/100 ms attack/release

#### API propagation
- `Mix::process` → `&mut self` (required for mutable limiter)
- `MixEngine::process_frame` → `&mut self`
- All test fixtures updated accordingly

**Test results:** 80 tests, 0 failed, 0 clippy warnings
- audio-engine: 20 tests
- mix-engine: 57 tests (50 original + 7 new limiter tests)
- doc-tests: 3

### 2026-09-09 — Phase 9: Admin API server-side routes + biquad validation

**Implemented:**

#### Admin API Routes (`server/api-server/src/routes/admin.rs`)
- 4 new endpoints, all Admin-role-only via `require_min_role`:
  - `GET /api/v1/admin/users` — list all users (id, username, role)
  - `DELETE /api/v1/admin/users/{id}` — delete user (204) or not found (404)
  - `GET /api/v1/admin/sessions` — list active refresh-token sessions
  - `DELETE /api/v1/admin/sessions/{id}` — revoke session by ID (204/404)
- All routes in protected Router (behind JWT auth middleware)
- `ApiError::NotFound(String)` variant added for 404 responses

#### DB Layer (`server/api-server/src/db.rs`)
- `list_users()` — parameterized SELECT, returns Vec<(i64, String, Role)>
- `delete_user(user_id: i64)` — DELETE, NotFound if affected==0
- `list_active_sessions(now_unix: u64)` — non-revoked, non-expired refresh tokens
- `revoke_session_by_id(session_id: i64)` — UPDATE, NotFound if affected==0
- Sessions cascade-deleted on user delete (existing FK ON DELETE CASCADE)

#### Admin CLI Fix (`server/admin-cli/src/main.rs`)
- `user create --username --password --role` (was `--name`)
- `session revoke --id` (was `--token`, now numeric ID)

#### Testing
- 8 new HTTP integration tests in api-server/tests/integration.rs
- 3 new DB unit tests
- Total: 157 → 168 tests, all green

#### Biquad Validation
- Python RBJ coefficients vs Rust implementation: max delta 5×10⁻⁸ (f32 rounding only)
- Identity check (gain=0): b0=1.0, b1=a1, b2=a2 — PASS in both

#### CI
- Added npm-audit job (HIGH severity gate for musician/engineer frontends)

**Test results:** 168 passed, 0 failed, 0 clippy warnings, code formatted
- admin-cli: 0 tests (binary)
- api-server: 27 tests (was 19, +8 admin integration)
- api-server DB: 16 tests (was 13, +3 admin DB unit)
- mix-engine: 79 tests, audio-engine: 20 tests, control-server: 13 tests, streaming: 7 tests, doc-tests: 3


### 2026-09-09 — Phase 10: mix assignment and ownership enforcement

Implemented SQLite mix assignments, JWT `uid`, Engineer/Admin assignment routes, musician-owned send gain/pan/mute routes, and control-state mix accessors. VPS validation remains SIMULATED for PipeWire/Opus.

Added three HTTP integration tests for assignment, musician ownership denial, and gain validation. Assignment writes now reject slot conflicts without `INSERT OR REPLACE` data loss.

**Verification:** cargo fmt PASS; cargo clippy --all-targets -- -D warnings PASS; cargo test --all PASS: 171 tests, 0 failures.


### 2026-09-09 — Phase 11: versioned release pipeline

Implemented `.github/workflows/release.yml` with 6-stage pipeline: version validation gate, quality gate (fmt+clippy+test+audit), Linux x86_64 build, Linux ARM64 cross-compile (Raspberry Pi 5 target), web frontend builds, and GitHub Release with artefacts + SHA-256 checksums.

Bumped workspace version `0.1.0` → `0.2.0`. Aligned `admin-cli` Cargo manifest to workspace. Added `server/.cargo/config.toml` for ARM64 linker.

**Limitations:** ARM64 cross-compiled but SIMULATED — not tested on real Pi hardware. Windows/macOS UNSUPPORTED (toolchain). PipeWire/Opus remain SIMULATED on VPS.

**Verification:** cargo fmt PASS; cargo clippy PASS; cargo test --all PASS: 171 tests, 0 failures. Static scan clean. Independent reviewer: passed=true.

### 2026-09-09 — Documentation validation hardening

**Goal:** validate required documentation and release metadata on every CI/development round.

**Implemented:** Added `scripts/validate-docs.sh`, checking required Markdown/PDF files, SemVer workspace version, single CHANGELOG `[Unreleased]` section, and non-empty Musician Guide PDF. Removed duplicate `[Unreleased]` heading from CHANGELOG.

**Tests:** Documentation validator PASS (`version 0.2.0`); `git diff --check` PASS. Rust and frontend tests remain green from this run.

**Limitations:** PDF content extraction and rendering remain outside this lightweight validator. PipeWire/Opus and ARM64 runtime remain SIMULATED.

**Next Step:** Add validator to CI, then continue HTTP/WebSocket integration hardening.
