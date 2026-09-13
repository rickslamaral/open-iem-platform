#!/usr/bin/env python3
"""Fail when phase completion lacks required evidence files."""
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
required = [
    root / "README.md",
    root / "CHANGELOG.md",
    root / "docs/TODO.md",
    root / "docs/DEVELOPMENT-LOG.md",
]
missing = [str(p.relative_to(root)) for p in required if not p.is_file()]
reviews = list((root / "docs/reviews").glob("PHASE-*-REVIEW.md"))
if not reviews:
    missing.append("docs/reviews/PHASE-<N>-REVIEW.md")

readme = (root / "README.md").read_text(encoding="utf-8")
if not re.search(r"desenvolvimento|development|desarrollo", readme, re.I):
    print("README lacks development status", file=sys.stderr)
    sys.exit(1)
if missing:
    print("Missing DoD files:", ", ".join(missing), file=sys.stderr)
    sys.exit(1)
print(f"phase documentation baseline: OK ({len(reviews)} review files)")
