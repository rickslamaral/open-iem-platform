# Audio Transport Evaluation

**Status:** PRELIMINARY — closes GAP-001 (partial)
**Date:** 2026-09-08
**Author:** Autonomous Engineering Agent (Phase 1)
**Phase:** Research (Phase 1) — Final selection deferred to Phase 5

---

## Evaluation Question

Which protocol should be used to stream real-time stereo audio from the Open IEM server to musician devices (phones, dedicated receivers)?

Constraint from START.md: "DO NOT blindly assume RTP/UDP is the final solution."

---

## Candidates

1. RTP/UDP (standard media streaming)
2. WebRTC (browser/mobile peer-to-peer or server-side)
3. WebTransport / QUIC (emerging browser API)
4. Custom UDP (raw framing, proprietary)

---

## Architecture Constraint: Control vs Audio Split

Per GAP-002 and ADR-005, browsers CANNOT receive arbitrary UDP audio. The architecture **must** separate:

```
Phone / Browser                    Phone / Native App
        │                                   │
   Control Client                     Audio Receiver
        │                                   │
   WebSocket (WS)              WebRTC / WebTransport / Custom UDP
        │                                   │
   Open IEM Server ───── Audio Plane ───────┘
```

The control path (faders, mute, mix settings) always uses WebSocket.
The audio path protocol is what this document evaluates.

---

## Evaluation Criteria

