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
- Linux (Raspberry Pi 5 target, x86_64 also supported)

## Current Status

**Phase 21 — Autenticação WebSocket sem token em URL** (controle operacional; áudio SIMULATED no VPS)

O cliente Musician envia access token no subprotocolo de autenticação `openiem.bearer.<JWT>` junto de `openiem.v1` durante o upgrade HTTP; `/ws/v1` valida ambos e ecoa somente `openiem.v1`. Query strings não carregam mais tokens. ACKs, snapshot REST, ownership, ordenação e limites permanecem ativos. Áudio, sessões WebRTC e telemetria permanecem SIMULATED no VPS.

See [Phase 21 review](docs/reviews/PHASE-21-REVIEW.md). [Phase 19 review](docs/reviews/PHASE-19-REVIEW.md). [Phase 18 review](docs/reviews/PHASE-18-REVIEW.md). [Phase 17 review](docs/reviews/PHASE-17-REVIEW.md).

## Development Phases

| Phase | Name | Status |
|-------|------|--------|
| 0 | Bootstrap & Specification Audit | ✅ Complete |
| 1 | Audio Engine POC | ✅ Complete |
| 2 | Mix Engine | ✅ Complete |
| 3 | Backend (Rust/REST/WS) | ✅ Complete |
| 4 | Musician PWA | ✅ Complete |
| 5 | Audio Transport | 🔄 Signaling scaffold complete; media SIMULATED |
| 6 | Engineer Console | ✅ Operational control dashboard; media SIMULATED |
| 7 | Scenes & Advanced DSP | 🔄 EQ/compressor delivered; scenes pending |
| 8 | Raspberry Pi Deployment | ⏳ Pending |
| 9 | Performance & Reliability | ⏳ Pending |
| 10 | ESP32 / Dedicated Receiver Research | ⏳ Pending |

## Repository Structure

```
.agents/skills/       — Project-specific agent skills
.github/workflows/    — CI/CD pipelines
docs/                 — All documentation
server/               — Rust backend
web/musician/         — Musician PWA (React/TypeScript)
web/engineer/         — Engineer console (React/TypeScript)
firmware/esp32/       — Future: ESP32 receiver firmware
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
- [ADR Index](docs/adr/)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

Apache 2.0 — see [LICENSE](LICENSE).


## Estado Phase 10

Mix assignments persist in SQLite. Engineer/Admin assign mix slots; musicians control only sends belonging to assigned mix. Send state and gain/pan/mute routes are protected by JWT role and ownership checks. Audio remains SIMULATED on VPS until PipeWire/Opus validation on Raspberry Pi 5.

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

