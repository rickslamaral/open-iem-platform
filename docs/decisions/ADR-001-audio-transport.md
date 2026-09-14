# ADR-001: Audio Transport

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
Signaling exists, media delivery does not.

## Requirements
Low-latency LAN media; bounded loss behavior; versioning; native receiver; security; Linux/RPi viability.

## Options Considered
WebRTC media; RTP/UDP custom; QUIC datagrams; TCP/WebSocket.

## Trade-offs
WebRTC has standard signaling, RTP, Opus, DTLS-SRTP, ICE and mature congestion behavior. Custom UDP lowers surface only if all protocol/security work is rebuilt. TCP has head-of-line blocking.

## Decision
Use WebRTC media transport with RTP/Opus and DTLS-SRTP. Use ICE for session setup; LAN deployment does not require Internet/NAT. Existing `str0m` signaling becomes media-session foundation, not proof of implementation.

## Rationale
Existing code already has SDP/ICE scaffolding; standard media/security path minimizes bespoke protocol risk and supports future native/browser receivers.

## Consequences
Media implementation must drive actual `poll_output`, media frames and receiver. LAN-only deployment avoids making WAN traversal a prerequisite.

## Risks
WebRTC integration complexity; congestion behavior may need LAN tuning; no current runtime proof.

## Validation Requirements
Integration test with real media frames; packet loss/reorder/jitter; DTLS-SRTP handshake; native receiver interoperability; Linux and Pi lab evidence.

## Implementation Dependencies
ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007.

## Related GAPs
GAP-001, GAP-002, GAP-005, GAP-017, GAP-018.

## Open Questions
Future WAN/NAT support may require additional ICE/TURN policy; not MVP blocker.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
