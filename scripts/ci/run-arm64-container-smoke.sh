#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
if ! command -v docker >/dev/null 2>&1; then
  echo 'ARM64 userspace smoke: BLOCKED (docker unavailable)' >&2
  exit 2
fi
if ! docker buildx version >/dev/null 2>&1; then
  echo 'ARM64 userspace smoke: BLOCKED (docker buildx unavailable)' >&2
  exit 2
fi
if docker buildx inspect --bootstrap >/dev/null 2>&1; then
  docker build -q --platform linux/arm64 -f "$ROOT/deployment/docker/raspberrypi-os-smoke/Dockerfile" "$ROOT" -t open-iem-platform/raspberrypi-os-smoke:latest >/dev/null
  docker run --rm --platform linux/arm64 open-iem-platform/raspberrypi-os-smoke:latest
  echo 'ARM64 Raspberry Pi OS-compatible userspace smoke: PASS'
else
  echo 'ARM64 userspace smoke: BLOCKED (no runnable binfmt/QEMU builder)' >&2
  exit 2
fi
