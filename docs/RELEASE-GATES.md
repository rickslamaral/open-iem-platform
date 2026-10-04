# Open IEM — Release Gates

## Current gate status — 2026-10-04

- `develop` HEAD: `220f5f25b44f4b2b7d9f26589b81b9e2871526d7`; `origin/develop` synchronized.
- PR #354 (`feat: expire QR generations hourly`) remains open against `main`; policy forbids merge from this job.
- PR #354 exact HEAD: 16/16 real CI jobs SUCCESS.
- Software evidence: CODE/CI/SIMULATED. Hardware certification and `v0.3.1` release remain pending.

## Evidence classes

- `CODE`: source, unit/integration tests and static checks.
- `CI`: exact-commit hosted jobs with completed success and artifacts.
- `SOFTWARE_RELEASE_GATE`: reproducible software/package validation; no physical device required.
- `RUNTIME_VALIDATED`: execution on a real Linux target.
- `HARDWARE_CERTIFICATION`: physical Raspberry Pi/audio/network evidence only.

A missing test or unavailable emulator is `PENDING`, never `PASS`.

## Software release gate

Release may proceed without Raspberry Pi hardware when all gates below pass on exact commit:

1. Rust fmt, clippy, unit/integration/security tests.
2. Frontend typecheck, tests and builds.
3. Deterministic DSP/PCM and virtual ALSA tests.
4. Opus/WebRTC software tests exposed by repository suites.
5. amd64 `.deb` build and validator.
6. arm64 `.deb` build or reproducible cross-build artifact and validator.
7. Install, upgrade, uninstall and purge tests in clean Debian-family containers.
8. systemd unit and package ownership/permissions validation.
9. ARM64 userspace smoke test with Docker/QEMU when available.
10. Bounded 60-minute software soak (`OPENIEM_SOAK_SECONDS=3600`); unavailable execution is `PENDING`.
11. Documentation, checksum and provenance validation.

## Hardware certification

These never block software release, but block hardware support/certification claims:

- physical Raspberry Pi 3 baseline and Pi 4/5 compatibility;
- real PipeWire/ALSA device discovery and playback/capture;
- USB audio, hot-plug, reboot recovery, XRUN and thermal stability;
- measured audio latency on target hardware;
- native receiver plus mixer/interface/network/IEM end-to-end;
- 60-minute physical endurance.

## Gate rule

`SOFTWARE_RELEASE_GATE=PASS` permits software release. It does not imply `HARDWARE_CERTIFICATION=PASS`.
