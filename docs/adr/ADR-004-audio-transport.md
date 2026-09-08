# ADR-004: Audio Transport Protocol Selection

## Status
**Accepted** — 2026-09-08

Supersedes the Proposed/Deferred state set at project bootstrap.

---

## Context

Open IEM Platform must stream independent stereo mixes from the Linux server to
musician devices (phones, dedicated hardware receivers). Multiple transport
protocols exist with different trade-offs.

**Critical browser constraint:** Browsers cannot receive arbitrary UDP audio.
The architecture must distinguish:

- Control client (WebSocket/PWA — browser compatible)
- Audio receiver (WebRTC, native app, or dedicated receiver)

### Evaluation performed

`docs/research/audio-transport/EVALUATION.md` compared four candidates across
latency, jitter handling, packet-loss recovery, browser/iOS/Android
compatibility, CPU overhead, and implementation complexity.

| Protocol | Latency | Browser PWA | Android | iOS | Complexity |
|----------|---------|-------------|---------|-----|------------|
| RTP/UDP | ~5ms | ❌ | ✅ (native) | ✅ (native) | LOW |
| **WebRTC** | **~20–40ms** | **✅** | **✅** | **✅** | **HIGH** |
| WebTransport/QUIC | ~20ms | ⚠️ (Safari partial) | ⚠️ | ❌ | HIGH |
| Custom UDP | ~5ms | ❌ | ✅ (native) | ✅ (native) | MEDIUM |

---

## Decision

**Primary transport: WebRTC (str0m Rust crate)**

Implemented in Phase 5 via `server/streaming/` crate using `str0m`.

**Secondary transport: RTP/UDP** — for dedicated hardware receivers (ESP32 etc.,
Phase 10). Not implemented in Phase 5.

**Rejected: WebTransport/QUIC** — incomplete iOS/Safari support as of 2026-09;
no built-in media layer; re-evaluate in Phase 8+.

**Rejected: Custom UDP for browser clients** — browsers cannot open raw UDP
sockets.

---

## Rationale

1. **Only WebRTC allows a PWA (Android/iOS) to receive audio** without a native
   app — satisfies the core product requirement.
2. **Built-in jitter buffer, FEC, PLC** reduce application-layer complexity
   significantly.
3. **Industry validated**: Zoom, Meet, Teams all use WebRTC for real-time audio.
4. **str0m** is a lightweight Sans-IO WebRTC implementation in pure Rust with no
   external media engine dependency — fits our latency-sensitive codebase.
5. **DTLS-SRTP mandatory** — encryption is built in, not bolted on.

---

## Consequences

### Positive
- PWA on Android/iOS can receive server audio without a native app.
- Encryption is inherent (DTLS-SRTP).
- Mature ecosystem; `str0m` crate actively maintained.

### Negative
- Inherent ~20ms jitter buffer adds latency vs raw UDP.
- SDP/ICE/DTLS negotiation adds implementation complexity.
- CPU cost higher than raw RTP (encoding + DTLS per session).

### Mitigations
- Jitter buffer latency acceptable for IEM monitoring (target ≤ 50ms end-to-end
  including mix processing on LAN).
- ICE candidates exchanged via trickle ICE HTTP endpoint to minimize negotiation
  time.
- Phase 10 adds RTP/UDP path for ESP32 clients where lowest latency is required.

---

## Implementation Notes

- Signaling: `POST /api/v1/audio/offer` + `POST /api/v1/audio/ice-candidate`
- Session keyed by JWT `sub`
- Phase 5 sends silence (placeholder); Phase 9 connects real PipeWire audio
- VPS deployment: SIMULATED (no real audio hardware)
- Raspberry Pi: real PipeWire audio, Phase 9+

---

## Related

- `docs/research/audio-transport/EVALUATION.md`
- `docs/specifications/PHASE-5-AUDIO-TRANSPORT.md`
- ADR-005 (browser audio constraint)
- ADR-001 (PipeWire)
