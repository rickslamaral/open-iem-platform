# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Documentation
- Added `docs/guides/WINDOWS-DOCKER-GUIDE.md` with Windows 10/11 + Docker Desktop prerequisites, JWT key generation, planned local smoke test, cleanup, troubleshooting and explicit audio validation limits.
- Documented Compose as BLOCKED until its three referenced Dockerfiles exist; no Windows Docker runtime support is claimed.

### Security — Failed WebSocket authentication limiting
- Added bounded in-process per-peer-IP limiting for failed `/ws/v1` authentication: 5 failures per 60-second window and at most 4,096 retained IP entries.
- Successful authenticated WebSocket upgrades do not consume failure budget; peer IP comes only from `ConnectInfo<SocketAddr>`.
- Blocked attempts return HTTP 429 with `Retry-After: 5`.

### Security — WebSocket admission quotas
- Added atomic in-process quotas of 4 upgraded connections per authenticated user and 16 per peer IP, alongside the global limit of 64.
- Quota reservations use RAII release when WebSocket handlers end; quota rejections return HTTP 429 with `Retry-After: 5`.
- Peer identity comes from `ConnectInfo<SocketAddr>`; forwarded headers are not trusted.

### Tests — WebSocket keepalive loop
- Added deterministic paused-clock coverage for the 30-second Ping interval and 60-second Pong timeout boundary.

### Fixed — WebSocket state recovery race
- Musician now records accepted revisions before applying ACK state, preventing a delayed initial REST snapshot from regressing the revision observed over WebSocket.

### Fixed — WebSocket state recovery
- Broadcast lag now emits an authoritative `State` revision notice; Musician refetches authenticated REST snapshot to recover missed deltas.
- Musician client now validates and applies `MasterAck` broadcasts to its local mix snapshot.

### Tests — WebSocket frame limits
- Added integration coverage proving oversized Text messages terminate transport before application parsing.

### Security — WebSocket frame limits
- Applied the 16 KiB protocol limit during WebSocket upgrade for both messages and frames, preventing oversized payload allocation before application validation.

### Fixed — WebSocket keepalive state machine
- Keepalive timeout now applies only while a Pong challenge is outstanding; a valid Pong cannot cause a later false timeout before the next Ping.

### Security — WebSocket keepalive challenge correlation
- Server now accepts Pong for liveness only when payload matches outstanding server Ping; unsolicited or stale Pong frames cannot bypass timeout.

### Changed — WebSocket error correlation
- Errors returned after successful envelope validation preserve originating `request_id`; parser failures continue using `server`.

### Security — WebSocket error redaction
- WebSocket protocol failures now expose stable generic codes/messages; parser and transport details stay server-side.
- Normal peer closes and receive failures no longer masquerade as JWT expiration.

### Security — WebSocket connection cap
- Added a process-wide semaphore limiting upgraded WebSocket connections to 64; excess upgrades return HTTP 503 with `Retry-After: 5`.

### Changed — WebSocket keepalive
- Extracted keepalive intervals and timeout into named policy constants; added boundary unit tests for timeout evaluation.

### Added — Local developer interface
- Added root `Makefile` with documented build, test, lint, docs, validation, diagnostics and lifecycle targets.
- Added `scripts/validate-environment.sh` with explicit `OK`, `OPTIONAL`, `SIMULATED` and `HARDWARE VALIDATION REQUIRED` states.

### Changed — Engineering contract
- Extended `START.md` with CI diagnostics, release blocking, recovery, hardware validation, traceability, audio test harness and architecture fitness requirements.
- Clarified `make run-local` as local API execution with simulated audio; it does not bypass server configuration.
- Docker Compose diagnostics now verify the Compose plugin or standalone executable instead of inferring support from Docker alone.

### Security — WebSocket protocol
- Binary WebSocket frames now receive `INVALID_MESSAGE` and close the connection instead of being silently discarded.

### Added — Phase 24 WebSocket resilience
- Added integration coverage for client Ping/Pong payload preservation and binary-frame rejection.
- WebSocket rate limiting counts every inbound frame, including control frames, preventing Ping/Pong floods from bypassing the per-connection quota.
- Added fail-closed integration coverage when the musician ownership database lookup fails.
- Added server-initiated WebSocket Ping every 30 seconds and `CONNECTION_TIMEOUT` after 60 seconds without Pong.
- Bounded every WebSocket send operation to 10 seconds, preventing stalled clients from retaining handler tasks indefinitely.

