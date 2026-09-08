# PipeWire Integration Research

**Status:** RESEARCH COMPLETE — closes GAP-003
**Date:** 2026-09-08
**Author:** Autonomous Engineering Agent (Phase 1)
**Environment:** SIMULATED — PipeWire not available on VPS (development environment only)

---

## Question

Which PipeWire integration API is best for the Open IEM Mix Engine?

Options:
1. **pw-filter (native PipeWire filter node)**
2. **pipewire-jack (JACK bridge)**
3. **pipewire-pulse (PulseAudio bridge)**
4. **ALSA direct (without PipeWire graph)**

---

## 1. pw-filter — Native PipeWire Filter Node

### Description
`pw_filter` is the PipeWire-native API for inserting a processing node into the PipeWire graph. The filter registers as a node with input and output ports. PipeWire calls the filter's process callback synchronously within its scheduling loop.

### API Surface
```c
// C API (also accessible via pw-sys Rust bindings)
struct pw_filter *pw_filter_new(struct pw_core *core,
    const char *name, struct pw_properties *props);

void *pw_filter_add_port(struct pw_filter *filter,
    enum pw_direction direction,
    enum pw_filter_port_flags flags, size_t port_data_size,
    struct pw_properties *props, const struct spa_pod **params,
    uint32_t n_params);

int pw_filter_connect(struct pw_filter *filter,
    enum pw_filter_flags flags,
    const struct spa_pod **params, uint32_t n_params);
```

### Process Callback
```c
static void on_process(void *userdata, struct spa_io_position *position) {
    // Called by PipeWire realtime scheduler
    // Must be: lock-free, no I/O, no heap alloc, bounded CPU
    struct pw_buffer *buf = pw_filter_dequeue_buffer(port);
    // process audio...
    pw_filter_queue_buffer(port, buf);
}
```

### Advantages
- **Native graph scheduling** — PipeWire manages driver election, quantum, and wakeup. No manual thread management.
- **Automatic realtime priority** — PipeWire promotes filter thread via rtkit automatically (no SCHED_FIFO setup needed in application).
- **Format negotiation** — PipeWire handles sample rate and format negotiation via SPA pods.
- **Graph routing** — Other PipeWire clients (pw-link, WirePlumber) can route into/out of our filter node.
- **Session management** — WirePlumber can auto-connect hardware nodes to the filter on device connect.

### Disadvantages
- **C API complexity** — SPA pod encoding is verbose; pw-sys Rust bindings are lower-level.
- **Tight PipeWire coupling** — Application is PipeWire-specific (acceptable given ADR-001 decision).
- **Less documentation** — pw-filter docs are thinner than JACK API docs.

### Rust bindings
- `pipewire` crate (pipewire-rs): provides safe Rust wrappers around pw_filter, pw_stream, pw_core.
- Maintained by the PipeWire project and freedesktop contributors.
- `pw-sys` for direct unsafe FFI if needed.

---

## 2. pipewire-jack (JACK Bridge)

### Description
JACK is an established pro-audio API with a large Rust ecosystem (`jack` crate). The `pipewire-jack` package provides a JACK-compatible library that transparently routes JACK API calls through PipeWire.

### API (via `jack` Rust crate)
```rust
let (client, _status) = jack::Client::new("open-iem", jack::ClientOptions::NO_START_SERVER)?;
let in_port = client.register_port("input_1", jack::AudioIn::default())?;
let out_port = client.register_port("output_1", jack::AudioOut::default())?;

let process = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
    // realtime callback — same rules: no I/O, no alloc
    let in_buf = in_port.as_slice(ps);
    let out_buf = out_port.as_mut_slice(ps);
    // process...
    jack::Control::Continue
};
client.activate_async((), jack::ClosureProcessHandler::new(process))?;
```

### Advantages
- **Mature Rust ecosystem** (`jack` crate is stable and well-documented).
- **JACK API is simple** — well understood by audio engineers.
- **No SPA pod complexity** — format negotiation is simpler.
- **pipewire-jack bridge is transparent** — runs on PipeWire without needing JACK server.

