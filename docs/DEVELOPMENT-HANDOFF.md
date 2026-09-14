# Open IEM Platform — Development Handoff

**Date:** 2026-09-14
**Canonical workspace:** `/workspace/open-iem-platform`
**Architecture status:** P0 decisions closed; implementation and validation remain.
**No ESP32.**

## Current Architecture

Canonical audio chain:

```text
Audio Sources
  ↓
Audio Interface
  ↓
Audio Backend
  ↓
Mix Engine
  ↓
Media Plane
  ↓
WebRTC/RTP Transport
  ↓
Native/headless Receiver
  ↓
Audio Output
  ↓
IEM
```

Control plane remains separate:

```text
PWA/Desktop
  ↓
HTTP/WebSocket
  ↓
Control API
  ↓
Authorization
  ↓
Mix State
```

MVP contract:

- 8 mono logical input channels.
- 2 independent stereo mixes.
- 2 musicians and 2 audio receivers, one receiver per musician.
- 48 kHz nominal stream, Opus, 20 ms frames.
- Channel Mode only. AUX Mono, AUX Stereo Pair, Playback Stereo and Hybrid deferred.
- Linux/PipeWire primary target; Raspberry Pi 5 headless server target.
- Server Ethernet; receiver Ethernet or supported 5 GHz Wi-Fi. 2.4 GHz has no support claim.
- LAN-only live audio. Internet optional for administration, never required for audio.
- PWA is control UI. Audio receiver is native/headless and survives UI disconnect.
- MVP promises independent monitoring, not sample/phase alignment between receivers.

## Final Technical Decisions

- **ADR-001:** WebRTC media with RTP/Opus/DTLS-SRTP and LAN ICE. Existing SDP/ICE is signaling foundation only.
- **ADR-002:** Opus, 48 kHz stereo, 20 ms frames. PCM diagnostic/lab only. PLC available; in-band FEC benchmark-gated.
- **ADR-003:** Native/headless receiver separate from PWA. Browser receiver future, not MVP baseline.
- **ADR-004:** Capture/interface sample timeline; receiver-local clock with bounded drift estimator and adaptive resampling. Independent monitoring only.
- **ADR-005:** Provisional acceptance gate: audio E2E p95 ≤50 ms and p99 ≤75 ms under defined MVP LAN conditions. Control RTT remains <100 ms target. These are not yet validated.
- **ADR-006:** Mandatory pairing, device identity, unknown receiver blocked, revocation and DTLS-SRTP media protection.
- **ADR-007:** Bounded lock-free SPSC/ring audio boundary; separate bounded control queue; no blocking RT operations.
- **ADR-008:** PipeWire native primary Linux/RPi backend; ALSA explicit fallback/validation path; Pi headless server, receiver future.
- **ADR-009:** Idempotent `soundtech` / `[REDACTED]` Engineer bootstrap after migrations; Argon2id hash only; no duplicate/reset; implementation still pending.
- **ADR-010:** L1 Docker/PipeWire and L2 ALSA virtual in CI; L3 physical Pi 5 + USB; L4 full mixer/interface/network/receiver/IEM. L3/L4 required for support claims.

ADRs: `docs/decisions/ADR-001` through `ADR-010`.

## Implemented Components

Evidence-backed current components:

- Rust `mix-engine` DSP chain: Sum → EQ → Compressor → Master Gain → Limiter.
- Rust API/control server, REST, WebSocket, RBAC and state broadcast.
- JWT/Ed25519 sessions and Argon2id password primitives.
- `streaming` SDP/ICE signaling scaffold via `str0m`.
- Simulated audio backend and deterministic audio tests.
- Musician and Engineer React/TypeScript control UIs.
- CI workflows for Rust, frontend, security, documentation, coverage and ARM64 cross-build.
- Release archive validation and checksum/provenance workflow.

Not implemented/validated: production WebRTC media integration, PipeWire runtime, Pi hardware output, bootstrap, topology/device manager. Opus receiver core exists in P0-004, with OS output and hardware validation pending.

## Remaining Architecture Gaps

Canonical registry: `docs/ARCHITECTURE-GAPS.md`.

Critical remaining gaps: GAP-001, GAP-003, GAP-004, GAP-005, GAP-006, GAP-007, GAP-008, GAP-009, GAP-010, GAP-017, GAP-018, GAP-019, GAP-020, GAP-025.

