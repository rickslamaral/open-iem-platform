## Phase 124 status — duplicate packet receiver path

- `ReceiverError::DuplicateSequence` added; `JitterBuffer::push` returns it for duplicate sequence numbers instead of `InvalidPacket`.
- `OpusReceiver::playout` routes `DuplicateSequence` push errors to `record_late()` instead of `record_dropped()`.
- `DuplicateProfile` added to `network-fault` crate: `new(interval)` validated, `apply(&[Packet])` injects copies at every `interval`-th position.
- `deterministic_duplicate_profile_classifies_duplicates_as_late` test confirms 6 originals received, 3 duplicates counted as `late_packets`, zero `packets_dropped`, zero PLC, zero output failures.
- Evidence level is `CODE` local only. WebRTC/DTLS-SRTP negotiation, real LAN duplicate injection, PipeWire/ALSA runtime and hardware remain unvalidated.

## Phase 123 status — deterministic receiver PLC burst limit enforcement

- `network-fault/tests/headless_receiver.rs` drives an eight-packet sequence with a five-packet gap (positions 2-6) into `OpusReceiver`; coverage confirms three delivered packets, four PLC frames, fail-safe mute on the next missing frame, one `output_failures` transition and no duplicate failure count on subsequent calls.
- Evidence level is `CODE` local only. WebRTC/DTLS-SRTP negotiation, real LAN outage, PipeWire/ALSA runtime and hardware remain unvalidated.

## Phase 121 status — deterministic reorder receiver path

- Revisado `JitterProfile`: múltiplos eventos agora calculam slots de entrega determinísticos sem deslocamento por `remove/insert`; cobertura confirma colisões e contagem de reordenação. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem não validados.

- `network-fault/tests/headless_receiver.rs` applies `ReorderProfile` to eight encoded Opus packets before `OpusReceiver`; coverage confirms reordered arrival, ordered playout, zero PLC, zero late packets and no output failures.
- Evidence level is `CODE` local only. WebRTC/DTLS-SRTP negotiation, real LAN reordering, PipeWire/ALSA runtime and hardware remain unvalidated.

## Phase 120 status — deterministic reconnect receiver path

- `network-fault/tests/headless_receiver.rs` aplica `ReconnectProfile` a sete payloads Opus reais, simula uma perda de fronteira, executa `OpusReceiver::reconnect` e valida recuperação do mix, contador de reconnect, estado `Playing`, áudio pós-reconexão e ausência de PLC/falha de saída.
- Evidência nível `CODE` local; WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e hardware permanecem não validados.

## Phase 119 status — deterministic outage receiver path

- `network-fault/tests/headless_receiver.rs` aplica `OutageProfile` a seis payloads Opus reais sobreviventes e valida dois frames PLC consecutivos no `OpusReceiver`, `plc_consecutive_max == 2`, estado `Playing` e ausência de falhas de saída.
- Evidência nível `CODE` local; WebRTC/DTLS-SRTP, PipeWire/ALSA, rede real e hardware permanecem não validados.

## Phase 118 status — deterministic jitter receiver path

- `network-fault/tests/headless_receiver.rs` now applies `JitterProfile` to eight encoded Opus packets before `OpusReceiver`; coverage confirms all packets survive reordering, PCM playout remains valid, and the receiver stays `Playing` without PLC, mute or output failures.
- Evidence level is `CODE` and CI only. This remains bounded/in-memory coverage; WebRTC/DTLS-SRTP negotiation, real LAN jitter, PipeWire/ALSA runtime and hardware remain unvalidated.

## Phase 117 status — deterministic network fault receiver path

- `network-fault/tests/headless_receiver.rs` now drives encoded Opus packets through `LossProfile` into `OpusReceiver`, covering deterministic packet loss, PLC concealment, decoded frame content, receiver state and shared metrics.
- Evidence level is `CODE` and CI only. This remains bounded/in-memory coverage; WebRTC/DTLS-SRTP negotiation, real LAN impairment, PipeWire/ALSA runtime and hardware remain unvalidated.

## Phase 113 status — Engineer receiver metrics

- Teste de regressão cobre falha HTTP do endpoint de métricas: Engineer Console mantém os sete contadores como `UNKNOWN`; evidência CODE local.

- Engineer Console consulta `GET /api/v1/metrics` durante refresh e renderiza `packets_received`, `packets_dropped`, `late_packets`, `reconnect_count`, `plc_frames_total`, `plc_consecutive_max` e `output_failures`. Payload ausente/parcial não quebra UI; falha do endpoint exibe `UNKNOWN`; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Phase 112 status — receiver reset coverage

