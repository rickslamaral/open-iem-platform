---
description: Audit repository before public release. Use when checking secrets, history, license, CI, and public-repo readiness.
argument-hint: [optional repository path]
---

# Public Repository Audit

Run from repository root. Treat all findings as evidence, not assumptions.

1. Inspect `git status`, tracked files, remotes, and recent history.
2. Scan tracked content and Git history for credentials and private material. Do not print secret values.
3. Check `.gitignore`, `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, README status, and workflow safety.
4. Run `.claude/scripts/validate-phase-completion.py` and `.claude/scripts/validate-trilingual-guides.py`.
5. Report blockers: secrets, personal data, missing license/security policy, misleading production claims, or unsafe workflows.
6. Never change GitHub visibility or push. Those require explicit user confirmation.

Output: PASS, BLOCKED, or NEEDS_REVIEW with exact files and commands.
