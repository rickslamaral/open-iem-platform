# Realtime Scheduling Research

**Status:** RESEARCH COMPLETE — closes GAP-008
**Date:** 2026-09-08
**Author:** Autonomous Engineering Agent (Phase 1)
**Environment:** SIMULATED — hardware validation pending

---

## Question

For the Open IEM Mix Engine:
1. Should the application explicitly set `SCHED_FIFO` for audio threads?
2. Or should it rely on PipeWire-managed priority via rtkit?
3. What is the role of rtkit in this architecture?

---

## Background: Linux Realtime Scheduling Basics

Linux provides two mechanisms for realtime thread priority:

### SCHED_FIFO (POSIX Realtime)
- Thread runs until it yields or is preempted by a higher-priority RT thread.
- Requires `CAP_SYS_NICE` or `rlimit` `RLIMIT_RTPRIO > 0` (typically via PAM/limits.conf).
- If misused (infinite loop, I/O blocking in RT thread), can hang the system.
- Priority range: 1 (lowest) to 99 (highest).

```c
struct sched_param sp = { .sched_priority = 80 };
pthread_setschedparam(pthread_self(), SCHED_FIFO, &sp);
```

### SCHED_RR (Round Robin)
- Like SCHED_FIFO but with a time slice — preempts after quantum expires.
- Less suitable for audio than SCHED_FIFO.

### PREEMPT_RT Kernel Patch
- Converts most non-preemptible kernel sections to preemptible with RT mutexes.
- Dramatically reduces worst-case latency (`wakeup latency` from ~100µs to ~20µs typical on RPi).
- Standard in Ubuntu 24.04+ with `linux-image-*-rt` kernel.
- Not available on VPS — development only.

---

## rtkit — The Realtime Policy Kit Daemon

### What it does
rtkit (`rtkit-daemon`) is a D-Bus daemon that grants realtime priority to unprivileged processes on request, within policy limits.

```
Application thread
    │
    │ D-Bus call: org.freedesktop.RealtimeKit1.MakeThreadRealtime
    ▼
rtkit-daemon
    │  (verifies: nice level, max RT time budget)
    ▼
pthread_setschedparam(SCHED_FIFO, priority=...)
```

### Policy enforcement
- Enforces CPU time budget (prevents RT starvation bugs from hanging system).
- Default: 200ms RT time budget per 5 seconds (configurable).
- Priority range granted: typically 10–80 (below kernel drivers, above normal processes).

### PipeWire's use of rtkit
PipeWire uses rtkit by default to promote its own threads and registered filter/stream threads to `SCHED_FIFO`. When an application creates a `pw_filter` or `pw_stream` node that joins the PipeWire graph:

1. PipeWire's scheduler detects the thread.
2. PipeWire calls rtkit on the thread's behalf (or directly if it has the capability).
3. The process callback runs at `SCHED_FIFO` priority without the application needing `CAP_SYS_NICE`.

---

## Option Analysis

### Option A: Let PipeWire Manage RT Priority (Recommended for Phase 1)

**Mechanism:**
- Register as `pw_filter` or JACK client (via pipewire-jack).
- PipeWire + rtkit promote the audio callback thread automatically.
- Application code never calls `pthread_setschedparam` directly.

**Advantages:**
- No `CAP_SYS_NICE` required in application.
- Consistent with PipeWire's security model.
- Works with WirePlumber session policy.
- Simpler deployment — no special systemd `AmbientCapabilities`.
- rtkit provides starvation protection budget.

**Disadvantages:**
- Application has no control over exact RT priority.
- Depends on rtkit being installed and D-Bus available.
- Priority ceiling is rtkit-policy-limited (usually ≤ 80).

**Suitable for:** Phase 1 MVP. Most production PipeWire audio applications use this.

---

### Option B: Explicit SCHED_FIFO in Application Thread

**Mechanism:**
Application sets its own RT priority using `pthread_setschedparam` or via `sched_setscheduler(2)`.

**Requires one of:**
- systemd service with `AmbientCapabilities=CAP_SYS_NICE`
- PAM limits: `@audio hard rtprio 80` in `/etc/security/limits.d/audio.conf`
- User in `audio` group with RTKit fallback

**Advantages:**
- Full control over priority (can set to 90+, above rtkit ceiling).
- Independent of D-Bus / rtkit availability.
- Useful for extreme low-latency (128-sample buffer or below).

**Disadvantages:**
- Requires privilege configuration (systemd unit or PAM).
- If audio thread blocks on I/O (bug), system can freeze without rtkit budget.
- More complex deployment (systemd unit must set capabilities).

**Suitable for:** Phase 2+ if XRUN measurement shows rtkit-granted priority is insufficient. Require PREEMPT_RT kernel for maximum effect.

---

### Option C: Hybrid — PipeWire-Managed with Fallback

**Mechanism:**
- Default: rely on PipeWire + rtkit (Option A).
- If rtkit unavailable: attempt direct `pthread_setschedparam` if application has capability.
- Log RT scheduling mode at startup for observability.

**Implementation:**
```rust
// Pseudocode — in mix-engine init (NOT in audio callback)
fn try_set_realtime_priority(priority: u32) -> Result<(), RtError> {
    // Try rtkit first via D-Bus
    if let Ok(_) = rtkit_request_realtime(priority) {
        log::info!("RT priority granted via rtkit: {}", priority);
        return Ok(());
    }
    // Fallback: direct syscall (requires CAP_SYS_NICE)
    if let Ok(_) = set_sched_fifo(priority) {
        log::warn!("RT priority set directly (rtkit unavailable): {}", priority);
        return Ok(());
    }
    log::warn!("Could not set realtime priority — running at normal priority");
    Err(RtError::Unavailable)
}
```

**Suitable for:** Phase 2 (when we implement the standalone mix-engine binary).

---

## PREEMPT_RT Kernel Recommendation

For Raspberry Pi 5 production deployment:

```bash
# Ubuntu 24.04 / Raspberry Pi OS
sudo apt install linux-image-6.x.x-rt-arm64

# Verify RT kernel
uname -a | grep PREEMPT_RT

# Check latency with cyclictest (rt-tests package)
sudo cyclictest --mlockall --smp -p 80 -t 4 -m -n -q --duration=60s
```

Target cyclictest result on RPi 5 + PREEMPT_RT:
- Max latency: < 100 µs (acceptable for 2.67ms buffer = 2670 µs budget)
- Typical: < 50 µs

---

## Decision

| Phase   | RT Strategy                                     |
|---------|-------------------------------------------------|
| Phase 1 | Option A — PipeWire + rtkit managed             |
| Phase 2 | Option A with Option C hybrid fallback logging  |
| Phase 8 | Option B available via systemd `CAP_SYS_NICE`   |
| Always  | PREEMPT_RT kernel on RPi 5 production           |

---

## systemd Unit Configuration (Phase 8 reference)

```ini
[Service]
# RT priority via systemd — requires kernel capability
AmbientCapabilities=CAP_SYS_NICE
LimitRTPRIO=80
LimitNICE=-15
# Alternative: rely on rtkit (no capability needed)
# rtkit must be running as system service
```

---

## Gap Status

- **GAP-008:** CLOSED — RT scheduling strategy defined.
- Phase 1: PipeWire + rtkit automatic management.
- Hardware validation required on RPi 5 with PREEMPT_RT kernel.