### Fixed — Phase 24 audit follow-up
- Stabilized WebSocket broadcast integration tests by yielding after observer handshake, preventing scheduler-dependent false failures.
- Confirmed local Rust workspace and both frontend quality gates pass; real PipeWire, WebRTC media and Raspberry Pi ARM64 runtime remain unvalidated.

### CI — Phase 24
- ARM64 cross-compilation configuration already contains the Rust target, cross-linker package and linker environment; no workflow change required.
- GitHub-hosted CI remains blocked before steps execute because runners are not allocated; local gates are the verified result.

### Fixed — Phase 23 follow-up
- WebSocket ownership lookup now logs database failures and denies forwarding/mutation instead of silently collapsing errors to `None`.

### Changed — Phase 23 documentation
- Guia do Músico atualizado para refletir estado e controles implementados até Phase 23.

### Fixed — Release 0.3.1 preparation
- Preparada consistência de versão entre manifests Rust, frontends e lockfiles após `v0.3.0` ter apontado para commit anterior à sincronização.
- Próximo tag de release deve ser `v0.3.1`; `v0.3.0` remoto permanece imutável e inválido para o gate atual.

### Changed — Release 0.3.0 preparation
- Sincronizadas versões Rust e frontend com tag `v0.3.0`; pipeline de release agora pode validar consistência.

### Added — Phase 23
- `SetMasterGain { mix_index, gain_db }` WebSocket client message: sets master gain for a mix (Engineer/Admin only).
- `SetMasterMute { mix_index, muted }` WebSocket client message: sets master mute for a mix (Engineer/Admin only).
- `MasterAck { mix_index, master_gain_db, master_muted, revision }` server response for master mutations.
- `MasterDelta` broadcast event: all sessions receive unsolicited `MasterAck` after any master mutation; Musician sessions receive only for their assigned mix.
- `master_event_tx` broadcast channel (capacity 256) in `AppState` for `MasterDelta` events.
- Integration tests: `ws_engineer_set_master_gain_returns_master_ack`, `ws_engineer_set_master_mute_returns_master_ack`, `ws_musician_denied_set_master_gain`, `ws_musician_denied_set_master_mute`, `ws_engineer_set_master_gain_invalid_gain_returns_error`, `ws_master_mutation_broadcasts_to_other_sessions`, `ws_master_broadcast_filtered_by_musician_assignment`.

### Security — Phase 23
- Musician role cannot send `SetMasterGain` or `SetMasterMute`; `check_permission` returns `false` for both with explicit match arm.
- Master mutation broadcast filtered by role and mix assignment, consistent with send-delta RBAC model.

### Added — Phase 22
- `deployment/caddy/Caddyfile`: LAN TLS configuration using mkcert certificate; `reverse_proxy` to `127.0.0.1:8080`; HTTP → HTTPS redirect; `X-Real-IP`, `X-Forwarded-For`, `X-Forwarded-Proto` headers forwarded.
- `deployment/systemd/openiem-server.service`: production systemd service with dedicated `openiem` user, loopback bind, `NoNewPrivileges`, `PrivateTmp`, `ProtectSystem=strict`, `ProtectHome`, `LimitNOFILE=65536`, `Restart=on-failure`.
- `deployment/raspberry-pi/README.md`: step-by-step RPi 5 deployment guide (binary install, key generation, systemd, Caddy+mkcert, CA trust per OS), mkcert CA private key protection instructions (chmod 600, no unencrypted backups, revocation procedure, certificate expiry check).
- `docs/adr/ADR-011-tls-deployment.md`: TLS-at-proxy strategy decision; consequences and alternatives rejected.

### Changed — Phase 22
- `docker-compose.yml`: corrected env var prefix from `OPEN_IEM_*` to `OPENIEM_*` to match server binary (`OPENIEM_BIND_ADDR`, `OPENIEM_ALLOW_INSECURE_HTTP`, `OPENIEM_DB_PATH`, `OPENIEM_JWT_PRIVATE_PEM`, `OPENIEM_JWT_PUBLIC_PEM`); healthcheck URL fixed to `/api/v1/health`; volume mount renamed from `./secrets` to `./keys`; added explicit WARNING that this file is dev-only.
- `server/api-server/src/main.rs`: startup `tracing::warn!` emitted for each detected legacy `OPEN_IEM_*` environment variable, preventing silent misconfiguration in environments that have not migrated.

