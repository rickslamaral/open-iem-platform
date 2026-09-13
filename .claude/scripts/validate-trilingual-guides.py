#!/usr/bin/env python3
"""Validate required PT/EN/ES guide roots and relative links."""
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
errors = []
for language in ("pt", "en", "es"):
    folder = root / "docs/guides" / language
    index = folder / "README.md"
    if not index.is_file():
        errors.append(str(index.relative_to(root)))
        continue
    for link in re.findall(r"\[[^]]+\]\(([^)#]+)", index.read_text(encoding="utf-8")):
        target = (index.parent / link).resolve()
        if not target.exists():
            errors.append(f"{index.relative_to(root)} -> {link}")
if errors:
    print("Broken guide links/files:", *errors, sep="\n- ", file=sys.stderr)
    sys.exit(1)
print("trilingual guide structure: OK")
