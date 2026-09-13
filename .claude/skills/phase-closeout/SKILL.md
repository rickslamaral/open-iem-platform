---
description: Close an Open IEM development phase with tests, evidence, and synchronized documentation.
argument-hint: <phase number>
---

# Phase Closeout

1. Read `CLAUDE.md`, `docs/TODO.md`, and the relevant phase review.
2. Run `.claude/scripts/run-all-gates.sh` or record blocked gates exactly.
3. Update TODO, DEVELOPMENT-LOG, CHANGELOG, README, and `docs/reviews/PHASE-<N>-REVIEW.md`.
4. Keep local, CI, release, installation, media, and hardware evidence separate.
5. Run `.claude/scripts/validate-phase-completion.py` and `.claude/scripts/validate-trilingual-guides.py`.
6. Run `git diff --check` and show the final diff summary.
7. Do not claim complete when required evidence is missing.
