# ADR-005: End-to-End Audio Latency

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
No E2E measurement exists.

## Requirements
Separate control RTT from audio latency; measurable segment budget; release evidence.

## Options Considered
Static target; loopback measurement; synthetic timestamps.

## Trade-offs
Static targets guide engineering but do not prove performance. Loopback is authoritative.

## Decision
Set provisional MVP acceptance gate: audio E2E p95 ≤50 ms and p99 ≤75 ms under defined LAN test conditions; control RTT target remains <100 ms. These are acceptance requirements, not achieved results. Jitter buffer is bounded and included in budget.

## Rationale
IEM requires materially lower latency than generic control. Percentiles expose tail behavior. Values must be validated and may be revised only by evidence/ADR.

## Consequences
Latency gate blocks support/release claims until measured.

## Risks
Wi-Fi and device buffers may exceed budget; early target may require tuning or revision.

## Validation Requirements
Electrical/acoustic loopback across capture→DSP→encode→network→jitter→decode→output; p50/p95/p99; hardware/OS/interface/network/codec recorded.

## Implementation Dependencies
ADR-001, ADR-002, ADR-003, ADR-004, ADR-007, ADR-008, ADR-010.

## Related GAPs
GAP-005, GAP-009, GAP-010, GAP-025.

## Open Questions
Test conditions are wired server + supported 5 GHz Wi-Fi receiver for MVP; 2.4 GHz not accepted.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
