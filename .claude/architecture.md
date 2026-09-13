# Open IEM Platform — Architecture Reference for Claude

## System Overview

```
Digital Mixer / Audio Interface
        │
  Linux IEM Server (PipeWire)
        │
    Mix Engine (Rust)
        │
Independent IEM Mixes (per musician)
        │
    Audio Transport (WebRTC signaling + Opus — SIMULATED)
        │
       Wi-Fi
  /     │     \
Phone  Phone  Phone
 │      │      │
IEM    IEM    IEM
```

## Crates (server/)

| Crate | Role |
|-------|------|
| `audio-engine` | PipeWire graph, ALSA fallback, SIMULATED harness |
| `mix-engine` | Channel gain/pan/mute, limiter, parametric EQ stub |
| `api-server` | REST + WebSocket API, SQLite, JWT auth |
| `open-iem-admin` | Admin CLI (`open-iem-admin`) |
| `iem-cli` | Developer CLI (`iem`) |

## Auth Model

- Roles: `musician`, `engineer`, `admin`
- JWT EdDSA (Ed25519); issued by `api-server`
- Argon2id for password storage
- Admin self-delete: 403

## WebSocket Protocol

- Endpoint: `/ws/musician` (musician role), `/ws/engineer` (engineer/admin)
- Messages: JSON envelopes with `version`, `request_id`, `type`
- Types musician → server: `SetSendGain`, `SetSendPan`, `SetSendMute`
- Types server → client: `State` (full snapshot), `SendAck` (delta), `MasterMuted`
- Revision ordering: client ignores stale revisions; reconciles on gap

## Audio State

| Component | Status |
|-----------|--------|
| Mix Engine (Rust) | ✅ Validated locally |
| PipeWire graph | ⏳ SIMULATED — pending RPi5 |
| Opus transport | ⏳ SIMULATED — pending RPi5 |
| ARM64 binary | ✅ Cross-compiled; hardware pending |

## Key Constraints

- 48 kHz / 32-bit float internal
- 8 inputs, 2 stereo mixes, 2 musicians (MVP)
- Lookahead limiter: 64 frames = 1.33 ms @ 48 kHz
- Latency budget: see `docs/audio/LATENCY-BUDGET.md`
- XRUN SLA: see `docs/audio/AUDIO-SLA.md`
- TLS required for all external endpoints (Caddy)
- Local LAN only in MVP

## ADR Index

See `docs/adr/` for all Architecture Decision Records.
Key ADRs: ADR-001 (Rust), ADR-002 (PipeWire), ADR-003 (EdDSA JWT), ADR-004 (SQLite).

## Known Gaps

See `docs/ARCHITECTURE-GAPS.md` for open gaps and their resolution status.
