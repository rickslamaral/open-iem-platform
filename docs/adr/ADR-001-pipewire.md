# ADR-001: PipeWire as Audio Graph Engine

## Status
Accepted

## Context
Open IEM Platform needs a Linux audio server to manage audio capture, routing, and processing. Options include JACK, PulseAudio, ALSA direct, and PipeWire.

PipeWire is the current Linux audio standard, shipping as default on major distributions including Ubuntu 22.04+, Fedora 34+, and Raspberry Pi OS Bookworm. It provides low-latency, realtime-capable audio with a unified API replacing both JACK and PulseAudio.

## Decision
Use PipeWire as the audio graph engine. Open IEM Platform integrates as a PipeWire client node (filter/sink/source), not as a replacement for PipeWire.

## Rationale
- PipeWire is the Linux audio standard — not integrating with it would isolate Open IEM from the ecosystem
- Provides realtime scheduling integration via `rtkit`
- Supports USB audio (ALSA backend), session management, and filter graph
- JACK compatibility layer available for legacy tools
- Active upstream development and wide distribution support
- Raspberry Pi OS Bookworm ships PipeWire by default

## Consequences
**Positive:**
- Native realtime audio support
- Works with standard USB audio interfaces
- Ecosystem compatibility
- Distribution support

**Negative:**
- PipeWire API is C-based; Rust bindings (`pipewire-rs`) exist but are less mature than C bindings
- PipeWire not available on VPS development environment — requires hardware for validation
- API changes between PipeWire versions require version pinning

## Alternatives Considered
- **JACK**: Mature, but being superseded by PipeWire; not default on modern distributions
- **ALSA direct**: Lowest level, highest complexity, no session management
- **PulseAudio direct**: Network audio limited, being superseded by PipeWire
