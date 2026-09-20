#!/usr/bin/env python3
"""Print Linux process start time without ambiguous comm parsing."""
import re
import sys

if len(sys.argv) != 2 or not sys.argv[1].isdigit():
    raise SystemExit(2)
try:
    text = open(f"/proc/{sys.argv[1]}/stat", encoding="ascii").read()
    match = re.match(r"^\d+ \(.*\) [A-Z] (.*)$", text, re.DOTALL)
    if match is None:
        raise ValueError
    print(match.group(1).split()[18])
except (OSError, ValueError, IndexError):
    raise SystemExit(1)
