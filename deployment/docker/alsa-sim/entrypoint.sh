#!/bin/bash
# Open IEM Platform — ALSA simulation entrypoint
# Runs inside the alsa-sim container; requires /dev/snd mounted from host.
#
# Host pre-requisite (snd-dummy must be loaded):
#   modprobe snd-dummy
#
# Usage:
#   docker run --rm --device /dev/snd ghcr.io/open-iem/alsa-sim:latest
#
# To target a specific card/device (e.g. USB audio hw:1,0):
#   docker run --rm --device /dev/snd ghcr.io/open-iem/alsa-sim:latest hw:1,0

DEVICE="${1:-hw:0,0}"

# Software-only CI fallback. `null` is an ALSA userspace PCM and needs no
# kernel sound module or /dev/snd. Real RPi/USB runs must pass hw:N,M.
if [ "${ALSA_SIM_MODE:-}" = "null" ]; then
    DEVICE="null"
fi

echo "=== ALSA device list (inside container) ==="
if [ "${DEVICE}" = "null" ]; then
    aplay -L 2>&1 | grep -E '^(null|Discard all samples)' || true
else
    aplay -l 2>&1
fi

echo ""
echo "=== HW params for ${DEVICE} ==="
aplay -D "${DEVICE}" --dump-hw-params /test_tone.wav 2>&1 | head -25

echo ""
echo "=== Playing test tone on ${DEVICE} ==="
aplay -D "${DEVICE}" /test_tone.wav 2>&1
STATUS=$?

if [ $STATUS -eq 0 ]; then
    echo "PASS: playback succeeded on ${DEVICE}"
else
    echo "FAIL: aplay exited with status ${STATUS}"
fi

exit $STATUS
