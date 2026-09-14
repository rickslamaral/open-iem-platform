# ADR-004: Clock / Drift / Synchronization

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
48 kHz does not synchronize physical clocks.

## Requirements
Stable individual monitoring; sample timestamps; bounded drift correction; no false multi-RX claim.

## Options Considered
Server master; capture master; receiver master; PTP-like external clock.

## Trade-offs
Capture timeline preserves source semantics. Receiver-local playout with estimator/resampling is simpler than global clock.

## Decision
Use capture/audio-interface sample timeline as media timestamp source. Receiver uses local output clock, bounded drift estimator and adaptive resampling around a target buffer. MVP promises independent monitoring only; no sample/phase alignment between receivers.

## Rationale
Avoids pretending server wall clock or nominal sample rate synchronizes devices. Meets MVP need without external clock hardware.

## Consequences
Long sessions may require resampling; synchronized monitoring is future scope.

## Risks
Bad estimator can cause artifacts or buffer runaway.

## Validation Requirements
Long-run drift simulation, timestamp/sequence tests, resampling artifact test, 60-minute physical receiver test.

## Implementation Dependencies
ADR-001, ADR-002, ADR-003, ADR-005, ADR-007.

## Related GAPs
GAP-004, GAP-005, GAP-017.

## Open Questions
Sample-aligned multi-receiver mode requires new product requirement and ADR.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
