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
command -v dbus-daemon >/dev/null || { echo 'PIPEWIRE_SOFTWARE_E2E: BLOCKED (dbus-daemon missing)' >&2; exit 2; }
WORK_DIR=$(mktemp -d "${TMPDIR:-/tmp}/openiem-pipewire.XXXXXX")
process_start_time() {
  python3 "$ROOT/scripts/ci/read-proc-start-time.py" "$1"
}
capture_start_time() {
  local pid=$1 start=
  for _ in $(seq 1 20); do
    start=$(process_start_time "$pid" 2>/dev/null || true)
    [[ -n "$start" ]] && { printf "%s\n" "$start"; return 0; }
    sleep 0.05
  done
  return 1
}

daemon_alive() {
  local pid=$1 expected_start=$2 state actual_start
  kill -0 "$pid" 2>/dev/null || return 1
  actual_start=$(process_start_time "$pid")
  [[ -n "$actual_start" && "$actual_start" == "$expected_start" ]] || return 1
  state=$(ps -o stat= -p "$pid" 2>/dev/null | tr -d '[:space:]' || true)
  [[ -n "$state" && "$state" != Z* && "$state" != T* && "$state" != t* ]]
}
stop_daemon() {
  local pid=$1 expected_start=$2 actual_start
  [[ -n "$pid" ]] || return 0
  # Never signal unverified PID. PID reuse can target unrelated process.
  [[ -n "$expected_start" ]] || return 1
  actual_start=$(process_start_time "$pid" 2>/dev/null || true)
  if [[ -z "$actual_start" || "$actual_start" != "$expected_start" ]]; then
    wait "$pid" 2>/dev/null || true
    return 0
  fi
  local term_status=0
  python3 "$ROOT/scripts/ci/signal-pid.py" "$pid" "$expected_start" TERM 2>/dev/null || term_status=$?
  for _ in $(seq 1 20); do
    actual_start=$(process_start_time "$pid" 2>/dev/null || true)
    [[ -z "$actual_start" || "$actual_start" != "$expected_start" ]] && {
      wait "$pid" 2>/dev/null || true
      return 0
    }
    sleep 0.1
  done
  local kill_status=0
  python3 "$ROOT/scripts/ci/signal-pid.py" "$pid" "$expected_start" KILL 2>/dev/null || kill_status=$?
  for _ in $(seq 1 20); do
    actual_start=$(process_start_time "$pid" 2>/dev/null || true)
    [[ -z "$actual_start" || "$actual_start" != "$expected_start" ]] && {
      wait "$pid" 2>/dev/null || true
      return 0
    }
    state=$(ps -o stat= -p "$pid" 2>/dev/null | tr -d '[:space:]' || true)
    [[ "$state" == Z* ]] && {
      wait "$pid" 2>/dev/null || true
      return 0
    }
    sleep 0.1
  done
  actual_start=$(process_start_time "$pid" 2>/dev/null || true)
  [[ -z "$actual_start" || "$actual_start" != "$expected_start" ]] && {
    wait "$pid" 2>/dev/null || true
    return 0
  }
  state=$(ps -o stat= -p "$pid" 2>/dev/null | tr -d '[:space:]' || true)
  [[ "$state" == Z* ]] && { wait "$pid" 2>/dev/null || true; return 0; }
  return 1
}
cleanup() {
  local status=$? cleanup_status=0
  set +e
  if [[ -n "${WIREPLUMBER_PID:-}" ]] && ! stop_daemon "$WIREPLUMBER_PID" "${WIREPLUMBER_START_TIME:-}"; then cleanup_status=1; fi
  if [[ -n "${PIPEWIRE_PID:-}" ]] && ! stop_daemon "$PIPEWIRE_PID" "${PIPEWIRE_START_TIME:-}"; then cleanup_status=1; fi
  if [[ -n "${DBUS_PID:-}" ]] && ! stop_daemon "$DBUS_PID" "${DBUS_START_TIME:-}"; then cleanup_status=1; fi
  rm -rf -- "$WORK_DIR"
  (( cleanup_status == 0 )) || {
    printf '%s\n' 'PIPEWIRE_SOFTWARE_E2E: FAIL (daemon cleanup could not verify termination)' >&2
    status=1
  }
  exit "$status"
}
trap cleanup EXIT INT TERM
RUNTIME_DIR="$WORK_DIR/runtime"
mkdir -m 700 "$RUNTIME_DIR"
PIPEWIRE_LOG="$WORK_DIR/pipewire.log"
WIREPLUMBER_LOG="$WORK_DIR/wireplumber.log"
NODE_LIST="$WORK_DIR/nodes.txt"
export XDG_RUNTIME_DIR="$RUNTIME_DIR"
DBUS_INFO_FILE="$WORK_DIR/dbus.info"
DBUS_ERROR_FILE="$WORK_DIR/dbus.error"
# Keep daemon in this shell process. Cleanup always has its PID, including
# malformed startup output, and identity checks still prevent PID reuse.
dbus-daemon --session --nofork --print-address=1 --print-pid=1 >"$DBUS_INFO_FILE" 2>"$DBUS_ERROR_FILE" &
DBUS_PID=$!
DBUS_START_TIME=''
DBUS_INFO=''
for _ in $(seq 1 40); do
  [[ -n "$DBUS_START_TIME" ]] || DBUS_START_TIME=$(capture_start_time "$DBUS_PID" 2>/dev/null || true)
  if [[ -r "$DBUS_INFO_FILE" ]]; then
    DBUS_INFO=$(python3 -c "import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_bytes()[:65537].decode('utf-8'),end='')" "$DBUS_INFO_FILE" 2>/dev/null || true)
    if (( ${#DBUS_INFO} > 65536 )); then
      DBUS_INFO=''
    elif [[ $(printf '%s' "$DBUS_INFO" | awk 'END { print NR }') -ge 2 ]]; then
      :
    else
      DBUS_INFO=''
    fi
  fi
  [[ -n "$DBUS_INFO" ]] && break
  daemon_alive "$DBUS_PID" "$DBUS_START_TIME" 2>/dev/null || true
  sleep 0.05
done
[[ -n "$DBUS_INFO" ]] || { echo 'PIPEWIRE_SOFTWARE_E2E: FAIL (D-Bus startup output timeout)' >&2; exit 1; }
mapfile -t DBUS_LINES < <(printf '%s\n' "$DBUS_INFO" | awk 'NF { print }')
DBUS_ADDRESS=${DBUS_LINES[0]-}
DBUS_REPORTED_PID=${DBUS_LINES[1]-}
if (( ${#DBUS_LINES[@]} != 2 )) || [[ "$DBUS_ADDRESS" != unix:* ]] ||
   ! [[ "$DBUS_PID" =~ ^[0-9]+$ && "$DBUS_REPORTED_PID" == "$DBUS_PID" ]] ||
   [[ -z "$DBUS_START_TIME" ]]; then
  echo 'PIPEWIRE_SOFTWARE_E2E: FAIL (invalid or unavailable private D-Bus startup identity)' >&2
  exit 1
fi
export DBUS_SESSION_BUS_ADDRESS="$DBUS_ADDRESS"
deadline=$((SECONDS + 30))
check_deadline() {
  (( SECONDS < deadline )) || { echo 'PIPEWIRE_SOFTWARE_E2E: FAIL (startup deadline exceeded)' >&2; exit 1; }
}
pw_cli() {
  check_deadline
  local status=0
  timeout --foreground 2s pw-cli "$@" || status=$?
  check_deadline
  return "$status"
}
pipewire >"$PIPEWIRE_LOG" 2>&1 & PIPEWIRE_PID=$!
PIPEWIRE_START_TIME=$(capture_start_time "$PIPEWIRE_PID" || true)
[[ -n "$PIPEWIRE_START_TIME" ]] || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
for _ in $(seq 1 50); do
  check_deadline
  pw_cli info 0 >/dev/null 2>&1 && break
  daemon_alive "$PIPEWIRE_PID" "$PIPEWIRE_START_TIME" || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
  sleep 0.1
done
pw_cli info 0 >/dev/null 2>&1 || { cat "$PIPEWIRE_LOG" >&2; exit 1; }
wireplumber >"$WIREPLUMBER_LOG" 2>&1 & WIREPLUMBER_PID=$!
WIREPLUMBER_START_TIME=$(capture_start_time "$WIREPLUMBER_PID" || true)
[[ -n "$WIREPLUMBER_START_TIME" ]] || { cat "$WIREPLUMBER_LOG" >&2; exit 1; }
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
SINK_ID=$(printf '%s\n' "$sink_output" | grep -Eo '(^|[[:space:]])id[[:space:]]+[0-9]+' | grep -Eo '[0-9]+' | tail -n 1 || true)
[[ "$SINK_ID" =~ ^[0-9]+$ ]] || { printf '%s\n' "$sink_output" >&2; exit 1; }
source_output=$(pw_cli create-node adapter '{ factory.name = support.null-audio-source node.name = openiem.virtual_source node.description = OpenIEM\ Virtual\ Source media.class = Audio/Source audio.rate = 48000 audio.channels = 2 }' 2>&1) || { printf '%s\n' "$source_output" >&2; exit 1; }
SOURCE_ID=$(printf '%s\n' "$source_output" | grep -Eo '(^|[[:space:]])id[[:space:]]+[0-9]+' | grep -Eo '[0-9]+' | tail -n 1 || true)
[[ "$SOURCE_ID" =~ ^[0-9]+$ ]] || { printf '%s\n' "$source_output" >&2; exit 1; }
assert_node_properties() {
  local node_id=$1 node_name=$2 media_class=$3
  awk -v node_id="$node_id" -v node_name="$node_name" -v media_class="$media_class" '
    BEGIN { RS = ""; found = 0 }
    index($0, "id " node_id ",") && index($0, "node.name = \"" node_name "\"") {
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
  assert_node_properties "$SINK_ID" openiem.virtual_sink Audio/Sink &&
    assert_node_properties "$SOURCE_ID" openiem.virtual_source Audio/Source && break
  sleep 0.1
done
assert_node_properties "$SINK_ID" openiem.virtual_sink Audio/Sink || { cat "$NODE_LIST" >&2; exit 1; }
assert_node_properties "$SOURCE_ID" openiem.virtual_source Audio/Source || { cat "$NODE_LIST" >&2; exit 1; }
printf '%s\n' 'PIPEWIRE_SOFTWARE_E2E: PASS (SOFTWARE/SIMULATED virtual sink/source enumeration; no hardware/WebRTC claim)'
