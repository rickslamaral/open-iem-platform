#!/usr/bin/env bash
# Headless audio validation: deterministic DSP always, ALSA loopback when available.
set -Eeuo pipefail
IFS=$'\n\t'

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
cd "$ROOT"
TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/open-iem-audio.XXXXXX")
LOOPBACK_MODULE_LOADED=0

cleanup() {
  local status=$?
  set +e
  [[ -n "${CAPTURE_PID:-}" ]] && kill "$CAPTURE_PID" 2>/dev/null
  [[ -n "${CAPTURE_PID:-}" ]] && wait "$CAPTURE_PID" 2>/dev/null
  rm -rf -- "$TMP_DIR"
  exit "$status"
}
trap cleanup EXIT INT TERM

printf '%s\n' '== headless audio diagnostics =='
uname -srmo
[[ -r /etc/os-release ]] && . /etc/os-release && printf 'os=%s\n' "${PRETTY_NAME:-unknown}"
command -v cargo >/dev/null || { echo 'cargo missing' >&2; exit 3; }

printf '%s\n' '== deterministic backend: full DSP pipeline =='
cargo test --manifest-path server/Cargo.toml --package audio-engine --test deterministic_harness

if ! command -v aplay >/dev/null || ! command -v arecord >/dev/null; then
  printf '%s\n' 'alsa=unavailable (alsa-utils missing); virtual backend passed'
  exit 0
fi

if command -v modprobe >/dev/null && modprobe -n snd-aloop >/dev/null 2>&1; then
  modprobe snd-aloop >/dev/null 2>&1 || true
fi
if aplay -l | grep -qi 'Loopback'; then
  LOOPBACK_MODULE_LOADED=1
fi

printf '%s\n' '== ALSA devices =='
aplay -l || true
arecord -l || true

if (( LOOPBACK_MODULE_LOADED == 0 )) || ! aplay -l | grep -qi 'Loopback'; then
  printf '%s\n' 'alsa_loopback=unavailable (virtual DSP backend remains release gate)'
  exit 0
fi

TONE="$TMP_DIR/tone.wav"
CAPTURE="$TMP_DIR/capture.wav"
python3 - "$TONE" <<'PY'
import math
import struct
import sys
import wave

path = sys.argv[1]
rate = 48000
frames = rate * 3
with wave.open(path, "wb") as wav:
    wav.setnchannels(1)
    wav.setsampwidth(2)
    wav.setframerate(rate)
    wav.writeframes(b"".join(struct.pack("<h", int(12000 * math.sin(2 * math.pi * 440 * i / rate))) for i in range(frames)))
PY

printf '%s\n' '== ALSA Loopback capture/injection =='
set +e
timeout --signal=TERM 15s arecord -D hw:Loopback,1,0 -f S16_LE -r 48000 -c 1 -d 3 "$CAPTURE" &
CAPTURE_PID=$!
sleep 0.25
timeout --signal=TERM 15s aplay -D hw:Loopback,0,0 "$TONE"
PLAY_STATUS=$?
wait "$CAPTURE_PID"
CAPTURE_STATUS=$?
CAPTURE_PID=
set -e
(( PLAY_STATUS == 0 )) || { printf 'alsa_play=FAIL exit=%s\n' "$PLAY_STATUS" >&2; exit "$PLAY_STATUS"; }
(( CAPTURE_STATUS == 0 )) || { printf 'alsa_capture=FAIL exit=%s\n' "$CAPTURE_STATUS" >&2; exit "$CAPTURE_STATUS"; }

python3 - "$CAPTURE" <<'PY'
import struct
import sys
import wave

with wave.open(sys.argv[1], "rb") as wav:
    assert wav.getframerate() == 48000, wav.getframerate()
    assert wav.getnchannels() == 1, wav.getnchannels()
    assert wav.getsampwidth() == 2, wav.getsampwidth()
    raw = wav.readframes(wav.getnframes())
assert len(raw) >= 48000 * 2, len(raw)
samples = struct.unpack("<%dh" % (len(raw) // 2), raw)
assert any(sample != 0 for sample in samples), "captured PCM is silent"
print(f"alsa_capture=PASS frames={len(samples)} non_silent=true")
PY