### Disadvantages
- **Indirect** — JACK → pipewire-jack bridge → PipeWire. Extra translation layer adds minor overhead.
- **Dependency on pipewire-jack** — requires `libjack.so` from the pipewire-jack package, not just PipeWire core.
- **Session management** — JACK auto-connection logic is separate from WirePlumber's policies.
- **Slightly higher base latency** — bridge adds ~0–1 period overhead in some configurations.

---

## 3. pipewire-pulse (PulseAudio Bridge)

### Not suitable.
PulseAudio bridge is designed for general-purpose audio playback, not low-latency processing. It adds buffering and resampling. Reject for Mix Engine.

---

## 4. ALSA Direct (no PipeWire graph)

### Not suitable for MVP.
Direct ALSA bypasses PipeWire entirely, which means:
- No graph integration — other apps can't route into the mix engine.
- Manual realtime scheduling required.
- Conflicts with PipeWire owning hardware devices.
- Would require exclusive hardware access (blocking PipeWire from operating).

Acceptable only for experiments or dedicated embedded hardware without PipeWire.

---

## Decision Matrix

| Criterion                  | pw-filter | pipewire-jack | pipewire-pulse | ALSA direct |
|----------------------------|-----------|---------------|----------------|-------------|
| Native graph integration   | ✅ Best   | ✅ Good       | ❌             | ❌          |
| Rust API quality           | ⚠️ Good   | ✅ Best       | ❌             | ⚠️ OK       |
| RT priority (automatic)    | ✅        | ✅            | ❌             | Manual      |
| SPA pod complexity         | ⚠️ High   | ✅ Low        | —              | Low         |
| WirePlumber compatibility  | ✅ Best   | ✅ Good       | ⚠️ Limited    | ❌          |
| Latency potential          | ✅ Best   | ✅ Equal      | ❌             | ✅          |
| Portability (non-PipeWire) | ❌        | ⚠️ JACK only | ❌             | ✅          |
| MVP suitability            | ✅        | ✅            | ❌             | ❌          |

---

## Recommendation

**Phase 1:** Use **pipewire-jack** via the `jack` Rust crate with `pipewire-jack` bridge.

**Rationale:**
1. Stable, mature Rust API (`jack` crate) reduces implementation risk.
2. Functionally equivalent to pw-filter via the bridge — no meaningful latency penalty.
3. The `jack` crate's process callback model maps directly to our Mix Engine design.
4. Enables testing with JACK-aware tools (Carla, Catia, qjackctl) for debugging.
5. Migration path to pw-filter native API is straightforward if needed (same callback model).

**Phase 2+:** Evaluate migrating to native `pw-filter` API via `pipewire-rs` if:
- The JACK bridge introduces measurable latency overhead.
- WirePlumber policy integration requires native node properties.
- The `pipewire-rs` crate reaches stable API status.

---

## ADR Reference

This research feeds into **ADR-001** (PipeWire integration). ADR-001 should be updated to reflect the pipewire-jack Phase 1 decision.

---

## Rust Dependencies

```toml
# server/mix-engine/Cargo.toml (Phase 1 — when PipeWire available)
[dependencies]
jack = "0.13"          # pipewire-jack bridge
# OR for native pw-filter:
pipewire = "0.8"       # pipewire-rs (native, less stable API)
```

**VPS / CI note:** `jack` crate requires `libjack-dev` or `libjack-jackd2-dev`. On VPS without PipeWire/JACK, compile with `--no-default-features` or use feature flags to gate audio I/O from pure DSP logic. The Mix Engine core (gain, pan, mute, revision) must compile and test without any audio system dependency.

---

## Gap Status

- **GAP-003:** CLOSED — integration model selected (pipewire-jack for Phase 1).
- Pending: Hardware validation on Raspberry Pi 5 with PipeWire + pipewire-jack installed.

