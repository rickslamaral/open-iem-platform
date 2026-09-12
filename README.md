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

**Fase incremental atual: Phase 63 — verificação fail-closed de arquivos assinados.** O verificador Ed25519 e o validador de archives exigem `O_NOFOLLOW`, validam arquivos regulares em descritores não bloqueantes e o verificador chama `/usr/bin/openssl` sem depender de `PATH` mutável. Execução validada permanece Linux com `/usr/bin/openssl`; CI remoto continua bloqueado por runner/permissões; hardware continua não validado. Ver [review da Phase 63](docs/reviews/PHASE-63-REVIEW.md), [Phase 62](docs/reviews/PHASE-62-REVIEW.md), [Phase 61](docs/reviews/PHASE-61-REVIEW.md), [Phase 60](docs/reviews/PHASE-60-REVIEW.md) e [Phase 59](docs/reviews/PHASE-59-REVIEW.md).

**Phase 56 — correção do lifetime do diretório temporário do Caddyfile.** A limpeza prematura removida do guia Raspberry Pi permite concluir cópia e instalação do Caddyfile; CI remoto e hardware continuam não validados. Ver [review da Phase 56](docs/reviews/PHASE-56-REVIEW.md).

**Phase 54 — provenance de artefatos de release.** O workflow gera attestation Sigstore/GitHub para archives de servidor x86_64 e ARM64 antes do upload, com permissões mínimas de OIDC; CI remoto e hardware continuam não validados. Ver [review da Phase 54](docs/reviews/PHASE-54-REVIEW.md).

**Phase 53 — validação estrutural de archives de release.** O workflow valida archives de servidor x86_64 e ARM64 antes de checksum/upload, rejeitando traversal, links, membros inesperados e binários ausentes. O empacotamento web agora falha se qualquer `dist/` faltar e uploads exigem arquivos. CI remoto e hardware continuam não validados. O guia Raspberry Pi restringe redirects do `curl` a HTTPS, valida os dois binários e instala ambos. CI e release continuam usando `ubuntu-latest`; isso não prova disponibilidade de runner. Os runs mais recentes `34645508776` (PR) e `34645504292` (push) falharam antes dos steps; todos os 9 jobs do PR terminaram com `runner_id=0` e `steps=[]`; PR #40 permanece aberto; merge bloqueado por runner/permissões Actions; sem release. Ver [review da Phase 53](docs/reviews/PHASE-53-REVIEW.md), [guia do músico](docs/guides/MUSICIANS-GUIDE.md), [matriz de validação](docs/validation/PLATFORM-VALIDATION-MATRIX.md), [review da Phase 51](docs/reviews/PHASE-51-REVIEW.md), [Phase 50](docs/reviews/PHASE-50-REVIEW.md), [Phase 49](docs/reviews/PHASE-49-REVIEW.md) e [Phase 48](docs/reviews/PHASE-48-REVIEW.md).

**Phase 34 — CLI administrativo**. `open-iem-admin` exige HTTPS fora de localhost, preserva união de colunas em tabelas JSON e não chama 404 de recurso não implementado.

**Phase 32 — harness determinístico do áudio** (verificação local passa; PR #40 aberto; CI bloqueado; não mergeado)

O harness `SIMULATED` cobre determinismo, isolamento entre mixes, ganho, pan, mute, limiter e finitude das amostras. `cargo fmt --all -- --check` e `cargo test -p audio-engine` passaram: 20 testes unitários, 4 testes de integração e doc-tests. O harness não cobre hardware, desempenho realtime ou stop/start. Áudio real, PipeWire e runtime ARM64 em Raspberry Pi continuam não validados.

Ver [Phase 32 review](docs/reviews/PHASE-32-REVIEW.md), [Phase 31 review](docs/reviews/PHASE-31-REVIEW.md), [Phase 29 review](docs/reviews/PHASE-29-REVIEW.md), [Phase 27 review](docs/reviews/PHASE-27-REVIEW.md), [Phase 24 review](docs/reviews/PHASE-24-REVIEW.md), [Phase 21 review](docs/reviews/PHASE-21-REVIEW.md), [Phase 19 review](docs/reviews/PHASE-19-REVIEW.md) e [Phase 17 review](docs/reviews/PHASE-17-REVIEW.md).

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
| 32 | Deterministic audio harness | ✅ Local tests pass; CI blocked; unmerged |
| 33 | CI branch trigger diagnosis | ✅ Local correction; CI remote blocked |
| 34 | Admin CLI output correctness | ✅ Local tests pass; CI blocked; unmerged |
| 35 | Developer CLI `iem` | ✅ Local tests pass; CI blocked; unmerged |
| 38 | CLI documentation consistency | ✅ Local validation; CI blocked |
| 39 | Makefile audio harness coverage | ✅ Local validation; CI blocked |
| 40 | Compose rebuild and localhost UI binding | ✅ Local validation; CI blocked |
| 41–51 | Documentation, release and ARM64 deployment hardening | ✅ Local validation; CI blocked; hardware pending |
| 52 | Verification follow-up: redirects, CI tool pins and TLS file installation | ✅ Local validation; CI blocked |
| 53 | Release archive validation | ✅ Local validation; CI blocked |
| 54 | Release artifact provenance | 🔄 Local workflow implementation; CI blocked |
| 55 | Deployment path hardening | ✅ Local validation; CI blocked; hardware pending |
| 56 | Caddyfile temporary-directory lifetime | ✅ Local validation; CI blocked; hardware pending |
| 58 | Detached release signature verification | ✅ Local tests; workflow key management pending |
| 59 | Release archive resource limits | ✅ Local tests; CI and hardware pending |
| 61 | Fail-closed release archive input | ✅ Local tests; CI and hardware pending |
| 62 | Python security tests in CI | 🔄 Local workflow implementation; CI and hardware pending |
| 63 | Fail-closed signed-file verification | 🔄 Local implementation; CI and hardware pending |

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
- [Review da Phase 33](docs/reviews/PHASE-33-REVIEW.md)
- [Review da Phase 32](docs/reviews/PHASE-32-REVIEW.md)
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