`RESOLVED` entries only cover selected decision/registry/CI evidence. They do not claim media, runtime or hardware support.

## P0 Queue

| ID | Priority | Component | Task | Why | Dependencies | Acceptance Criteria | Tests | Validation Level | Blocking |
|---|---|---|---|---|---|---|---|---|---|
| P0-001 | P0 | RT boundary | Replace blocking JACK/audio callback path with bounded SPSC/ring boundary and separate control queue | RT safety precedes hardware/media | ADR-007 | No mutex/I/O/filesystem/unbounded allocation in callback; overflow policy documented | Rust unit/stress/lock scan | CODE + CI + L1 | Complete in HEAD; runtime/hardware pending |
| P0-002 | P0 | Audio Lab | Create L1 Docker/PipeWire and L2 ALSA virtual harness | Reproducible audio validation | ADR-010 | Profiles run deterministically and emit metrics | CI lab tests | CI + SIMULATED | Yes |
| P0-003 | P0 | Media Plane | Connect MixEngine frames to WebRTC media session and drive output | Current signaling has no media | ADR-001/002/007 | Real frames leave MixEngine; versioned stream metadata; bounded path | Integration/media tests | L1 first | Yes |
| P0-004 | P0 | Receiver | Implemented receiver core: bounded ingress/jitter, Opus decode, fail-safe mute and reconnect (SIMULATED); OS output/hardware pending with output, jitter and reconnect | No receiver exists | ADR-002/003/004/006 | Decode, playout, mute-on-failure, pairing and reconnect | Receiver integration/fault tests | L1 then L3 | Yes |
| P0-005 | P0 | Clock | Implement sample timestamps, sequence, drift estimator and adaptive resampling | 48 kHz is not sync | ADR-004 | Long-run bounded drift and no unbounded buffer | Simulation/soak tests | L1/L2 | Yes |
| P0-006 | P0 | Security | Implement pairing, receiver identity, revocation and DTLS-SRTP session binding | Unknown receivers must be blocked | ADR-006 | Rogue/revoked receiver cannot receive/control audio | Negative/replay/revoke tests | L1 then runtime | Yes |
| P0-007 | P0 | Auth/DB | Add idempotent `soundtech` bootstrap and versioned migration boundary | Explicit product contract absent | ADR-009 | Fresh/repeat/changed password/concurrent startup pass | DB/auth integration tests | CODE + CI | Yes |
| P0-008 | P0 | Backend | Implement PipeWire native backend boundary and safe device failure | Current audio is simulated | ADR-008 | Device discovery, callback safety, fail-safe mute | Backend tests | L1/L2 then L3 | Yes |
| P0-009 | P0 | Latency | Instrument capture→IEM and publish p50/p95/p99 report | No E2E evidence | ADR-005 | p95≤50ms/p99≤75ms under MVP test conditions, or reopen ADR | Loopback/latency tests | L3 required | Yes |
| P0-010 | P0 | Pi validation | Run physical Pi 5 + USB audio gate | Cross-build is not hardware | ADR-008/010 | L3 report with OS/device/buffer/XRUN/recovery | Hardware test suite | HARDWARE | Yes |

## P1 Queue

| ID | Component | Task | Dependencies | Acceptance |
|---|---|---|---|---|
| P1-001 | Topology | Add capability model and Channel Mode validation; later AUX/pair/playback/hybrid | P0-008 | Explicit source mapping and invalid-config tests |
| P1-002 | Device Manager | Capability discovery, hot-plug and recovery state machine | P1-001/P0-008 | Device loss/reconnect safe mute and restore |
| P1-003 | Observability | XRUN, device, stream, receiver and network quality metrics | P0-004/P0-008 | Truthful telemetry and fault reports |
| P1-004 | Recovery | Server/device/network/receiver fault handling | P0-004/P1-002 | Other musician survives client failure; reconnect restores assigned mix |
| P1-005 | Network Tests | Loss/jitter/reorder/outage/reconnect fault profiles | P0-003/P0-004 | Automated profiles and thresholds |
| P1-006 | Release | Install, artifact, checksum and v0.3.1 release validation | P0 gates | No release claim before evidence |
| P1-007 | Backup | Config backup/restore without secrets | P0-007/P1-004 | Clean-environment restore |
| P1-008 | API/UI | Reconcile missing domain routes and Phase 93 EQ UI | P0 contracts | Contract/typecheck/frontend tests |