- Teste de reset agora confirma limpeza de `output_failures` e `late_packets`, além dos contadores existentes. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Phase 111 status — decoder/output failure metrics

- `OpusReceiver` registra `output_failures` para falhas de decode normal e PLC, duração PCM inválida, exaustão do orçamento PLC e erro de saída. O latch de mute impede dupla contagem em chamadas posteriores. Evidência CODE local; integração headless, runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.

## Phase 110 status — late_packets counter

- `ReceiverMetrics` agora distingue `late_packets` (stale/duplicados) de `packets_dropped` (overflow/inválido); `OpusReceiver` chama `record_late()` no caminho stale. `GET /api/v1/metrics` expõe o campo. Evidência CODE; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware permanecem pendentes.


## Phase 109 status — receiver fail-safe output metrics

- `ReceiverMetrics` e snapshot REST agora incluem `output_failures`; `OpusReceiver` conta falhas de decode, PLC/output e mute latch sem duplicar chamadas já mutadas. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware continuam pendentes.

## Phase 108 status — receiver metrics round-trip coverage

- Teste de integração confirma métricas compartilhadas no fluxo OpusReceiver: pacote válido incrementa `packets_received`, payload inválido incrementa `packets_dropped` e `reconnect` incrementa `reconnect_count`. Evidência CODE local; binário headless, runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware continuam pendentes.

## Phase 107 status — receiver snapshot coverage

- Testes unitários cobrem snapshot com contadores populated e serialização dos nomes estáveis (`schema_version`, `packets_received`, `packets_dropped`, `reconnect_count`, `plc_frames_total`, `plc_consecutive_max`). Evidência CODE local; integração com binário headless, runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e hardware continuam pendentes.

# Open IEM Platform — Development Handoff

**Date:** 2026-09-21
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
- Linux-first primary target: Debian, Ubuntu and Raspberry Pi OS on amd64/arm64; Raspberry Pi 3 is minimum family baseline, while Pi 4/5/future models are capability-based targets.
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
- **ADR-009:** Idempotent `soundtech` / `[REDACTED]` Engineer bootstrap after migrations; Argon2id hash only; first login returns `must_change_password`; authenticated `PUT /api/v1/auth/password` clears bootstrap flag; auth issuance and replacement are serialized; ordinary users cannot use bootstrap-only endpoint.
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
- Headless audio verification: deterministic DSP always; optional ALSA `snd-aloop` injection/capture with PCM frame validation; installer `--run-tests` produces a gate report.
- Release archive validation and checksum/provenance workflow.

Implemented but not production-validated: bounded MixEngine-to-media bridge (`streaming::MediaBridge`) routes processed frames into per-session queues without blocking. `MediaWriter` encodes one bounded 48 kHz stereo frame to Opus; `SessionRegistry::drive_once` drives bounded bridge input, attaches negotiated media to `str0m::media::Writer`, and polls bounded Sans-IO WebRTC output; `TransportAdapter` owns bounded UDP delivery and requeues failed sends within capacity. Current media frame representation is a stereo sample pair expanded to 20 ms for deterministic CODE/SIMULATED coverage. Production WebRTC/RTP/DTLS-SRTP runtime, PipeWire runtime, Pi hardware output, bootstrap and topology hot-plug integration remain pending. Opus receiver core exists in P0-004, with OS output and hardware validation pending.

## Remaining Architecture Gaps

Canonical registry: `docs/ARCHITECTURE-GAPS.md`.

Critical remaining gaps: GAP-001, GAP-003, GAP-004, GAP-005, GAP-006, GAP-007, GAP-008, GAP-009, GAP-010, GAP-017, GAP-018, GAP-019, GAP-020, GAP-025.

`RESOLVED` entries only cover selected decision/registry/CI evidence. They do not claim media, runtime or hardware support.

## P0 Queue

