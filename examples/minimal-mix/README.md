# Minimal Mix Scenario

Minimal working example of the Open IEM Platform mix engine. Demonstrates
two independent monitor mixes from two input channels (vocals + kick drum),
with per-send gain/pan/mute, master gain, and master mute.

## Signal graph

```
CH0 (Vocals, +3 dB trim) ─┬─► MixSend(mix=0, 0 dB, centre) ─► Mix 0 — Vocalist monitor
                           └─► MixSend(mix=1, –6 dB, L pan) ─► Mix 1 — Drummer monitor
CH1 (Kick drum, 0 dB)    ──► MixSend(mix=1, 0 dB, centre)  ─► Mix 1
                              MixSend(mix=0, muted)
```

Audio chain per mix: **Sum → EQ → Compressor → Master Gain → Limiter**

## Running

```bash
cargo run --example minimal_mix --manifest-path server/Cargo.toml
```

Expected output:

```
Open IEM Platform — minimal mix example
Sample rate: 48000 Hz

Engine revision after setup: 4

─── Output (one frame) ───────────────────────────────────────
  Mix 0 (Vocalist): L = 0.7071  R = 0.7071
  Mix 1 (Drummer) : L = 1.2736  R = 0.5657

All assertions passed.

─── Muting Mix 0 master ──────────────────────────────────────
  Mix 0 (muted)   : L = 0.0000  R = 0.0000
Mute verified: Mix 0 is silent.

Example complete.
```

## Source

`server/mix-engine/examples/minimal_mix.rs`

## Notes

- Audio processing is **SIMULATED** — no real hardware required.
- All DSP constants (sample rate, channel/mix limits) come from the `mix-engine` crate.
- The example is self-checking: it asserts expected signal-flow invariants and panics on failure.
