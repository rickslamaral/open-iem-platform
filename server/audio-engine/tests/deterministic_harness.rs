//! **SIMULATED** deterministic audio-engine integration harness.
//!
//! No hardware, network, JACK, `PipeWire`, or wall-clock values participate in
//! audio assertions. `cpu_us` is intentionally ignored.

use audio_engine::backend::{simulated::SimulatedBackend, Backend};
use mix_engine::{Channel, Limiter, Mix, MixEngine, MixSend};

const SAMPLE_RATE: u32 = 48_000;
const BUFFER_FRAMES: u32 = 64;
const FRAMES: u32 = 256;
const EPSILON: f32 = 1.0e-6;

fn configured_backend(mix_config: impl FnOnce(&mut Mix, &mut Mix)) -> SimulatedBackend {
    let mut engine = MixEngine::new();
    engine
        .set_channel(0, Channel::new(0, "SIMULATED input"))
        .expect("channel slot is valid");
    engine
        .set_channel(1, Channel::new(1, "SIMULATED input 2"))
        .expect("channel slot is valid");

    let mut mix0 = Mix::new(0, "SIMULATED mix 0");
    let mut mix1 = Mix::new(1, "SIMULATED mix 1");
    mix_config(&mut mix0, &mut mix1);
    engine.set_mix(0, mix0).expect("mix slot is valid");
    engine.set_mix(1, mix1).expect("mix slot is valid");

    SimulatedBackend::new(engine, SAMPLE_RATE, BUFFER_FRAMES)
}

fn run(config: impl FnOnce(&mut Mix, &mut Mix)) -> [Vec<(f32, f32)>; 2] {
    let mut backend = configured_backend(config);
    backend.activate().expect("SIMULATED backend activates");
    let stats = backend
        .process_n_frames(FRAMES)
        .expect("SIMULATED processing succeeds");
    assert_eq!(stats.frames, FRAMES);
    assert!(!stats.xrun);
    [
        backend.output_samples(0).expect("mix 0 output exists"),
        backend.output_samples(1).expect("mix 1 output exists"),
    ]
}

fn assert_samples_equal(left: &[(f32, f32)], right: &[(f32, f32)]) {
    assert_eq!(left.len(), right.len());
    for (index, ((ll, lr), (rl, rr))) in left.iter().zip(right).enumerate() {
        assert!((ll - rl).abs() <= EPSILON, "left sample differs at {index}");
        assert!(
            (lr - rr).abs() <= EPSILON,
            "right sample differs at {index}"
        );
    }
}

fn one_send(mix: &mut Mix, channel: usize) {
    mix.set_send(channel, MixSend::new(channel as u32, mix.id))
        .expect("send slot is valid");
}

#[test]
fn simulated_determinism_ignores_cpu_time() {
    let first = run(|mix0, mix1| {
        one_send(mix0, 0);
        one_send(mix1, 0);
    });
    let second = run(|mix0, mix1| {
        one_send(mix0, 0);
        one_send(mix1, 0);
    });
    assert_samples_equal(&first[0], &second[0]);
    assert_samples_equal(&first[1], &second[1]);
}

#[test]
fn simulated_mix_outputs_are_isolated() {
    let baseline = run(|mix0, mix1| {
        one_send(mix0, 0);
        one_send(mix1, 0);
    });
    let changed_mix0 = run(|mix0, mix1| {
        one_send(mix0, 0);
        mix0.set_send_gain_db(0, 6.0).expect("send exists");
        one_send(mix1, 0);
    });
    assert_samples_equal(&baseline[1], &changed_mix0[1]);
    assert!(changed_mix0[0]
        .iter()
        .zip(&baseline[0])
        .any(|((l, r), (bl, br))| (l - bl).abs() > EPSILON || (r - br).abs() > EPSILON));
}

#[test]
fn simulated_gain_pan_and_mute_behave_as_configured() {
    let unity = run(|mix0, mix1| {
        one_send(mix0, 0);
        one_send(mix1, 0);
    });
    let gain = run(|mix0, mix1| {
        one_send(mix0, 0);
        mix0.set_send_gain_db(0, -6.0).expect("send exists");
        one_send(mix1, 0);
    });
    let panned = run(|mix0, mix1| {
        one_send(mix0, 0);
        mix0.set_send_pan(0, -1.0).expect("send exists");
        one_send(mix1, 0);
    });
    let muted = run(|mix0, mix1| {
        one_send(mix0, 0);
        mix0.set_send_muted(0, true).expect("send exists");
        one_send(mix1, 0);
    });

    let index = 10;
    assert!((gain[0][index].0 / unity[0][index].0 - 10.0_f32.powf(-6.0 / 20.0)).abs() < 1.0e-5);
    assert!(panned[0].iter().all(|(_, right)| right.abs() <= EPSILON));
    assert!(muted[0]
        .iter()
        .all(|(left, right)| left.abs() <= EPSILON && right.abs() <= EPSILON));
}

#[test]
fn simulated_limiter_is_bounded_and_finite() {
    let output = run(|mix0, mix1| {
        one_send(mix0, 0);
        mix0.limiter = Limiter::new_enabled(-6.0);
        one_send(mix1, 0);
    });
    let threshold = 10.0_f32.powf(-6.0 / 20.0) + EPSILON;
    for samples in output {
        for (left, right) in samples {
            assert!(left.is_finite() && right.is_finite());
            assert!(left.abs() <= threshold && right.abs() <= threshold);
        }
    }
}
