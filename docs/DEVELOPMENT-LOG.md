# Development Log

All significant milestones documented here in reverse chronological order.

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