## P2 Queue

- Scenes/state-store and durable/transient state model.
- Playback/AUX/Hybrid topology after Channel Mode evidence.
- Windows WASAPI/ASIO runtime validation.
- Full Engineer matrix, meters, locks and device UI.
- Multi-receiver synchronization, only if product requirement changes.
- Telemetry/privacy policy before any remote telemetry.
- Scalability beyond 8 channels/2 mixes/2 receivers.

## Validation Gates

1. **CODE VALIDATED:** local tests, fmt, clippy, typecheck, build and security tests.
2. **CI VALIDATED:** real GitHub jobs with runner/steps/conclusion on exact commit.
3. **SIMULATED:** L1/L2 virtual audio; never hardware claim.
4. **RUNTIME VALIDATED:** real Linux PipeWire/ALSA execution.
5. **HARDWARE VALIDATED:** physical Pi/interface report.
6. **RELEASE VALIDATED:** artifact, checksum, install, upgrade, runtime and required hardware evidence.

## Hardware Gates

- L3: Raspberry Pi 5, named USB audio interface, fixed OS/image, sample rate, buffer, network, XRUN, hot-plug, reboot, 60-minute stability and latency percentiles.
- L4: mixer + interface + network + native receiver + IEM end-to-end.
- QEMU, Docker, ARM64 cross-build and simulated backend never satisfy L3/L4.

## CI Gates

- Existing CI must pass every relevant commit.
- Add L1/L2 Audio Lab jobs before media merge.
- CI must report exact commit, job, runner, steps and artifacts.
- Hardware evidence remains separate unless a controlled hardware runner exists.

## Release Gates

- v0.3.1 remains blocked until artifact, install, PipeWire, media, latency and required hardware gates pass.
- No support claim from cross-compile alone.
- No release marked validated with missing receiver/media evidence.
- Independent Ed25519 public-key distribution remains required where release signing is claimed.

## Development Order

1. P0-001 RT boundary — code complete in HEAD; runtime/hardware validation remains pending.
2. P0-002 L1/L2 Audio Lab.
3. P0-003 media plane.
4. P0-004 native receiver.
5. P0-005 clock/drift.
6. P0-006 pairing/security.
7. P0-007 auth/migrations.
8. P0-008 PipeWire backend.
9. P1 topology/device/recovery/observability.
10. P0-009 latency measurement.
11. P0-010 L3 Pi validation, then L4 system validation.
12. P1 release gates and Phase 93/UI work.

Each task: read START and this handoff → implement smallest unit → test → review → update GAP/ADR/docs → branch/PR → CI → merge only after gates.

## Definition of Done

- Requirement and ADR traceability exists.
- Smallest safe implementation merged through PR.
- Relevant tests and security review pass.
- CI passes on exact commit.
- Runtime/hardware claims use correct evidence level.
- GAP registry, TODO, DEVELOPMENT-LOG and user docs updated.
- No secrets, ESP32, unsupported platform or optimistic status claims.

## Risks

- WebRTC media integration may expose str0m API/runtime constraints.
- Native receiver packaging increases platform work.
- Pi USB/PipeWire behavior may miss latency gate.
- Fixed bootstrap credential requires forced operational hygiene.
- Wi-Fi tail latency may violate p99 gate.
- Current JACK mutex must not reach hardware.

## Known Limitations

- No physical Pi/audio/interface evidence in this handoff.
- No measured E2E latency.
- No production media or receiver.
- No Audio Lab implementation yet.
- Windows native backend absent.
- v0.3.1 release not validated/published.
- Architecture decisions can be reopened only on contradictory evidence: stop implementation, document evidence, assess impact, update ADR/GAP, then resume.

## Agent Entry Protocol

```text
START.md
  ↓
DEVELOPMENT-HANDOFF.md
  ↓
NEXT DEVELOPMENT QUEUE
  ↓
smallest safe task
  ↓
test → review → docs/GAP update → PR/CI
```

**Next development item:** `P0-006 — Security: pairing, receiver identity, revocation e DTLS-SRTP binding (depende de P0-005 ✓)` — registry de identidade/credencial implementado em código; integração DTLS-SRTP e API permanecem pendentes.
