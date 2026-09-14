# ADR-003: Audio Receiver

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
No receiver implementation exists.

## Requirements
Audio must survive UI/browser failure; output device control; reconnect; identity; bounded jitter; Linux/RPi validation.

## Options Considered
Browser/PWA; native desktop; dedicated Pi; hybrid.

## Trade-offs
Browser is easy to distribute but scheduling, permissions and screen-lock behavior are unsuitable as baseline for low-latency IEM. Native process is controllable. Dedicated Pi adds hardware scope.

## Decision
Use a native/headless receiver process separate from PWA. MVP receiver target is Linux x86_64 and Raspberry Pi only after hardware validation. PWA controls state and displays status; it is not required for audio playout. Browser receiver is future optional.

## Rationale
Separates UI failure from audio execution and gives deterministic device/buffer control without forcing a dedicated hardware product in MVP.

## Consequences
Adds native packaging and a receiver lifecycle. One receiver maps to one musician/mix in MVP.

## Risks
Native receiver packaging; device selection; pairing/reconnect; platform expansion later.

## Validation Requirements
Decode→output integration; UI kill test; screen-lock irrelevant to native process; device disconnect/reconnect; 60-minute stability; L1/L3.

## Implementation Dependencies
ADR-001, ADR-002, ADR-004, ADR-005, ADR-006, ADR-008.

## Related GAPs
GAP-003, GAP-019, GAP-020, GAP-028.

## Open Questions
Future browser/mobile receiver requires separate ADR amendment and measured evidence.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
