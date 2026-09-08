# Latency Budget — End-to-End

**Status:** DEFINED — closes GAP-005
**Date:** 2026-09-08
**Author:** Autonomous Engineering Agent (Phase 1)
**Verification:** SIMULATED — hardware measurements pending (Raspberry Pi 5 + USB audio interface)

---

## Goal

Define the maximum acceptable end-to-end latency from instrument capture to IEM output for the Open IEM Platform MVP. Without a target, hardware validation has no pass/fail criterion.

---

## Reference: Human Perception Thresholds

| Latency   | Perception                                      |
|-----------|-------------------------------------------------|
| < 5 ms    | Imperceptible — studio standard                 |
| 5–10 ms   | Imperceptible to most musicians                 |
| 10–20 ms  | Detectable by trained ears, acceptable live     |
| 20–25 ms  | Edge — some musicians notice slight looseness   |
| 25–35 ms  | Noticeable latency — acceptable for IEM at venue scale |
| > 35 ms   | Unacceptable for live monitoring                |
| > 50 ms   | Echo-like — unusable                            |

Sources: AES SC-02-02 Working Group recommendations; Haas effect threshold ~30ms.

---

## Open IEM Platform Targets

| Metric            | Target   | Maximum  | Notes                                   |
|-------------------|----------|----------|-----------------------------------------|
| **End-to-end**    | ≤ 20 ms  | ≤ 35 ms  | Instrument → ADC → server → DAC → IEM  |
| **Audio buffer**  | ≤ 5 ms   | ≤ 10 ms  | PipeWire/ALSA hardware period           |
| **Mix Engine**    | ≤ 1 ms   | ≤ 2 ms   | DSP processing within audio callback    |
| **Transport**     | ≤ 5 ms   | ≤ 15 ms  | Audio frame over LAN (Wi-Fi)            |
| **Receiver decode**| ≤ 2 ms  | ≤ 5 ms   | Decode + playout buffer                 |
| **Control (WS)**  | ≤ 50 ms  | ≤ 200 ms | UI → fader change reflected             |
| **Scene recall**  | ≤ 100 ms | ≤ 500 ms | Full mix state applied                  |

---

## End-to-End Budget Decomposition

```
Instrument
    │
    ▼ ADC conversion          ~0.5 ms
USB Audio Interface
    │
    ▼ USB frame transfer       ~1 ms (USB FS/HS frame)
PipeWire Hardware Node
    │
    ▼ ALSA period buffer       ~2.5 ms (48kHz, 128 samples)
PipeWire Graph Processing
    │
    ▼ Mix Engine callback      ~0.5 ms (SIMD gain/pan/sum — 8ch stereo)
Audio Frame Output
    │
    ▼ Network serialization    ~0.5 ms
LAN / Wi-Fi
    │
    ▼ Wi-Fi air time           ~2–8 ms (802.11n/ac, 2.4/5 GHz)
Receiver
    │
    ▼ Decode + jitter buffer   ~2–5 ms
DAC / Headphone Amp
    │
    ▼ DAC conversion           ~0.5 ms
IEM
─────────────────────────────────────────
TOTAL (best case):            ~10 ms
TOTAL (target):               ~20 ms
TOTAL (maximum):              ~35 ms
```

---

## PipeWire Buffer Sizing

| Buffer (samples @ 48kHz) | Period (ms) | Use Case                        |
|--------------------------|-------------|----------------------------------|
| 64                       | 1.33 ms     | Extreme low latency (XRUN risk) |
| 128                      | 2.67 ms     | Low latency — Phase 1 target    |
| 256                      | 5.33 ms     | Balanced — Phase 1 fallback     |
| 512                      | 10.67 ms    | Stable — degraded mode          |
| 1024                     | 21.33 ms    | High-latency fallback           |

**Phase 1 target:** 256 samples (5.33 ms) for VPS/simulation; 128 samples (2.67 ms) on RPi 5 with USB audio.

---

## Validation Requirements

When hardware is available, validate with:

```bash
# ALSA round-trip latency
alsa-utils: speaker-test + capture analysis

# PipeWire latency measurement
pw-top  # observe quantum and latency

# Dedicated latency measurement
jack_iodelay  # via pipewire-jack bridge
```

All hardware latency measurements **MUST** be committed to:
`docs/testing/LATENCY-MEASUREMENTS.md`

---

## SLA Reference

See `docs/audio/AUDIO-SLA.md` for XRUN SLA linked to buffer size.

---

## Gap Status

- **GAP-005:** CLOSED — latency budget defined.
- Pending: Hardware verification on Raspberry Pi 5 + USB audio interface.

