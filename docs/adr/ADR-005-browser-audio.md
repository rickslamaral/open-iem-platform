# ADR-005: Browser Audio Receiver Architecture

## Status
Proposed — PENDING RESEARCH

## Context
Browsers (mobile and desktop) cannot receive arbitrary UDP audio streams. WebRTC is the standard browser audio API for real-time audio receive. However, WebRTC adds significant implementation complexity (ICE, DTLS, SRTP, SDP negotiation).

Alternative: dedicated native receiver that can use lower-level protocols, only if measured requirements justify it.

This creates a two-tier architecture decision:
1. Control plane: always WebSocket/PWA (browser compatible)
2. Audio plane: WebRTC (browser) vs native receiver vs dedicated hardware

## Decision
**DEFERRED** — requires transport evaluation and browser compatibility research.

## Architecture Options

### Option A: WebRTC for all clients
```
Phone (Browser PWA)
  ├── Control: WebSocket /ws/v1
  └── Audio:   WebRTC (browser RTC API)
```
Pro: No native app required. Con: WebRTC complexity on server.

### Option B: Native receiver app
```
Phone
  ├── Control: WebSocket (PWA or native)
  └── Audio:   Native app (RTP/UDP or custom)
```
Pro: Lower latency, simpler protocol. Con: App distribution required.

### Option C: Hybrid (PWA control + WebRTC audio)
```
PWA: control only (WebSocket)
WebRTC receiver: audio (browser or native)
```
Pro: Separation of concerns. Con: Two components on device.

## Constraints
- **DO NOT** assume phone browser can receive UDP audio
- MVP may use WebRTC as the path of least resistance for browser support
- Dedicated receiver remains conditional backlog work; it requires measured technical justification and an ADR
