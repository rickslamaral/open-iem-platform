# Open IEM Platform

> Open-source professional personal In-Ear Monitoring platform for Linux.

## What is this?

Open IEM Platform is an open-source, Linux-native system for personal in-ear monitor (IEM) mixing in live audio environments. It allows each musician on stage to control their own independent monitor mix from a smartphone, using a Raspberry Pi or similar Linux SBC as the audio server.

## Architecture

```
Digital Mixer / Audio Interface
        |
  Linux IEM Server (PipeWire)
        |
    Mix Engine
        |
Independent IEM Mixes (per musician)
        |
    Audio Transport
        |
       Wi-Fi
  /     |     \
Phone  Phone  Phone
 |      |      |
IEM    IEM    IEM
```

## MVP Scope

- 8 inputs
- 2 stereo mixes
- 2 musicians / 2 clients
- 48 kHz / 32-bit float internal
- Gain, pan, mute, master volume, limiter
- WebSocket control
- PWA mobile interface
- Local LAN only
- Targets atuais: Windows x64, Linux x64 e Raspberry Pi 5 ARM64; suporte validado e runtime de áudio continuam condicionados a evidência. macOS, Android e iPadOS permanecem backlog

## Current Status

**Fase incremental atual: Phase 31 — JWT post-issuance revocation + fail-closed follow-up**. Implementação está commitada nesta branch e aguarda merge no PR #40; release permanece bloqueada por CI remoto sem jobs executados.

