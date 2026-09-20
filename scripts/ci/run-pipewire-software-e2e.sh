#!/usr/bin/env bash
# PipeWire/WirePlumber software graph smoke. No hardware, realtime, WebRTC network,
# latency, or Raspberry Pi support claim. Evidence level: SOFTWARE/SIMULATED.
set -Eeuo pipefail
IFS=$'\n\t'
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"
unset PIPEWIRE_REMOTE PIPEWIRE_RUNTIME_DIR PIPEWIRE_CONFIG WIREPLUMBER_CONFIG_DIR
command -v pw-cli >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (pw-cli missing)' >&2; exit 2; }
command -v timeout >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (timeout missing)' >&2; exit 2; }
timeout --foreground 1s true >/dev/null 2>&1 || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (timeout lacks --foreground)' >&2; exit 2; }
command -v pipewire >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (pipewire missing)' >&2; exit 2; }
command -v wireplumber >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (wireplumber missing)' >&2; exit 2; }
python3 -c 'import os; raise SystemExit(0 if hasattr(os, "pidfd_open") and hasattr(os, "pidfd_send_signal") else 1)' || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (pidfd unavailable)' >&2; exit 2; }
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/openiem-pipewire.XXXXXX")
process_start_time() {
  python3 "$ROOT/scripts/ci/read-proc-start-time.py" "$1"
}

daemon_alive() {
  local pid=$1 expected_start=$2 state actual_start
  kill -0 "$pid" 2>/dev/null || return 1
  actual_start=$(process_start_time "$pid")
  [[ -n "$actual_start" && "$actual_start" == "$expected_start" ]] || return 1
  state=$(ps -o stat= -p "$pid" 2>/dev/null || true)
  [[ -n "$state" && "$state" != Z* && "$state" != T* && "$state" != t* ]]
}
stop_daemon() {
  local pid=$1 expected_start=$2
  [[ -n "$pid" && -n "$expected_start" ]] || return 0
  daemon_alive "$pid" "$expected_start" || return 0
  python3 "$ROOT/scripts/ci/signal-pid.py" "$pid" "$expected_start" TERM 2>/dev/null || true
  for _ in $(seq 1 20); do
    daemon_alive "$pid" "$expected_start" || { wait "$pid" 2>/dev/null || true; return; }
    sleep 0.1
  done
  python3 "$ROOT/scripts/ci/signal-pid.py" "$pid" "$expected_start" KILL 2>/dev/null || true
  for _ in $(seq 1 20); do
    daemon_alive "$pid" "$expected_start" || { wait "$pid" 2>/dev/null || true; return; }
    sleep 0.1
  done
  wait "$pid" 2>/dev/null || true
}
cleanup() {
  local status=$?
  set +e
  [[ -n "${WIREPLUMBER_PID:-}" ]] && stop_daemon "$WIREPLUMBER_PID" "${WIREPLUMBER_START_TIME:-}"
  [[ -n "${PIPEWIRE_PID:-}" ]] && stop_daemon "$PIPEWIRE_PID" "${PIPEWIRE_START_TIME:-}"
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
deadline=$((SECONDS + 30))
check_deadline() {
  (( SECONDS < deadline )) || { echo 'PIPEWIRE_SOFTWARE_E2E: FAIL (startup deadline exceeded)' >&2; exit 1; }
}
pw_cli() {
  check_deadline
  timeout --foreground 2s pw-cli "$@"
  local status=$?
  check_deadline
  return "$status"
}
pipewire >"$PIPEWIRE_LOG" 2>&1 & PIPEWIRE_PID=$!
PIPEWIRE_START_TIME=$(process_start_time "$PIPEWIRE_PID")
for _ in $(seq 1 50); do
  check_deadline
  pw_cli info 0 >/dev/null 2>&1 && break
  daemon_alive "$PIPEWIRE_PID" "$PIPEWIRE_START_TIME" || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
  sleep 0.1
done
pw_cli info 0 >/dev/null 2>&1 || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
wireplumber >"$WIREPLUMBER_LOG" 2>&1 & WIREPLUMBER_PID=$!
WIREPLUMBER_START_TIME=$(process_start_time "$WIREPLUMBER_PID")
for _ in $(seq 1 50); do
  check_deadline
  nodes=$(pw_cli list-objects Node 2>/dev/null || true)
  grep -q 'node.name' <<<"$nodes" && break
  daemon_alive "$WIREPLUMBER_PID" "$WIREPLUMBER_START_TIME" || { cat "$WIREPLUMBER_LOG" >&2; exit 1; }
  sleep 0.1
done
daemon_alive "$PIPEWIRE_PID" "$PIPEWIRE_START_TIME" || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
daemon_alive "$WIREPLUMBER_PID" "$WIREPLUMBER_START_TIME" || { cat "$WIREPLUMBER_LOG" >&2; exit 1; }
pw_cli list-objects Node >"$NODE_LIST"
# Create deterministic null sink/source nodes in private userspace graph. These prove
# virtual graph plumbing only; they do not prove physical devices or WebRTC media.
sink_output=$(pw_cli create-node adapter '{ factory.name = support.null-audio-sink node.name = openiem.virtual_sink node.description = OpenIEM\ Virtual\ Sink media.class = Audio/Sink audio.rate = 48000 audio.channels = 2 }' 2>&1) || { printf '%s\n' "$sink_output" >&2; exit 1; }
source_output=$(pw_cli create-node adapter '{ factory.name = support.null-audio-source node.name = openiem.virtual_source node.description = OpenIEM\ Virtual\ Source media.class = Audio/Source audio.rate = 48000 audio.channels = 2 }' 2>&1) || { printf '%s\n' "$source_output" >&2; exit 1; }
assert_node_properties() {
  local node_name=$1 media_class=$2
  awk -v node_name="$node_name" -v media_class="$media_class" '
    BEGIN { RS = ""; found = 0 }
    index($0, "node.name = \"" node_name "\"") {
      has_class = index($0, "media.class = \"" media_class "\"")
      has_rate = ($0 ~ /(^|[[:space:]])audio.rate[[:space:]]*=[[:space:]]*"?48000"?([[:space:]]|$)/)
      has_channels = ($0 ~ /(^|[[:space:]])audio.channels[[:space:]]*=[[:space:]]*"?2"?([[:space:]]|$)/)
      if (has_class && has_rate && has_channels) found = 1
    }
    END { exit(found ? 0 : 1) }
  ' "$NODE_LIST"
}
for _ in $(seq 1 20); do
  check_deadline
  pw_cli list-objects Node >"$NODE_LIST"
  assert_node_properties openiem.virtual_sink Audio/Sink &&
    assert_node_properties openiem.virtual_source Audio/Source && break
  sleep 0.1
done
assert_node_properties openiem.virtual_sink Audio/Sink || { cat "$NODE_LIST" >&2; exit 1; }
assert_node_properties openiem.virtual_source Audio/Source || { cat "$NODE_LIST" >&2; exit 1; }
printf '%s\n' 'PIPEWIRE_SOFTWARE_E2E: PASS (SOFTWARE/SIMULATED virtual sink/source enumeration; no hardware/WebRTC claim)'
