# ADR-004: Audio Transport Protocol Selection

## Status
Proposed — PENDING RESEARCH

## Context
Open IEM Platform must stream independent stereo mixes from the Linux server to musician devices (phones, dedicated receivers). Multiple transport protocols exist with different trade-offs for latency, jitter, packet loss handling, browser compatibility, and implementation complexity.

Candidate protocols:
- RTP/UDP
- WebRTC
- Custom UDP
- WebTransport/QUIC

**Critical browser constraint**: Browsers cannot receive arbitrary UDP audio. The architecture must distinguish between:
- Control client (WebSocket/PWA — browser compatible)
- Audio receiver (WebRTC, native app, or dedicated receiver)

## Decision
**DEFERRED** — requires formal evaluation before decision.

See: `docs/research/audio-transport/EVALUATION.md` (to be created in Phase 5 research).

## Required Research

Before this ADR can be accepted, `docs/research/audio-transport/EVALUATION.md` must exist and compare:

| Protocol | Latency | Jitter | Loss Recovery | Browser | Android | iOS | CPU | Complexity |
|----------|---------|--------|---------------|---------|---------|-----|-----|------------|
| RTP/UDP | | | | | | | | |
| WebRTC | | | | | | | | |
| UDP custom | | | | | | | | |
| WebTransport/QUIC | | | | | | | | |

## Constraints
- **DO NOT implement audio streaming until this ADR is Accepted**
- Phase 5 is gated on this decision
- Must consider browser-incapable UDP reception
