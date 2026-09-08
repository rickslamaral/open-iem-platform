# Specification Audit

**Date:** 2026-09-08
**Phase:** 0 — Bootstrap & Specification Audit
**Source:** `START.md` (2135 lines)

---

## 1. Existing Requirements (from START.md)

### Functional

| ID | Requirement | Status |
|----|-------------|--------|
| FR-01 | 8 audio inputs | Specified |
| FR-02 | 2 independent stereo mixes | Specified |
| FR-03 | 2 musicians / 2 client connections | Specified |
| FR-04 | Per-channel: gain, pan, mute, solo, enabled, locked | Specified |
| FR-05 | Per-mix: master, limiter, revision | Specified |
| FR-06 | Scenes: create, save, rename, duplicate, recall, delete, export, import | Specified |
| FR-07 | REST API `/api/v1` with versioning | Specified |
| FR-08 | WebSocket `/ws/v1` with envelope format | Specified |
| FR-09 | Musician PWA: volume, pan, mute, master | Specified |
| FR-10 | Engineer UI: channels, musicians, mixes, devices, scenes, locks, audio, network, system, logs | Specified |
| FR-11 | Role-based access: ADMIN, ENGINEER, MUSICIAN | Specified |
| FR-12 | CLI: `open-iem status/audio/channels/mixes/devices/diagnostics/scenes` | Specified |
| FR-13 | Revision control on mixes (stale command rejection) | Specified |
| FR-14 | Scene import/export | Specified |

### Non-Functional

| ID | Requirement | Status |
|----|-------------|--------|
| NFR-01 | Realtime-safe audio path (no blocking I/O in audio callback) | Specified |
| NFR-02 | 48 kHz / 24-bit capture / 32-bit float internal | Specified |
| NFR-03 | ARM64 + x86_64 support | Specified |
| NFR-04 | Linux-native (PipeWire, ALSA, systemd) | Specified |
| NFR-05 | Local LAN only (no cloud dependency) | Specified |
| NFR-06 | Limiter on master output (audio safety) | Specified |
| NFR-07 | Safe startup: mute until state verified | Specified |
| NFR-08 | Observability: CPU, RAM, XRUN, latency, jitter, packet loss | Specified |
| NFR-09 | Hardware abstraction (no RPi hard-coding in core) | Specified |
| NFR-10 | Open source (Apache 2.0 implied) | Specified |

---

## 2. Missing Requirements

| ID | Missing Requirement | Risk | Recommendation |
|----|---------------------|------|----------------|
| MR-01 | Latency budget (end-to-end target in ms) | HIGH — no target means no validation | Define: target ≤25ms E2E, ≤5ms audio buffer latency |
| MR-02 | Maximum number of XRUNs acceptable per session | MEDIUM | Define SLA: 0 XRUNs per 1h session |
| MR-03 | Session management / token lifetime | HIGH — security gap | Define token expiry and refresh strategy |
| MR-04 | Audio transport selection (RTP, WebRTC, etc.) | BLOCKER for Phase 5 | Research required before implementation |
| MR-05 | Quantified packet loss tolerance | MEDIUM | Define: ≤1% packet loss before degraded mode |
| MR-06 | Maximum connected musicians | MEDIUM | Define MVP limit: 2, extensible to 16+ |
| MR-07 | USB audio device hot-plug behavior | MEDIUM | Define: mute on disconnect, reconnect auto-recover |
| MR-08 | Upgrade / migration path for SQLite schema | MEDIUM | Define: sqlx-migrate, reversible migrations |
| MR-09 | PWA update strategy (service worker) | LOW | Define: prompt user on update |
| MR-10 | Log retention / rotation policy | LOW | Define: structured JSON, 7-day retention |

---

## 3. Contradictions

| ID | Description | Resolution |
|----|-------------|------------|
| C-01 | Browser audio constraint not resolved: START.md says "do not assume browser can receive UDP" but doesn't mandate a specific transport | Requires formal evaluation — ADR-004 deferred to Phase 5 research |
| C-02 | VPS used for development but PipeWire not available on VPS | Expected and documented — VPS for API/backend/CI, RPi for audio hardware |

---

## 4. Risks

| ID | Risk | Severity | Mitigation |
|----|------|----------|------------|
| R-01 | Audio transport selection may invalidate PWA-only approach | HIGH | Complete transport evaluation before Phase 5 |
| R-02 | PipeWire API stability | MEDIUM | Pin PipeWire version in deployment, document tested version |
| R-03 | Realtime constraints on Raspberry Pi 5 without PREEMPT_RT kernel | MEDIUM | Research RPi5 RT kernel availability |
| R-04 | Browser WebRTC for audio has significant implementation complexity | MEDIUM | Evaluate native receiver as alternative |
| R-05 | ESP32 receiver viability unproven | LOW | Research gate in place (Phase 10 only after research) |
| R-06 | Supply chain: Rust crate ecosystem dependencies | LOW | `cargo audit` in CI, pin dependency versions |

---

## 5. Open Decisions

| ID | Decision | Owner | Due |
|----|----------|-------|-----|
| OD-01 | Audio transport protocol (RTP, WebRTC, WebTransport, custom) | architect-designer + realtime-audio-engineer | Before Phase 5 |
| OD-02 | Authentication mechanism (JWT, session token, mTLS) | architect-designer + security-review | Before Phase 3 |
| OD-03 | PipeWire filter node vs external graph management | realtime-audio-engineer | Before Phase 1 |
| OD-04 | WebSocket auth model (per-connection vs per-message) | senior-backend + security-review | Before Phase 3 |
| OD-05 | Latency budget (E2E target ms) | realtime-audio-engineer | Before Phase 1 |

---

## 6. Recommended Changes

1. Add quantified latency budget to NFR (target ≤25ms E2E) before Phase 1
2. Define XRUN SLA before audio hardware testing
3. Complete audio transport evaluation before any Phase 5 work
4. Define authentication mechanism in ADR-008 before Phase 3
5. Add USB hot-plug behavior specification to `docs/product/features/`
