use std::sync::Arc;

use network_fault::{
    CombinedFaultProfile, DuplicateProfile, JitterProfile, LossProfile, Packet, ReconnectProfile,
    ReorderProfile, Stage,
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

#[test]
fn deterministic_bandwidth_profile_drives_opus_receiver_plc() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let minimum_payload = encoded
        .iter()
        .map(|packet| packet.payload.len())
        .min()
        .unwrap();
    let profile = CombinedFaultProfile::new(vec![Stage::Bandwidth(
        network_fault::BandwidthProfile::new(minimum_payload, 2).unwrap(),
    )])
    .unwrap();
    let delivered = profile.apply(&encoded);
    assert!(!delivered.is_empty());
    assert!(delivered.len() < encoded.len());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), delivered.len());
    assert_eq!(snapshot.packets_received, delivered.len() as u64);
    assert_eq!(snapshot.output_failures, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
}

#[test]
fn combined_bandwidth_loss_drives_opus_receiver() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=10u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let budget = encoded[0].payload.len() + encoded[1].payload.len();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(budget, 4).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 6, 9],
        "bandwidth then loss must preserve deterministic stage composition"
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(delivered.len(), 4);
    assert_eq!(snapshot.packets_received, 4);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.plc_consecutive_max, 2);
    assert_eq!(snapshot.output_failures, 0);
    assert_eq!(output.muted, 0);
    assert_eq!(output.frames.len(), delivered.len());
    assert_eq!(receiver.state(), ReceiverState::Playing);
}

#[test]
fn deterministic_bandwidth_then_outage_drives_opus_receiver_plc() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let bandwidth_budget = encoded
        .iter()
        .take(11)
        .map(|packet| packet.payload.len())
        .sum();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(bandwidth_budget, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 2).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 6, 7, 8, 9, 10, 11]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..11 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 11);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 9);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.plc_consecutive_max, 2);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn deterministic_bandwidth_then_duplicate_classifies_receiver_late_packets() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }
    let bandwidth_budget = encoded
        .iter()
        .take(6)
        .map(|packet| packet.payload.len())
        .sum();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(bandwidth_budget, 8).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(2).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 2, 3, 4, 4, 5, 6, 6]
    );
    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..6 {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 6);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.late_packets, 3);
    assert_eq!(snapshot.packets_dropped, 0);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.plc_consecutive_max, 0);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn deterministic_bandwidth_outage_duplicate_drives_receiver_metrics() {
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=10u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let bandwidth_budget = encoded
        .iter()
        .take(8)
        .map(|packet| packet.payload.len())
        .sum();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(bandwidth_budget, 10).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(2, 2).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(2).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 2, 5, 6, 6, 7, 8, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
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
    assert_eq!(snapshot.late_packets, 3);
    assert_eq!(snapshot.packets_dropped, 0);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.plc_consecutive_max, 2);
    assert_eq!(snapshot.output_failures, 0);
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

