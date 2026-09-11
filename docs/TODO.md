# TODO

Priority labels: **BLOCKER** | **HIGH** | **MEDIUM** | **LOW** | **RESEARCH**

---

## BLOCKER

- [ ] P0 — Desbloquear GitHub Actions/runner — status BLOCKED — workflow agora cobre branches `feat/**`; runs remotos ainda falham antes dos steps com `runner_id=0`; sem CI remoto não há release `v0.3.1`.

## Phase 45 — correção do label de runner

- [x] Trocar `ubuntu-24.04` por `ubuntu-latest` em todos os jobs CI.
- [x] Confirmar resultado após push: run `34607886129` falhou antes dos steps em todos os jobs com `runner_id=0` e `steps=[]`.
- [ ] Desbloquear GitHub Actions/runner; runs `34611098109` e `34611094092` também falharam pré-steps (`runner_id=0`, `steps=[]`); API de permissões e runners retorna HTTP 403 para token atual; sem CI remoto verde não há release `v0.3.1`.

## Phase 43 — verification gates

- [x] Adicionar `server/Cargo.lock` e remover exclusão global do lockfile.
- [x] Adicionar job CI para documentação, PDF, skills e whitespace.
- [x] Renumerar ADRs duplicados 012 para 013/014.

## READY / CURRENT PHASE

- [x] P1 — Tornar `make up` reconstruível por padrão — Phase 40 — `docker compose up -d --build`; evita imagens obsoletas no desenvolvimento.
- [x] P1 — Validar Musician Guide PDF de forma reproduzível — Phase 42 — `make docs` extrai texto e renderiza PDF; `mutool` cobre ambiente sem `pdftotext`.
- [x] P1 — JWT pós-emissão revogável — Phase 31 — access-session mappings checked by middleware and WebSocket message/keepalive loops; local verification passes, CI remains blocked.

- [x] P1 — Corrigir Docker Compose dev — Phase 30 — Dockerfiles de desenvolvimento adicionados para API e UIs; `docker compose config` passa sem aviso de `version` obsoleto. Build completo depende de chaves JWT locais e daemon Docker disponível.
- [x] P1 — Implementar CLI `iem` com paridade documentada ao Makefile — Phase 35 — dispatcher tipado para help/status/diagnostics/docs/test/build/up/down; versão `0.3.1`; sem instalação global e sem `iem run`.
- [x] P1 — Corrigir saída do `open-iem-admin` — Phase 34 — HTTPS obrigatório fora de localhost, tabelas preservam união de colunas e 404 não afirma recurso planejado.
- [x] P1 — Criar harness de áudio determinístico — Phase 32 — cobre determinismo, sinais sintéticos, isolamento, ganho, pan, mute, limiter e finitude; não substitui hardware. Recuperação stop/start permanece follow-up.
- [x] P1 — Criar matriz formal de validação Docker/Linux/Raspberry Pi — Phase 37 — matriz publicada; execução em Raspberry Pi 5 continua pendente.
- [x] P1 — Adicionar sincronização de observabilidade ao desenvolvimento — Phase 36 — Engineer Console consulta telemetria e mantém métricas desconhecidas como `UNKNOWN`; áudio real segue pendente.

## BLOCKED / VALIDATION REQUIRED

- [ ] P0 — Validar PipeWire/ALSA e áudio real no Raspberry Pi 5 — status HARDWARE VALIDATION REQUIRED.
- [ ] P0 — Validar mídia WebRTC, latência, jitter, perda e recuperação — status HARDWARE VALIDATION REQUIRED.

---

## HIGH

- [x] Add API snapshot/telemetry contract before expanding Engineer Console (Phase 14; audio metrics remain SIMULATED)
- [ ] Install PipeWire on target hardware (Raspberry Pi 5) for Phase 1 validation
- [x] Evaluate audio transport options; WebRTC selected in ADR-004 (Phase 5 signaling scaffold complete)
- [x] Wire authenticated audio signaling routes and engineer session listing (Phase 5)
- [x] Establish authentication mechanism decision (ADR-008 — Accepted and implemented; TLS deployment validation pending)
- [x] HTTP integration tests for api-server (Phase 6 — 19 tests green)
- [x] Trickle ICE real injection in Sans-IO loop (Phase 6 — str0m Candidate::from_sdp_string)
- [x] WebSocket send mutations: SetSendGain/SetSendPan/SetSendMuted with musician ownership enforcement (Phase 16)
- [x] Biquad EQ real DSP (Phase 7 — TDF2 peaking biquad, RBJ coefficients, no heap)
- [x] RMS Compressor real DSP (Phase 7 — stereo-linked, exp-RMS, smoothed gain reduction)
- [ ] Add ARM64 cross-compilation to CI (requires `aarch64-unknown-linux-gnu` setup)
- [x] Musician mix ownership enforcement (mix sends and audio signaling; Phase 13)
- [x] Admin API server-side routes (Phase 9 — complete)