### Security — Phase 22
- `OPENIEM_ALLOW_INSECURE_HTTP` is absent from the production systemd service, enforcing fail-closed HTTP-on-non-loopback behavior.
- Documented mkcert CA private key protection requirements: `rootCA.key` must be `chmod 600`, excluded from unencrypted backups, with revocation instructions.
- Closes HIGH gate from Phase 3 security follow-up: HTTPS/TLS listener and fail-closed transport configuration.

### Changed — Phase 21
- WebSocket authentication moved from URL query parameters to negotiated `Sec-WebSocket-Protocol: openiem.bearer.<JWT>`.
- Server selects and echoes authenticated subprotocol during `/ws/v1` upgrade; missing or empty credentials are rejected.

### Security — Phase 21
- Access tokens no longer appear in WebSocket URLs, reducing exposure through proxy/access logs and browser history.

### Added — Phase 20
- Musician hook tests cover authenticated REST snapshot, malformed nested state, delayed snapshot protection and `SendAck` reconciliation.
- WebSocket client validates protocol version, bounded request ID, ACK ranges and aborts snapshot fetch during connection cleanup.

### Security — Phase 20
- REST snapshot uses `Authorization: Bearer <token>`.
- Invalid ACK gain/pan values and malformed envelopes are rejected before local state mutation.
- WebSocket query-string token remains a documented limitation until cookie/subprotocol authentication is implemented.

### Added — Phase 19
- Musician client reconciles assigned mix snapshot through authenticated `GET /api/v1/state` after WebSocket connection.
- Nested snapshot validation rejects malformed or non-finite channel, mix and send values.
- `SendAck` updates local send state; stale REST snapshots are rejected by monotonic revision tracking.

### Changed — Phase 18
- Musician WebSocket client consumes `SendAck` revisions and ignores `State`/ACK messages older than current revision, preventing stale UI state after delayed broadcasts.

### Added — Phase 17
- WebSocket send mutation broadcasts: connected Engineer/Admin sessions receive unsolicited `SendAck` deltas after another session changes gain, pan or mute.
- Musician sessions receive broadcast deltas only for their currently assigned mix; originator receives only direct acknowledgement.
- Broadcast integration coverage added for cross-session delivery.

### Security — Phase 17
- WebSocket send ownership check and dispatch now share `mix_assignment_lock`, closing the assignment TOCTOU window.
- Broadcast ordering is serialized with assignment/mutation operations; ownership filtering builds payload under lock and releases lock before outbound socket I/O.

### Added — Phase 16
- WebSocket send mutations: `SetSendGain`, `SetSendPan`, `SetSendMuted` client messages via `/ws/v1`.
- `SendAck` server message: echoes current gain_db, pan, muted and state revision after each send mutation.
- Musician ownership enforcement: ws_handler rejects send mutations targeting a mix not assigned to the authenticated musician.
- Input validation in dispatch: `gain_db` must be finite and within `GAIN_DB_MIN..=GAIN_DB_MAX`; `pan` must be finite and within ±1.0.
- 17 new tests (7 WS integration, 6 control-server unit, 4 protocol round-trip).
- `axum-test` ws feature enabled in api-server dev-dependencies.

### Fixed — Phase 15 follow-up
- Musician Guide LAN example now uses actual server environment variable names (`OPENIEM_BIND_ADDR` and `OPENIEM_ALLOW_INSECURE_HTTP`).

### Changed — Phase 15 follow-up
- README, Musician Guide e pacotes web agora refletem Engineer Console operacional, ownership atômico e versão `0.2.0`.
- CI deixou de mascarar falhas de instalação, typecheck e teste nos frontends Musician e Engineer; scripts existentes agora são gates obrigatórios.
- Quality gate de release executa typecheck, testes e `npm audit` dos dois frontends.

### Security — Phase 15
- `mix_assignment_lock` agora cobre ownership e leitura/mutação de sends, fechando janela TOCTOU entre assignment persistido e alteração de estado.

### Added — Phase 14
- Contrato versionado de snapshot em `GET /api/v1/state`, com canais, mixes e sends filtrados por ownership de músico.
- `GET /api/v1/telemetry` para Engineer/Admin, reportando backend `simulated` e métricas `null` quando áudio real não está conectado.

### Security — Phase 13
- `POST /api/v1/audio/offer` agora valida `mix_id` de músicos contra assignment persistido antes de criar sessão WebRTC.

### Added — Phase 6 Engineer Console
- Engineer Console funcional em `web/engineer`: login, refresh por cookie HttpOnly após 401, dashboard autenticado, polling limitado, revisão do estado, sessões WebRTC e atribuição/remoção de mixes.
- Indicador explícito `SIMULATED` para áudio sem PipeWire no VPS.

