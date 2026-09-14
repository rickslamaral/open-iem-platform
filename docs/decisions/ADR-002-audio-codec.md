# ADR-002: Audio Codec and Frame Profile

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
No codec exists.

## Requirements
Predictable low latency, CPU bounded, native receiver support, future browser interoperability.

## Options Considered
Opus; PCM diagnostic mode; other codecs.

## Trade-offs
Opus reduces bandwidth and provides PLC/in-band FEC options; PCM is simple but expensive and loss-intolerant.

## Decision
Use Opus, 48 kHz, stereo per mix stream, 20 ms frames, constrained bitrate configured by benchmark. PCM only for lab/diagnostic loopback, never default media. Enable PLC; enable in-band FEC only when measurements show benefit.

## Rationale
Opus aligns with WebRTC/RTP and is available across likely receiver targets. Fixed 20 ms matches existing frame constants and keeps implementation bounded.

## Consequences
Codec profile becomes versioned protocol contract. Exact bitrate and complexity remain benchmark parameters, not unsupported claims.

## Risks
CPU cost on Pi; codec settings may increase latency or artifacts.

## Validation Requirements
Round-trip decode; CPU/memory; bitrate; PLC/FEC loss tests; p50/p95/p99 latency on Linux and Pi L1/L3.

## Implementation Dependencies
ADR-001, ADR-003, ADR-004, ADR-005.

## Related GAPs
GAP-001, GAP-002, GAP-005, GAP-017.

## Open Questions
Benchmark fixes bitrate/complexity within accepted quality and latency gates.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