| ID | Priority | Component | Task | Why | Dependencies | Acceptance Criteria | Tests | Validation Level | Blocking |
|---|---|---|---|---|---|---|---|---|---|
| P0-001 | P0 | RT boundary | Replace blocking JACK/audio callback path with bounded SPSC/ring boundary and separate control queue | RT safety precedes hardware/media | ADR-007 | No mutex/I/O/filesystem/unbounded allocation in callback; overflow policy documented | Rust unit/stress/lock scan | CODE + CI + L1 | Complete in HEAD; runtime/hardware pending |
| P0-002 | P0 | Audio Lab | Create L1 Docker/PipeWire and L2 ALSA virtual harness | Reproducible audio validation | ADR-010 | Profiles run deterministically and emit metrics | CI lab tests | CI + HEADLESS/EMULATED | Complete in HEAD; target runtime/hardware pending |
| P0-003 | P0 | Media Plane | Connect MixEngine frames to WebRTC media session and drive output | Current signaling has no media | ADR-001/002/007 | Real frames leave MixEngine; versioned stream metadata; bounded path | Integration/media tests | L1 first | Yes |
| P0-004 | P0 | Receiver | Implemented receiver core: bounded ingress/jitter, Opus decode, fail-safe mute and reconnect (SIMULATED); OS output/hardware pending with output, jitter and reconnect | No receiver exists | ADR-002/003/004/006 | Decode, playout, mute-on-failure, pairing and reconnect | Receiver integration/fault tests | L1 then L3 | Yes |
| P0-005 | P0 | Clock | Implement sample timestamps, sequence, drift estimator and adaptive resampling | 48 kHz is not sync | ADR-004 | Long-run bounded drift and no unbounded buffer | Simulation/soak tests | L1/L2 | Yes |
| P0-006 | P0 | Security | Implement pairing, receiver identity, revocation and DTLS-SRTP session binding | Unknown receivers must be blocked | ADR-006 | Rogue/revoked receiver cannot receive/control audio | Negative/replay/revoke tests | L1 then runtime | Yes |
| P0-007 | P0 | Auth/DB | Add idempotent `soundtech` bootstrap and versioned migration boundary | Explicit product contract absent | ADR-009 | Fresh/repeat/changed password/concurrent startup pass | DB/auth integration tests | CODE + CI | Yes |
| P0-008 | P0 | Backend | ALSA explicit fallback backend (CODE+CI, PR #63); PipeWire native backend and device discovery remain pending | Explicit ALSA path needed before PipeWire | ADR-008 | Device discovery, callback safety, fail-safe mute | Backend tests | CODE+CI; HARDWARE pending | P1-001 |
| P0-009 | P0 | Latency | Instrument capture→IEM and publish p50/p95/p99 report | No E2E evidence | ADR-005 | p95≤50ms/p99≤75ms under MVP test conditions, or reopen ADR | Loopback/latency tests | L3 required | Yes |
| P0-010 | P0 | Pi validation | Run physical Pi 5 + USB audio gate | Cross-build is not hardware | ADR-008/010 | L3 report with OS/device/buffer/XRUN/recovery | Hardware test suite | HARDWARE | Yes |

## P1 Queue

| ID | Component | Task | Dependencies | Acceptance |
|---|---|---|---|---|
| P1-001 | Topology | Add capability model and Channel Mode validation; later AUX/pair/playback/hybrid | P0-008 | Explicit source mapping and invalid-config tests |
| P1-002 | Device Manager | COMPLETE: bounded capability discovery, snapshot and recovery state machine; API route `/api/v1/devices` protected by Engineer/Admin RBAC | P1-001/P0-008 | CODE + CI; backend hot-plug/runtime integration remains pending |
| P1-003 | Observability | XRUN, device, stream, receiver and network quality metrics | P0-004/P0-008 | Truthful telemetry and fault reports |
| P1-004 | Recovery | COMPLETE: RecoveryRegistry in AppState and Musician WebSocket lifecycle; duplicate ownership guard and DB assignment conflict handling | P0-004/P1-002 | Other musician survives client failure; reconnect restores assigned mix |
| P1-005 | Network Tests | Loss/jitter/reorder/outage/reconnect fault profiles | P0-003/P0-004 | Automated profiles and thresholds |
| P1-006 | Release | Install, artifact, checksum and v0.3.1 release validation | P0 gates | No release claim before evidence |
| P1-007 | Backup | COMPLETE: config-backup crate and local `iem config backup/restore` CLI serialize/restore channel, mix, EQ, compressor, limiter and sends without secrets | P0-007/P1-004 | CLI CODE evidence; clean-environment restore and API/operational deployment validation remain pending |
| P1-008 | API/UI | COMPLETE: EQ UI via #74; domain routes `GET /api/v1/system` + `GET /api/v1/channels` via #75; Musician scene/preset read-only catalogs and Engineer scene/preset controls are implemented in code/CI; runtime remains pending | P0 contracts | Contract/typecheck/frontend tests |

## P2 Queue

- Scene duplication implemented in API and Engineer Console: `POST /api/v1/scenes/{id}/duplicate`; CODE+CI evidence is merged, runtime validation remains pending.

- Scenes/state-store durable path exists via `SCENE_STORE_PATH`; `GET/PUT /api/v1/scenes/backup` provides validated atomic export/restore; Engineer Console supports list/create/recall/edit-revision/delete; runtime validation remains pending.
- Playback/AUX/Hybrid topology after Channel Mode evidence.
- Windows WASAPI/ASIO runtime validation.
- Full Engineer matrix, meters, locks and device UI. Built-in channel preset catalog/application is implemented in code/CI; runtime/hardware execution and validation remain pending. Preset authoring, persistence and mix presets remain deferred.
- Multi-receiver synchronization, only if product requirement changes.
- Telemetry/privacy policy before any remote telemetry.
- Scalability beyond 8 channels/2 mixes/2 receivers.

## Linux-first release architecture

- Supported platforms: Debian, Ubuntu and Raspberry Pi OS.
- Supported architectures: amd64 and arm64.
- `.deb` is first official package; lifecycle and systemd checks are software gates.
- Physical Raspberry Pi, USB, thermal, controller-specific behavior, physical latency, hot-plug and hardware XRUN are `HARDWARE_CERTIFICATION`, not software-release blockers.
- Canonical details: `docs/COMPATIBILITY.md`, `docs/PACKAGING.md`, `docs/RELEASE-GATES.md`, `docs/HARDWARE-CERTIFICATION.md`.

## Phase 106 status — receiver metrics continuation

- Phase 106 está reconciliada: `OpusReceiver` registra recebidos, drops por overflow e payload inválido, reconnect e métricas PLC; endpoint REST serializa snapshot com schema versionado. Evidência CODE local nos commits `2ac4a77`, `3ba17dc`, `48fa714` e `49e0d1d`; integração do snapshot ao binário headless receiver ainda não existe. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 continuam pendentes.


- `OpusReceiver::enqueue` agora registra rejeições de payload inválido (`empty` ou >1500 bytes) em `ReceiverMetrics::packets_dropped`; cobertura unitária confirma duas rejeições, duas contagens. Evidência CODE; runtime/hardware permanecem pendentes.

- Phase 105 receiver ingress overflow metric is implemented and covered by 71 streaming tests. Phase 104/105 metrics wire-up remains CODE-only; no headless receiver binary currently consumes `AppState.metrics.receiver`. Runtime WebRTC/DTLS-SRTP, PipeWire/ALSA and Raspberry Pi 5 remain pending.

## Phase 105 status — receiver ingress drop metric

- `OpusReceiver::enqueue` registra `ReceiverMetrics::record_dropped()` somente quando `try_send` rejeita por fila de ingress cheia; canal desconectado não é contado como pacote descartado.
- Teste dedicado confirma overflow bounded contado uma vez. Evidência CODE local; runtime WebRTC/DTLS-SRTP, PipeWire/ALSA e Raspberry Pi 5 permanecem pendentes.

## Phase 99 status — PipeWire/WirePlumber virtual graph CI smoke

- CI now installs `pipewire`, `pipewire-bin` and `wireplumber` and runs `scripts/ci/run-pipewire-software-e2e.sh`.
- Deterministic Opus writer/receiver round-trip remains covered by CODE + CI.
- Phase 100 adds a two-packet out-of-order Opus round-trip test with jitter-buffer reordering and per-frame decoded-content assertions; this remains CODE evidence, not network/runtime WebRTC validation.
- Evidence level remains `SOFTWARE/SIMULATED`: this does not validate a target virtual sink/source, WebRTC over network, latency, or hardware.

## Phase 114 status — headless UDP/Opus loopback

- `TransportAdapter` agora tem teste bounded de loopback UDP entrega payload Opus ao socket e o encaminha manualmente ao `OpusReceiver`; valida duração/canais PCM estéreo de 20 ms e confirma `packets_received` sem `output_failures`.
- Evidência permanece `CODE` local. Isso não prova negociação WebRTC completa, DTLS-SRTP, PipeWire/ALSA, runtime de produção ou hardware.

## Validation Gates

1. **CODE VALIDATED:** local tests, fmt, clippy, typecheck, build and security tests.
2. **CI VALIDATED:** real GitHub jobs with runner/steps/conclusion on exact commit.
3. **HEADLESS/EMULATED:** deterministic DSP, Docker ALSA userspace (`ALSA_SIM_MODE=null`) and host ALSA Loopback; valid for headless regression/release gates, never a hardware claim.
4. **RUNTIME VALIDATED:** real Linux PipeWire/ALSA execution on target host.
5. **HARDWARE VALIDATED:** physical Pi/interface report.
6. **RELEASE VALIDATED:** artifact, checksum, install, upgrade, runtime and required hardware evidence.

## Hardware Gates

- L3: Raspberry Pi 5, named USB audio interface, fixed OS/image, sample rate, buffer, network, XRUN, hot-plug, reboot, 60-minute stability and latency percentiles.
- L4: mixer + interface + network + native receiver + IEM end-to-end.
- QEMU, Docker, ARM64 cross-build and simulated backend never satisfy L3/L4.

## CI Gates

- Existing CI must pass every relevant commit.
- L1/L2 Audio Lab jobs are present and passing; maintain exact-HEAD evidence for every relevant commit.
- CI must report exact commit, job, runner, steps and artifacts.
- Hardware evidence remains separate unless a controlled hardware runner exists.

## Release Gates

- v0.3.1 remains blocked until artifact, install, PipeWire, media, latency and required hardware gates pass.
- No support claim from cross-compile alone.
- No release marked validated with missing receiver/media evidence.
- Independent Ed25519 public-key distribution remains required where release signing is claimed.

## Development Order

1. P0-001 RT boundary — code complete in HEAD; runtime/hardware validation remains pending.
2. P0-002 Audio Lab — HEADLESS/EMULATED complete; target runtime/hardware validation remains pending.
3. P0-003 media plane.
4. P0-004 native receiver.
5. P0-005 clock/drift.
6. P0-006 pairing/security; CODE binding now includes canonical DTLS-SRTP fingerprint matching, while runtime remains pending.
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
- Headless audio evidence: deterministic DSP, Docker ALSA userspace and host ALSA Loopback passed; classified `HEADLESS/EMULATED`. CI run `35176577755` passed its Audio Lab job; target PipeWire/ALSA and physical validation remain pending.
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

**Current status (Phase 119):** P1-002 Device Manager, P1-003 observability, P1-004 recovery (PR #81), P1-005 network, P1-007 backup library (PR #73), P1-008 API/UI and Phase 116 metrics reset and Phase 117 deterministic network-fault receiver coverage are complete at their recorded evidence levels. P1-002 evidence: CODE + CI; `GET /api/v1/devices` exposes protected snapshots, while backend hot-plug/runtime integration remains pending. P1-004 evidence: CODE + CI run `35055191463` (13/13), plus local fmt, clippy, tests and documentation gates PASS. Runtime, PipeWire/ALSA, WebRTC/Opus and Raspberry Pi hardware remain unvalidated. These are `PENDING` or `HARDWARE_CERTIFICATION`, not automatic software-release blockers. P1-006 software/package release proceeds when `SOFTWARE_RELEASE_GATE` and `PACKAGE_RELEASE_GATE` pass; publication still requires explicit confirmation.

**Current next step:** Continue receiver/media work only where a concrete CODE boundary exists; Phase 117 now covers deterministic loss/PLC receiver behavior in CODE/CI; otherwise select next independent P2 product slice. Phase 105 receiver packet metrics are CODE-only and require headless receiver integration before runtime claims; amd64 and arm64 `.deb` lifecycle are now CODE + PACKAGE_RELEASE_GATE PASS in CI run `35533396414`; local `iem config backup/restore` CLI is implemented with bounded input and symlink rejection; clean-environment restore and deployment validation remain pending. Built-in channel preset application is implemented for Engineer/Admin via `POST /api/v1/presets/{id}/apply`, with fail-closed channel bounds, payload validation and locked-channel protection before mutation; preset creation, editing, persistence and mix-preset application remain unimplemented. P0-002 Audio Lab L1/L2 is implemented in CI as deterministic CODE/SIMULATED coverage. PR #125 merged with an explicit bounded `TransportAdapter` owning UDP socket I/O; `SessionRegistry` remains Sans-IO and requeues failed sends within bounded capacity. P0-003 remains CODE/SIMULATED: no runtime, PipeWire/ALSA, WebRTC/Opus deployment or Raspberry Pi 5 hardware claim. SceneStore persistence remains covered by fresh `AppState` reconstruction over an explicit SQLite path. P1-006 release remains blocked by explicit confirmation and physical validation.
