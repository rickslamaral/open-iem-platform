# ADR-007: Real-Time Audio Boundary

- **Status:** DECIDED
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
MixEngine is tested but current control state uses mutexes and JACK callback uses Mutex.

## Requirements
No blocking mutex/I/O/filesystem/logging/allocation in callback; bounded ownership; explicit overflow.

## Options Considered
Mutex; SPSC ring; bounded channel; pull model.

## Trade-offs
Mutex is unsafe for RT. SPSC/ring is predictable; bounded channel can be correct if allocation-free and bounded. Pull couples scheduling.

## Decision
Use a bounded lock-free SPSC/ring boundary for each producer/consumer audio stream. Control mutations cross a separate bounded command queue. Callback performs only bounded buffer operations and DSP; overflow increments metric and applies fail-safe drop/mute policy.

## Rationale
Single producer/consumer matches MVP two mixes and minimizes RT unpredictability. Concrete crate/API choice remains implementation detail, not architectural reopening.

## Consequences
Requires redesign around current mutex path before hardware. Control state is never audio buffer transport.

## Risks
Incorrect overflow policy, stale frames, hidden allocation.

## Validation Requirements
Static callback audit; allocation/lock scan; stress/overrun; XRUN; L1/L2 then real backend.

## Implementation Dependencies
ADR-001, ADR-004, ADR-008.

## Related GAPs
GAP-006, GAP-007, GAP-001, GAP-026.

## Open Questions
Scale beyond MVP may require a different fan-out boundary, requiring ADR review.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
