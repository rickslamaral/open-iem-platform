# Claude Code — Settings & Behavior

## Permissions

### Allowed without confirmation
- Read any file in the repository
- Run `cargo fmt`, `cargo clippy`, `cargo test`, `cargo build`
- Run `npm run typecheck`, `npm test`, `npm run build` in `web/`
- Run `python3 tests/validate_archive.py` and other test scripts
- Edit source files in `server/`, `web/`, `scripts/`, `deployment/`, `docs/`, `tests/`, `experiments/`
- Create new files anywhere in the repo
- `git add`, `git commit` (message must follow Conventional Commits)

### Requires explicit confirmation
- `git push` — always confirm with user before pushing
- Tag creation (`git tag`)
- Any `docker compose` command that affects running services
- Edits to `.github/workflows/`
- Edits to `scripts/install.sh`
- Any command that writes outside the repository root

### Never do
- Hardcode secrets, tokens, passwords, or private keys in any file
- Commit `.env`, `*.pem`, `*.key`, `*.secret` files
- Use `--force` push without explicit instruction
- Run the installer (`bash scripts/install.sh`) without `--dry-run` unless explicitly asked

## Commit Message Convention

Follow Conventional Commits: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `ci`, `chore`, `perf`, `security`

Scopes: `audio-engine`, `mix-engine`, `api-server`, `admin-cli`, `iem-cli`, `musician`, `engineer`, `deployment`, `ci`, `scripts`, `docs`

Example: `feat(api-server): add pan control to musician WebSocket protocol`

## Phase Workflow

1. Create/update items in `docs/TODO.md`
2. Implement changes — run all local gates
3. Update `CHANGELOG.md` under `[Unreleased]`
4. Add entry to `docs/DEVELOPMENT-LOG.md`
5. Create `docs/reviews/PHASE-<N>-REVIEW.md`
6. Update `README.md → Current Status`
7. Commit with `docs: Phase <N> review, DEVELOPMENT-LOG, TODO, CHANGELOG`
