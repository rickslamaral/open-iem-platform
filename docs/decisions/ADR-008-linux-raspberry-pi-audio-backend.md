# ADR-008: Linux / Raspberry Pi Audio Backend

- **Status:** DECIDED — VALIDATION REQUIRED
- **Date:** 2026-09-14
- **Scope:** Final P0 architecture closure; implementation follows DEVELOPMENT-HANDOFF.md
- **Evidence:** `/tmp/open-iem-p0-architecture-decisions.md`, `/tmp/open-iem-product-lan-requirements.md`, `/tmp/open-iem-product-decision-gate.md`, current repository source/tests

## Context
- MVP: 8 mono logical input channels, 2 independent stereo mixes, 2 musicians, 2 audio receivers, one receiver per musician.
- MVP topology: Channel Mode only. AUX Mono, AUX Stereo Pair, Playback Stereo and Hybrid remain later.
- LAN-only live audio. Internet optional for administration, never required for audio.
- Server uses Ethernet; receivers may use Ethernet or 5 GHz Wi-Fi. 2.4 GHz is not a support claim.
- Receiver is separate native/headless audio process; PWA remains control UI.
- Independent monitoring is MVP; no sample/phase synchronization claim.
- 48 kHz nominal stream rate; not a clock synchronization mechanism.


## Problem
Current path is feature-gated JACK/PipeWire and simulated; no physical runtime evidence.

## Requirements
Linux native graph; RT-safe callback; device capability; hot-plug; Pi deployment; no false support claim.

## Options Considered
PipeWire native; JACK bridge; ALSA direct; simulated backend.

## Trade-offs
PipeWire native is the primary Linux graph; ALSA remains device-layer fallback where needed. JACK bridge preserves compatibility but current callback is unsafe and is not primary architecture.

## Decision
Use PipeWire native as primary Linux/RPi backend; use ALSA only as explicit low-level fallback/validation path. SimulatedBackend remains test-only. Pi role is headless server in MVP; Pi receiver is future.

## Rationale
Matches START Linux graph, reduces bridge dependence and keeps core/backend abstraction.

## Consequences
PipeWire API integration and Pi runtime are implementation/validation work. Windows remains separate future backend.

## Risks
PipeWire runtime differences; USB devices; thermal/power; missing Pi hardware.

## Validation Requirements
L1 Docker/PipeWire, L2 ALSA virtual, L3 Pi5+USB, L4 full system; nodes, loopback, XRUN, hot-plug, reboot, 60-minute and latency gates.

## Implementation Dependencies
ADR-005, ADR-007, ADR-010.

## Related GAPs
GAP-007, GAP-009, GAP-010, GAP-026, GAP-029.

## Open Questions
Primary PipeWire API details are implementation research under this decision, not a product decision.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
