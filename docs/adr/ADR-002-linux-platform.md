# ADR-002: Linux as Target Platform

## Status
Accepted

## Context
Open IEM Platform needs a host OS. The product targets live sound engineers who need a cost-effective, reliable audio server for stage use.

## Decision
Linux ARM64/x86_64 is the only currently validated server platform. Windows x64 and Raspberry Pi 5 ARM64 remain project targets, not validated support claims, until native audio/runtime validation completes.

Reference hardware: Raspberry Pi 5 (ARM64).
Alternative hardware: Intel N100/N150, other x86_64 mini PCs.

## Rationale
- Linux provides realtime audio scheduling (PREEMPT_RT, rtkit)
- PipeWire and ALSA are Linux-native
- Raspberry Pi is low-cost, low-power, deployable on stage
- Linux is the only OS where all required audio stack components exist together
- Open source toolchain (Rust, GCC, systemd) fully available

## Consequences
**Positive:**
- Full control over realtime scheduling
- No audio middleware licensing costs
- Raspberry Pi 5 is affordable stage hardware
- ARM64 + x86_64 covers all likely deployment scenarios

**Negative:**
- macOS remains future/backlog; Windows target remains unvalidated until a native backend exists
- Cross-compilation required for ARM64 builds on x86_64 CI

## Constraints
- **NEVER** hard-code Raspberry Pi-specific behavior into core architecture
- Raspberry Pi is a supported hardware profile, not the only target
- ARM64 cross-compilation must be validated in CI
