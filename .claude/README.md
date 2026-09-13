# `.claude/`

Configuration and persistent context for Claude Code and compatible agents.

- `CLAUDE.md` — project context, architecture, rules, DoD
- `settings.md` — permissions and workflow policy
- `commands.md` — command reference
- `architecture.md` — technical architecture reference
- `settings.json` — shared permissions and hooks configuration
- `rules/` — path-scoped engineering and documentation rules
- `agents/` — reusable security, test, CI, release, hardware, and documentation reviewers
- `hooks/` — local safety backstops
- `scripts/` — repeatable validation and gate runners
- `references/` — CI, release, and DoD contracts
- `templates/` — phase review, ADR, log, and PR templates
- `skills/task-lifecycle/` — backlog-to-merge workflow
- `agents/task-orchestrator.md` — task lifecycle coordinator

Keep files updated whenever architecture, commands, security policy, or development workflow changes. User-facing guides live under `docs/guides/{pt,en,es}/`.
