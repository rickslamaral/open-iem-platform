# Contributing to Open IEM Platform

## Welcome

Open IEM Platform is an open-source project. Contributions are welcome.

## Development Setup

### Prerequisites

- Linux (Ubuntu 22.04+ recommended)
- Rust stable (`rustup`)
- Node.js 22+
- Git

For audio hardware testing:
- Raspberry Pi 5 (or compatible Linux ARM64 hardware)
- USB audio interface
- PipeWire installed

### Getting Started

```bash
git clone https://github.com/rickslamaral/open-iem-platform
cd open-iem-platform
source ~/.cargo/env
```

### Running the Skill Validator

```bash
bash scripts/validate-skills.sh
```

## Development Methodology

All contributions must follow the development loop:

```
SPEC → PLAN → IMPLEMENT → TEST → SECURITY REVIEW → CODE REVIEW → DOCUMENT → COMMIT
```

**Do not skip stages for significant features.**

## Commit Convention

Use semantic commits:

```
feat:     New feature
fix:      Bug fix
refactor: Code restructure (no behavior change)
test:     Tests only
docs:     Documentation only
build:    Build system, CI, dependencies
ci:       CI/CD configuration
perf:     Performance improvement
chore:    Maintenance tasks
```

Commits must be small and meaningful. Do not create giant commits containing unrelated changes.

## Branch Strategy

- `main` — always deployable
- `feature/*` — new features
- `fix/*` — bug fixes
- `experiment/*` — research/experiments

## Code Standards

### Rust

- Use `cargo fmt` before committing
- Zero `cargo clippy -- -D warnings`
- No `unwrap()` or `expect()` in production paths (tests only)
- All errors return structured types
- No unsafe code without explicit safety comment

### TypeScript

- Strict mode enabled
- No `any` type in production code
- All components must be accessible (WCAG 2.1 AA)

## Realtime Safety (Critical)

**Never** perform blocking operations inside realtime audio processing:
- No database I/O
- No network I/O
- No filesystem I/O
- No unbounded allocation
- No heavy locks

See `.agents/skills/realtime-audio-engineer/SKILL.md` for full requirements.

## Audio Safety (Critical)

- Limiter must be active on all master outputs
- Safe startup: mute until state verified
- Any gain/routing change must be reviewed for accidental loud-output risk

## Testing Requirements

Every feature must have:
- Unit test
- Integration test (where applicable)
- Acceptance criteria verification

Audio features additionally require hardware tests (labeled `SIMULATED` or `HARDWARE VERIFIED`).

## Documentation

Documentation must describe **actual behavior**, not intended behavior.

## Security

See [SECURITY.md](SECURITY.md) for the security policy.

## License

By contributing, you agree your contributions are licensed under Apache 2.0.
