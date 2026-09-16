# Architecture Decision Records

Final P0 decisions. Implementation remains subject to DEVELOPMENT-HANDOFF.md and validation gates.

| ADR | Decision | Status | Key dependency |
|---|---|---|---|
| ADR-001 | WebRTC media, RTP/Opus, DTLS-SRTP, LAN ICE | DECIDED | Receiver, codec, security |
| ADR-002 | Opus 48 kHz stereo, 20 ms; PCM diagnostic | DECIDED | Transport, latency |
| ADR-003 | Native/headless receiver, PWA control-only | DECIDED | Backend, platform validation |
| ADR-004 | Capture timeline + receiver drift correction; independent monitoring | DECIDED | Receiver, latency |
| ADR-005 | Provisional p95 ≤50 ms / p99 ≤75 ms; validation required | DECIDED — VALIDATION REQUIRED | Hardware/network evidence |
| ADR-006 | Mandatory pairing, identity, revocation, DTLS-SRTP | DECIDED | Auth/bootstrap |
| ADR-007 | Bounded lock-free SPSC audio boundary | DECIDED | Backend, topology |
| ADR-008 | PipeWire native primary; ALSA fallback; Pi headless server | DECIDED — VALIDATION REQUIRED | L1–L4 gates |
| ADR-009 | Idempotent fixed `soundtech` bootstrap; implementation gap | DECIDED — IMPLEMENTATION GAP | Migrations/auth |
| ADR-010 | L1/L2 CI lab; L3/L4 hardware evidence | DECIDED | Release gates |
| ADR-011 | Durable scenes, immutable revisions, transient runtime state | PROPOSED — implementation pending | GAP-027, state store |

## Non-negotiable boundaries

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

`PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`

No ESP32. No Internet dependency for live audio. No hardware claim from Docker, QEMU, ARM64 cross-build or simulated tests.
