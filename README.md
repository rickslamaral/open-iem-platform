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

**Phase 0 — Bootstrap & Specification Audit** (in progress)

See [docs/reviews/PHASE-0-REVIEW.md](docs/reviews/PHASE-0-REVIEW.md) for current phase status.

## Development Phases

| Phase | Name | Status |
|-------|------|--------|
| 0 | Bootstrap & Specification Audit | 🔄 In Progress |
| 1 | Audio Engine POC | ⏳ Pending |
| 2 | Mix Engine | ⏳ Pending |
| 3 | Backend (Rust/REST/WS) | ⏳ Pending |
| 4 | Musician PWA | ⏳ Pending |
| 5 | Audio Transport | ⏳ Pending |
| 6 | Engineer Console | ⏳ Pending |
| 7 | Scenes & Advanced DSP | ⏳ Pending |
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
