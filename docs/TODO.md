# TODO

Priority labels: **BLOCKER** | **HIGH** | **MEDIUM** | **LOW** | **RESEARCH**

---

## BLOCKER

- None currently.

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
- [x] Configure Dependabot for dependency updates
- [ ] Set up code coverage reporting
- [ ] LOW: Log DB errors in master broadcast fan-out (currently silenced via unwrap_or(None); fail-closed but not observable)
- [ ] LOW: mix_assignment_lock held during DB read in broadcast fan-out — may contend under load; evaluate read-only lookup without lock

---

## RESEARCH

- [x] **Audio Transport** — Preliminary evaluation complete (docs/research/audio-transport/EVALUATION.md); WebRTC selected as primary; final benchmarks deferred to Phase 5
- [ ] **Browser Audio Constraint** — Verify what browsers can receive (WebRTC vs native receiver architecture)
- [ ] **ESP32-S3 / ESP32-P4** — I2S, DAC, Wi-Fi, latency, power budget (Phase 10)
- [x] **PipeWire filter node API** — pipewire-jack selected for Phase 1 (docs/research/pipewire-integration.md)
- [ ] **Raspberry Pi 5 realtime tuning** — PREEMPT_RT kernel, PipeWire latency config, USB audio device selection
- [ ] **JPMixer architecture** — Study WebSocket/scene/mix model as UX reference (verify license before using code)
- [x] **Linux realtime scheduling** — PipeWire+rtkit for Phase 1; hybrid in Phase 2 (docs/research/realtime-scheduling.md)

---

## PHASE 3 SECURITY FOLLOW-UP

- [x] HIGH: Add HTTPS/TLS listener and fail-closed transport configuration — **DONE Phase 22** (Caddy config, systemd, RPi5 guide, ADR-011)
- [x] HIGH: Authorize every WebSocket message by role and musician mix ownership (Phase 23 — SetMasterGain/SetMasterMute RBAC complete; all WS messages now have explicit role checks)
- [x] HIGH: Enforce WebSocket expiry and rate limits; revocation remains bounded by JWT TTL
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
- [ ] Add client-side revision ordering/reconciliation for missed or lagged broadcasts

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
- [ ] Publish `v0.3.1` and verify full pipeline on GitHub Actions
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
