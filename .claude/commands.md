# Open IEM Platform — Agent Commands

Quick reference for Claude Code and AI agents working in this repository.

## Rust (server/)

```bash
# Format
cargo fmt --all

# Lint (zero warnings required)
cargo clippy --all-targets --all-features -- -D warnings

# All tests
cargo test --workspace

# Specific crate
cargo test -p audio-engine
cargo test -p mix-engine
cargo test -p api-server

# Build debug
cargo build

# Build release
cargo build --release

# Cross-compile ARM64 (requires cross)
cross build --release --target aarch64-unknown-linux-gnu -p api-server
cross build --release --target aarch64-unknown-linux-gnu -p open-iem-admin
```

## Frontend

```bash
# Musician PWA
cd web/musician
npm install
npm run typecheck
npm test -- --run
npm run build

# Engineer Console
cd web/engineer
npm install
npm run typecheck
npm test -- --run
npm run build
```

## Python tests

```bash
cd tests
python3 validate_archive.py
python3 -m pytest -v
```

## Docker

```bash
# Start dev stack
docker compose up --build

# Reset
docker compose down -v
```

## Installer (local dry-run only)

```bash
# Never run without --dry-run unless explicitly asked by user
bash scripts/install.sh --dry-run --skip-deps --ref <40-CHAR-SHA>

# Help
bash scripts/install.sh --help
```

## Git

```bash
# Status
git status && git log --oneline -5

# Stage and commit (Conventional Commits)
git add -p
git commit -m "feat(scope): description"

# Push — always confirm with user first
git push
```
