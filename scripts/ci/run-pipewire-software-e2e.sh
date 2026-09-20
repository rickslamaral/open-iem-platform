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
# Create deterministic null sink/source nodes in private userspace graph. These prove
# virtual graph plumbing only; they do not prove physical devices or WebRTC media.
sink_output=$(pw-cli create-node adapter '{ factory.name = support.null-audio-sink node.name = openiem.virtual_sink node.description = OpenIEM\ Virtual\ Sink media.class = Audio/Sink audio.rate = 48000 audio.channels = 2 }' 2>&1) || { printf '%s\n' "$sink_output" >&2; exit 1; }
source_output=$(pw-cli create-node adapter '{ factory.name = support.null-audio-source node.name = openiem.virtual_source node.description = OpenIEM\ Virtual\ Source media.class = Audio/Source audio.rate = 48000 audio.channels = 2 }' 2>&1) || { printf '%s\n' "$source_output" >&2; exit 1; }
assert_node_properties() {
  local node_name=$1 media_class=$2
  awk -v node_name="$node_name" -v media_class="$media_class" '
    BEGIN { RS = ""; found = 0 }
    index($0, "node.name = \"" node_name "\"") &&
    index($0, "media.class = \"" media_class "\"") &&
    index($0, "audio.rate = 48000") &&
    index($0, "audio.channels = 2") { found = 1 }
    END { exit(found ? 0 : 1) }
  ' "$NODE_LIST"
}
for _ in $(seq 1 20); do
  pw-cli list-objects Node >"$NODE_LIST"
  assert_node_properties openiem.virtual_sink Audio/Sink &&
    assert_node_properties openiem.virtual_source Audio/Source && break
  sleep 0.1
done
assert_node_properties openiem.virtual_sink Audio/Sink || { cat "$NODE_LIST" >&2; exit 1; }
assert_node_properties openiem.virtual_source Audio/Source || { cat "$NODE_LIST" >&2; exit 1; }
printf '%s\n' 'PIPEWIRE_SOFTWARE_E2E: PASS (SOFTWARE/SIMULATED virtual sink/source enumeration; no hardware/WebRTC claim)'
