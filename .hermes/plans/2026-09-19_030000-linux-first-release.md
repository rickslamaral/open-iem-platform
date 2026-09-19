# Linux-first Release Architecture Implementation Plan

> **For Hermes:** Execute task-by-task; preserve evidence boundaries and do not commit/push.

**Goal:** Transform Open IEM into Linux-first software-release product supporting Debian, Ubuntu and Raspberry Pi OS on amd64/arm64, with `.deb` packaging and no mandatory Raspberry Pi hardware gate for software release.

**Architecture:** Keep Raspberry Pi as a supported Linux ARM64 family target, not a required physical appliance. Add explicit evidence levels: CODE, CI, SOFTWARE_RELEASE_GATE, and HARDWARE_CERTIFICATION. Build identical server/systemd package payloads for amd64 and arm64, with package lifecycle scripts for install/upgrade/remove/purge. Use Docker/QEMU/virtual audio for reproducible software validation; reserve physical Pi/USB/PipeWire latency and endurance evidence for certification.

**Tech Stack:** Rust workspace, Debian packaging (`dpkg-deb`/`dpkg-query`), systemd, Docker/Buildx/QEMU where available, GitHub Actions, Bash/Python validators.

---

## Task 1: Define release contract and compatibility matrix

**Files:**
- Modify: `START.md`
- Modify: `docs/DEVELOPMENT-HANDOFF.md`
- Modify: `docs/TODO.md`
- Modify: `docs/DEVELOPMENT-LOG.md`
- Modify: `CHANGELOG.md`
- Create: `docs/RELEASE-GATES.md`
- Create: `docs/COMPATIBILITY-MATRIX.md`
- Create/update: `docs/decisions/ADR-012-linux-first-release.md`

**Acceptance:** Debian/Ubuntu/Raspberry Pi OS, amd64/arm64, Pi 3 baseline, SOFTWARE_RELEASE_GATE vs HARDWARE_CERTIFICATION, and release without physical Pi are explicit and non-contradictory.

## Task 2: Add Debian package builder

**Files:**
- Create: `packaging/deb/build-deb.sh`
- Create: `packaging/deb/DEBIAN/control.in`
- Create: `packaging/deb/DEBIAN/conffiles`
- Create: `packaging/deb/DEBIAN/postinst`
- Create: `packaging/deb/DEBIAN/prerm`
- Create: `packaging/deb/DEBIAN/postrm`
- Create: `packaging/deb/etc/openiem/openiem-server.env.example`
- Create: `packaging/deb/usr/lib/systemd/system/openiem-server.service`
- Create: `packaging/deb/usr/share/doc/openiem/README.Debian`

**Acceptance:** Build from prebuilt binaries for amd64/arm64; package installs binary, systemd unit, config dirs, user/group, preserves config on upgrade, removes package files on uninstall, purges state only when explicitly requested by dpkg purge; no secrets generated in package.

## Task 3: Add package validator and lifecycle tests

**Files:**
- Create: `scripts/validate-deb-package.py`
- Create: `tests/test_validate_deb_package.py`
- Modify: `Makefile`

**Acceptance:** Static package checks cover metadata, architecture, paths, permissions, systemd unit, lifecycle scripts, no embedded secrets, and reproducible file list. Test both architecture metadata variants without requiring foreign execution.

## Task 4: Add software-only ARM64 emulation and release matrix scripts

**Files:**
- Create: `scripts/ci/run-software-release-gates.sh`
- Create: `scripts/ci/run-arm64-container-smoke.sh`
- Create: `deployment/docker/raspberrypi-os-smoke/Dockerfile`
- Create: `deployment/docker/raspberrypi-os-smoke/run.sh`
- Modify: `Makefile`
- Modify: `scripts/install.sh` only if needed for package/source compatibility

**Acceptance:** x86 native gates, arm64 cross-build/package gates, ARM64 userspace smoke using Debian Bookworm/Raspberry Pi OS-compatible image, virtual audio, Opus/WebRTC software tests where existing suites expose them, and bounded 60-minute stability test are runnable without physical hardware. Any unavailable optional emulator is reported PENDING, never PASS.

## Task 5: Add CI software release gates and `.deb` artifacts

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.github/workflows/release.yml`
- Create: `.github/workflows/software-release-gates.yml` if separation is cleaner

**Acceptance:** CI tests amd64 and arm64 package builds, package validator, install/upgrade/uninstall/purge in containers, ARM64 smoke under emulation when runner supports it, and publishes `.deb` only after software gates. Physical hardware is never a required CI dependency.

## Task 6: Execute full validation and correct failures

**Commands:**
- `bash -n` all new scripts
- `git diff --check`
- Rust fmt/clippy/tests/build
- frontend typecheck/tests/build
- deterministic DSP and ALSA virtual/loopback tests
- package build/validator/tests for amd64/arm64
- Docker smoke if Docker/QEMU available
- documentation validators

**Acceptance:** Report exact PASS/FAIL/PENDING evidence. Do not commit or push. Final output includes artifacts, platforms, tests, gates, hardware-only items, START changes, `git status`, and `git diff --stat`.
