//! Clock+media pipeline integration: `SampleTimestamp` propagates through
//! `MediaPlane`, `MediaWriter`, and into `MediaPacket`.

use mix_engine::FrameOutput;
use streaming::{
    clock::{AdaptiveResampler, DriftEstimator, SampleTimestamp, NOMINAL_SAMPLE_RATE},
    MediaBridge, MediaFrame, MediaPlane, MediaWriter, StreamMetadata,
};

fn frame_output() -> FrameOutput {
    FrameOutput {
        mixes: [(0.1_f32, -0.1_f32), (0.2_f32, -0.2_f32)],
    }
}

#[test]
fn sample_timestamp_propagates_through_media_frame_to_packet() {
    let ts = SampleTimestamp::new(5, 240_000);
    let frame = MediaFrame {
        metadata: StreamMetadata {
            stream_id: "mix_0".into(),
            mix_index: 0,
            revision: 1,
            sequence: 0,
            sample_rate: 48_000,
            channels: 2,
            frame_duration_ms: 20,
            capture_timestamp: Some(ts),
        },
        samples: (0.1, -0.1),
    };
    let packet = MediaWriter::new().unwrap().encode(&frame).unwrap();
    assert_eq!(packet.capture_timestamp, Some(ts));
}

#[test]
fn none_capture_timestamp_propagates() {
    let frame = MediaFrame {
        metadata: StreamMetadata {
            stream_id: "mix_0".into(),
            mix_index: 0,
            revision: 1,
            sequence: 0,
            sample_rate: 48_000,
            channels: 2,
            frame_duration_ms: 20,
            capture_timestamp: None,
        },
        samples: (0.1, -0.1),
    };
    let packet = MediaWriter::new().unwrap().encode(&frame).unwrap();
    assert_eq!(packet.capture_timestamp, None);
}

#[tokio::test]
async fn bridge_carries_capture_timestamp_to_media_plane() {
    let bridge = MediaBridge::new();
    let plane = MediaPlane::new();
    plane.register_session("alice", 0).await.unwrap();

    let ts = SampleTimestamp::new(1, 48_000);
    bridge.try_send(frame_output(), 3, Some(ts)).unwrap();
    bridge.drain_to(&plane).await;

    let frames = plane
        .drain_session_frames_with_budget("alice", 1)
        .await
        .unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].metadata.capture_timestamp, Some(ts));
}

#[tokio::test]
async fn bridge_none_timestamp_does_not_block_frame() {
    let bridge = MediaBridge::new();
    let plane = MediaPlane::new();
    plane.register_session("alice", 0).await.unwrap();

    bridge.try_send(frame_output(), 7, None).unwrap();
    bridge.drain_to(&plane).await;

    let frames = plane
        .drain_session_frames_with_budget("alice", 1)
        .await
        .unwrap();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].metadata.capture_timestamp, None);
}

#[test]
#[allow(clippy::cast_sign_loss)]
fn drift_estimator_soak_10k_frames_stays_bounded() {
    const FRAMES: u64 = 10_000;
    const SAMPLES_PER_FRAME: u64 = 960;
    let drift_ppm_true = 100.0_f64;

    let mut estimator = DriftEstimator::new(NOMINAL_SAMPLE_RATE, 0.1);
    let mut resampler = AdaptiveResampler::new(256);
    let mut buffered = 256_usize;

    for i in 0..FRAMES {
        let remote = i * SAMPLES_PER_FRAME;
        let local = (remote as f64 * (1.0 + drift_ppm_true / 1_000_000.0)).max(0.0) as u64;
        let ppm = estimator.update(remote, local);
        let ratio = resampler.update(ppm, buffered);
        assert!(ppm.abs() <= 500.0, "ppm out of bounds at frame {i}: {ppm}");
        assert!(
            (0.9995..=1.0005).contains(&ratio),
            "ratio out of bounds at frame {i}: {ratio}"
        );
        if ratio > 1.0 {
            buffered = buffered.saturating_add(1);
        } else {
            buffered = buffered.saturating_sub(1);
        }
    }
    let final_ppm = estimator.estimate_ppm();
    assert!(final_ppm > 0.0, "estimator should detect positive drift");
}

#[test]
fn sample_timestamp_fields_preserved() {
    let ts = SampleTimestamp::new(42, 2_016_000);
    assert_eq!(ts.sequence, 42);
    assert_eq!(ts.sample, 2_016_000);
    let default_ts = SampleTimestamp::default();
    assert_eq!(default_ts.sequence, 0);
    assert_eq!(default_ts.sample, 0);
}
