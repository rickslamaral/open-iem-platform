# Phase 91 Review — Biquad Reference Validation

**Status:** PASS WITH CONDITIONS — coefficient validation is local-only; audio runtime and Raspberry Pi 5 remain unvalidated.

## Scope

Added deterministic reference vectors for the RBJ peaking-EQ coefficient calculation in `server/mix-engine/src/eq.rs`. Four combinations cover low, mid, high and near-Nyquist frequencies, positive and negative gain, and varied Q values. `scripts/validate_biquad_reference.py` independently recomputes same equations with Python stdlib `math`; it is executable without project dependencies.

## Acceptance criteria

- [x] Compare normalized `b0`, `b1`, `b2`, `a1`, and `a2`.
- [x] Use an explicit `5e-6` tolerance for `f32` rounding.
- [x] Keep reference values independent from production calculation at test runtime.
- [x] Preserve existing DSP behavior and public API.
- [ ] Validate real-time audio behavior on Raspberry Pi 5 hardware.

## Evidence

- `cargo fmt --all --manifest-path server/Cargo.toml`: PASS.
- `cargo test --manifest-path server/Cargo.toml -p mix-engine`: 80 unit tests and 3 doc-tests passed.
- Python baseline: `python3 -m pytest --tb=no -q`: 60 passed before edits.

## Limitations

Reference vectors validate coefficient arithmetic only. PipeWire, ALSA, audio latency, and Raspberry Pi 5 execution remain `SIMULATED` or pending physical validation.
