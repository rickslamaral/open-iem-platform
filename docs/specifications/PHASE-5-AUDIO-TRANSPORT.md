# Phase 5 Specification: Audio Transport

**Status:** ACCEPTED  
**Date:** 2026-09-08  
**Author:** Autonomous Engineering Agent  
**Phase:** 5 — Audio Transport

---

## 1. Problem

The Open IEM Platform control plane (Phases 1–4) is complete. Musician phones
can authenticate, adjust faders, and receive real-time mix-state updates via
WebSocket. The audio plane is absent: no audio reaches the phones.

Phase 5 closes this gap by implementing the audio transport layer: the signaling
path that sets up an audio delivery channel from the server to each musician's
device.

---

## 2. Context and Constraints

### 2.1 Research baseline
`docs/research/audio-transport/EVALUATION.md` evaluated four protocols (RTP/UDP,
WebRTC, WebTransport/QUIC, Custom UDP) and recommended:

- **Primary:** WebRTC — sole standard that allows a PWA on Android/iOS to
  receive audio without a native app.
- **Secondary:** RTP/UDP — for dedicated hardware receivers (ESP32, Phase 10).

### 2.2 VPS environment (Phase 5)
The VPS has no PipeWire. All audio processing is **SIMULATED**. The transport
layer must compile and test without real audio hardware.

### 2.3 Raspberry Pi target
On the Pi, the audio plane is: PipeWire → Mix Engine → encoded Opus frames →
WebRTC transport → musician phone. Phase 5 delivers the plumbing; real audio
flows after Pi integration (Phase 9+).

### 2.4 Non-goals for Phase 5
- Real PipeWire audio capture (Phase 9)
- Actual Opus encoding (Phase 7 DSP / Phase 9 pipeline)
- Multi-peer SFU scheduling (Phase 8)
- ESP32 RTP path (Phase 10)

---

## 3. Requirements

### Functional
| ID | Requirement |
|----|-------------|
| F-01 | Server exposes `POST /api/v1/audio/offer` — accepts an SDP offer from a client, returns an SDP answer |
| F-02 | Server creates one `str0m` WebRTC peer per musician session |
| F-03 | Server opens an Opus audio track on the peer connection and sends silence frames at 48kHz/stereo/20ms |
| F-04 | ICE candidates are exchanged via `POST /api/v1/audio/ice-candidate` (trickle ICE) |
| F-05 | A session is identified by the JWT `sub` claim already validated by `jwt_auth` middleware |
| F-06 | `GET /api/v1/audio/sessions` (engineer role only) lists active WebRTC sessions |
| F-07 | Peer connection is removed when WebSocket session ends or JWT expires |

### Non-functional
| ID | Requirement |
|----|-------------|
| NF-01 | No blocking calls on the Tokio runtime (str0m input/output loops must run on spawn_blocking or dedicated task) |
| NF-02 | All routes protected by existing `jwt_auth` middleware and RBAC |
| NF-03 | No hardcoded secrets or credentials |
| NF-04 | Compiles on x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu (no platform-specific code in Phase 5) |
| NF-05 | 0 clippy warnings |
| NF-06 | Minimum 15 new unit tests |

---

## 4. Architecture

```
Musician PWA (browser)
    │
    │  POST /api/v1/audio/offer  (SDP offer)
    ▼
api-server  ──jwt_auth──▶  audio_routes
                              │
                              │  creates PeerSession
                              ▼
                        streaming crate
                        (str0m WebRTC)
                              │
                              │  SDP answer
                              ▼
    Musician PWA (browser)
    │
    │  POST /api/v1/audio/ice-candidate  (trickle ICE)
    ▼
api-server  ──▶  streaming crate  ──▶  ICE negotiation
                              │
                              │  DTLS-SRTP tunnel established
                              ▼
                         Opus silence frames (placeholder)
                              │
                              ▼
                         Musician phone speaker (Phase 9+: real mix)
```

### Crate boundaries

| Crate | Role |
|-------|------|
| `streaming` | New crate. WebRTC session lifecycle: SDP negotiation, ICE, track management, frame pump. |
| `api-server` | New routes: `/api/v1/audio/*`. Delegates to `streaming`. |
| `mix-engine` | Unchanged in Phase 5. Will feed frames in Phase 9. |

---

## 5. Implementation Plan

1. Create `server/streaming/` crate with `str0m` dependency.
2. Implement `PeerSession` struct: wraps `str0m::Rtc`, holds `sub`, `mix_id`.
3. Implement `SessionRegistry`: `Arc<Mutex<HashMap<String, PeerSession>>>`.
4. Implement `negotiate_offer(sdp_offer) -> Result<String>` (SDP answer).
5. Validate and route `add_ice_candidate(sub, candidate)` to session boundary;
   candidate injection is completed with the dedicated Sans-IO drive loop in the
   next transport increment.
6. Define silence-frame contract (20ms, 48kHz, stereo); Opus encoding and frame
   pump wait for Phase 7/9 media integration.
7. Wire HTTP routes into `api-server` in the next increment after the signaling
   API contract stabilizes.
8. Tests: SDP validation, bounds, session list, candidate validation, and
   silence-frame contract.

---

## 6. Acceptance Criteria

- [ ] `POST /api/v1/audio/offer` returns 200 + SDP answer for valid JWT musician
- [ ] `POST /api/v1/audio/offer` returns 401 for missing JWT
- [ ] `POST /api/v1/audio/ice-candidate` routes candidate to correct session
- [ ] `GET /api/v1/audio/sessions` returns 403 for musician role, 200 for engineer role
- [ ] `cargo test --workspace` ≥ 118 tests, 0 failures
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo fmt --all -- --check` clean
- [ ] No hardcoded secrets in diff
- [ ] Independent reviewer returns `passed: true`

---

## 7. Security Notes

- `POST /api/v1/audio/offer` must be gated by `jwt_auth` (musician or engineer role).
- ICE candidate endpoint: same auth gate, session keyed by JWT `sub`.
- SDP input treated as untrusted data; str0m parses it — no eval/exec of SDP content.
- No DTLS keys are hardcoded; str0m generates ephemeral keys per session.
- Rate limiting: leverage existing `MAX_MESSAGES_PER_MINUTE` pattern; HTTP routes bounded by 16 KiB global body limit.

---

## 8. Gap Closure

| Gap | Status after Phase 5 |
|-----|----------------------|
| GAP-001 (audio transport selection) | CLOSED — ADR-004 Accepted |
| GAP-002 (browser audio reception) | CLOSED — WebRTC path implemented |
| Audio plane placeholder | CLOSED — silence frames prove wiring |
