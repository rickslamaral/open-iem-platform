# Linux Installation

> 🚧 In preparation. Do not run in production. Real installation still requires dedicated Linux host validation.

Use [`scripts/install.sh`](../../../scripts/install.sh) with a full commit SHA. Run complete local gates after installation:

```bash
curl -fsSL https://raw.githubusercontent.com/rickslamaral/open-iem-platform/main/scripts/install.sh -o install.sh
chmod 700 install.sh
export OPENIEM_REF=<COMMIT-SHA-40-CHARS>
sudo bash install.sh --ref "$OPENIEM_REF" --run-tests --test-report /tmp/openiem-tests.txt
cat /tmp/openiem-tests.txt
```

Headless audio validation:

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends alsa-utils
sudo modprobe snd-aloop
make audio-test
```

The test uses deterministic 440 Hz DSP and, when available, ALSA Loopback with 48 kHz mono `S16_LE`. It discovers the Loopback card by name. Missing `snd-aloop` does not invalidate deterministic DSP, but it does not prove physical ALSA, PipeWire, USB, WebRTC/Opus, or Raspberry Pi support. See [`docs/HEADLESS-AUDIO-TESTING.md`](../../HEADLESS-AUDIO-TESTING.md). Real audio and Raspberry Pi 5 remain pending.
