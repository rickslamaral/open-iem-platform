# ADR-006: Audio Security / Pairing

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
Control auth does not secure media packets.

## Requirements
Unknown receivers blocked; pairing; device identity; revocation; encrypted/authenticated media; lost credential recovery.

## Options Considered
Trusted LAN; managed LAN; hostile/zero-trust LAN. DTLS-SRTP; QUIC/TLS; Noise.

## Trade-offs
Trusted LAN alone fails on rogue/lost devices. DTLS-SRTP aligns with selected WebRTC and avoids custom cryptography.

## Decision
Threat model: LAN may contain rogue clients and lost/compromised receivers. Pairing is mandatory; unknown receivers are blocked; media uses DTLS-SRTP; server authorization binds device identity to musician/mix; revocation invalidates sessions/keys. Credential loss requires admin re-pair/revoke or local recovery procedure, never silent bypass.

## Rationale
Standard WebRTC security plus explicit device lifecycle gives strong MVP boundary without inventing custom crypto.

## Consequences
Pairing becomes install/runtime prerequisite. Control JWT remains separate from media key establishment.

## Risks
Pairing UX and recovery can block musicians; protect admin recovery and audit all changes.

## Validation Requirements
Negative auth, rogue receiver, replay, revocation, reconnect, key rotation and lost-device tests.

## Implementation Dependencies
ADR-001, ADR-003, ADR-009.

## Related GAPs
GAP-018, GAP-019, GAP-001, GAP-008.

## Open Questions
Exact pairing UX may evolve without changing security invariants.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