| Criterion                  | Weight | Description                                      |
|----------------------------|--------|--------------------------------------------------|
| Latency (one-way)          | HIGH   | Lower is better; target ≤ 15ms over LAN         |
| Jitter handling            | HIGH   | Built-in jitter buffer / RTCP / FEC              |
| Packet loss recovery       | HIGH   | Retransmit / FEC / concealment                   |
| Browser compatibility      | MEDIUM | Can a PWA on Android/iOS receive audio?          |
| Android native support     | HIGH   | Target platform                                  |
| iOS support                | MEDIUM | Secondary target                                 |
| CPU overhead (server)      | MEDIUM | Encoding + packetization cost                    |
| CPU overhead (client)      | MEDIUM | Decode cost on phone                             |
| Implementation complexity  | MEDIUM | Dev effort, Rust crate availability              |
| LAN-only security          | LOW    | Local network; no public internet                |
| Synchronization (multi-mix)| MEDIUM | Multiple musicians in sync                       |
| Scalability (# clients)    | LOW    | MVP = 2 clients; future = 16                    |

---

## Candidate Analysis

### 1. RTP / UDP

**Protocol:** IETF RFC 3550 (RTP), RFC 3551 (audio profiles)

**Description:**
Raw UDP packets containing audio frames with RTP header (SSRC, sequence number, timestamp). Typically paired with RTCP for statistics. Codec: Opus (RFC 7587) at 48kHz is standard.

**Latency:**
- Wire latency: ≤ 1ms on LAN (UDP, no acknowledgment).
- One-way: dominated by codec frame size (Opus 20ms frame = 20ms minimum).
- With 2.5ms Opus frame size: ~5ms one-way on LAN.

**Jitter handling:**
- RTCP provides round-trip statistics but no built-in jitter buffer spec.
- Application must implement jitter buffer.
- Industry standard: 20–40ms jitter buffer.

**Packet loss:**
- No built-in retransmit (UDP).
- Opus has built-in PLC (Packet Loss Concealment) at moderate loss (<10%).
- FEC available in Opus (in-band, adds ~50% bandwidth).

**Browser support:**
- ❌ Browsers cannot receive raw UDP RTP (no socket API for arbitrary UDP).
- ✅ Native Android app: full support via Java/NDK.
- ✅ Native iOS app: full support via Swift/ObjC.

**Server implementation (Rust):**
- `rtp` crate or manual packetization.
- Opus: `opus` crate (wraps libopus).
- Low complexity.

**Assessment:**
- Best latency of all options on LAN.
- Cannot be used from browser PWA directly.
- Requires native app or WebRTC bridge for browser clients.

---

### 2. WebRTC

**Protocol:** W3C WebRTC API, RFC 8825 (overview)

**Description:**
Full media stack: ICE, DTLS, SRTP, Opus codec. Designed for browser-to-browser and browser-to-server real-time audio/video.

**Latency:**
- One-way on LAN: ~15–30ms typical (ICE + DTLS overhead + jitter buffer).
- Minimum achievable: ~10ms with aggressive buffer tuning.
- Latency higher than raw RTP due to DTLS + ICE negotiation and mandatory jitter buffer.

**Jitter handling:**
- Built-in jitter buffer (Adaptive Jitter Buffer, AJB) in browsers and WebRTC stacks.
- Standard configuration adds 20–60ms buffer.
- Can be tuned via `RTCRtpReceiver.jitterBufferTarget`.

**Packet loss:**
- Built-in FEC, NACK, PLC.
- Excellent packet loss resilience.

**Browser support:**
- ✅ All modern browsers (Chrome, Firefox, Safari) on Android and iOS.
- ✅ No native app required — PWA can receive WebRTC audio.

**Server implementation (Rust):**
- `webrtc` crate (pure Rust, actively maintained, `webrtc.rs`).
- `str0m` crate (lighter WebRTC implementation, no media engine bundled).
- Complexity: HIGH (ICE, DTLS, SRTP, SDP negotiation, signaling server).

**Assessment:**
- Only option that allows browser PWA to receive audio directly.
- Highest implementation complexity.
- Jitter buffer adds ~20ms that is hard to eliminate.
- Security built-in (DTLS/SRTP mandatory).

---

### 3. WebTransport / QUIC

**Protocol:** W3C WebTransport API (WHATWG), RFC 9000 (QUIC), RFC 9114 (HTTP/3)

**Description:**
QUIC-based transport accessible from browsers. Provides unreliable datagrams (like UDP) and reliable streams (like TCP) over an encrypted connection. Does NOT include a media layer (no codec, no RTP).

**Latency:**
- QUIC datagrams: similar to UDP (~1ms on LAN) but with TLS overhead.
- No built-in jitter buffer or media processing — application must implement.

**Jitter handling:**
- None built-in at protocol level.
- Application-layer jitter buffer required.

**Packet loss:**
- QUIC retransmit available on reliable streams.
- For datagrams: no retransmit — same as UDP.

**Browser support:**
- ✅ Chrome 97+ (Android, desktop).
- ⚠️ Safari: limited support (as of 2026, WebTransport support partial).
- ✅ Firefox: enabled by default in recent versions.
- **Requires HTTPS / valid TLS certificate** — LAN deployment needs self-signed cert with trust installed.

**Server implementation (Rust):**
- `quinn` crate (QUIC/HTTP3, mature).
- `webtransport-quinn` or `wtransport` crate.
- Need to implement audio framing, codec (Opus), and jitter buffer on top.

**Assessment:**
- Browser-compatible with modern Chrome/Firefox.
- Safari support still limited — iOS coverage incomplete.
- Does not include media stack — significant additional implementation burden.
- Requires TLS (adds deployment complexity on LAN).
- Promising for Phase 5+ but not ready for Phase 1 MVP.

---

### 4. Custom UDP

**Protocol:** Proprietary

**Description:**
Custom binary protocol over UDP: minimal header (sequence number, timestamp, mix ID, payload). Opus or PCM payload. No standard.

**Latency:**
- Equal to raw RTP — pure UDP, no overhead.

**Jitter handling:**
- Must implement from scratch.

**Packet loss:**
- Must implement FEC or PLC from scratch.

**Browser support:**
- ❌ Not accessible from browsers.

**Assessment:**
- Maximum control, maximum implementation burden.
- Appropriate only for dedicated hardware receivers (future ESP32 receiver).
- Not suitable for phone/browser clients.

---

## Comparison Matrix

| Criterion              | RTP/UDP     | WebRTC       | WebTransport/QUIC | Custom UDP  |
|------------------------|-------------|--------------|-------------------|-------------|
| One-way latency (LAN)  | ≤ 5 ms      | 10–30 ms     | ≤ 5 ms            | ≤ 5 ms      |
| Built-in jitter buffer | ❌ Manual   | ✅ AJB       | ❌ Manual         | ❌ Manual   |
| Built-in FEC/PLC       | ⚠️ Opus PLC | ✅ Full      | ❌                | ❌          |
| Browser PWA audio      | ❌          | ✅           | ⚠️ Chrome/FF      | ❌          |
| Android native         | ✅          | ✅           | ✅                | ✅          |
| iOS support            | ✅          | ✅           | ⚠️ Partial        | ✅          |
| Server CPU (Rust)      | Low         | Medium-High  | Medium            | Low         |
| Implementation effort  | Low         | High         | Medium-High       | Medium      |
| Security (LAN)         | Manual      | DTLS/SRTP    | TLS/QUIC          | Manual      |
| Multi-mix sync         | Manual      | RTCP/SSRC    | Manual            | Manual      |
| Rust crate maturity    | Good        | Good (webrtc.rs) | Good (quinn)  | N/A         |

---

## Phase 1 Recommendation (POC, no network streaming yet)

Phase 1 does NOT include audio transport. Focus: local mix engine only.

---

## Phase 5 Recommendation

**Primary: WebRTC** for phone/browser clients.

**Rationale:**
1. Only standard that allows a browser PWA to receive audio — satisfies GAP-002.
2. Built-in jitter buffer, FEC, PLC reduce application-layer complexity.
3. Industry validated for real-time audio (Zoom, Meet, Teams all use WebRTC).
4. `webrtc.rs` crate is production-ready for Rust servers.
5. iOS + Android both supported natively via WebRTC library.

**Secondary: RTP/UDP** for dedicated hardware receivers (ESP32, future Phase 10).

**Tertiary: Custom UDP** for ESP32 if RTP overhead is problematic on constrained hardware.

**Reject for now: WebTransport** — iOS/Safari support incomplete; adds TLS complexity without a media stack; re-evaluate in Phase 5 when browser support matures.

---

## Required Next Steps (before Phase 5)

1. [ ] Implement WebRTC signaling endpoint on control server (`/api/v1/audio/offer`)
2. [ ] Benchmark `webrtc.rs` SFU: latency, CPU on RPi 5 with 2 concurrent streams
3. [ ] Measure jitter buffer delay under Wi-Fi congestion (2.4GHz vs 5GHz)
4. [ ] Test RTP/UDP + jitter buffer as reference comparison
5. [ ] Create ADR-004 final version with measured data
6. [ ] Define audio codec parameters: Opus 48kHz, stereo, frame size 10ms

---

## Gap Status

- **GAP-001:** PARTIALLY CLOSED — evaluation complete, WebRTC selected as primary candidate.
- Final closure requires: Phase 5 benchmarks and ADR-004 update with measured data.

