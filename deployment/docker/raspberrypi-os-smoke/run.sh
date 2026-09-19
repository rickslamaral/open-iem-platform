#!/usr/bin/env bash
set -euo pipefail
arch=$(dpkg --print-architecture)
test "$arch" = arm64 || { printf 'userspace architecture: %s (expected arm64)\n' "$arch" >&2; exit 1; }
. /etc/os-release
test -n "${ID:-}" && test -n "${VERSION_ID:-}" || { echo 'missing OS identity' >&2; exit 1; }
printf 'userspace architecture: %s\n' "$arch"
printf 'OS: %s %s\n' "$ID" "$VERSION_ID"
printf 'Raspberry Pi OS-compatible ARM64 userspace smoke: PASS\n'
printf 'physical kernel/audio/hardware certification: NOT RUN\n'
