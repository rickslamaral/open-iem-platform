#!/usr/bin/env python3
"""Print Linux process start time without ambiguous comm parsing."""
import re
import sys

if len(sys.argv) != 2 or not sys.argv[1].isdigit():
    raise SystemExit(2)
try:
    text = open(f"/proc/{sys.argv[1]}/stat", encoding="ascii").read()
    closing = text.rfind(")")
    if closing < 0:
        raise ValueError
    fields = text[closing + 1 :].split()
    if len(fields) <= 19 or len(fields[0]) != 1 or not fields[0].isalpha():
        raise ValueError
    print(fields[19])
except (OSError, ValueError, IndexError):
    raise SystemExit(1)
