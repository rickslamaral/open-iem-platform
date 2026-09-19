# ADR-012: Linux-first portable software release

## Status
Accepted for implementation.

## Decision
Open IEM targets Debian, Ubuntu and Raspberry Pi OS on amd64 and arm64. Raspberry Pi 3 is the minimum family baseline. Software release uses package, CI, virtual audio, ARM64 and emulated userspace evidence. Physical Raspberry Pi tests are classified `HARDWARE_CERTIFICATION` and do not block software release.

## Consequences

- `.deb` artifacts and lifecycle tests become first-class release outputs.
- Hardware support claims require separate certification reports.
- QEMU/container success cannot be described as physical Pi, USB or PipeWire validation.
- Missing emulator or runtime evidence remains `PENDING`.