---

## MEDIUM

- [x] Initialize Rust workspace (`server/Cargo.toml`)
- [x] Initialize frontend projects (`web/musician/`, `web/engineer/`) — Phase 4 complete
- [x] Create Docker Compose for local development
- [x] Define PipeWire filter node architecture for Mix Engine (pipewire-jack — see docs/research/pipewire-integration.md)
- [x] Design channel/mix state machine (revision control — implemented in mix-engine)
- [x] Define WebSocket message type catalog (`server/control-protocol/`; Phase 3 foundation)

---

## LOW

- [x] Set up `cargo audit` in CI (Phase 8 — existing job)
- [x] Set up `npm audit` in CI (Phase 9 — npm-audit job added)
- [ ] Create `examples/` with minimal mix scenario
- [x] Include deterministic `audio-engine` integration harness in `make test` — Phase 39
- [x] Configure Dependabot for dependency updates
- [ ] Set up code coverage reporting
- [x] LOW: Log DB errors in master broadcast fan-out (fail-closed and observable)
- [ ] LOW: mix_assignment_lock held during DB read in broadcast fan-out — may contend under load; evaluate read-only lookup without lock
- [x] LOW: Test DB failure during musician WebSocket ownership lookup — fail-closed coverage added Phase 24
- [x] WebSocket keepalive: server Ping/Pong timeout and bounded socket sends — Phase 24 (policy timing covered; transport timing integration remains pending)
- [x] LOW: WebSocket keepalive: add deterministic state-machine coverage without waiting 30/60 seconds — tracker e loop cobrem timeout pendente, Pong incorreto, Pong correlacionado, ausência de falso timeout após Pong válido e fronteiras do intervalo 30 s/timeout 60 s
- [x] LOW: Aplicar limite de 16 KiB em mensagem/frame no `WebSocketUpgrade`, antes da alocação do payload — Phase 26 follow-up
- [x] LOW: Testar rejeição de frame/mensagem acima de 16 KiB via integração WebSocket — teste de transporte confirma encerramento para mensagem Text acima do limite (Phase 26 follow-up)
- [x] LOW: WebSocket: revogação pós-emissão de JWT — Phase 31; middleware, keepalive e mensagens revalidam access-session mapping persistido
- [x] LOW: WebSocket: quotas agregadas por usuário/IP — 4 por usuário, 16 por IP, 64 global; reserva atômica e liberação RAII (Phase 28)
- [x] HIGH: Limitar falhas de autenticação `/ws/v1` por IP — 5 falhas por janela de 60 s; estado bounded em memória, peer via `ConnectInfo<SocketAddr>` (Phase 29)
- [x] LOW: WebSocket: ressincronização após broadcast lag — servidor emite `State` com revisão autoritativa; Musician refaz snapshot REST (Phase 27); revisão monotônica cobre ACK antes do primeiro snapshot
- [x] LOW: WebSocket protocol control coverage: client Ping/Pong payload preservation and binary-frame rejection (Phase 24 follow-up)
- [x] LOW: WebSocket rate quota counts every inbound frame, including control frames, preventing Ping/Pong flood bypass (Phase 24 follow-up)
- [x] MEDIUM: Enforce process-wide WebSocket connection quota (64 permits); per-user/IP quotas and post-issuance JWT revocation checks implemented in Phases 28/31
- [x] LOW: Add explicit WebSocket logging redaction policy and generic client-facing protocol errors — Phase 25; parser/transport details redacted, normal close classified separately
- [x] LOW: Preserve validated WebSocket `request_id` in authorization errors — Phase 26; invalid envelopes use synthetic `server`

---

## RESEARCH

- [x] **Audio Transport** — Preliminary evaluation complete (docs/research/audio-transport/EVALUATION.md); WebRTC selected as primary; final benchmarks deferred to Phase 5
- [ ] **Browser Audio Constraint** — Verify what browsers can receive (WebRTC vs native receiver architecture)
- [x] **PipeWire filter node API** — pipewire-jack selected for Phase 1 (docs/research/pipewire-integration.md)
- [ ] **Raspberry Pi 5 realtime tuning** — PREEMPT_RT kernel, PipeWire latency config, USB audio device selection
- [ ] **JPMixer architecture** — Study WebSocket/scene/mix model as UX reference (verify license before using code)
- [ ] **Windows x64 audio backend** — Implement and validate WASAPI/ASIO backend; current server backend is not native Windows audio.
- [ ] **Platform validation matrix** — Keep Windows x64, Linux x64 and Raspberry Pi 5 ARM64 evidence separate; macOS, Android and iPadOS remain future/backlog.
- [x] **Linux realtime scheduling** — PipeWire+rtkit for Phase 1; hybrid in Phase 2 (docs/research/realtime-scheduling.md)

