# Open IEM — Compatibility Matrix

| Family | OS baseline | Architectures | Software release | Hardware certification |
|---|---|---:|---|---|
| Debian | Debian 12 Bookworm | amd64, arm64 | Supported when package gates pass | Optional target evidence |
| Ubuntu | 22.04+ | amd64, arm64 | Supported when package gates pass | Optional target evidence |
| Raspberry Pi OS | Bookworm | arm64; Pi 3 baseline | Supported through ARM64 userspace/package gates | Physical Pi 3 baseline, then Pi 4/5 |

## CPU baseline

- `amd64`: native CI/package artifact.
- `arm64`: cross-build and ARM64 userspace smoke; QEMU/binfmt may execute package tests.
- Raspberry Pi family baseline: Pi 3 capability target. Pi 4/5 and future models remain compatible when their supported Raspberry Pi OS provides required 64-bit userspace and kernel interfaces.

## Evidence boundary

Docker, VM, QEMU, ARM64 cross-build, virtual ALSA and Raspberry Pi OS containers validate software portability only. They do not validate physical GPIO, USB timing, thermal behavior, real PipeWire/ALSA devices or audio latency.
