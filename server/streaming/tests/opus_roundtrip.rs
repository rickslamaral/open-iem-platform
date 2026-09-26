use std::sync::Arc;

use mix_engine::FrameOutput;

use observability::ReceiverMetrics;
use streaming::{
    AudioOutput, MediaBridge, MediaFrame, MediaPlane, MediaWriter, OpusReceiver, OutputError,
    ReceiverError, ReceiverState, StreamMetadata, OPUS_MAX_PACKET_BYTES,
};

#[tokio::test]
async fn media_bridge_plane_writer_receiver_round_trip_preserves_frame() {
    let bridge = MediaBridge::new();
    let plane = MediaPlane::new();
    plane.register_session("alice", 0).await.unwrap();
    bridge
        .try_send(
            FrameOutput {
                mixes: [(0.2, -0.1), (0.0, 0.0)],
            },
            7,
            None,
        )
        .unwrap();
    bridge.drain_to(&plane).await;

    let frame = plane
        .drain_session_frames_with_budget("alice", 1)
        .await
        .unwrap()
        .pop()
        .unwrap();
    let packet = MediaWriter::new().unwrap().encode(&frame).unwrap();
    let mut receiver = OpusReceiver::new().unwrap();
    receiver.enqueue(packet.sequence, &packet.payload).unwrap();

    let mut output = Capture {
        samples: Vec::new(),
        muted: 0,
    };
    receiver.playout(&mut output).unwrap();

    assert_eq!(frame.metadata.sequence, 0);
    assert_eq!(frame.metadata.revision, 7);
    assert_eq!(output.samples.len(), 1_920);
    assert_eq!(output.muted, 0);
    assert!(
        output
            .samples
            .iter()
            .step_by(2)
            .map(|sample| sample.abs())
            .sum::<f32>()
            > 1.0
    );
    assert!(
        output
            .samples
            .iter()
            .skip(1)
            .step_by(2)
            .map(|sample| sample.abs())
            .sum::<f32>()
            > 1.0
    );
}

struct Capture {
    samples: Vec<f32>,
    muted: usize,
}

impl AudioOutput for Capture {
    fn write(&mut self, samples: &[f32], channels: u8) -> Result<(), OutputError> {
        assert_eq!(channels, 2);
        self.samples.extend_from_slice(samples);
        Ok(())
    }

    fn mute(&mut self) {
        self.muted += 1;
    }
}

