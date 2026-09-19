# Linux compatibility

| Platform | amd64 | arm64 | Status |
|---|---:|---:|---|
| Debian 12+ | yes | yes | software target |
| Ubuntu 22.04+ | yes | yes | software target |
| Raspberry Pi OS Bookworm | n/a | yes | software target; Pi 3 baseline |

Runtime capability detection must inspect architecture, systemd, ALSA, PipeWire, WirePlumber, capture/playback devices, Opus and WebRTC availability. Never branch on Pi model when Linux capability detection is sufficient.

Docker, VM, QEMU, ARM64 cross-build and virtual audio prove software portability only. They do not certify physical Pi, USB, thermal, power, controller behavior or real-device latency.
