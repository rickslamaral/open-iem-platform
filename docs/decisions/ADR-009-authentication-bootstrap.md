# ADR-009: Authentication Bootstrap

- **Status:** DECIDED — IMPLEMENTATION GAP
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
START requires automatic idempotent `soundtech` / `[REDACTED]` Engineer bootstrap; code/tests absent.

## Requirements
Create after migration; no duplicate; no reset; Argon2id hash only; Web UI login; server RBAC; preserve users.

## Options Considered
Fixed default; forced enrollment; installer secret.

## Trade-offs
User explicitly preserves fixed credentials. Security mitigation is mandatory first-login/password change and restricted exposure; enrollment can be added later without violating no-reset.

## Decision
Implement idempotent bootstrap of username `soundtech`, password `[REDACTED]`, role Engineer/Mixing Engineer after migrations. Store only Argon2id hash; create only if absent; never overwrite existing user/password. Require password change on first login before normal operation; extend current auth model if needed to enforce this fail-closed; keep server-side RBAC. Until this implementation and its tests exist, bootstrap remains blocked and is not release-ready.

## Rationale
This is explicit user/product contract, not a technical choice to rewrite. Safety rules prevent repeated startup from resetting credentials.

## Consequences
Creates a known initial secret; release/install must refuse normal operation until mandatory change completes. This ADR does not claim that enforcement exists yet.

## Risks
Known credential exposure; migration ordering; race/duplicate creation.

## Validation Requirements
Fresh DB, repeated startup, pre-existing changed password, concurrent startup, hash inspection, login, RBAC and revocation tests.

## Implementation Dependencies
DB migration/bootstrap design; ADR-006; API auth code.

## Related GAPs
GAP-008, GAP-015, GAP-018, GAP-025.

## Open Questions
If fixed password must be removed later, reopen ADR with migration plan.

## Canonical Audio Chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

Control plane remains independent: `PWA/Desktop → HTTP/WebSocket → Control API → Authorization → Mix State`.
