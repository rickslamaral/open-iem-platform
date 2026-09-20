# Architecture GAP Registry

**Baseline:** 2026-09-16
**Source:** architecture audit, reconciliation, final P0 ADR closure
**Rule:** code/tests/CI/hardware evidence remain separate. Documentation or compilation alone never resolves runtime/hardware GAPs.

## Status vocabulary

`IMPLEMENTATION GAP` · `VALIDATION REQUIRED` · `HARDWARE VALIDATION REQUIRED` · `RESEARCH REQUIRED` · `BLOCKED` · `DEFERRED` · `ACCEPTED RISK` · `RESOLVED`

## Canonical chain

`Audio Sources → Audio Interface → Audio Backend → Mix Engine → Media Plane → Transport → Receiver → Audio Output → IEM`

## Registry

| ID | Priority | Category | Description | Evidence | Status | Decision/ADR | Dependencies | Implementation Action | Validation | Blocking |
|---|---|---|---|---|---|---|---|---|---|---|
| GAP-001 | P0 | Media | Production media path MixEngine→network lacks runtime evidence | Bounded `MediaBridge`, Opus encoder, negotiated `str0m::media::Writer` handoff and explicit bounded `TransportAdapter` socket owner route frames to Sans-IO/session socket boundary; runtime not exercised | VALIDATION REQUIRED | ADR-001/002/003 | 006,007 | Exercise adapter against real WebRTC media drive and frame bridge | Real media integration/L1–L4 | Yes |
| GAP-002 | P0 | Transport | Media transport was previously undecided | ADR-001 now selects WebRTC media/RTP/Opus/DTLS-SRTP | RESOLVED | ADR-001 | 002,003,006 | Implement selected transport | Interop + impairment tests | No |
| GAP-003 | P0 | Receiver | Native/headless decoder core exists; OS output and runtime reconnect evidence remain absent | Bounded ingress/jitter, Opus decode, fail-safe mute and reconnect core with CODE/SIMULATED coverage; no OS output/runtime evidence | VALIDATION REQUIRED | ADR-003 | 001,002,008 | Validate OS output and runtime reconnect | Decode/output/reconnect | Yes |
| GAP-004 | P0 | Clock | Timestamp, drift estimator and adaptive resampling code exists; physical clock evidence absent | Sample timestamps, bounded drift estimator and adaptive resampling with CODE/SIMULATED coverage; no long-run hardware evidence | VALIDATION REQUIRED | ADR-004 | 001,003,005 | Validate long-run drift and physical clock behavior | Long-run drift/physical test | Yes |
| GAP-005 | P0 | Latency | No E2E measurement | Local buffer only; no loopback | VALIDATION REQUIRED | ADR-005 | 001–004,007,008 | Instrument segment latency | p95≤50ms/p99≤75ms provisional gate | Yes |
| GAP-006 | P0 | RT boundary | Bounded MixEngine→media bridge implementation exists; runtime evidence pending | `audio-engine::rt_boundary` in HEAD; bounded non-blocking control queue | VALIDATION REQUIRED | ADR-007 | 001,007 | Preserve bounded boundary; add runtime/stress evidence | Static RT audit/stress/L1 | Yes |
| GAP-007 | P0 | RT backend | JACK callback no longer uses Mutex; runtime evidence pending | `backend/jack.rs` callback owns `RealtimeProcessor`; feature/hardware not executed | VALIDATION REQUIRED | ADR-007/008 | 006,008 | Preserve callback safety; validate JACK/PipeWire lifecycle | Feature build/L1/L2 then L3 | Yes |
| GAP-008 | P0 | Auth | Bootstrap password safety and first-access enforcement | `Db::bootstrap_soundtech` is idempotent, Argon2id-only and M001-backed; startup now fails closed when `OPENIEM_SOUNDTECH_PASSWORD` is absent/empty; first-access password-change enforcement is implemented by login response and bootstrap-only password endpoint | RESOLVED (CODE) | ADR-009 | migrations, auth | Preserve first-access enforcement and add runtime evidence | Fresh/repeat/changed password/RBAC | Yes for runtime/support claims |
| GAP-009 | P0 | Hardware | No Pi 5 physical validation | Cross-build only | HARDWARE VALIDATION REQUIRED | ADR-008/010 | 005,007,010 | Execute L3 Pi+USB gate | Physical report | Yes |
| GAP-010 | P0 | Backend | No real PipeWire/ALSA runtime evidence | Simulated/JACK feature path | VALIDATION REQUIRED | ADR-008/010 | 007,009 | Implement/execute PipeWire path | L1–L3 runtime | Yes |
| GAP-011 | P0 | CI | Current software CI evidence exists | Run 34836841835 recorded 11/11 green | RESOLVED | ADR-010 | none | Keep per-HEAD evidence | Jobs/steps green | No |
| GAP-012 | P1 | Topology | AUX/pairs/playback/hybrid absent | Logical Channel only | DEFERRED | Future ADR | 010, backend | MVP Channel Mode; design later modes | Topology profiles | No |
| GAP-013 | P1 | Devices | Capability/hot-plug runtime integration absent | Bounded `DeviceManager` capability validation, discovery snapshot and recovery state machine exist; API snapshot is Engineer/Admin protected; backend hot-plug/runtime integration remains absent | VALIDATION REQUIRED | ADR-008 | 012,010 | Integrate backend discovery and validate device loss/reconnect | Device loss/recovery | Yes for full topology |
| GAP-014 | P1 | Lab | Audio Lab L1/L2 SIMULATED | 8 testes em audio_lab_l1_l2.rs, CI job audio-lab | RESOLVED (CI/SIMULATED) | ADR-010 | 001,012 | L3/L4 hardware pendentes | CI virtual audio | No (CI level done) |
| GAP-015 | P1 | DB | Versioned migration boundary | `Db::migrate` tracks M001 in `migrations`; legacy column upgrade covered | RESOLVED | ADR-009 | 008 | Add future migrations through tracked IDs | Fresh/upgrade/restore | Yes for bootstrap |
| GAP-016 | P1 | API | START API surface exceeds routes | Domain route coverage expanded incrementally | RESOLVED (CODE+CI) | Future ADR | topology/state | GET `/api/v1/system` + GET `/api/v1/channels`, scenes REST, and built-in preset catalog/application have contract and RBAC coverage (PRs #144–#151; integration tests); runtime remains pending | Contract tests | No |
| GAP-017 | P1 | Network | Loss/FEC/congestion policy absent | No media implementation | RESEARCH REQUIRED | ADR-001/002/005 | 001,004 | Benchmark PLC/FEC and impairment | Loss/reorder/jitter tests | Yes for media gate |
| GAP-018 | P1 | Security | Media key/session binding absent; receiver identity registry exists | PairingRegistry integrated into api-server via PR #189 (CI 16/16 SUCCESS on d80b7ef); pair/revoke/offer-auth API routes implemented; DTLS-SRTP session binding remains pending | PARTIAL (CODE+CI) | ADR-006 | 001,003,009 | Bind authenticated identity to DTLS-SRTP session | Negative/replay/revoke tests | Yes |
| GAP-019 | P1 | Pairing | Pairing registry exists; runtime reconnect integration absent | PairingRegistry in AppState; pair/revoke/offer-auth integration tests (7 tests) pass in CODE+CI via PR #189; runtime reconnect and hardware validation remain pending | PARTIAL (CODE+CI) | ADR-006 | 003,018 | Integrate registry with receiver lifecycle | Pair/revoke/reconnect | Yes |
| GAP-020 | P1 | Recovery | Recovery lifecycle integration implemented; runtime recovery evidence pending | RecoveryRegistry in AppState; Musician WS reconnect/disconnect lifecycle; duplicate ownership guard; DB assignment conflict handling; PR #81; CI run `35055191463` 13/13; 5 recovery tests | RESOLVED (CODE+CI/SIMULATED) | ADR-003/008 | 003,013,023 | Preserve fail-safe recovery and add runtime evidence | Fault injection, real device/media recovery | Yes for runtime/support claims |
| GAP-021 | P1 | Docs | ADR traceability now created | ADRs 001–010 exist | RESOLVED | README + ADRs | none | Maintain links | Cross-reference audit | No |
| GAP-022 | P1 | Registry | Registry now reconciled | This file covers canonical GAPs | RESOLVED | this registry | audit evidence | Update only with evidence | Registry review | No |
| GAP-023 | P1 | Observability | Audio/XRUN/device/network metrics absent | observability crate P1-003, PR #68 | RESOLVED (CODE+CI/SIMULATED) | ADR-010 | 010,020 | Bounded AtomicU64 metrics for XRUN/device/stream/receiver/network | 16 unit tests, fmt+clippy clean | No (runtime evidence pending) |
| GAP-024 | P1 | Operations | Backup/restore library and operational API implemented; clean restore absent | `config-backup` crate in PR #73; 8 unit tests; CODE+CI/SIMULATED | VALIDATION REQUIRED | Future ADR | 015,020, P1-004 | Maintain operational backup/restore API and validate clean-environment restore | Clean restore, secret exclusion | No |
| GAP-025 | P1 | Release | v0.3.1 release/assets not validated | Tag exists; release absent | BLOCKED | ADR-005/008/010 | 005,009,010 | Clear validation/release gates | Artifact/install/release evidence | Yes |
| GAP-026 | P1 | DSP | Runtime safe-default validation absent | DSP/unit tests simulated | VALIDATION REQUIRED | ADR-007/008 | 005,007 | Validate chain/overload/limiter | Runtime loopback | Yes for support |
| GAP-027 | P2 | State | Scenes/state-store durable path and lifecycle | SceneStore SQLite + REST implemented | PARTIAL (CODE+CI) | Future ADR | 015,016 | File-backed path via `SCENE_STORE_PATH`; `GET/PUT /api/v1/scenes/backup` provides atomic durable restore; runtime validation remains | Reopen persistence, recall/rollback tests | No |
| GAP-028 | P2 | UI | EQ/full matrix/device/audio status incomplete | Current UI subset | DEFERRED | Future ADR | 001,003,016 | Phase 93 UI after contracts | Browser integration | No |
| GAP-029 | P2 | Platform | Windows/native support unvalidated | WASAPI/ASIO absent | VALIDATION REQUIRED | ADR-008 | backend | Validate each claim separately | Target runtime | No MVP |
| GAP-030 | P2 | Network testing | Fault injection suite implemented; physical network validation absent | `network-fault` crate integrated with recovery/observability; [PR #72](https://github.com/rickslamaral/open-iem-platform/pull/72); CODE+CI/SIMULATED | VALIDATION REQUIRED | ADR-010 | 001,005,017 | Validate profiles against real LAN and receiver runtime | Automated profiles plus physical loss/jitter/reconnect evidence | Yes for release |
| GAP-031 | P2 | Docs | Phase reviews/guides incomplete | Canonical reviews now cover Phases 92–95; remaining historical review/guides still require reconciliation | IMPLEMENTATION GAP | Future docs task | registry | Reconcile remaining claims | Docs validator | No |
| GAP-032 | P2 | Scale | 8 channels/2 mixes hardcoded | MVP constants | ACCEPTED RISK | Future ADR | topology | Keep explicit MVP boundary | MVP tests | No |
| GAP-033 | P3 | Privacy | Telemetry policy absent | Local-first/no remote telemetry contract | DEFERRED | Future ADR | security | Decide before remote telemetry | Privacy review | No |

## P0 dependency order

1. GAP-006/GAP-007 RT boundary and backend safety validation.
2. GAP-014 L1/L2 lab foundation.
3. GAP-001 media implementation using ADR-001/002.
4. GAP-003 receiver and GAP-018/019 pairing/security.
5. GAP-004 clock/drift and GAP-020 recovery.
6. GAP-010 real PipeWire, then GAP-009 physical Pi.
7. GAP-005 E2E latency validation.
8. GAP-008 auth bootstrap in parallel with DB migration.
9. GAP-025 release only after gates above.

## Evidence boundaries

- `RESOLVED` here means decision/registry/CI evidence only where stated; it does not imply audio runtime or hardware support.
- L1 Docker/PipeWire and L2 ALSA virtual are not L3 Raspberry Pi validation.
- ARM64 cross-build is not hardware validation.
- Simulated DSP is not media delivery.
