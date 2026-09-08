# TODO

Priority labels: **BLOCKER** | **HIGH** | **MEDIUM** | **LOW** | **RESEARCH**

---

## BLOCKER

- None currently.

---

## HIGH

- [ ] Install PipeWire on target hardware (Raspberry Pi 5) for Phase 1 validation
- [ ] Evaluate audio transport options before Phase 5 (`docs/research/audio-transport/EVALUATION.md`)
- [ ] Establish authentication mechanism decision (ADR-008 — Proposed state)
- [ ] Add ARM64 cross-compilation to CI (requires `aarch64-unknown-linux-gnu` setup)

---

## MEDIUM

- [ ] Initialize Rust workspace (`server/Cargo.toml`)
- [ ] Initialize frontend projects (`web/musician/`, `web/engineer/`)
- [ ] Create Docker Compose for local development
- [ ] Define PipeWire filter node architecture for Mix Engine
- [ ] Design channel/mix state machine (revision control)
- [ ] Define WebSocket message type catalog

---

## LOW

- [ ] Set up `cargo audit` in CI
- [ ] Set up `npm audit` in CI
- [ ] Create `examples/` with minimal mix scenario
- [ ] Configure Dependabot for dependency updates
- [ ] Set up code coverage reporting

---

## RESEARCH

- [ ] **Audio Transport** — Compare RTP/UDP, WebRTC, WebTransport/QUIC, custom UDP for latency/jitter/browser compat
- [ ] **Browser Audio Constraint** — Verify what browsers can receive (WebRTC vs native receiver architecture)
- [ ] **ESP32-S3 / ESP32-P4** — I2S, DAC, Wi-Fi, latency, power budget (Phase 10)
- [ ] **PipeWire filter node API** — Best approach for Mix Engine integration
- [ ] **Raspberry Pi 5 realtime tuning** — PREEMPT_RT kernel, PipeWire latency config, USB audio device selection
- [ ] **JPMixer architecture** — Study WebSocket/scene/mix model as UX reference (verify license before using code)
- [ ] **Linux realtime scheduling** — SCHED_FIFO vs PipeWire-managed priority for audio threads

---

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