---

## PHASE 3 SECURITY FOLLOW-UP

- [x] HIGH: Add HTTPS/TLS listener and fail-closed transport configuration — **DONE Phase 22** (Caddy config, systemd, RPi5 guide, ADR-011)
- [x] HIGH: Authorize every WebSocket message by role and musician mix ownership (Phase 23 — SetMasterGain/SetMasterMute RBAC complete; all WS messages now have explicit role checks)
- [x] HIGH: Enforce WebSocket expiry, rate limits and post-issuance revocation through persistent access-session mappings (Phase 31); revocation is checked on handshake, messages and keepalive ticks
- [x] HIGH: Make refresh rotation atomic in one SQLite transaction
- [x] MEDIUM: Add Origin/CSRF validation and bounded auth request inputs
- [x] MEDIUM: Bound HTTP request body to 16 KiB; route payload validation remains pending
- [x] MEDIUM: Parse bind address as SocketAddr and fail closed for non-loopback HTTP without explicit dev override
- [x] MEDIUM: Preflight refresh token owner and JWT before atomic rotation to avoid token loss on issuance failure
- [x] MEDIUM: Make musician mix ownership checks and send mutations atomic under `mix_assignment_lock` (Phase 15)

## PHASE 7 STATUS

- [x] Biquad peaking EQ real processing (TDF2, RBJ coefficients, stereo biquad state, 48kHz)
- [x] Stereo-linked RMS compressor (exp-RMS detector, smoothed gain reduction, mutators)
- [x] Docker Compose dev environment (api-server, musician-ui, engineer-ui)
- [x] Musician Guide pt-BR (docs/guides/MUSICIANS-GUIDE.md)
- [x] PHASE-6-REVIEW.md and PHASE-7-REVIEW.md
- [x] Integrate EQ + Compressor into Mix::process audio chain (Phase 8)
- [x] Create admin CLI for user management (Phase 8)

## PHASE 9 STATUS

- [x] Admin API server-side routes: GET/DELETE /api/v1/admin/users, GET/DELETE /api/v1/admin/sessions
- [x] DB methods: list_users, delete_user, list_active_sessions, revoke_session_by_id
- [x] ApiError::NotFound(String) variant
- [x] 8 HTTP integration tests for admin endpoints
- [x] 3 DB unit tests for new methods
- [x] Admin CLI fix: --username/--password for user create, --id for session revoke
- [x] Biquad coefficient validation vs Python/scipy (delta ≤ 5×10⁻⁸)
- [x] npm audit CI job (HIGH severity gate)
- [x] Admin self-delete protection (policy gate — deferred Phase 10)
- [x] Musician mix ownership enforcement (mix sends and audio signaling; Phase 13)

## PHASE 8 STATUS

- [x] EQ + Compressor wired into Mix::process chain (Sum → EQ → Comp → Master Gain → Limiter)
- [x] Mix struct exposes `eq: ParametricEq` and `compressor: Compressor` fields
- [x] 5 new mix-engine integration tests (EQ boost, compressor reduction, passthrough, chain order)
- [x] Admin CLI binary (`server/admin-cli/`) — user/session/health commands, JSON/table output
- [x] Musician Guide PDF (`docs/guides/MUSICIANS-GUIDE.pdf`) — pandoc+xelatex, 61 KB
- [x] PHASE-8-REVIEW.md
- [ ] Admin API server-side routes (Phase 9)
- [ ] Biquad coefficient validation vs reference implementation (scipy)
- [ ] Generate Musician Guide PDF (Phase 8 documentation) — DONE Phase 8
- [ ] Biquad coefficient validation vs reference implementation (Phase 8) — deferred Phase 9

## PHASE 1 STATUS

- [x] Audio engine crate with simulated backend and feature-gated JACK/PipeWire bridge
- [x] Phase 1 review completed: PASS WITH CONDITIONS
- [ ] Validate real PipeWire graph on Raspberry Pi 5 (hardware blocker)

## PHASE 21 STATUS