#[tokio::test]
async fn media_bridge_plane_writer_receiver_preserves_multi_frame_order_and_metadata() {
    let bridge = MediaBridge::new();
    let plane = MediaPlane::new();
    plane.register_session("alice", 0).await.unwrap();

    for (revision, left, right) in [(17_u64, 0.1_f32, -0.1_f32), (18, 0.3, -0.3)] {
        bridge
            .try_send(
                FrameOutput {
                    mixes: [(left, right), (0.0, 0.0)],
                },
                revision,
                None,
            )
            .unwrap();
    }
    bridge.drain_to(&plane).await;

    let frames = plane
        .drain_session_frames_with_budget("alice", 2)
        .await
        .unwrap();
    assert_eq!(frames.len(), 2);
    assert_eq!(
        frames
            .iter()
            .map(|frame| (frame.metadata.sequence, frame.metadata.revision))
            .collect::<Vec<_>>(),
        vec![(0, 17), (1, 18)]
    );

    let mut writer = MediaWriter::new().unwrap();
    let packets = frames
        .iter()
        .map(|frame| writer.encode(frame).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(packets[0].sequence, 0);
    assert_eq!(packets[1].sequence, 1);
    assert_eq!(packets[0].revision, 17);
    assert_eq!(packets[1].revision, 18);
    assert_eq!(
        packets[1]
            .rtp_timestamp
            .wrapping_sub(packets[0].rtp_timestamp),
        960
    );

    let mut receiver = OpusReceiver::new().unwrap();
    receiver
        .enqueue(packets[1].sequence, &packets[1].payload)
        .unwrap();
    receiver
        .enqueue(packets[0].sequence, &packets[0].payload)
        .unwrap();
    let mut output = Capture {
        samples: Vec::new(),
        muted: 0,
    };
    receiver.playout(&mut output).unwrap();
    receiver.playout(&mut output).unwrap();

    assert_eq!(output.samples.len(), 3_840);
    assert_eq!(output.muted, 0);
    let first_left_mean: f32 = output.samples[..1_920].iter().step_by(2).sum::<f32>() / 960.0;
    let second_left_mean: f32 = output.samples[1_920..].iter().step_by(2).sum::<f32>() / 960.0;
    assert!((first_left_mean - 0.1).abs() < 0.08);
    assert!((second_left_mean - 0.3).abs() < 0.08);
}

#[test]
fn deterministic_frame_survives_opus_writer_receiver_round_trip() {
    let frame = MediaFrame {
        metadata: StreamMetadata {
            stream_id: "mix_0".into(),
            mix_index: 0,
            revision: 7,
            sequence: 41,
            sample_rate: 48_000,
            channels: 2,
            frame_duration_ms: 20,
            capture_timestamp: None,
        },
        samples: (0.2, -0.1),
    };
    let packet = MediaWriter::new().unwrap().encode(&frame).unwrap();
    assert_eq!(packet.sequence, 41);
    assert_eq!(packet.rtp_timestamp, 0);

    let mut receiver = OpusReceiver::new().unwrap();
    receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    let mut output = Capture {
        samples: Vec::new(),
        muted: 0,
    };
    receiver.playout(&mut output).unwrap();

    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(output.samples.len(), 1_920);
    assert_eq!(output.muted, 0);
    let left_energy: f32 = output
        .samples
        .iter()
        .step_by(2)
        .map(|sample| sample.abs())
        .sum();
    let right_energy: f32 = output
        .samples
        .iter()
        .skip(1)
        .step_by(2)
        .map(|sample| sample.abs())
        .sum();
    assert!(left_energy > 1.0, "decoded left channel is silent");
    assert!(right_energy > 1.0, "decoded right channel is silent");
    let left_mean: f32 = output.samples.iter().step_by(2).sum::<f32>() / 960.0;
    let right_mean: f32 = output.samples.iter().skip(1).step_by(2).sum::<f32>() / 960.0;
    assert!(
        (left_mean - 0.2).abs() < 0.08,
        "left channel corrupted: {left_mean}"
    );
    assert!(
        (right_mean + 0.1).abs() < 0.08,
        "right channel corrupted: {right_mean}"
    );
}

#[test]
fn consecutive_opus_packets_preserve_order_and_frame_timestamps() {
    let mut writer = MediaWriter::new().unwrap();
    let mut receiver = OpusReceiver::new().unwrap();
    let mut packets = Vec::new();
    for (sequence, samples) in [(10_u64, (0.1, -0.1)), (11, (0.3, -0.3))] {
        let mut frame = test_frame();
        frame.metadata.sequence = sequence;
        frame.samples = samples;
        let packet = writer.encode(&frame).unwrap();
        assert_eq!(packet.sequence, sequence);
        packets.push(packet);
    }
    assert_eq!(packets[0].rtp_timestamp, 0);
    assert_eq!(
        packets[1]
            .rtp_timestamp
            .wrapping_sub(packets[0].rtp_timestamp),
        960
    );
    // Both packets enter ingress before first playout, so receiver establishes
    // expected sequence from sorted jitter-buffer head, not arrival order.
    // Jitter buffer must restore sequence order, not arrival order.
    receiver
        .enqueue(packets[1].sequence, &packets[1].payload)
        .unwrap();
    receiver
        .enqueue(packets[0].sequence, &packets[0].payload)
        .unwrap();
    let mut output = Capture {
        samples: Vec::new(),
        muted: 0,
    };
    receiver.playout(&mut output).unwrap();
    receiver.playout(&mut output).unwrap();
    assert_eq!(output.samples.len(), 3_840);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    let first_left_mean: f32 = output.samples[..1_920].iter().step_by(2).sum::<f32>() / 960.0;
    let second_left_mean: f32 = output.samples[1_920..].iter().step_by(2).sum::<f32>() / 960.0;
    assert!((first_left_mean - 0.1).abs() < 0.08);
    assert!((second_left_mean - 0.3).abs() < 0.08);
}

#[test]
fn receiver_metrics_follow_roundtrip_drop_and_reconnect() {
    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    let packet = MediaWriter::new().unwrap().encode(&test_frame()).unwrap();
    receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    receiver.enqueue(packet.sequence + 1, &[]).unwrap_err();

    let mut output = Capture {
        samples: Vec::new(),
        muted: 0,
    };
    receiver.playout(&mut output).unwrap();
    receiver.reconnect(&mut output);

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.packets_received, 1);
    assert_eq!(snapshot.packets_dropped, 1);
    assert_eq!(snapshot.reconnect_count, 1);
}

fn test_frame() -> MediaFrame {
    MediaFrame {
        metadata: StreamMetadata {
            stream_id: "mix_0".into(),
            mix_index: 0,
            revision: 7,
            sequence: 41,
            sample_rate: 48_000,
            channels: 2,
            frame_duration_ms: 20,
            capture_timestamp: None,
        },
        samples: (0.2, -0.1),
    }
}

#[test]
fn writer_and_receiver_share_opus_packet_limit() {
    let packet = MediaWriter::new().unwrap().encode(&test_frame()).unwrap();
    assert!(packet.payload.len() <= OPUS_MAX_PACKET_BYTES);

    let receiver = OpusReceiver::new().unwrap();
    let at_limit = vec![0_u8; OPUS_MAX_PACKET_BYTES];
    assert_eq!(receiver.enqueue(1, &at_limit), Ok(()));
    let oversized = vec![0_u8; OPUS_MAX_PACKET_BYTES + 1];
    assert_eq!(
        receiver.enqueue(2, &oversized),
        Err(ReceiverError::InvalidPacket)
    );
}
