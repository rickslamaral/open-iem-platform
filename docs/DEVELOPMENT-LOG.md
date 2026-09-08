# Development Log

All significant milestones documented here in reverse chronological order.

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