- [x] Replace WebSocket query-token authentication with negotiated subprotocol authentication
- [x] Reject missing, empty and query-only WebSocket credentials
- [x] Preserve server-selected subprotocol during HTTP upgrade
- [x] Update Musician client and integration coverage

## PHASE 20 STATUS

- [x] Add independent hook tests for REST snapshot, malformed nested state, delayed snapshot and `SendAck`
- [x] Validate WebSocket envelope version/request ID and ACK ranges client-side
- [x] Abort snapshot fetches when WebSocket connection is cleaned up
- [x] Replace WebSocket query-token authentication with cookie or subprotocol authentication — DONE Phase 21

## PHASE 18 STATUS

- [x] Musician client accepts `SendAck` revisions
- [x] Musician client ignores stale `State`/`SendAck` revisions
- [x] Add client reconciliation of full channel/send state after revision gap (Phase 19; REST snapshot + SendAck application)

## PHASE 17 STATUS

- [x] Broadcast WebSocket deltas for send gain/pan/mute
- [x] Engineer/Admin cross-session `SendAck` delivery
- [x] Musician mix ownership filtering for broadcasts
- [x] Assignment lock coordination and integration coverage
- [x] Independent security/code review; findings fixed
- [x] Add client-side revision ordering/reconciliation for missed or lagged broadcasts — Phase 27; stale revisions ignored and lagged snapshots reconciled

## COMPLETED

- [x] Repository initialized with correct GitHub remote
- [x] Environment audited and documented
- [x] Project directory structure created
- [x] 11 project skills created in `.agents/skills/`
- [x] Documentation structure created (`docs/`)
- [x] ADR baseline created (ADR-001 through ADR-008)
- [x] CI foundation created (GitHub Actions)
- [x] `scripts/validate-skills.sh` created
- [x] Root project files created (README, CONTRIBUTING, SECURITY, LICENSE, CHANGELOG, .gitignore)
- [x] Latency budget defined (`docs/audio/LATENCY-BUDGET.md`) — GAP-005 CLOSED
- [x] XRUN SLA defined (`docs/audio/AUDIO-SLA.md`) — GAP-006 CLOSED
- [x] Mix Engine Rust crate implemented (Channel, MixSend, Mix, Limiter, MixEngine)
- [x] 53 unit + doc tests passing — 0 clippy warnings

- [x] Lookahead brick-wall limiter (LOOKAHEAD_FRAMES=64 @ 48kHz = 1.33ms)
- [x] ParametricEq stub (passthrough — biquad DSP Phase 7)
- [x] Compressor stub (passthrough — dynamics DSP Phase 7)
- [x] 80 tests passing — 0 clippy warnings


## PHASE 10 STATUS

- [x] SQLite mix assignment model and Engineer/Admin API
- [x] JWT numeric user identity (`uid`)
- [x] Musician-owned send gain/pan/mute routes
- [x] Ownership and assignment validation
- [x] Add dedicated Phase 10 integration coverage for assignment lifecycle and musician ownership routes


## PHASE 11 STATUS

- [x] Versioned release pipeline `.github/workflows/release.yml`
- [x] Version consistency gate (tag == workspace version)
- [x] Quality gate (fmt + clippy + tests + cargo-audit) required before builds
- [x] Linux x86_64 server artefact with SHA-256
- [x] Linux ARM64 cross-compile for Raspberry Pi 5 (SIMULATED — not hardware-validated)
- [x] Musician PWA + Engineer UI artefacts
- [x] GitHub Release with CHANGELOG excerpt
- [x] Workspace version bumped to 0.2.0; admin-cli aligned to workspace
- [ ] Tag v0.2.0 and verify full pipeline on GitHub Actions
- [x] Investigate `v0.3.0` release gate failure; tag points to pre-sync commit
- [ ] Publish `v0.3.1` and verify full pipeline on GitHub Actions — código local validado; CI remoto falha antes dos steps; logs bloqueados por permissão do token atual
- [ ] Validate ARM64 binary on real Raspberry Pi 5 hardware
- [x] Dependabot for Cargo + npm

## PHASE 12 STATUS

- [x] Fix release artifact naming (validate-version chain in build jobs)
- [x] Dependabot: Cargo + npm (musician/engineer) + GitHub Actions, weekly
- [x] Admin self-delete protection: 403 when caller deletes own UID
- [x] SBOM: cargo-sbom in release pipeline, best-effort non-blocking

- [x] SBOM generation (cargo-sbom in release pipeline)
- [x] Sincronizar versões com tag v0.3.0 após falha do gate de consistência
- [ ] Reapontar tag v0.3.0 e verificar pipeline completo no GitHub Actions
