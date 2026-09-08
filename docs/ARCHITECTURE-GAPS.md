# Architecture Gaps

**Date:** 2026-09-08
**Phase:** 0 — Bootstrap & Specification Audit

---

## Summary

Gaps identified by analyzing `START.md` against known requirements for a production realtime audio platform. Each gap must be closed before the phase that depends on it.

---

## GAP-001: Audio Transport Protocol — UNRESOLVED (Critical)

**Required by:** Phase 5
**Description:** No protocol selected for streaming audio from server to musician devices. START.md explicitly forbids assuming RTP/UDP is the final solution and mandates a formal evaluation.
**Risk:** Wrong transport choice → rework entire streaming stack
**Resolution:** Complete `docs/research/audio-transport/EVALUATION.md` before Phase 5 begins. Create ADR-004.

---

## GAP-002: Browser Audio Receive Constraint — UNRESOLVED (High)

**Required by:** Phase 4 (PWA) / Phase 5 (Transport)
**Description:** Browsers cannot receive arbitrary UDP audio. Architecture must formally separate:
- Control client (WebSocket/PWA — browser)
- Audio receiver (WebRTC, native app, or dedicated hardware)

The current spec does not resolve this split with a concrete decision.
**Resolution:** Define in ADR-005. Audio receiver path must be designed independently of the PWA control path.

---

## GAP-003: PipeWire Integration Model — UNRESOLVED (High)

**Required by:** Phase 1
**Description:** START.md states Open IEM should register as a PipeWire filter node. The exact integration approach (pw-filter API vs pipewire-jack vs pipewire-pulse) is not specified. PipeWire API surface is large and the correct approach depends on latency requirements.
**Resolution:** Research and document in `docs/research/pipewire-integration.md`. Create ADR.

---

## GAP-004: Authentication Mechanism — UNRESOLVED (High)

**Required by:** Phase 3
**Description:** Security model defines roles (ADMIN, ENGINEER, MUSICIAN) but no authentication mechanism is specified. JWT, session tokens, mTLS, and pre-shared keys are all possible given the local LAN context.
**Resolution:** Complete ADR-008. Consider local LAN threat model — full mTLS may be over-engineered for MVP.

---

## GAP-005: Latency Budget — UNDEFINED (High)

**Required by:** Phase 1 (audio hardware testing)
**Description:** No end-to-end latency target is specified. Without a target, latency validation is impossible.
**Proposed:** 
- Audio buffer: ≤5ms (at 48kHz, 240-sample buffer)
- Control latency: ≤50ms (WebSocket round-trip)
- End-to-end (instrument → IEM): ≤25ms target, ≤35ms maximum
**Resolution:** Quantify and document in `docs/audio/LATENCY-BUDGET.md`. Confirm with realtime-audio-engineer.

---

## GAP-006: XRUN SLA — UNDEFINED (Medium)

**Required by:** Phase 1 (audio testing)
**Description:** No acceptable XRUN rate defined. Without this, hardware validation has no pass/fail criterion.
**Proposed:** 0 XRUNs per 1-hour session at target buffer size.
**Resolution:** Document in `docs/audio/AUDIO-SLA.md`.

---

## GAP-007: USB Hot-Plug Behavior — UNSPECIFIED (Medium)

**Required by:** Phase 1
**Description:** Behavior when audio interface disconnects is not specified. Audio safety requires defined fallback.
**Proposed:** On device disconnect → immediate mute all outputs → log event → attempt auto-recover → notify clients.
**Resolution:** Document in `docs/product/features/USB-HOTPLUG.md`.

---

## GAP-008: Realtime Thread Priority Model — UNRESOLVED (Medium)

**Required by:** Phase 1
**Description:** Whether realtime scheduling is managed by PipeWire or by the application is not resolved. PipeWire typically manages realtime priority for its own threads; application nodes may require additional configuration.
**Resolution:** Research in `docs/research/realtime-scheduling.md`.

---

## GAP-009: SQLite Concurrency Model — UNRESOLVED (Medium)

**Required by:** Phase 3
**Description:** SQLite has concurrency limitations (WAL mode, connection pooling). The architecture does not specify how the control server manages concurrent WebSocket clients writing mix state to SQLite.
**Proposed:** WAL mode + single writer + sqlx connection pool with bounded size.
**Resolution:** Document in `docs/architecture/DATABASE.md`.

---

## GAP-010: Deployment Secrets Management — UNSPECIFIED (Medium)

**Required by:** Phase 8 (RPi deployment)
**Description:** How auth tokens, session secrets, and configuration secrets are managed on the deployed system is not specified. Raspberry Pi deployments often use plaintext config files.
**Proposed:** systemd `EnvironmentFile` with restricted permissions (600, root:root).
**Resolution:** Document in `docs/deployment/SECRETS.md`.

---

## Gaps by Phase Dependency

| Gap | Phase Required | Severity |
|-----|---------------|----------|
| GAP-005: Latency Budget | Phase 1 | High |
| GAP-006: XRUN SLA | Phase 1 | Medium |
| GAP-007: USB Hot-Plug | Phase 1 | Medium |
| GAP-008: RT Thread Priority | Phase 1 | Medium |
| GAP-003: PipeWire Integration | Phase 1 | High |
| GAP-004: Authentication | Phase 3 | High |
| GAP-009: SQLite Concurrency | Phase 3 | Medium |
| GAP-002: Browser Audio | Phase 4/5 | High |
| GAP-001: Audio Transport | Phase 5 | Critical |
| GAP-010: Secrets Management | Phase 8 | Medium |
