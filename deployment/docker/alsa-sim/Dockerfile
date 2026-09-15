FROM debian:bookworm-slim

# ALSA dummy driver simulation — no real audio hardware required.
# Mirrors the snd-dummy environment used in VPS/CI validation.
# On a real RPi with USB audio: replace hw:0,0 with hw:1,0 (or as shown by aplay -l).

RUN apt-get update && apt-get install -y --no-install-recommends \
        alsa-utils \
        sox \
    && rm -rf /var/lib/apt/lists/*

# Pre-generate a 3-second 440 Hz sine test tone (S16_LE stereo 44100 Hz).
# Compatible with snd-dummy and USB class-compliant audio interfaces.
RUN sox -n -r 44100 -c 2 -e signed -b 16 /test_tone.wav synth 3 sine 440

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