### Changed — Phase 6
- Assignment usa ID numérico porque catálogo de usuários continua restrito à API Admin.

### Added — Phase 12
- Admin self-delete protection: `DELETE /api/v1/admin/users/{id}` returns 403 if caller matches target user ID.
- Dependabot configuration (`.github/dependabot.yml`): Cargo (weekly, limit 5), npm musician/engineer (weekly, limit 3), GitHub Actions (weekly).
- SBOM generation in release pipeline (`cargo-sbom`, best-effort, non-blocking); output `open-iem-server-<ver>-sbom.json` attached to GitHub Release.

### Fixed — Phase 12
- Release artifact naming bug: `build-server-x86`, `build-server-arm64`, `build-web` jobs now have `needs: [quality-gate, validate-version]` so version string is populated in artifact filenames.

---

## [0.2.0] — 2026-09-09

### Added — Phase 11
- Versioned release pipeline (`release.yml`): semver tag triggers quality gate + multi-target builds + GitHub Release.
- Linux x86_64 server artefact: `api-server` + `open-iem-admin` + README + LICENSE + CHANGELOG, SHA-256 checksum.
- Linux ARM64 (aarch64) server artefact: cross-compiled for Raspberry Pi 5 target; SIMULATED on VPS until real Pi validates PipeWire/Opus.
- Musician PWA and Engineer UI artefacts bundled from `web/musician/dist` and `web/engineer/dist`.
- Version consistency gate: release workflow rejects if tag version ≠ `[workspace.package].version` in `server/Cargo.toml`.
- `server/.cargo/config.toml` linker config for ARM64 cross-compilation.
- `admin-cli` Cargo manifest aligned to workspace semver.

### Changed
- Workspace `version` bumped `0.1.0` → `0.2.0`.

---

### Added — Phase 10
- Persistent SQLite mix assignments with one mix per user.
- Engineer/Admin mix assignment API.
- Musician-owned mix send API for gain, pan, mute, and state reads.
- Numeric user ID (`uid`) in access JWT claims.

### Security
- Musician send mutations require assigned mix ownership; channel master controls remain Engineer+.


### Added — Phase 9
- **Admin API server-side routes** (`server/api-server/src/routes/admin.rs`): 4 endpoints, all require Admin role:
  - `GET /api/v1/admin/users` — list all users (id, username, role)
  - `DELETE /api/v1/admin/users/{id}` — delete user by numeric ID (204 / 404)
  - `GET /api/v1/admin/sessions` — list active (non-revoked, non-expired) refresh-token sessions
  - `DELETE /api/v1/admin/sessions/{id}` — revoke session by numeric ID (204 / 404)
- **DB methods** added to `Db`: `list_users`, `delete_user`, `list_active_sessions`, `revoke_session_by_id` — all parameterized, no format-string SQL.
- **`ApiError::NotFound(String)`** variant added (was `&'static str`); HTTP 404 response.
- **8 new HTTP integration tests** for all admin endpoints (RBAC enforcement, success paths, 404 paths). api-server: 19 → 27 tests.
- **3 new DB unit tests**: `list_users_empty_and_populated`, `delete_user_ok_and_not_found`, `list_and_revoke_sessions`.
- **npm audit job** added to CI (HIGH severity gate, non-blocking until dependencies present).
- **Biquad coefficient validation** vs Python/scipy: max delta ≈ 5×10⁻⁸ (f32 rounding only), algorithm confirmed identical.

### Fixed — Phase 9
- **Admin CLI `user create`** command: `--name` → `--username`, added required `--password` arg. Server body now `{username, password, role}` matching `CreateUserRequest`.
- **Admin CLI `session revoke`** command: `--token: String` → `--id: u64`, dispatches `DELETE /api/v1/admin/sessions/{id}`.

### Changed — Phase 9
- Total workspace tests: 157 → 168 (+11: 8 integration + 3 DB unit).
- `GET /api/v1/admin/users` added alongside existing `POST` (combined route).


