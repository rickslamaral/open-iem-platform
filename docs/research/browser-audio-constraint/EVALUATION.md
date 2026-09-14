# Browser Audio Constraint Evaluation

**Status:** COMPLETE — WebRTC remains selected for browser audio reception
**Date:** 2026-09-13
**Phase:** 90 — Research follow-up
**Scope:** Determine what audio a browser/PWA can receive, and whether WebRTC should remain the browser audio path.

## Decision

Use **WebRTC media** for audio delivered to Musician PWA. Keep WebSocket for control; use authenticated HTTP signaling routes for SDP offer/answer and trickle ICE. Do not build raw RTP/UDP or a WebTransport media path for the MVP.

Reason: browsers expose WebRTC as a standardized audio receiver with codec negotiation, jitter handling, packet-loss mechanisms and playback through `MediaStream`/`AudioContext`. Browsers do not expose arbitrary UDP sockets, so a PWA cannot directly receive the project's RTP/UDP or custom UDP stream.

WebTransport remains a future experiment only. It provides transport primitives, not an audio media pipeline; Opus framing, timing, jitter buffering, loss behavior, synchronization and playback integration would become Open IEM responsibilities. Safari/iOS coverage also requires separate validation before treating it as a product path.

## Browser-receivable paths

| Path | Browser PWA | Audio API | Assessment |
|---|---:|---|---|
| WebRTC audio track | Yes | `RTCPeerConnection` → `MediaStreamTrack` / `<audio>` / `AudioContext` | **Selected** |
| WebSocket PCM/Opus bytes | Not as native media | Application decode + Web Audio scheduling | Reject for MVP; no native jitter/media transport |
| WebTransport datagrams | Chromium/Firefox capable; cross-browser status requires validation | Application framing + Web Audio | Future research; too much media work |
| RTP/UDP or custom UDP | No arbitrary UDP API | None | Native receiver only |

A WebSocket connection can carry control messages and signaling data. It must not be confused with a browser-native audio transport: receiving bytes does not make browser playback, clock recovery, jitter control or packet-loss handling available.

## Browser API boundary

### WebRTC

The browser can:

1. Create an `RTCPeerConnection`.
2. Receive a negotiated remote audio track through the `track` event.
3. Attach the resulting `MediaStream` to an `<audio>` element or route it into `AudioContext`.
4. Use WebRTC's negotiated Opus media path and browser-managed transport security.

Open IEM already implements the signaling control plane: authenticated SDP offer/answer and trickle ICE. The server-side streaming scaffold defines a silence-frame contract but does not yet provide a validated live media frame pump; media remains **SIMULATED** until real PipeWire/Opus/Raspberry Pi 5 validation.

### Web Audio

`AudioContext` and `AudioWorklet` can process audio that the browser already decoded or captured. They are not network transports. An `AudioWorklet` does not solve arbitrary UDP reception, packet reordering, jitter buffering or Opus decoding by itself.

Use Web Audio only for optional client-side metering, routing or processing after WebRTC supplies a `MediaStream`.

### WebTransport

WebTransport exposes reliable streams and unreliable datagrams over an encrypted HTTP/3 connection. It does not define audio framing, codec negotiation, RTP timestamps, jitter buffering, loss concealment or playback semantics. A production implementation would need all of those layers plus browser compatibility and TLS deployment validation.

## Verification evidence

### Documentation sources

Checked 2026-09-13; each endpoint returned HTTP 200:

- [MDN `RTCPeerConnection`](https://developer.mozilla.org/en-US/docs/Web/API/RTCPeerConnection)
- [MDN `MediaStreamTrack`](https://developer.mozilla.org/en-US/docs/Web/API/MediaStreamTrack)
- [MDN `AudioWorklet`](https://developer.mozilla.org/en-US/docs/Web/API/AudioWorklet)
- [MDN `WebTransport`](https://developer.mozilla.org/en-US/docs/Web/API/WebTransport)
- [WebRTC API specification](https://w3c.github.io/webrtc-pc/)
- [Web Audio API specification](https://webaudio.github.io/web-audio-api/)
- [WebTransport specification](https://w3c.github.io/webtransport/)

### Repository evidence

- `docs/research/audio-transport/EVALUATION.md` evaluates RTP/UDP, WebRTC, WebTransport and custom UDP.
- `docs/specifications/PHASE-5-AUDIO-TRANSPORT.md` defines WebRTC signaling and browser reception as the selected architecture.
- `server/streaming/` contains the current Sans-IO WebRTC signaling/media scaffold.
- `docs/guides/MUSICIANS-GUIDE.md` labels WebRTC media as **SIMULATED**.

## Acceptance criteria

- [x] Identify browser-native audio reception mechanism.
- [x] State whether arbitrary RTP/UDP can reach a PWA.
- [x] Separate WebSocket control/signaling from audio media.
- [x] Compare WebRTC and WebTransport against browser constraints.
- [x] Record selected architecture and rejected MVP alternatives.
- [x] Preserve **SIMULATED** status for media and hardware.

## Follow-up

- Validate actual `<audio>` playback and `AudioContext` routing in supported browsers.
- Validate WebRTC Opus media, latency, jitter, loss and recovery on Raspberry Pi 5.
- Revisit WebTransport only if measured requirements make a custom browser media pipeline worthwhile.
