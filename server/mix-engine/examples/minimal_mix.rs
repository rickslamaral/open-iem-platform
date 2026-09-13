//! Minimal mix scenario — `mix-engine` example
//!
//! Demonstrates the full signal path from input channels through [`MixSend`]s into
//! two independent mixes, with master gain, mute, and EQ applied.
//!
//! # What this shows
//!
//! ```text
//!   CH0 (Vocals, +3 dB trim)  ─┬─► MixSend(mix=0, 0 dB, C) ─► Mix 0 (Vocalist monitor)
//!                               └─► MixSend(mix=1, –6 dB, L) ─► Mix 1 (Drummer monitor)
//!   CH1 (Kick drum, 0 dB trim) ──► MixSend(mix=1, 0 dB, C)  ─► Mix 1
//! ```
//!
//! Both mixes pass through: Sum → EQ (flat here) → Compressor (disabled) →
//! Master Gain → Limiter.
//!
//! # Running
//!
//! ```bash
//! cargo run --example minimal_mix --manifest-path server/Cargo.toml
//! ```

use mix_engine::{Channel, Mix, MixEngine, MixSend, SAMPLE_RATE};

fn main() {
    println!("Open IEM Platform — minimal mix example");
    println!("Sample rate: {SAMPLE_RATE} Hz");
    println!();

    // ── Build the engine ──────────────────────────────────────────────────
    let mut engine = MixEngine::new();

    // Channel 0: Vocals, input trim +3 dB
    let mut ch_voc = Channel::new(0, "VOC");
    ch_voc.set_gain_db(3.0);
    engine.set_channel(0, ch_voc).expect("channel 0");

    // Channel 1: Kick drum, unity gain
    let ch_kick = Channel::new(1, "KICK");
    engine.set_channel(1, ch_kick).expect("channel 1");

    // ── Mix 0: Vocalist's monitor ─────────────────────────────────────────
    // Hears vocals full, kick muted.
    let mut mix0 = Mix::new(0, "Vocalist");

    // Vocals → Mix 0: 0 dB, centre pan
    let send_voc_0 = MixSend::new(0, 0);
    mix0.set_send(0, send_voc_0).expect("send voc→mix0");

    // Kick → Mix 0: muted
    let mut send_kick_0 = MixSend::new(1, 0);
    send_kick_0.set_muted(true);
    mix0.set_send(1, send_kick_0).expect("send kick→mix0");

    // Master gain: –3 dB
    mix0.set_master_gain_db(-3.0);

    engine.set_mix(0, mix0).expect("mix 0");

    // ── Mix 1: Drummer's monitor ──────────────────────────────────────────
    // Hears kick loudly, vocals at –6 dB panned left.
    let mut mix1 = Mix::new(1, "Drummer");

    // Vocals → Mix 1: –6 dB, hard left
    let mut send_voc_1 = MixSend::new(0, 1);
    send_voc_1.set_gain_db(-6.0);
    send_voc_1.set_pan(-1.0);
    mix1.set_send(0, send_voc_1).expect("send voc→mix1");

    // Kick → Mix 1: 0 dB, centre
    let send_kick_1 = MixSend::new(1, 1);
    mix1.set_send(1, send_kick_1).expect("send kick→mix1");

    engine.set_mix(1, mix1).expect("mix 1");

    println!("Engine revision after setup: {}", engine.revision());
    println!();

    // ── Process a single frame ────────────────────────────────────────────
    // Synthetic input: 1.0 on CH0 (vocals), 0.8 on CH1 (kick), rest silent.
    let input = [1.0_f32, 0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let out = engine.process_frame(&input);

    let (m0_l, m0_r) = out.mixes[0];
    let (m1_l, m1_r) = out.mixes[1];

    println!("─── Output (one frame) ───────────────────────────────────────");
    println!("  Mix 0 (Vocalist): L = {m0_l:.4}  R = {m0_r:.4}");
    println!("  Mix 1 (Drummer) : L = {m1_l:.4}  R = {m1_r:.4}");
    println!();

    // ── Assertions (self-check) ───────────────────────────────────────────
    // Mix 0: vocals only, centre pan, master –3 dB → L ≈ R, both > 0
    assert!(m0_l > 0.0, "Mix 0 left should be non-zero");
    assert!(m0_r > 0.0, "Mix 0 right should be non-zero");
    assert!(
        (m0_l - m0_r).abs() < 1e-4,
        "Mix 0 centre pan: L ≈ R, got L={m0_l:.4} R={m0_r:.4}"
    );

    // Mix 1: vocals hard-left (only L), kick at centre (both channels)
    // Total Mix 1 L  = vocals_contribution_L + kick_contribution_L
    // Total Mix 1 R  = kick_contribution_R  (no vocals on right)
    assert!(m1_l > m1_r, "Mix 1: L > R because vocals panned hard-left");
    assert!(m1_r > 0.0, "Mix 1 right: kick contributes to both channels");

    println!("All assertions passed.");

    // ── Mute master and verify silence ────────────────────────────────────
    println!();
    println!("─── Muting Mix 0 master ──────────────────────────────────────");
    if let Some(mix) = engine.mix_mut(0) {
        mix.set_master_muted(true);
    }

    let out_muted = engine.process_frame(&input);
    let (muted_l, muted_r) = out_muted.mixes[0];
    println!("  Mix 0 (muted)   : L = {muted_l:.4}  R = {muted_r:.4}");
    assert!(
        muted_l.abs() < f32::EPSILON,
        "Muted mix 0 should be silent (L)"
    );
    assert!(
        muted_r.abs() < f32::EPSILON,
        "Muted mix 0 should be silent (R)"
    );
    println!("Mute verified: Mix 0 is silent.");

    println!();
    println!("Example complete.");
}