- **EQ + Compressor integrated into `Mix::process`** audio chain: Sum → EQ → Compressor → Master Gain → Limiter. Both are disabled by default (passthrough when disabled).
- **`Mix` struct fields** `eq: ParametricEq` and `compressor: Compressor` exposed for per-mix DSP configuration.
- **5 new mix-engine tests** covering EQ boost at frequency, compressor reduction on loud signal, EQ passthrough when disabled, compressor passthrough when disabled, and EQ→Compressor chain order verification.
- **Admin CLI binary** (`server/admin-cli/`): `open-iem-admin` — commands: `user list/create/delete`, `session list/revoke`, `health`. Global `--server`, `--token` (or `OPEN_IEM_ADMIN_TOKEN` env), `--json` flags. Graceful 404 handling (Phase 9 server-side routes).
- **Musician Guide PDF** (`docs/guides/MUSICIANS-GUIDE.pdf`): generated via pandoc+xelatex from `MUSICIANS-GUIDE.md`. 61 KB, PDF 1.5. Emoji glyphs (⚠, ✅, ❌) render as blank in lmroman — cosmetic only.
- **`PHASE-8-REVIEW.md`** in `docs/reviews/`.

### Changed — Phase 8
- mix-engine test count: 74 → 79 (+5 new chain integration tests).
- Total workspace tests: 152 → 157.
- `server/Cargo.toml` workspace members: added `admin-cli`.

### Added — Phase 7
- **Biquad Parametric EQ** (`mix-engine/src/eq.rs`): real Type-II Transposed DF2 peaking filter replacing Phase 2 passthrough stub. RBJ Audio EQ Cookbook coefficients, SAMPLE_RATE=48000, stereo biquad state inline (no heap), all 4 bands independent.
- **BiquadCoeffs** struct: `identity()` + `peaking(frequency_hz, gain_db, q)` — coefficients recomputed on `set_band`.
- **BiquadState** struct: stereo delay lines `w1_l/w2_l/w1_r/w2_r`, `tick()` for per-sample processing.
- **RMS Compressor** (`mix-engine/src/compressor.rs`): stereo-linked RMS detector + smoothed gain reduction replacing Phase 2 passthrough stub.
- Compressor mutators: `set_threshold`, `set_ratio`, `set_attack_ms`, `set_release_ms` — all bump revision and recompute coefficients.
- **Docker Compose** dev environment (`docker-compose.yml`): api-server, musician-ui, engineer-ui services with health checks.
- **Musician Guide** (`docs/guides/MUSICIANS-GUIDE.md`): 14-section guide in pt-BR covering Linux requirements, server setup, JWT key generation, login, mix controls, WebRTC status, permissions, diagnostics, LAN, security, known limitations.
- **PHASE-6-REVIEW.md** and **PHASE-7-REVIEW.md** added to `docs/reviews/`.
- `cargo fmt --all` applied across workspace.

### Changed — Phase 7
- `ParametricEq::process` signature changed from `&self` to `&mut self` (required for biquad state mutation).
- mix-engine test count: 57 → 74 (+17 new DSP tests for biquad EQ and compressor).
- Total workspace tests: 135 → 152.

### Added — Phase 6
- Real trickle-ICE candidate injection via `Candidate::from_sdp_string` + `Rtc::add_remote_candidate` in streaming crate (SIMULATED on VPS)
- Named constants replace magic numbers in streaming crate (`MAX_SDP_BYTES`, `MAX_CANDIDATE_BYTES`, `MAX_USER_ID_BYTES`)
- HTTP integration test suite for api-server (19 tests: auth, RBAC, CSRF, audio routes, channel controls)
- Test fixture role casing corrected (`"MUSICIAN"`, `"ADMIN"`, `"ENGINEER"` — SCREAMING_SNAKE_CASE)
- `axum-test` pinned to `"21"`, `jsonwebtoken` gains `rust_crypto` feature
- Project directory structure (all planned directories)
- 11 project-specific agent skills in `.agents/skills/`
- Documentation structure (`docs/` with all subdirectories)
- ADR baseline (ADR-001 through ADR-008)
- `docs/SPEC-AUDIT.md` — specification audit with 14 FRs, 10 NFRs, 10 missing requirements, 10 architecture gaps
- `docs/ARCHITECTURE-GAPS.md` — 10 identified gaps with phase dependencies
- `docs/SKILLS.md` — skills registry and orchestration guide
- `docs/TODO.md` — prioritized backlog
- `docs/DEVELOPMENT-LOG.md` — development history
- `docs/DEVELOPMENT-ENVIRONMENT.md` — environment audit
- GitHub Actions CI foundation (Rust lint/test/build, ARM64 cross-build, frontend, skill validation)
- `scripts/validate-skills.sh` — skill validation script
- Symlinks: `.hermes/skills/` and `.claude/skills/` → `.agents/skills/`
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `LICENSE` (Apache 2.0), `.gitignore`

## [0.0.1] - 2026-09-07

### Added
- Initial repository creation
- Minimal README
