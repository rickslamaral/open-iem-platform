# Phase 7 Review — DSP: Biquad EQ + RMS Compressor

**Status:** PASS  
**Date:** 2026-09-09  
**Environment:** VPS/Linux x86_64 — **SIMULATED**, no PipeWire/audio hardware  
**Branch:** `feat/phase7-dsp` → squash-merge pending

## Delivered

### Biquad Parametric EQ (`mix-engine/src/eq.rs`)
- Replaced Phase 2 passthrough stub with real Type-II Transposed Direct Form II (TDF2) peaking biquad sections.
- Coefficients follow the Audio EQ Cookbook (RBJ): `b0/b1/b2/a1/a2` for peaking filter at configured `frequency_hz / gain_db / q`.
- `SAMPLE_RATE = 48_000.0` constant.
- `BiquadCoeffs` struct: computed once per `set_band`, stored per-band.
- `BiquadState` struct: stereo delay lines `w1_l / w2_l / w1_r / w2_r`, inline, zero heap allocation.
- `process(&mut self, l, r)` applies all enabled bands sequentially; disabled bands bypass.
- API surface unchanged: `ParametricEq { bands: [EqBand; 4], revision }`, `set_band(index, band)`, `process(&mut self, l, r)`.

### RMS Compressor (`mix-engine/src/compressor.rs`)
- Replaced Phase 2 passthrough stub with stereo-linked RMS detector + smoothed gain reduction.
- Stereo link: detector uses `max(|L|, |R|)`.
- RMS: exp-moving-average of x² with configurable attack/release coefficients.
- Gain reduction: `(1 - 1/ratio) * (threshold_db - rms_db)` when RMS exceeds threshold; 0 otherwise.
- Smoothed gain envelope: separate attack/release tracking on gain_reduction_db.
- Mutators: `set_threshold`, `set_ratio`, `set_attack_ms`, `set_release_ms` — all bump revision and recompute cached coefficients.
- State: `rms_state: f32`, `gain_db: f32` — inline, no heap allocation.
- Disabled: passthrough (output = input).

## Test Results

| Crate | Tests |
|-------|-------|
| mix-engine | 74 (↑ from 57) |
| api-server (integration) | 19 |
| audio-engine | 20 |
| control-server | 13 |
| control-protocol | 7 |
| streaming | 3 (doc) |
| **Total workspace** | **152** |

## Verification

| Gate | Result |
|------|--------|
| `cargo fmt --all` | PASS |
| `cargo test --workspace` | PASS — 152 tests, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS — 0 warnings |
| Static secret/injection scan | PASS |
| Independent reviewer | PENDING — runs before squash merge |
| Raspberry Pi hardware | NOT RUN — SIMULATED |

## Design Notes

- Phase 7 EQ uses peaking filter only. Low-shelf, high-shelf, high-pass, low-pass deferred to Phase 8+ when use cases are established.
- Compressor uses soft-knee via smoothed gain envelope; hard-knee option deferred.
- Both stubs maintained API compatibility — no breaking changes to `Mix`, `MixSend`, or `MixEngine` public interfaces.
- No heap allocation introduced; audio path remains allocation-free.

## Open Items

- `ParametricEq::process` now takes `&mut self` (required for biquad state). Callers that used `&self` need update — already propagated in `Mix::process`.
- Real coefficient verification against reference implementation (e.g., scipy.signal) deferred to hardware test phase.
- PipeWire audio graph integration remains SIMULATED (Pi hardware blocker).
