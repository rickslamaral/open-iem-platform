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

## Phase and Task Workflow

Never commit or push directly to `main` from an automated agent.

1. Read task from GitHub Issue/Project or `docs/TODO.md`
2. Update local `main` with `git pull --ff-only origin main`
3. Create dedicated `feat/`, `fix/`, `refactor/`, or `docs/` branch before edits
4. Implement changes — run all local gates
5. Update `CHANGELOG.md` under `[Unreleased]`
6. Add entry to `docs/DEVELOPMENT-LOG.md`
7. Create `docs/reviews/PHASE-<N>-REVIEW.md` when applicable
8. Update `README.md → Current Status` when applicable
9. Commit only on dedicated branch
10. Push branch and open PR against `main`
11. Wait for real CI; `runner_id=0` and `steps=[]` are not success
12. Fix failures and report `READY_TO_MERGE`
13. Merge, tag, release, and visibility changes require explicit human confirmation
14. Never claim merged without verified PR URL and merge SHA