**Phase 31 — JWT post-issuance revocation + fail-closed follow-up** (local verification passes; PR #40 open; CI blocked; not merged)

O cliente Musician envia access token no subprotocolo de autenticação `openiem.bearer.<JWT>` junto de `openiem.v1` durante o upgrade HTTP; `/ws/v1` valida ambos e ecoa somente `openiem.v1`. Query strings não carregam mais tokens. Erros após envelope válido preservam `request_id`; erros de parsing usam `server`. ACKs, snapshot REST, ownership, ordenação, revisão monotônica antes de aplicar `State`/`SendAck`/`MasterAck`, limites de mensagem/frame de 16 KiB aplicados no upgrade (com teste de transporte para mensagem Text acima do limite), keepalive Ping/Pong com desafio correlacionado (30 s / timeout 60 s), máquina de estados sem falso timeout após Pong válido e teste determinístico com relógio pausado, rejeição de frames binários, limite process-wide de 64 conexões, quotas de 4 conexões por usuário e 16 por IP com reserva atômica e liberação RAII, sinalização de ressincronização após lag de broadcast, limiter bounded de 5 falhas de autenticação por IP a cada 60 s (com 4.096 entradas máximas) e logs de falha de consulta de ownership permanecem ativos. Áudio, sessões WebRTC reais, PipeWire e runtime ARM64 em Raspberry Pi continuam `SIMULATED`/não validados.

Ver [Phase 29 review](docs/reviews/PHASE-29-REVIEW.md), [Phase 27 review](docs/reviews/PHASE-27-REVIEW.md), [Phase 24 review](docs/reviews/PHASE-24-REVIEW.md), [Phase 21 review](docs/reviews/PHASE-21-REVIEW.md), [Phase 19 review](docs/reviews/PHASE-19-REVIEW.md) e [Phase 17 review](docs/reviews/PHASE-17-REVIEW.md).

## Development Phases

| Phase | Name | Status |
|-------|------|--------|
| 0 | Bootstrap & Specification Audit | ✅ Complete |
| 1 | Audio Engine POC | ✅ Complete with hardware validation pending |
| 2 | Mix Engine | ✅ Complete |
| 3 | Backend (Rust/REST/WS) | ✅ Complete |
| 4 | Musician Client | ✅ Complete |
| 5 | Audio Transport | 🔄 Signaling scaffold complete; media SIMULATED |
| 6 | Engineer UI | ✅ Operational control dashboard; media SIMULATED |
| 7 | Advanced DSP | 🔄 EQ/compressor delivered; scenes and hardware audio pending |
| 8 | Raspberry Pi Deployment | ⏳ Pending |
| 9 | Cross-Platform Builds | ⏳ Pending |
| 10 | Release Engineering | ⏳ Pending |
| 11 | Musicians Guide | ⏳ Pending |
| 12 | Future Receivers | ⏳ Conditional; requires measured technical justification |
| 27 | WebSocket State Recovery | ✅ Local implementation; CI blocked |
| 28 | WebSocket Connection Fairness | ✅ Local implementation; CI blocked |
| 29 | WebSocket Failed-Auth Limiting | ✅ Local implementation; CI blocked |
| 30 | Docker Compose development | ✅ Configuration validated; runtime pending |
| 31 | JWT post-issuance revocation | ✅ Local implementation and tests; CI blocked; unmerged |

## Repository Structure

```
.agents/skills/       — Project-specific agent skills
.github/workflows/    — CI/CD pipelines
docs/                 — All documentation
server/               — Rust backend
web/musician/         — Musician PWA (React/TypeScript)
web/engineer/         — Engineer console (React/TypeScript)
deployment/           — RPi, systemd, Docker
experiments/          — Audio transport experiments
tests/                — Cross-cutting tests
scripts/              — Development utilities
```

## Documentation

- [Architecture Gaps](docs/ARCHITECTURE-GAPS.md)
- [Specification Audit](docs/SPEC-AUDIT.md)
- [Skills Registry](docs/SKILLS.md)
- [TODO](docs/TODO.md)
- [Development Log](docs/DEVELOPMENT-LOG.md)
- [Windows + Docker Desktop Guide](docs/guides/WINDOWS-DOCKER-GUIDE.md)
- [ADR Index](docs/adr/)
- [Interface CLI](docs/CLI.md)
- [Review da Phase 31](docs/reviews/PHASE-31-REVIEW.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

Apache 2.0 — see [LICENSE](LICENSE).


## Estado Phase 10

Mix assignments persist in SQLite. Engineer/Admin assign mix slots; musicians control only sends belonging to assigned mix. Send state and gain/pan/mute routes are protected by JWT role and ownership checks. Audio remains SIMULATED on VPS until PipeWire/Opus validation on Raspberry Pi 5.

## Release v0.3.1 — Versioned Build Pipeline (preparation)

Release pipeline in `.github/workflows/release.yml`. Planned trigger: semver tag (`v0.3.1`). `v0.3.0` remains an inconsistent historical tag: its commit predates synchronized `0.3.0` manifests and failed version validation.

## Release v0.2.0 — Versioned Build Pipeline

Release pipeline in `.github/workflows/release.yml`. Push a semver tag (`v0.2.0`) to trigger:

1. **Version consistency gate** — tag must match `[workspace.package].version` in `server/Cargo.toml`.
2. **Quality gate** — fmt + clippy + tests + cargo-audit (required before any build).
3. **Linux x86_64 build** — `api-server` + `open-iem-admin` bundled with checksum.
4. **Linux ARM64 build** — cross-compiled for Raspberry Pi 5; marked SIMULATED until hardware validation.
5. **Web artefacts** — Musician PWA + Engineer UI dist bundles with checksums.
6. **GitHub Release** — all artefacts + SHA-256 checksums attached; release notes from CHANGELOG.

> **Note:** PipeWire/Opus audio is SIMULATED on VPS. ARM64 artefact is cross-compiled and untested on real Pi 5 hardware. Mark SIMULATED until physical validation.

## Phase 12 — Security & Automation

- **Dependabot** enabled for Cargo, npm (musician/engineer), and GitHub Actions — weekly updates.
- **Admin self-delete protection** — `DELETE /api/v1/admin/users/{id}` returns 403 if caller's UID matches target.
- **SBOM** — `cargo-sbom` generates `open-iem-server-<ver>-sbom.json` in release pipeline (best-effort, non-blocking).
- **Artifact naming fix** — build jobs now depend on `validate-version` so version is non-empty in filenames.

