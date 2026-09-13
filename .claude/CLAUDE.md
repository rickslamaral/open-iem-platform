# Open IEM Platform — Claude Configuration

> This file is the authoritative context for Claude Code (and compatible AI coding agents) working on this repository.
> Keep it accurate and up to date as the project evolves.

---

## Project at a Glance

**Open IEM Platform** is an open-source, Linux-native In-Ear Monitoring system for live audio.
Musicians control independent monitor mixes from a smartphone. The server runs on a Raspberry Pi 5 (ARM64) or any Linux x86_64 machine.

- **Stack:** Rust (backend), React/TypeScript (PWA + Engineer UI), PipeWire/ALSA (audio), systemd, Caddy (TLS), SQLite
- **Version:** 0.3.1 (see `VERSION` and `server/Cargo.toml`)
- **Status:** 🚧 **Under active development** — final phases, tests, and hardware validation in progress

---

## Repository Layout

```
.agents/skills/        Project-specific AI agent skills (architect-designer, senior-backend, etc.)
.claude/               Claude Code config — you are here
.github/workflows/     CI/CD (GitHub Actions)
deployment/            RPi5, systemd, Docker, Caddy configs
docs/                  All documentation, ADRs, reviews, guides
experiments/           Audio transport experiments (NOT production code)
firmware/              Embedded/hardware stubs
scripts/               install.sh, dev utilities
server/                Rust workspace (audio-engine, mix-engine, api-server, admin-cli, iem-cli)
tests/                 Cross-cutting Python/integration tests
web/musician/          Musician PWA (React/TypeScript/Vite)
web/engineer/          Engineer Console (React/TypeScript/Vite)
```

---

## Development Rules — Read Before Touching Anything

### Non-negotiable gates before ANY commit

1. `cargo fmt --all -- --check` — zero diffs
2. `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings
3. `cargo test --workspace` — all pass
4. `cd web/musician && npm run typecheck && npm test && npm run build`
5. `cd web/engineer && npm run typecheck && npm test && npm run build`
6. `python3 tests/validate_archive.py` (when touching release scripts)
7. Security: no hardcoded secrets, no plain-text keys, no `panic!` in production paths

### CI is currently blocked (quota/permissions on GitHub Actions)

All local gates above must pass. Do NOT declare a phase done until local gates pass.
CI remote status is a separate concern — log it, don't block work on it.

### Architecture constraints

- **Audio is SIMULATED** until validated on real Raspberry Pi 5 hardware — label it explicitly
- TLS is required for all external endpoints (Caddy handles this)
- JWT with EdDSA (Ed25519) — no RSA, no symmetric JWT in new code
- Argon2id for password hashing — no bcrypt, no MD5, no SHA-1 for passwords
- No `unsafe` Rust without an ADR and security review
- No shell script runs without `set -euo pipefail`
- Installer (`scripts/install.sh`) requires `--ref <40-char-SHA>` — never accept mutable refs

### Role model for new features

| Concern | Correct approach |
|---------|-----------------|
| New API endpoint | REST for CRUD, WebSocket for real-time push |
| Auth | JWT Bearer (EdDSA); musician/engineer/admin roles |
| DB | SQLite via sqlx; migrations in `server/migrations/` |
| Config | Environment variables; no config files with secrets |
| Frontend state | React hooks + WebSocket context; no Redux |
| Audio graph | PipeWire node per mix; SIMULATED on non-RPi hosts |

---

## Phase Tracking

Phases are tracked in `docs/TODO.md` and summarized in `README.md → Development Phases`.

Current phase: **Phase 83 — Pan and master-muted indicator in Musician UI**

Next open gates:
- [ ] CI remote green (blocked: Actions quota/permissions)
- [ ] Physical RPi5 hardware validation
- [ ] Release `v0.3.1` on GitHub

---

## Available AI Skills (`.agents/skills/`)

Load these when working on the relevant area:

| Skill | When to load |
|-------|-------------|
| `architect-designer` | System design, ADRs, component boundaries |
| `senior-backend` | Rust server, API, audio engine, mix engine |
| `senior-frontend` | React, TypeScript, PWA, Engineer UI |
| `realtime-audio-engineer` | PipeWire, ALSA, audio graph, latency |
| `security-review` | Auth, JWT, TLS, installer, supply chain |
| `code-review` | Pre-commit review |
| `test-master` | Writing tests, coverage, harness design |
| `code-documenter` | Docs, ADRs, guides |
| `pr-review` | Pull request review |
| `product-spec` | Feature specification |
| `superpowers` | Complex multi-skill orchestration |

---

## Key Documentation

- `docs/TODO.md` — canonical backlog
- `docs/DEVELOPMENT-LOG.md` — decision history
- `CHANGELOG.md` — versioned history
- `docs/adr/` — Architecture Decision Records
- `docs/reviews/` — Phase review reports
- `docs/guides/` — User-facing guides (pt / en / es)
- `docs/ARCHITECTURE-GAPS.md` — known gaps

---

## Common Commands

```bash
# Rust
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --release

# Musician PWA
cd web/musician && npm install && npm run typecheck && npm test && npm run build

# Engineer UI
cd web/engineer && npm install && npm run typecheck && npm test && npm run build

# Docker dev
docker compose up --build

# Install (Linux, requires 40-char SHA)
bash scripts/install.sh --ref <COMMIT-SHA-40-CHARS>

# Dry-run install
bash scripts/install.sh --dry-run --skip-deps --ref <COMMIT-SHA-40-CHARS>
```

---

## Secrets — Never Commit

The following are **always** in GitHub Secrets / environment variables, never in files:

| Secret | Usage |
|--------|-------|
| `OPENIEM_RELEASE_SIGNING_KEY_PEM` | Ed25519 private key for release signing |
| `OPENIEM_RELEASE_SIGNING_PUBLIC_KEY_PEM` | Corresponding public key |
| `OPENIEM_RELEASE_SIGNING_PUBLIC_KEY_FINGERPRINT` | SHA-256 hex fingerprint |
| `JWT_SECRET` / `JWT_PRIVATE_KEY` | Runtime JWT signing key (generated by installer) |

`.gitignore` covers `.env`, `*.pem`, `*.key`, `*.secret`, `secrets/`.

---

## Definition of Done (DoD)

A phase is **done** when ALL of the following are true:

- [ ] All local gates pass (fmt, clippy, tests, typecheck, build)
- [ ] `docs/TODO.md` updated — phase items checked off
- [ ] `docs/DEVELOPMENT-LOG.md` entry added
- [ ] `CHANGELOG.md` updated under `[Unreleased]`
- [ ] `README.md → Current Status` updated
- [ ] Phase review doc created in `docs/reviews/PHASE-<N>-REVIEW.md`
- [ ] ADR created/updated if an architectural decision was made
- [ ] No regressions in existing tests

CI remote and hardware validation are tracked separately and do NOT block local DoD.

---

## Language

- Code, comments, commit messages: **English**
- User-facing docs and guides: **Portuguese (pt-BR)** primary, **English (en)** and **Spanish (es)** mirrors
- ADRs, reviews, internal docs: **English** preferred; Portuguese acceptable
