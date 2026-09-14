# ADR-010: CI / Audio Lab / Hardware Validation

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
Green software CI does not prove media, runtime or hardware.

## Requirements
Evidence levels must remain separate and tied to commit/config/artifact.

## Options Considered
Hosted CI; L1/L2 Audio Lab; self-hosted hardware; manual evidence.

## Trade-offs
Hosted CI is scalable but software-only. L1/L2 are reproducible but virtual. L3/L4 are authoritative but operationally costly.

## Decision
Adopt four levels: L1 Docker/PipeWire virtual, L2 ALSA virtual, L3 Pi5+USB real, L4 mixer+interface+network+receiver+IEM real. CI gates L1/L2 and software; release/support gates require L3/L4 evidence where claimed. Do not add hardware runner requirement before ownership exists; use signed/manual evidence package initially.

## Rationale
Preserves current CI value while preventing ARM64/Docker from becoming hardware claim.

## Consequences
More evidence work before release; manual hardware evidence needs strict template and artifact linkage.

## Risks
Environment drift, incomplete reports, false release confidence.

## Validation Requirements
Run current CI jobs; add L1/L2 tests; record L3/L4 reports with commit, hardware, OS, sample rate, buffer, interface, network, codec, receiver and metrics.

## Implementation Dependencies
ADR-001, ADR-005, ADR-008.

## Related GAPs
GAP-009, GAP-010, GAP-011, GAP-014, GAP-025, GAP-030.

## Open Questions
Hardware runner can be revisited after repeatable L3 ownership exists.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
