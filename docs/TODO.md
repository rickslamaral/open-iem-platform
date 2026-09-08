# TODO

Priority labels: **BLOCKER** | **HIGH** | **MEDIUM** | **LOW** | **RESEARCH**

---

## BLOCKER

- None currently.

---

## HIGH

- [ ] Install PipeWire on target hardware (Raspberry Pi 5) for Phase 1 validation
- [x] Evaluate audio transport options; WebRTC selected in ADR-004 (Phase 5 signaling scaffold complete)
- [x] Wire authenticated audio signaling routes and engineer session listing (Phase 5)
- [x] Establish authentication mechanism decision (ADR-008 — Accepted; implementation pending)
- [ ] Add ARM64 cross-compilation to CI (requires `aarch64-unknown-linux-gnu` setup)

---

## MEDIUM

- [x] Initialize Rust workspace (`server/Cargo.toml`)
- [x] Initialize frontend projects (`web/musician/`, `web/engineer/`) — Phase 4 complete
- [ ] Create Docker Compose for local development
- [x] Define PipeWire filter node architecture for Mix Engine (pipewire-jack — see docs/research/pipewire-integration.md)
- [x] Design channel/mix state machine (revision control — implemented in mix-engine)
- [x] Define WebSocket message type catalog (`server/control-protocol/`; Phase 3 foundation)

---

## LOW

- [ ] Set up `cargo audit` in CI
- [ ] Set up `npm audit` in CI
- [ ] Create `examples/` with minimal mix scenario
- [ ] Configure Dependabot for dependency updates
- [ ] Set up code coverage reporting

---

## RESEARCH

- [x] **Audio Transport** — Preliminary evaluation complete (docs/research/audio-transport/EVALUATION.md); WebRTC selected as primary; final benchmarks deferred to Phase 5
- [ ] **Browser Audio Constraint** — Verify what browsers can receive (WebRTC vs native receiver architecture)
- [ ] **ESP32-S3 / ESP32-P4** — I2S, DAC, Wi-Fi, latency, power budget (Phase 10)
- [x] **PipeWire filter node API** — pipewire-jack selected for Phase 1 (docs/research/pipewire-integration.md)
- [ ] **Raspberry Pi 5 realtime tuning** — PREEMPT_RT kernel, PipeWire latency config, USB audio device selection
- [ ] **JPMixer architecture** — Study WebSocket/scene/mix model as UX reference (verify license before using code)
- [x] **Linux realtime scheduling** — PipeWire+rtkit for Phase 1; hybrid in Phase 2 (docs/research/realtime-scheduling.md)

---

## PHASE 3 SECURITY FOLLOW-UP

- [ ] HIGH: Add HTTPS/TLS listener and fail-closed transport configuration
- [ ] HIGH: Authorize every WebSocket message by role and musician mix ownership
- [x] HIGH: Enforce WebSocket expiry and rate limits; revocation remains bounded by JWT TTL
- [x] HIGH: Make refresh rotation atomic in one SQLite transaction
- [x] MEDIUM: Add Origin/CSRF validation and bounded auth request inputs
- [x] MEDIUM: Bound HTTP request body to 16 KiB; route payload validation remains pending
- [x] MEDIUM: Parse bind address as SocketAddr and fail closed for non-loopback HTTP without explicit dev override
- [x] MEDIUM: Preflight refresh token owner and JWT before atomic rotation to avoid token loss on issuance failure
- [ ] MEDIUM: Enforce musician mix ownership after mix assignment model exists

## PHASE 1 STATUS

- [x] Audio engine crate with simulated backend and feature-gated JACK/PipeWire bridge
- [x] Phase 1 review completed: PASS WITH CONDITIONS
- [ ] Validate real PipeWire graph on Raspberry Pi 5 (hardware blocker)

## COMPLETED

- [x] Repository initialized with correct GitHub remote
- [x] Environment audited and documented
- [x] Project directory structure created
- [x] 11 project skills created in `.agents/skills/`
- [x] Documentation structure created (`docs/`)
- [x] ADR baseline created (ADR-001 through ADR-008)
- [x] CI foundation created (GitHub Actions)
- [x] `scripts/validate-skills.sh` created
- [x] Root project files created (README, CONTRIBUTING, SECURITY, LICENSE, CHANGELOG, .gitignore)
- [x] Latency budget defined (`docs/audio/LATENCY-BUDGET.md`) — GAP-005 CLOSED
- [x] XRUN SLA defined (`docs/audio/AUDIO-SLA.md`) — GAP-006 CLOSED
- [x] Mix Engine Rust crate implemented (Channel, MixSend, Mix, Limiter, MixEngine)
- [x] 53 unit + doc tests passing — 0 clippy warnings

- [x] Lookahead brick-wall limiter (LOOKAHEAD_FRAMES=64 @ 48kHz = 1.33ms)
- [x] ParametricEq stub (passthrough — biquad DSP Phase 7)
- [x] Compressor stub (passthrough — dynamics DSP Phase 7)
- [x] 80 tests passing — 0 clippy warnings
