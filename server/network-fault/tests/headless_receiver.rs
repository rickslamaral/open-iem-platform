use std::sync::Arc;

use network_fault::{
    DuplicateProfile, JitterProfile, LossProfile, Packet, ReconnectProfile, ReorderProfile,
};
use observability::ReceiverMetrics;
use streaming::{
    AudioOutput, MediaFrame, MediaWriter, OpusReceiver, OutputError, ReceiverState, StreamMetadata,
};

struct Capture {
    frames: Vec<Vec<f32>>,
    muted: usize,
}

impl AudioOutput for Capture {
    fn write(&mut self, samples: &[f32], channels: u8) -> Result<(), OutputError> {
        assert_eq!(channels, 2);
        assert_eq!(samples.len(), 1_920);
        self.frames.push(samples.to_vec());
        Ok(())
    }

    fn mute(&mut self) {
        self.muted += 1;
    }
}

#[test]
fn deterministic_loss_profile_drives_opus_receiver_plc() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let result = LossProfile::new(3).unwrap().apply(&encoded);
    assert_eq!(result.dropped, 2);
    assert_eq!(
        result
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 5, 7, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &result.delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 8);
    for (index, expected_left) in [(0, 0.1), (1, 0.2), (3, 0.4), (4, 0.5), (6, 0.7), (7, 0.8)] {
        let left_mean: f32 = output.frames[index].iter().step_by(2).sum::<f32>() / 960.0;
        assert!(
            (left_mean - expected_left).abs() < 0.08,
            "decoded frame at playout {index} has wrong source: {left_mean}"
        );
    }
    for index in [2, 5] {
        let plc_peak = output.frames[index]
            .iter()
            .map(|sample| sample.abs())
            .fold(0.0_f32, f32::max);
        assert!(plc_peak > 0.001, "PLC frame at playout {index} is silent");
    }
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.plc_consecutive_max, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn deterministic_outage_profile_drives_opus_receiver_plc_burst() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let result = network_fault::OutageProfile::new(3, 2)
        .unwrap()
        .apply(&encoded);
    assert_eq!(result.dropped, 2);
    assert_eq!(
        result
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 6, 7, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &result.delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 8);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.plc_consecutive_max, 2);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn deterministic_outage_profile_enforces_receiver_plc_burst_limit() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let result = network_fault::OutageProfile::new(2, 5)
        .unwrap()
        .apply(&encoded);
    assert_eq!(result.dropped, 5);
    assert_eq!(result.delivered.len(), 3);
    assert_eq!(
        result
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &result.delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    receiver.playout(&mut output).unwrap();
    receiver.playout(&mut output).unwrap();
    for _ in 0..4 {
        receiver.playout(&mut output).unwrap();
    }
    let failure = receiver.playout(&mut output);
    assert!(
        failure.is_err(),
        "fifth consecutive missing frame must fail closed"
    );

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 6);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Muted);
    assert_eq!(snapshot.packets_received, 3);
    assert_eq!(snapshot.plc_frames_total, 4);
    assert_eq!(snapshot.plc_consecutive_max, 4);
    assert_eq!(snapshot.output_failures, 1);

    let second_failure = receiver.playout(&mut output);
    assert!(second_failure.is_err());
    assert_eq!(metrics.snapshot().output_failures, 1);
    assert_eq!(output.muted, 2);
}

#[test]
fn deterministic_jitter_profile_drives_reordered_opus_receiver() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let result = JitterProfile::new(4, 2).unwrap().apply(&encoded);
    assert_eq!(result.dropped, 0);
    assert_eq!(result.jitter_events, 2);
    assert_eq!(result.delivered.len(), encoded.len());
    assert!(result.reordered > 0);
    assert_eq!(
        result
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 5, 6, 4, 7, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &result.delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 8);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 8);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
    assert_eq!(snapshot.late_packets, 0);
    for (index, expected_left) in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8].iter().enumerate() {
        let left_mean: f32 = output.frames[index].iter().step_by(2).sum::<f32>() / 960.0;
        assert!(
            (left_mean - expected_left).abs() < 0.08,
            "decoded frame at playout {index} has wrong source: {left_mean}"
        );
    }
}

#[test]
fn deterministic_reorder_profile_drives_ordered_opus_receiver() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let result = ReorderProfile::new(3).unwrap().apply(&encoded);
    assert_eq!(result.dropped, 0);
    assert_eq!(result.reordered, 4);
    assert_eq!(
        result
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 3, 5, 7, 6, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &result.delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 8);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 8);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.late_packets, 0);
    assert_eq!(snapshot.output_failures, 0);
    for (index, expected_left) in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8].iter().enumerate() {
        let left_mean: f32 = output.frames[index].iter().step_by(2).sum::<f32>() / 960.0;
        assert!(
            (left_mean - expected_left).abs() < 0.08,
            "decoded frame at playout {index} has wrong source: {left_mean}"
        );
    }
}

#[test]
fn deterministic_reconnect_profile_resumes_opus_receiver() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let result = ReconnectProfile::new(4, 1, "musician-1", 0)
        .unwrap()
        .apply(&encoded);
    assert_eq!(result.pre_disconnect.len(), 4);
    assert_eq!(result.post_reconnect.len(), 3);
    assert_eq!(result.lost_at_disconnect, 1);
    assert_eq!(result.recovered_mix_id, Some(0));

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &result.pre_disconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &result.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);
    for packet in &result.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &result.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 7);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 7);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
    for (index, expected_left) in [0.1, 0.2, 0.3, 0.4, 0.6, 0.7, 0.8].iter().enumerate() {
        let left_mean: f32 = output.frames[index].iter().step_by(2).sum::<f32>() / 960.0;
        assert!(
            (left_mean - expected_left).abs() < 0.08,
            "decoded frame at playout {index} has wrong source: {left_mean}"
        );
    }
}

#[test]
fn deterministic_duplicate_profile_classifies_duplicates_as_late() {
    // Encode 6 real Opus packets. DuplicateProfile(interval=2) replays every
    // 2nd packet. The receiver must:
    // - deliver all 6 original packets (packets_received == 6)
    // - classify each duplicate as late_packets (not packets_dropped)
    // - play out 6 frames without PLC or output failure
    // - stay in Playing state

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=6u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    // interval=2 → duplicates at positions 2,4,6 → 3 duplicates injected
    let with_dupes = DuplicateProfile::new(2).unwrap().apply(&encoded);
    assert_eq!(with_dupes.len(), 9, "6 originals + 3 duplicates");

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &with_dupes {
        // Enqueue may return DuplicateSequence for replays; that is expected.
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..6 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 6, "six frames played");
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6, "six unique packets received");
    assert_eq!(snapshot.late_packets, 3, "three duplicates counted as late");
    assert_eq!(snapshot.packets_dropped, 0, "no packets dropped");
    assert_eq!(snapshot.plc_frames_total, 0, "no PLC needed");
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

fn test_frame(sequence: u64) -> MediaFrame {
    MediaFrame {
        metadata: StreamMetadata {
            stream_id: "mix_0".into(),
            mix_index: 0,
            revision: 1,
            sequence,
            sample_rate: 48_000,
            channels: 2,
            frame_duration_ms: 20,
            capture_timestamp: None,
        },
        samples: (0.0, 0.0),
    }
}