#[test]
fn combined_loss_reorder_duplicate_headless_receiver() {
    // 12 Opus packets (seq 1..=12)
    // → LossProfile(4) drops 1-indexed positions 4,8,12 (seqs 4,8,12) → 9 survive: 1,2,3,5,6,7,9,10,11
    // → ReorderProfile(3) swaps 0-indexed pos 2 and 5 → seqs: 1,2,5,3,6,9,7,10,11
    // → DuplicateProfile(3) duplicates every 3rd (1-indexed pos 3,6,9) → 3 duplicates → 12 total
    // Receiver sees 12 packets; 9 unique seqs + 3 duplicates
    // Playout covers seq 1..=11 (seq 12 dropped): 9 real frames + 2 PLC (seqs 4,8 missing)
    // Expected: packets_received=9, late_packets=3, packets_dropped=0, plc_frames_total=2

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Loss(LossProfile::new(4).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();

    let result = combined.apply(&encoded);
    // 9 originals after loss + 3 duplicates = 12 total
    assert_eq!(result.len(), 12, "12 packets after combined pipeline");

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &result {
        // Duplicates return DuplicateSequence; ignore that error.
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // seq 1..=11 span (seq 12 dropped) → 11 playout calls: 9 real + 2 PLC (seqs 4,8)
    for _ in 0..11 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        11,
        "eleven frames played (9 real + 2 PLC)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 9, "nine unique packets received");
    assert_eq!(snapshot.late_packets, 3, "three duplicates counted as late");
    assert_eq!(snapshot.packets_dropped, 0, "no packets dropped");
    assert_eq!(
        snapshot.plc_frames_total, 2,
        "PLC for seqs 4 and 8 (dropped by loss)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_bandwidth_then_jitter_drives_opus_receiver() {
    // Phase 131
    // Generate 10 Opus packets (seq 1..=10)
    // BandwidthProfile budget = sum of first 7 payload lengths, window=10 → admits seqs 1-7, drops 8-10
    // JitterProfile(3,1) reorders but does not drop → 7 packets, all seqs 1-7 present
    // Receiver plays 7 frames with no gaps → no PLC

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=10u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let budget: usize = encoded[0..7].iter().map(|p| p.payload.len()).sum();

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(budget, 10).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    // Bandwidth admits 7, jitter reorders only → still 7
    assert_eq!(
        delivered.len(),
        7,
        "7 packets after bandwidth+jitter pipeline"
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        receiver.enqueue(pkt.sequence, &pkt.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..7 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 7, "seven frames played");
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(
        snapshot.packets_received, 7,
        "seven unique packets received"
    );
    assert_eq!(snapshot.plc_frames_total, 0, "no PLC: all seqs 1-7 present");
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_bandwidth_then_reorder_drives_opus_receiver() {
    // Phase 132
    // Generate 9 Opus packets (seq 1..=9)
    // BandwidthProfile budget = sum of first 6 payload lengths, window=9 → admits seqs 1-6, drops 7-9
    // ReorderProfile(3) swaps pos3↔4 (seqs 3,4 swap) → result [1,2,4,3,5,6]
    // Receiver plays 6 frames with no gaps → no PLC

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let budget: usize = encoded[0..6].iter().map(|p| p.payload.len()).sum();

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(budget, 9).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    // Bandwidth admits 6, reorder swaps but no drop → still 6
    assert_eq!(
        delivered.len(),
        6,
        "6 packets after bandwidth+reorder pipeline"
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        receiver.enqueue(pkt.sequence, &pkt.payload).unwrap();
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
    assert_eq!(snapshot.plc_frames_total, 0, "no PLC: all seqs 1-6 present");
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_jitter_then_loss_drives_opus_receiver_plc() {
    // Phase 133
    // Generate 9 Opus packets (seq 1..=9)
    // JitterProfile(3,1): delays pos3(seq3),pos6(seq6),pos9(seq9) by 1 slot each
    //   Jitter output: [seq1,seq2,seq4,seq3,seq5,seq7,seq6,seq8,seq9]
    // LossProfile(3): drops 1-indexed positions 3,6,9 → drops seq4, seq7, seq9
    //   Surviving: [seq1,seq2,seq3,seq5,seq6,seq8] (6 packets)
    // Gaps at seq4 and seq7 → 2 PLC frames
    // Playout 8 times (span seq1..seq8)

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    // Jitter keeps all 9, loss drops positions 3,6,9 → 6 survive
    assert_eq!(delivered.len(), 6, "6 packets after jitter+loss pipeline");

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        receiver.enqueue(pkt.sequence, &pkt.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // Span seq1..seq8 (seq9 dropped); 6 real + 2 PLC (seqs 4,7) = 8 playout calls
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        8,
        "eight frames played (6 real + 2 PLC)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6, "six unique packets received");
    assert_eq!(
        snapshot.plc_frames_total, 2,
        "PLC for seqs 4 and 7 (dropped by loss)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_bandwidth_loss_reorder_drives_opus_receiver_plc() {
    // Phase 164: bandwidth, loss and reorder receiver path.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }
    let bandwidth = network_fault::BandwidthProfile::new(12_000, 8).unwrap();
    let bandwidth_result = bandwidth.apply(&encoded);
    assert_eq!(
        bandwidth_result.delivered.len(),
        encoded.len(),
        "bandwidth admits all encoded packets"
    );
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(bandwidth),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Reorder(ReorderProfile::new(2).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(delivered.len(), 6);
    assert_eq!(
        delivered.iter().map(|p| p.sequence).collect::<Vec<_>>(),
        vec![1, 4, 2, 7, 5, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
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
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_bandwidth_loss_duplicate_drives_opus_receiver_late_packets() {
    // Phase 165: bandwidth, loss and duplicate receiver path.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(12_000, 8).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(2).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(delivered.len(), 9);
    assert_eq!(
        delivered.iter().map(|p| p.sequence).collect::<Vec<_>>(),
        vec![1, 2, 2, 4, 5, 5, 7, 8, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
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
    assert_eq!(snapshot.late_packets, 3);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_bandwidth_loss_jitter_drives_opus_receiver_plc() {
    // Phase 166: bandwidth, loss and jitter receiver path.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let budget = encoded
        .iter()
        .take(9)
        .map(|packet| packet.payload.len())
        .sum();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(budget, 9).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 5, 4, 7, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
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
    assert_eq!(snapshot.plc_consecutive_max, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_bandwidth_loss_outage_drives_opus_receiver_plc() {
    // Phase 167: bandwidth, loss and outage receiver path.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }
    let budget = encoded.iter().map(|packet| packet.payload.len()).sum();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(budget, 9).unwrap()),
        Stage::Loss(LossProfile::new(4).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 2).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 7, 9]
    );
    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..9 {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 9);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 5);
    assert_eq!(snapshot.plc_frames_total, 4);
    assert_eq!(snapshot.plc_consecutive_max, 3);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_bandwidth_outage_jitter_drives_opus_receiver_plc() {
    // Phase 168: bandwidth, outage and jitter receiver path.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let bandwidth_budget = encoded[..9].iter().map(|packet| packet.payload.len()).sum();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(bandwidth_budget, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 2).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 6, 3, 7, 9, 8]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..7 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 7);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 7);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.plc_consecutive_max, 2);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_bandwidth_jitter_duplicate_drives_opus_receiver_late_packets() {
    // Phase 170: bandwidth, jitter and duplicate receiver path.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 10.0, -(sequence as f32) / 10.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }
    let bandwidth_budget = encoded[..6].iter().map(|packet| packet.payload.len()).sum();
    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(bandwidth_budget, 9).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(2).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 2, 4, 3, 3, 5, 6, 6]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..6 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 6);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.late_packets, 3);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
}
