#!/usr/bin/env bash
set -euo pipefail

# PreToolUse hook backstop. Receives Claude Code JSON on stdin.
input=$(cat)
if printf '%s' "$input" | grep -Eiq -- '-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----|ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}'; then
  printf '%s\n' 'Blocked: possible secret detected in tool input.' >&2
  exit 2
fi
exit 0
