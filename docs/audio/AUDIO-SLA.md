# Audio SLA — XRUN & Reliability Targets

**Status:** DEFINED — closes GAP-006
**Date:** 2026-09-08
**Author:** Autonomous Engineering Agent (Phase 1)
**Verification:** SIMULATED — hardware measurements pending

---

## What is an XRUN?

An XRUN (overrun/underrun) occurs when the audio hardware requests a buffer that the application failed to fill (underrun) or read (overrun) in time. Every XRUN produces an audible click, dropout, or silence artifact.

---

## XRUN SLA Targets

| Condition                     | Buffer Size | Acceptable XRUNs     |
|-------------------------------|-------------|----------------------|
| Production session (1h)       | 128 samples | **0 XRUNs**          |
| Production session (1h)       | 256 samples | **0 XRUNs**          |
| Stress session (24h)          | 256 samples | ≤ 1 XRUN total       |
| Degraded mode (network storm) | 512 samples | ≤ 2 XRUNs / hour     |
| VPS / development (no RT)     | 1024 samples| Best-effort, not SLA |

> **Rule:** If the system produces any XRUN during a 1-hour production session at the target buffer size, the session is considered a **FAIL** for hardware validation.

---

## CPU Budget (realtime audio thread)

| Phase    | Budget         | Notes                                |
|----------|----------------|--------------------------------------|
| Phase 1  | ≤ 10% CPU core | 8 channels, no DSP, gain/pan/sum only|
| Phase 2  | ≤ 20% CPU core | + limiter, EQ stub                   |
| Phase 5  | ≤ 30% CPU core | + audio transport serialization      |

**Hard rule:** The mix engine audio callback must NEVER block on I/O, mutex with unbounded hold time, or allocate heap memory.

---

## Latency Jitter SLA

| Metric             | Target   | Maximum  |
|--------------------|----------|----------|
| Period jitter      | < 0.5 ms | < 1 ms   |
| Network jitter     | < 2 ms   | < 5 ms   |
| Receiver playout jitter | < 1 ms | < 3 ms |

---

## Startup / Recovery SLA

| Event                     | Target Recovery Time | Action                            |
|---------------------------|----------------------|-----------------------------------|
| PipeWire restart           | ≤ 2 s               | Auto-reconnect, mute during gap   |
| USB audio interface reconnect | ≤ 5 s           | Hot-plug handler, mute → restore  |
| Client reconnect (WebSocket) | ≤ 3 s            | State sync on reconnect           |
| Scene recall               | ≤ 100 ms            | Atomic state apply                |

---

## Monitoring & Observability

XRUN events **MUST** be:
1. Counted in a monotonic `xrun_total` counter
2. Logged with timestamp, buffer size, CPU load snapshot
3. Exposed via observability endpoint (`GET /api/v1/system/audio-health`)
4. Surfaced in Engineer UI as a warning badge

---

## Hardware Validation Protocol

When testing on Raspberry Pi 5 + USB audio interface:

```bash
# Monitor XRUNs via ALSA
arecord -D hw:0,0 -f S32_LE -r 48000 -c 2 --period-size=128 \
        --buffer-size=512 -d 3600 /tmp/test.wav 2>&1 | grep -i xrun

# PipeWire XRUN observation
journalctl -f -u pipewire | grep -i xrun

# pw-top for real-time stats
pw-top
```

Results template:
```
Hardware: <model>
OS: <distro + kernel>
Kernel: PREEMPT / PREEMPT_RT
PipeWire: <version>
Sample rate: 48000
Buffer: <samples>
Duration: <h>
XRUNs: <n>
Max jitter: <ms>
Result: PASS / FAIL
```

---

## Phase 1 Gate Criteria

Phase 1 audio hardware validation **PASSES** when:
- [ ] 0 XRUNs in 1-hour test session at 256-sample buffer
- [ ] Mix engine callback ≤ 10% CPU
- [ ] End-to-end latency ≤ 35 ms (measured, not estimated)
- [ ] XRUN counter exposed in observability endpoint

---

## Gap Status

- **GAP-006:** CLOSED — XRUN SLA defined.
- Pending: Hardware verification on Raspberry Pi 5.

