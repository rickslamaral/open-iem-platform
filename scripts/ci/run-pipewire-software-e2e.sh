#!/usr/bin/env bash
# PipeWire/WirePlumber software graph smoke. No hardware, realtime, WebRTC network,
# latency, or Raspberry Pi support claim. Evidence level: SOFTWARE/SIMULATED.
set -Eeuo pipefail
IFS=$'\n\t'
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"
unset PIPEWIRE_REMOTE PIPEWIRE_RUNTIME_DIR PIPEWIRE_CONFIG WIREPLUMBER_CONFIG_DIR
command -v pw-cli >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (pw-cli missing)' >&2; exit 2; }
command -v pipewire >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (pipewire missing)' >&2; exit 2; }
command -v wireplumber >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (wireplumber missing)' >&2; exit 2; }
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/openiem-pipewire.XXXXXX")
daemon_alive() {
  local pid=$1 state
  kill -0 "$pid" 2>/dev/null || return 1
  state=$(ps -o stat= -p "$pid" 2>/dev/null || true)
  [[ -n "$state" && "$state" != Z* && "$state" != T* && "$state" != t* ]]
}
stop_daemon() {
  local pid=$1
  kill "$pid" 2>/dev/null || true
  for _ in $(seq 1 20); do
    daemon_alive "$pid" || { wait "$pid" 2>/dev/null || true; return; }
    sleep 0.1
  done
  kill -KILL "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
}
cleanup() {
  local status=$?
  set +e
  [[ -n "${WIREPLUMBER_PID:-}" ]] && stop_daemon "$WIREPLUMBER_PID"
  [[ -n "${PIPEWIRE_PID:-}" ]] && stop_daemon "$PIPEWIRE_PID"
  rm -rf -- "$WORK_DIR"
  exit "$status"
}
trap cleanup EXIT INT TERM
RUNTIME_DIR="$WORK_DIR/runtime"
mkdir -m 700 "$RUNTIME_DIR"
PIPEWIRE_LOG="$WORK_DIR/pipewire.log"
WIREPLUMBER_LOG="$WORK_DIR/wireplumber.log"
NODE_LIST="$WORK_DIR/nodes.txt"
export XDG_RUNTIME_DIR="$RUNTIME_DIR"
pipewire >"$PIPEWIRE_LOG" 2>&1 & PIPEWIRE_PID=$!
for _ in $(seq 1 50); do
  pw-cli info 0 >/dev/null 2>&1 && break
  daemon_alive "$PIPEWIRE_PID" || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
  sleep 0.1
done
pw-cli info 0 >/dev/null 2>&1 || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
wireplumber >"$WIREPLUMBER_LOG" 2>&1 & WIREPLUMBER_PID=$!
for _ in $(seq 1 50); do
  nodes=$(pw-cli list-objects Node 2>/dev/null || true)
  grep -q 'node.name' <<<"$nodes" && break
  daemon_alive "$WIREPLUMBER_PID" || { cat "$WIREPLUMBER_LOG" >&2; exit 1; }
  sleep 0.1
done
daemon_alive "$PIPEWIRE_PID" || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
daemon_alive "$WIREPLUMBER_PID" || { cat "$WIREPLUMBER_LOG" >&2; exit 1; }
pw-cli list-objects Node >"$NODE_LIST"
# Core graph existence proves only userspace daemon startup, not audio device support.
grep -q 'node.name' "$NODE_LIST" || { cat "$WIREPLUMBER_LOG" >&2; exit 1; }
printf '%s\n' 'PIPEWIRE_SOFTWARE_E2E: PASS (SOFTWARE/SIMULATED userspace graph; no hardware/WebRTC claim)'
