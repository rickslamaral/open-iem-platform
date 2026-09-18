# Development Environment

## Environment Audit — 2026-09-08

### System

| Property | Value |
|----------|-------|
| OS | Ubuntu 25.10 (Questing Quokka) |
| Kernel | 6.17.0-23-generic |
| Architecture | x86_64 |
| CPUs | 4 |
| RAM | 15 GiB total, ~8.4 GiB available |
| Disk (/workspace) | 193 GiB total, 142 GiB available |

### Runtime Availability

| Tool | Version | Status |
|------|---------|--------|
| git | 2.51.0 | ✅ |
| rustc | 1.98.1 | ✅ (installed 2026-09-08) |
| cargo | 1.98.1 | ✅ |
| node | 22.22.3 | ✅ |
| npm | 10.9.8 | ✅ |
| python3 | 3.13.7 | ✅ |
| docker | 29.5.3 | ✅ |
| pw-cli | — | ❌ Not installed (VPS — expected) |
| pipewire | — | ❌ Not installed (VPS — expected) |

### Notes

- **VPS environment**: PipeWire may be unavailable. This is expected; use [`HEADLESS-AUDIO-TESTING.md`](HEADLESS-AUDIO-TESTING.md) for deterministic DSP and optional ALSA Loopback.
- `snd-aloop` requires kernel support and host privilege; Docker cannot load it internally.
- PipeWire validation requires Raspberry Pi or Linux desktop with audio hardware.
- Rust installed via `rustup` into `/root/.cargo/`. Source `~/.cargo/env` in new shells.
- Node 22 LTS — suitable for all frontend tooling.
- Docker available for containerized development workflows.

## Missing Tools (Non-Fatal)

The following tools are missing and required for full development:

| Tool | Purpose | Install |
|------|---------|---------|
| pw-cli | PipeWire CLI | `apt install pipewire` (on audio hardware only) |
| pipewire | Audio server | `apt install pipewire` (on audio hardware only) |
| aarch64-unknown-linux-gnu toolchain | Cross-compile for ARM64 | `rustup target add aarch64-unknown-linux-gnu` |

## VPS vs Hardware Boundary

| Task | VPS | Raspberry Pi / Audio Hardware |
|------|-----|-------------------------------|
| Backend (Rust) | ✅ | ✅ |
| Frontend (React) | ✅ | ✅ |
| API tests | ✅ | ✅ |
| Database | ✅ | ✅ |
| CI | ✅ | ✅ |
| PipeWire integration | ❌ SIMULATED | ✅ |
| ALSA / USB audio | ❌ SIMULATED | ✅ |
| Latency measurement | ❌ SIMULATED | ✅ |
| XRUN validation | ❌ SIMULATED | ✅ |
| Real streaming | ❌ SIMULATED | ✅ |

## Rust Setup

```bash
source ~/.cargo/env
rustc --version  # 1.98.1
cargo --version  # 1.98.1
```

To add ARM64 cross-compilation target:

```bash
rustup target add aarch64-unknown-linux-gnu
apt install gcc-aarch64-linux-gnu
```

## Project Workspace

```
Canonical path: /workspace/open-iem-platform/
GitHub remote:  https://github.com/rickslamaral/open-iem-platform
Branch:         main
```
