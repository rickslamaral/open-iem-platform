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
fn reconnect_after_jitter_resumes_opus_receiver() {
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

    let jittered = JitterProfile::new(3, 1).unwrap().apply(&encoded);
    assert_eq!(jittered.delivered.len(), encoded.len());
    assert!(jittered.reordered > 0);
    assert_eq!(
        jittered
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 3, 5, 7, 6, 8],
        "reconnect input must retain jittered sequence order",
    );

    let reconnect = ReconnectProfile::new(4, 1, "musician-jitter", 0)
        .unwrap()
        .apply(&jittered.delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 4);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert_eq!(reconnect.post_reconnect.len(), 3);
    assert_eq!(reconnect.recovered_mix_id, Some(0));
    assert_eq!(
        reconnect
            .pre_disconnect
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 3],
    );
    assert_eq!(
        reconnect
            .post_reconnect
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![7, 6, 8],
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &reconnect.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &reconnect.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 7);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 7);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
    let means: Vec<f32> = output
        .frames
        .iter()
        .map(|frame| frame.iter().step_by(2).sum::<f32>() / 960.0)
        .collect();
    for (mean, expected) in means.iter().zip([0.1, 0.2, 0.3, 0.4, 0.6, 0.7, 0.8]) {
        assert!(
            (mean - expected).abs() < 0.08,
            "unexpected playout source: {mean}"
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
fn reconnect_after_loss_resumes_opus_receiver() {
    // Phase 144: loss before reconnect, then bounded post-reconnect playout.
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

    let loss = LossProfile::new(3).unwrap().apply(&encoded);
    assert_eq!(loss.dropped, 3);
    assert_eq!(
        loss.delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 5, 7, 8, 10],
    );

    let reconnect = ReconnectProfile::new(3, 1, "musician-loss", 2)
        .unwrap()
        .apply(&loss.delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 3);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert_eq!(reconnect.post_reconnect.len(), 3);
    assert_eq!(reconnect.recovered_mix_id, Some(2));
    assert_eq!(
        reconnect
            .pre_disconnect
            .iter()
            .chain(reconnect.post_reconnect.iter())
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 7, 8, 10],
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &reconnect.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &reconnect.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 6);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 3);
    assert_eq!(snapshot.output_failures, 0);
    assert!(output
        .frames
        .iter()
        .all(|frame| frame.iter().any(|sample| sample.abs() > 0.001)));
}

#[test]
fn reconnect_after_bandwidth_resumes_opus_receiver() {
    // Phase 145: bandwidth admission before bounded reconnect recovery.
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

    let bandwidth = network_fault::BandwidthProfile::new(5_000, 10)
        .unwrap()
        .apply(&encoded);
    assert_eq!(bandwidth.delivered.len(), 10);
    assert_eq!(bandwidth.dropped, 0);
    assert_eq!(
        bandwidth
            .delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        (1..=10).collect::<Vec<_>>()
    );

    let reconnect = ReconnectProfile::new(4, 2, "musician-bandwidth", 3)
        .unwrap()
        .apply(&bandwidth.delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 4);
    assert_eq!(reconnect.lost_at_disconnect, 2);
    assert_eq!(reconnect.post_reconnect.len(), 4);
    assert_eq!(reconnect.recovered_mix_id, Some(3));

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &reconnect.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &reconnect.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 8);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 8);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
    assert!(output
        .frames
        .iter()
        .all(|frame| frame.iter().any(|sample| sample.abs() > 0.001)));
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
fn combined_outage_then_jitter_drives_opus_receiver_plc() {
    // Phase 134
    // Generate 10 Opus packets (seq 1..=10, 0-indexed 0..9)
    // OutageProfile(start=3, window=3): drops 0-indexed positions 3,4,5 → seqs 4,5,6 lost
    //   Survivors (in order): [seq1, seq2, seq3, seq7, seq8, seq9, seq10] — 7 packets
    // JitterProfile(interval=3, delay_slots=1): delays 1-indexed positions 3 and 6
    //   1-indexed pos3 = seq3, pos6 = seq9 → each moved back 1 slot
    //   Delivered order: [seq1, seq2, seq7, seq3, seq8, seq10, seq9]
    // Receiver enqueues in that order; JitterBuffer reorders by sequence number.
    // Span seq1..seq10 → playout calls = 10 (span of received seqs + 3 PLC for seqs 4,5,6)
    // PLC = 3 (gaps at seqs 4, 5, 6)

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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    // Outage drops 3, jitter reorders but keeps all 7 survivors
    assert_eq!(
        delivered.len(),
        7,
        "7 packets survive outage; jitter reorders but does not drop"
    );
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 7, 3, 8, 10, 9],
        "outage+jitter must preserve deterministic survivor order",
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
    // Span seq1..seq10: 7 real + 3 PLC (seqs 4,5,6) = 10 playout calls
    for _ in 0..10 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        10,
        "ten frames played (7 real + 3 PLC)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(
        snapshot.packets_received, 7,
        "seven unique packets received"
    );
    assert_eq!(
        snapshot.plc_frames_total, 3,
        "PLC for seqs 4, 5 and 6 (outage window)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn reconnect_after_combined_outage_jitter_resumes_opus_receiver() {
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
    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered.iter().map(|p| p.sequence).collect::<Vec<_>>(),
        vec![1, 2, 7, 3, 8, 10, 9]
    );
    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered[..2] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..2 {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &delivered[2..] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 10);
    // Frames 0,1 decoded pre-disconnect (seqs 1,2). Frame 2 decoded post-reconnect (seq 3).
    // Frames 3,4,5 are PLC-concealed (seqs 4,5,6 were lost in outage). Frames 6..9 decoded (seqs 7,8,9,10).
    // Only assert amplitude for real (non-PLC) decoded frames.
    // Frame 6 (seq 7) is the first real frame after 3 consecutive PLC frames;
    // the codec state is in recovery, so amplitude is lower than nominal — skip it.
    let real_frames: &[(usize, f32)] =
        &[(0, 0.1), (1, 0.2), (2, 0.3), (7, 0.8), (8, 0.9), (9, 1.0)];
    for &(index, expected_left) in real_frames {
        let left_mean: f32 = output.frames[index].iter().step_by(2).sum::<f32>() / 960.0;
        assert!(
            (left_mean - expected_left).abs() < 0.08,
            "decoded frame at playout {index} has wrong source: {left_mean} (expected ~{expected_left})"
        );
    }
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 7);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 3);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_outage_then_loss_drives_opus_receiver_plc() {
    // Phase 135
    // Generate 10 Opus packets (seq 1..=10, 0-indexed 0..9)
    // OutageProfile(start=2, window=3): drops 0-indexed positions 2,3,4 -> seqs 3,4,5 lost
    //   Survivors (in order): [seq1, seq2, seq6, seq7, seq8, seq9, seq10] -- 7 packets
    // LossProfile(4) on 7 survivors: drops every 4th (1-indexed pos 4 = seq7)
    //   After loss: [seq1, seq2, seq6, seq8, seq9, seq10] -- 6 packets
    // Receiver gaps: seqs 3,4,5 (outage) and seq7 (loss) -> 4 PLC frames
    // Span seq1..seq10: 6 real + 4 PLC = 10 playout calls

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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(2, 3).unwrap()),
        Stage::Loss(LossProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    // 7 survive outage; loss drops 1 of 7 -> 6 remain
    assert_eq!(delivered.len(), 6, "6 packets survive outage+loss pipeline");

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
    // Span seq1..seq10: 6 real + 4 PLC (seqs 3,4,5,7) = 10 playout calls
    for _ in 0..10 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        10,
        "ten frames played (6 real + 4 PLC)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6, "six unique packets received");
    assert_eq!(
        snapshot.plc_frames_total, 4,
        "PLC for seqs 3,4,5 (outage) and seq7 (loss)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_outage_then_duplicate_classifies_receiver_late_packets() {
    // Phase 136
    // Generate 10 Opus packets (seq 1..=10, 0-indexed 0..9)
    // OutageProfile(start=1, window=2): drops 0-indexed positions 1,2 -> seqs 2,3 lost
    //   Survivors (in order): [seq1, seq4, seq5, seq6, seq7, seq8, seq9, seq10] -- 8 packets
    // DuplicateProfile(4) on 8 survivors: duplicates every 4th (1-indexed pos 4 = seq7, pos 8 = seq10)
    //   Delivered: [seq1, seq4, seq5, seq6, seq7, seq7_dup, seq8, seq9, seq10, seq10_dup] -- 10 packets
    // Receiver: duplicates return DuplicateSequence -> late_packets += 2
    // PLC: seqs 2,3 (outage) -> 2 PLC frames
    // Span seq1..seq10: 8 real + 2 PLC = 10 playout calls

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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(1, 2).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    // 8 survive outage; duplicate adds 2 copies -> 10 total
    assert_eq!(
        delivered.len(),
        10,
        "10 packets after outage+duplicate pipeline (8 originals + 2 duplicates)"
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        // Duplicates return DuplicateSequence; count them as late.
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // Span seq1..seq10: 8 real + 2 PLC (seqs 2,3) = 10 playout calls
    for _ in 0..10 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        10,
        "ten frames played (8 real + 2 PLC)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(
        snapshot.packets_received, 8,
        "eight unique packets received"
    );
    assert_eq!(
        snapshot.late_packets, 2,
        "two duplicate packets counted as late"
    );
    assert_eq!(
        snapshot.plc_frames_total, 2,
        "PLC for seqs 2 and 3 (outage window)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_outage_then_reorder_drives_opus_receiver() {
    // Phase 137
    // 10 packets seq 1..=10
    // OutageProfile(start=3, window=2): drops 0-indexed positions 3,4 -> seqs 4,5 lost
    //   Survivors: [seq1,seq2,seq3,seq6,seq7,seq8,seq9,seq10] -- 8 packets
    // ReorderProfile(3) on 8 survivors: swaps pos2<->3 (seq3<->seq6) and pos5<->6 (seq8<->seq9)
    //   Delivered: [seq1,seq2,seq6,seq3,seq7,seq9,seq8,seq10]
    // Receiver: seq3 arrives after seq6 — jitter buffer reorders -> no PLC for seq3 (within window)
    //   PLC: seqs 4,5 (outage) -> 2 PLC frames
    // Span seq1..seq10: 8 real + 2 PLC = 10 playout calls
    // Assert: delivered.len()==8, output.frames.len()==10, output.muted==0, state==Playing,
    //         packets_received==8, plc_frames_total==2, output_failures==0

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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 2).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered.len(),
        8,
        "8 packets survive outage+reorder pipeline"
    );
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 6, 3, 7, 9, 8, 10],
        "outage+reorder must preserve deterministic survivor order",
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // Span seq1..seq10: 8 real + 2 PLC (seqs 4,5 from outage) = 10 playout calls
    for _ in 0..10 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        10,
        "ten frames played (8 real + 2 PLC)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(
        snapshot.packets_received, 8,
        "eight unique packets received"
    );
    assert_eq!(
        snapshot.plc_frames_total, 2,
        "PLC for seqs 4 and 5 (outage window)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn reconnect_after_combined_outage_reorder_resumes_opus_receiver() {
    // Phase 150: reconnect after composed outage and reorder.
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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 2).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 6, 3, 7, 9, 8, 10],
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered[..2] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..2 {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);

    for packet in &delivered[2..] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 10);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 8);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_jitter_then_reorder_drives_opus_receiver() {
    // Phase 138
    // 8 packets seq 1..=8
    // JitterProfile::new(20, 3).unwrap() reorders but delivers all 8 packets
    // ReorderProfile(4) on 8 delivered: swaps pos3<->4 (additional reorder)
    // All 8 packets delivered (reordered only, no drops)
    // Receiver handles reordered packets via jitter buffer
    // Assert: delivered.len()==8, output.frames.len()==8, output.muted==0,
    //         state==Playing, packets_received==8, plc_frames_total==0, output_failures==0

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 8.0, -(sequence as f32) / 8.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Jitter(JitterProfile::new(20, 3).unwrap()),
        Stage::Reorder(ReorderProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered.len(),
        8,
        "8 packets after jitter+reorder (no drops)"
    );
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 5, 4, 6, 7, 8],
        "jitter+reorder must execute both deterministic reorder stages",
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // All 8 packets delivered (reordered only), jitter buffer handles reorder
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        8,
        "eight frames played (all real, no PLC)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(
        snapshot.packets_received, 8,
        "eight unique packets received"
    );
    assert_eq!(
        snapshot.plc_frames_total, 0,
        "no PLC frames (all packets delivered)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_jitter_then_duplicate_drives_opus_receiver() {
    // Phase 139
    // 8 packets seq 1..=8
    // JitterProfile::new(20, 3).unwrap() reorders but delivers all 8 packets
    // DuplicateProfile(4) on 8 delivered: inserts copies at pos4 and pos8 -> 10 total
    // Receiver: 8 unique + 2 duplicates (late_packets += 2)
    // All real packets delivered (no PLC gaps)
    // Assert: delivered.len()==10, output.frames.len()==8, output.muted==0,
    //         state==Playing, packets_received==8, late_packets==2, plc_frames_total==0, output_failures==0
    // Note: playout calls == 8 (not 10) because duplicates are rejected at enqueue

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 8.0, -(sequence as f32) / 8.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Jitter(JitterProfile::new(20, 3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    // 8 originals + 2 duplicates (after pos4 and pos8) = 10
    assert_eq!(
        delivered.len(),
        10,
        "10 packets after jitter+duplicate pipeline (8 originals + 2 duplicates)"
    );
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 4, 5, 6, 7, 8, 8],
        "jitter+duplicate must preserve deterministic order and duplicate positions",
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        // Duplicates return DuplicateSequence; count them as late.
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // playout calls == 8 (duplicates rejected at enqueue, no PLC needed)
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        8,
        "eight frames played (8 real, duplicates rejected)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(
        snapshot.packets_received, 8,
        "eight unique packets received"
    );
    assert_eq!(
        snapshot.late_packets, 2,
        "two duplicate packets counted as late"
    );
    assert_eq!(
        snapshot.plc_frames_total, 0,
        "no PLC frames (all unique packets delivered)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_loss_then_reorder_drives_opus_receiver() {
    // Phase 140
    // 9 packets seq 1..=9
    // LossProfile(3): drops positions 3,6,9 (1-indexed) -> seqs 3,6,9 lost; 6 survivors [seq1,seq2,seq4,seq5,seq7,seq8]
    // ReorderProfile(3) on 6 survivors: swaps 0-indexed pos2<->3 (seq4<->seq5); pos5+1=6 OOB, no swap there.
    //   Delivered: [seq1,seq2,seq5,seq4,seq7,seq8]
    // Receiver: seq4 arrives after seq5 -> jitter buffer reorders -> no PLC for seq4
    //   PLC: seqs 3,6,9 (loss) -> 3 PLC frames
    // Span seq1..seq9: 6 real + 3 PLC = 9 playout calls
    // Assert: delivered.len()==6, output.frames.len()==9, packets_received==6, plc_frames_total==3, output_failures==0

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 9.0, -(sequence as f32) / 9.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert_eq!(delivered.len(), 6, "6 survivors after loss(3) on 9 packets");

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // 6 real + 2 PLC (seq3,seq6) = 8 playout calls; seq9 is trailing loss, no PLC
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        8,
        "eight frames played (6 real + 2 PLC for seq3,seq6)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6, "six unique packets received");
    assert_eq!(
        snapshot.plc_frames_total, 2,
        "two PLC frames for lost seqs 3,6 (seq9 trailing, no PLC)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_loss_then_duplicate_classifies_receiver_late_packets() {
    // Phase 141
    // 9 packets seq 1..=9
    // LossProfile(3): drops positions 3,6,9 -> seqs 3,6,9 lost; 6 survivors [seq1,seq2,seq4,seq5,seq7,seq8]
    // DuplicateProfile(3) on 6 survivors: inserts copy after pos3 (seq4) and pos6 (seq8) -> 8 total
    //   Delivered: [seq1,seq2,seq4,seq4_dup,seq5,seq7,seq8,seq8_dup]
    // Receiver: 6 unique + 2 duplicates (late_packets+=2)
    // PLC: seqs 3,6,9 (loss) -> 3 PLC frames
    // Span seq1..seq9: 6 real + 3 PLC = 9 playout calls
    // Assert: delivered.len()==8, output.frames.len()==9, packets_received==6, late_packets==2, plc_frames_total==3, output_failures==0

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 9.0, -(sequence as f32) / 9.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered.len(),
        8,
        "8 packets after loss(3)+duplicate(3) on 9 packets (6 survivors + 2 dups)"
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        // Duplicates return DuplicateSequence; count them as late.
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // 6 real + 2 PLC (seq3,seq6) = 8 playout calls; seq9 is trailing loss, no PLC
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        8,
        "eight frames played (6 real + 2 PLC for seq3,seq6)"
    );
    assert_eq!(output.muted, 0, "no muted frames");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6, "six unique packets received");
    assert_eq!(
        snapshot.late_packets, 2,
        "two duplicate packets counted as late"
    );
    assert_eq!(
        snapshot.plc_frames_total, 2,
        "two PLC frames for lost seqs 3,6 (seq9 trailing, no PLC)"
    );
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn combined_outage_loss_duplicate_drives_opus_receiver() {
    // Phase 146: compose a contiguous outage, periodic loss and duplicates.
    // Outage removes seq4..=6; Loss(3) then removes seq6 and seq9 from the
    // nine survivors; Duplicate(3) duplicates seq7 and seq11.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 7, 7, 8, 10, 11, 11],
        "outage, loss and duplicate stages must preserve deterministic composition",
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..11 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        11,
        "six real frames plus five PLC frames"
    );
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.late_packets, 2);
    assert_eq!(snapshot.plc_frames_total, 5);
    assert_eq!(snapshot.plc_consecutive_max, 4);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_outage_loss_resumes_opus_receiver() {
    // Phase 147: reconnect after composed outage and periodic loss.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 7, 8, 10, 11],
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered[..2] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..2 {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);

    for packet in &delivered[2..] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in 0..4 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 6);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_outage_duplicate_resumes_opus_receiver() {
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
    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(1, 2).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(4).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 4, 5, 6, 6, 7, 8, 9, 10, 10]
    );
    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    receiver
        .enqueue(delivered[0].sequence, &delivered[0].payload)
        .unwrap();
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    receiver.playout(&mut output).unwrap();
    receiver.reconnect(&mut output);
    for packet in &delivered[1..] {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..7 {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 8);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 8);
    assert_eq!(snapshot.late_packets, 2);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_outage_resumes_opus_receiver() {
    // Phase 151: bandwidth admission followed by an outage and reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 10).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 7, 8, 9, 10, 11, 12],
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered[..3] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..3 {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);

    for packet in &delivered[3..] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in 0..6 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 9);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 9);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_reorder_resumes_opus_receiver() {
    // Phase 152: bandwidth admission followed by deterministic reorder and reconnect.
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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 10).unwrap()),
        Stage::Reorder(network_fault::ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(delivered.len(), 10);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 3, 5, 7, 6, 8, 10, 9]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered[..5] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..5 {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &delivered[5..] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in 0..5 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 10);
    for (index, expected_left) in (1..=10).map(|sequence| sequence as f32 / 10.0).enumerate() {
        let left_mean: f32 = output.frames[index].iter().step_by(2).sum::<f32>() / 960.0;
        let right_mean: f32 = output.frames[index].iter().skip(1).step_by(2).sum::<f32>() / 960.0;
        assert!(
            (left_mean - expected_left).abs() < 0.08 && (right_mean + expected_left).abs() < 0.08,
            "decoded frame at playout {index} has wrong stereo source: ({left_mean}, {right_mean})"
        );
    }
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 10);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_outage_resumes_opus_receiver() {
    // Phase 142: reconnect after outage gap
    // 12 packets seq 1..=12
    // OutageProfile(start=3, window=4): drops 0-indexed pos 3,4,5,6 -> seqs 4,5,6,7 lost
    //   Survivors: [seq1,seq2,seq3,seq8,seq9,seq10,seq11,seq12] -- 8 packets
    // Enqueue first 3 (seq1-seq3), then call receiver.reconnect(), then enqueue remaining 5 (seq8-seq12)
    // Playout 3 calls after initial enqueue (no gaps in seq1-seq3)
    // Playout 5 calls after reconnect enqueue (seq8-seq12, no internal gaps)
    // Total 8 frames: assert output.frames.len()==8, output.muted==1 (from reconnect), state==Playing
    // assert snapshot.reconnect_count==1, output_failures==0

    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    // OutageProfile(start=3, window=4): drops 0-indexed pos 3,4,5,6 -> seqs 4,5,6,7 lost
    let outage = network_fault::OutageProfile::new(3, 4).unwrap();
    let result = outage.apply(&encoded);
    let survivors = result.delivered;
    assert_eq!(survivors.len(), 8, "8 survivors after outage drops pos 3-6");

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    // Enqueue first 3 packets (seq1-seq3)
    let pre_reconnect = &survivors[..3];
    for pkt in pre_reconnect {
        receiver.enqueue(pkt.sequence, &pkt.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..3 {
        receiver.playout(&mut output).unwrap();
    }

    // Simulate reconnect
    receiver.reconnect(&mut output);

    // Enqueue remaining 5 packets (seq8-seq12)
    let post_reconnect = &survivors[3..];
    for pkt in post_reconnect {
        receiver.enqueue(pkt.sequence, &pkt.payload).unwrap();
    }
    for _ in 0..5 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(
        output.frames.len(),
        8,
        "eight total frames played (3 pre + 5 post reconnect)"
    );
    assert_eq!(output.muted, 1, "one mute call from reconnect");
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1, "one reconnect recorded");
    assert_eq!(snapshot.output_failures, 0, "no output failures");
}

#[test]
fn reconnect_after_combined_bandwidth_loss_resumes_opus_receiver() {
    // Phase 154: bandwidth admission followed by loss and reconnect.
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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 10).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 5, 7, 8, 10],
    );

    let reconnect = ReconnectProfile::new(3, 1, "musician-bandwidth-loss", 2)
        .unwrap()
        .apply(&delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 3);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert_eq!(reconnect.post_reconnect.len(), 3);
    assert_eq!(reconnect.recovered_mix_id, Some(2));

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &reconnect.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &reconnect.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 6);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 6);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 3);
    assert_eq!(snapshot.output_failures, 0);
    assert!(output
        .frames
        .iter()
        .all(|frame| frame.iter().any(|sample| sample.abs() > 0.001)));
}

#[test]
fn reconnect_after_combined_bandwidth_jitter_resumes_opus_receiver() {
    // Phase 153: bandwidth admission followed by deterministic jitter and reconnect.
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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 10).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert_eq!(delivered.len(), 10);
    assert_eq!(
        delivered
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![1, 2, 4, 3, 5, 7, 6, 8, 10, 9]
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered[..5] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..5 {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &delivered[5..] {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in 0..5 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(output.frames.len(), 10);
    for (index, expected_left) in (1..=10).map(|sequence| sequence as f32 / 10.0).enumerate() {
        let left_mean: f32 = output.frames[index].iter().step_by(2).sum::<f32>() / 960.0;
        let right_mean: f32 = output.frames[index].iter().skip(1).step_by(2).sum::<f32>() / 960.0;
        assert!(
            (left_mean - expected_left).abs() < 0.08 && (right_mean + expected_left).abs() < 0.08,
            "decoded frame at playout {index} has wrong stereo source: ({left_mean}, {right_mean})"
        );
    }
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.packets_received, 10);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_reorder_then_loss_drives_opus_receiver_plc() {
    // Phase 156: ReorderProfile(3) swaps positions, then LossProfile(4) drops every 4th.
    // ReorderProfile(3): swaps at 0-indexed positions 2,5,8
    //   swap idx2(seq3)<->idx3(seq4): [1,2,4,3,5,6,7,8,9]
    //   swap idx5(seq6)<->idx6(seq7): [1,2,4,3,5,7,6,8,9]
    //   idx8(seq9)<->idx9: doesn't exist -> no swap
    //   Result: [seq1,seq2,seq4,seq3,seq5,seq7,seq6,seq8,seq9]
    // LossProfile(4): drops 1-indexed pos4,8 -> drops seq3 (pos4), seq8 (pos8)
    //   Surviving: [seq1,seq2,seq4,seq5,seq7,seq6,seq9] (7 packets)
    //   Missing seqs: 3 and 8 -> 2 PLC frames
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=9u64 {
        let frame = test_frame(sequence);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
        Stage::Loss(LossProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert_eq!(delivered.len(), 7, "7 packets survive after reorder+loss");

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
    // seq1..=seq9 span, gaps at seq3 and seq8 -> 9 playout calls
    for _ in 0..9 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(delivered.len(), 7);
    assert_eq!(snapshot.packets_received, 7);
    assert_eq!(snapshot.plc_frames_total, 2);
    assert_eq!(snapshot.output_failures, 0);
    assert_eq!(output.frames.len(), 9);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
}

#[test]
fn combined_reorder_then_duplicate_classifies_receiver_late_packets() {
    // Phase 157: ReorderProfile(4) swaps, then DuplicateProfile(4) inserts copies.
    // ReorderProfile(4): swaps at 0-indexed positions 3,7
    //   swap idx3(seq4)<->idx4(seq5): [1,2,3,5,4,6,7,8]
    //   swap idx7(seq8)<->idx8: idx8 doesn't exist -> no swap
    //   Result: [seq1,seq2,seq3,seq5,seq4,seq6,seq7,seq8]
    // DuplicateProfile(4): duplicates at 1-indexed pos4,8
    //   pos4=seq5 -> copy after, pos8=seq8 -> copy after
    //   Result: [seq1,seq2,seq3,seq5,seq5_dup,seq4,seq6,seq7,seq8,seq8_dup] -> 10 packets
    //   2 late/duplicate packets (seq5_dup seen after seq5, seq8_dup after seq8)
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=8u64 {
        let frame = test_frame(sequence);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Reorder(ReorderProfile::new(4).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert_eq!(delivered.len(), 10, "8 originals + 2 duplicates = 10");

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for pkt in &delivered {
        let _ = receiver.enqueue(pkt.sequence, &pkt.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    // seq 1..=8, no gaps -> 8 playout calls
    for _ in 0..8 {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert_eq!(delivered.len(), 10);
    assert_eq!(snapshot.packets_received, 8);
    assert_eq!(snapshot.late_packets, 2);
    assert_eq!(snapshot.plc_frames_total, 0);
    assert_eq!(snapshot.output_failures, 0);
    assert_eq!(output.frames.len(), 8);
    assert_eq!(output.muted, 0);
    assert_eq!(receiver.state(), ReceiverState::Playing);
}

#[test]
fn reconnect_after_reorder_resumes_opus_receiver() {
    // Phase 158: ReorderProfile(3) reorders packets, then reconnect splits pre/post.
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

    // ReorderProfile(3) on 8 packets -> [1,2,4,3,5,7,6,8]
    let reordered = ReorderProfile::new(3).unwrap().apply(&encoded);
    assert_eq!(reordered.dropped, 0);
    assert!(reordered.reordered > 0);

    let reconnect = ReconnectProfile::new(4, 1, "musician-reorder", 1)
        .unwrap()
        .apply(&reordered.delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 4);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert!(!reconnect.post_reconnect.is_empty());
    assert_eq!(reconnect.recovered_mix_id, Some(1));

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for packet in &reconnect.pre_disconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &reconnect.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);

    for packet in &reconnect.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &reconnect.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_duplicate_resumes_opus_receiver() {
    // Phase 159: ReconnectProfile splits original packets first, then DuplicateProfile
    // applied to pre_disconnect only. Duplicates yield late_packets on enqueue.
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

    // Split pre/post on original encoded list
    let reconnect = ReconnectProfile::new(4, 1, "musician-dup", 1)
        .unwrap()
        .apply(&encoded);
    assert_eq!(reconnect.pre_disconnect.len(), 4);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert!(!reconnect.post_reconnect.is_empty());

    // Apply DuplicateProfile(4) to pre_disconnect only: duplicates seq4 (pos4)
    let pre_with_dups = DuplicateProfile::new(4)
        .unwrap()
        .apply(&reconnect.pre_disconnect);
    assert!(
        pre_with_dups.len() > reconnect.pre_disconnect.len(),
        "duplicates inserted"
    );

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for packet in &pre_with_dups {
        // Duplicates return DuplicateSequence error; counted as late_packets
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &reconnect.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);

    for packet in &reconnect.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &reconnect.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
    assert!(
        snapshot.late_packets > 0,
        "duplicates must be counted as late_packets"
    );
}

#[test]
fn reconnect_after_combined_jitter_loss_resumes_opus_receiver() {
    // Phase 160: JitterProfile(3,1) + LossProfile(3) via CombinedFaultProfile, then reconnect.
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
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());
    assert!(
        delivered.len() < encoded.len(),
        "jitter+loss must drop some packets"
    );

    let reconnect = ReconnectProfile::new(3, 1, "musician-jitter-loss", 2)
        .unwrap()
        .apply(&delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 3);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert!(!reconnect.post_reconnect.is_empty());
    assert_eq!(reconnect.recovered_mix_id, Some(2));

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for packet in &reconnect.pre_disconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in &reconnect.pre_disconnect {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);

    for packet in &reconnect.post_reconnect {
        receiver.enqueue(packet.sequence, &packet.payload).unwrap();
    }
    for _ in &reconnect.post_reconnect {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_reorder_duplicate_resumes_opus_receiver() {
    // Phase 161: ReorderProfile(3) + DuplicateProfile(4) via CombinedFaultProfile, then reconnect.
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
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let reconnect = ReconnectProfile::new(5, 1, "musician-reorder-dup", 1)
        .unwrap()
        .apply(&delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 5);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert!(!reconnect.post_reconnect.is_empty());

    // Count unique sequences in each segment for correct playout call count.
    let pre_unique: std::collections::HashSet<u64> = reconnect
        .pre_disconnect
        .iter()
        .map(|p| p.sequence)
        .collect();
    let post_unique: std::collections::HashSet<u64> = reconnect
        .post_reconnect
        .iter()
        .map(|p| p.sequence)
        .collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for packet in &reconnect.pre_disconnect {
        receiver
            .enqueue(packet.sequence, &packet.payload)
            .expect("pre-reconnect packet must enter bounded receiver ingress");
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..pre_unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);

    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..post_unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_jitter_duplicate_resumes_opus_receiver() {
    // Phase 162: JitterProfile(3,1) + DuplicateProfile(4) via CombinedFaultProfile, then reconnect.
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
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(4).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let reconnect = ReconnectProfile::new(5, 1, "musician-jitter-dup", 0)
        .unwrap()
        .apply(&delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 5);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert!(!reconnect.post_reconnect.is_empty());

    // Count unique sequences in each segment for correct playout call count.
    let pre_unique: std::collections::HashSet<u64> = reconnect
        .pre_disconnect
        .iter()
        .map(|p| p.sequence)
        .collect();
    let post_unique: std::collections::HashSet<u64> = reconnect
        .post_reconnect
        .iter()
        .map(|p| p.sequence)
        .collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..pre_unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);

    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..post_unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_loss_duplicate_resumes_opus_receiver() {
    // Phase 163: LossProfile(3) + DuplicateProfile(3) via CombinedFaultProfile, then reconnect.
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

    let combined = CombinedFaultProfile::new(vec![
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();

    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let reconnect = ReconnectProfile::new(4, 1, "musician-loss-dup", 2)
        .unwrap()
        .apply(&delivered);
    assert_eq!(reconnect.pre_disconnect.len(), 4);
    assert_eq!(reconnect.lost_at_disconnect, 1);
    assert!(!reconnect.post_reconnect.is_empty());

    // Count unique sequences in each segment for correct playout call count.
    let pre_unique: std::collections::HashSet<u64> = reconnect
        .pre_disconnect
        .iter()
        .map(|p| p.sequence)
        .collect();
    let post_unique: std::collections::HashSet<u64> = reconnect
        .post_reconnect
        .iter()
        .map(|p| p.sequence)
        .collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));

    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..pre_unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    receiver.reconnect(&mut output);

    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..post_unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
    let _ = snapshot.late_packets;
}

#[test]
fn combined_bandwidth_outage_loss_drives_opus_receiver() {
    // Phase 164: bandwidth admission, outage window, then loss.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.plc_frames_total > 0);
}

#[test]
fn combined_bandwidth_outage_jitter_drives_opus_receiver() {
    // Phase 165: bandwidth admission, outage window, then jitter.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.plc_frames_total > 0);
}

#[test]
fn combined_bandwidth_outage_reorder_drives_opus_receiver() {
    // Phase 166: bandwidth admission, outage window, then reorder.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.plc_frames_total > 0);
}

#[test]
fn combined_bandwidth_outage_duplicate_drives_opus_receiver() {
    // Phase 167: bandwidth admission, outage window, then duplicate.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let unique: std::collections::HashSet<u64> = delivered.iter().map(|p| p.sequence).collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.late_packets > 0);
}

#[test]
fn combined_bandwidth_loss_jitter_drives_opus_receiver() {
    // Phase 168: bandwidth admission, loss, then jitter.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.plc_frames_total > 0);
}

#[test]
fn combined_bandwidth_loss_reorder_drives_opus_receiver() {
    // Phase 169: bandwidth admission, loss, then reorder.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.plc_frames_total > 0);
}

#[test]
fn combined_bandwidth_loss_duplicate_drives_opus_receiver() {
    // Phase 170: bandwidth admission, loss, then duplicate.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let unique: std::collections::HashSet<u64> = delivered.iter().map(|p| p.sequence).collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.late_packets > 0);
}

#[test]
fn combined_bandwidth_jitter_reorder_drives_opus_receiver() {
    // Phase 171: bandwidth admission, jitter, then reorder.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..delivered.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn combined_bandwidth_jitter_duplicate_drives_opus_receiver() {
    // Phase 172: bandwidth admission, jitter, then duplicate.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let unique: std::collections::HashSet<u64> = delivered.iter().map(|p| p.sequence).collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &delivered {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }

    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..unique.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.output_failures, 0);
    assert!(snapshot.late_packets > 0);
}

#[test]
fn reconnect_after_combined_bandwidth_outage_loss_resumes_opus_receiver() {
    // Phase 173: Bandwidth->Outage->Loss triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-outage-loss", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        receiver
            .enqueue(packet.sequence, &packet.payload)
            .expect("post-reconnect packet must enter bounded receiver ingress");
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_outage_jitter_resumes_opus_receiver() {
    // Phase 174: Bandwidth->Outage->Jitter triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-outage-jitter", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_outage_reorder_resumes_opus_receiver() {
    // Phase 175: Bandwidth->Outage->Reorder triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-outage-reorder", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_outage_duplicate_resumes_opus_receiver() {
    // Phase 176: Bandwidth->Outage->Duplicate triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-outage-dup", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let pre_unique: std::collections::HashSet<u64> = reconnect
        .pre_disconnect
        .iter()
        .map(|p| p.sequence)
        .collect();
    let post_unique: std::collections::HashSet<u64> = reconnect
        .post_reconnect
        .iter()
        .map(|p| p.sequence)
        .collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..pre_unique.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..post_unique.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_loss_jitter_resumes_opus_receiver() {
    // Phase 177: Bandwidth->Loss->Jitter triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-loss-jitter", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_loss_reorder_resumes_opus_receiver() {
    // Phase 178: Bandwidth->Loss->Reorder triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-loss-reorder", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_loss_duplicate_resumes_opus_receiver() {
    // Phase 179: Bandwidth->Loss->Duplicate triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-loss-dup", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let pre_unique: std::collections::HashSet<u64> = reconnect
        .pre_disconnect
        .iter()
        .map(|p| p.sequence)
        .collect();
    let post_unique: std::collections::HashSet<u64> = reconnect
        .post_reconnect
        .iter()
        .map(|p| p.sequence)
        .collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..pre_unique.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..post_unique.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_jitter_reorder_resumes_opus_receiver() {
    // Phase 180: Bandwidth->Jitter->Reorder triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Reorder(ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-jitter-reorder", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_bandwidth_jitter_duplicate_resumes_opus_receiver() {
    // Phase 181: Bandwidth->Jitter->Duplicate triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Bandwidth(network_fault::BandwidthProfile::new(5_000, 12).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(!delivered.is_empty());

    let split_at = std::cmp::max(2, delivered.len() / 2);
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-bw-jitter-dup", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let pre_unique: std::collections::HashSet<u64> = reconnect
        .pre_disconnect
        .iter()
        .map(|p| p.sequence)
        .collect();
    let post_unique: std::collections::HashSet<u64> = reconnect
        .post_reconnect
        .iter()
        .map(|p| p.sequence)
        .collect();

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..pre_unique.len() {
        receiver.playout(&mut output).unwrap();
    }
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..post_unique.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(!output.frames.is_empty());
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_outage_loss_jitter_resumes_opus_receiver() {
    // Phase 182: Outage->Loss->Jitter triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Jitter(JitterProfile::new(3, 1).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(delivered.len() >= 2);

    let split_at = delivered.len() / 2;
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-outage-loss-jitter", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let pre_reconnect_frames = output.frames.len();
    receiver.reconnect(&mut output);
    for packet in &reconnect.post_reconnect {
        let _ = receiver.enqueue(packet.sequence, &packet.payload);
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(output.frames.len() > pre_reconnect_frames);
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_outage_loss_reorder_resumes_opus_receiver() {
    // Phase 183: Outage->Loss->Reorder triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Reorder(network_fault::ReorderProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(delivered.len() >= 2);

    let split_at = delivered.len() / 2;
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-outage-loss-reorder", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        receiver
            .enqueue(packet.sequence, &packet.payload)
            .expect("pre-reconnect packet must enter bounded receiver ingress");
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let pre_reconnect_frames = output.frames.len();
    receiver.reconnect(&mut output);
    let packets_before_reconnect = metrics.snapshot().packets_received;
    for packet in &reconnect.post_reconnect {
        receiver
            .enqueue(packet.sequence, &packet.payload)
            .expect("post-reconnect packet must enter bounded receiver ingress");
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let snapshot = metrics.snapshot();
    assert!(snapshot.packets_received > packets_before_reconnect);
    assert!(output.frames.len() > pre_reconnect_frames);
    assert!(
        output.frames[pre_reconnect_frames..]
            .iter()
            .any(|frame| frame.iter().any(|sample| sample.abs() > 0.001)),
        "post-reconnect output is silent"
    );
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.output_failures, 0);
}

#[test]
fn reconnect_after_combined_outage_loss_duplicate_resumes_opus_receiver() {
    // Phase 184: Outage->Loss->Duplicate triple fault then reconnect.
    let mut writer = MediaWriter::new().unwrap();
    let mut encoded = Vec::new();
    for sequence in 1..=12u64 {
        let mut frame = test_frame(sequence);
        frame.samples = (sequence as f32 / 12.0, -(sequence as f32) / 12.0);
        let packet = writer.encode(&frame).unwrap();
        encoded.push(Packet {
            sequence: packet.sequence,
            payload: packet.payload,
        });
    }

    let combined = CombinedFaultProfile::new(vec![
        Stage::Outage(network_fault::OutageProfile::new(3, 3).unwrap()),
        Stage::Loss(LossProfile::new(3).unwrap()),
        Stage::Duplicate(DuplicateProfile::new(3).unwrap()),
    ])
    .unwrap();
    let delivered = combined.apply(&encoded);
    assert!(delivered.len() >= 2);
    assert!(delivered
        .windows(2)
        .any(|packets| packets[0].sequence == packets[1].sequence));

    let split_at = delivered.len() / 2;
    let reconnect = ReconnectProfile::new(split_at, 1, "musician-outage-loss-duplicate", 2)
        .unwrap()
        .apply(&delivered);
    assert!(!reconnect.pre_disconnect.is_empty());
    assert!(!reconnect.post_reconnect.is_empty());

    let metrics = Arc::new(ReceiverMetrics::default());
    let mut receiver = OpusReceiver::new()
        .unwrap()
        .with_metrics(Arc::clone(&metrics));
    for packet in &reconnect.pre_disconnect {
        receiver
            .enqueue(packet.sequence, &packet.payload)
            .expect("pre-reconnect packet must enter bounded receiver ingress");
    }
    let mut output = Capture {
        frames: Vec::new(),
        muted: 0,
    };
    for _ in 0..reconnect.pre_disconnect.len() {
        receiver.playout(&mut output).unwrap();
    }
    let pre_reconnect_frames = output.frames.len();
    receiver.reconnect(&mut output);
    let packets_before_reconnect = metrics.snapshot().packets_received;
    for packet in &reconnect.post_reconnect {
        receiver
            .enqueue(packet.sequence, &packet.payload)
            .expect("post-reconnect packet must enter bounded receiver ingress");
    }
    for _ in 0..reconnect.post_reconnect.len() {
        receiver.playout(&mut output).unwrap();
    }

    let snapshot = metrics.snapshot();
    assert!(snapshot.packets_received > packets_before_reconnect);
    assert!(output.frames.len() > pre_reconnect_frames);
    assert!(output.frames[pre_reconnect_frames..]
        .iter()
        .any(|frame| frame.iter().any(|sample| sample.abs() > 0.001)));
    assert_eq!(output.muted, 1);
    assert_eq!(receiver.state(), ReceiverState::Playing);
    assert_eq!(snapshot.reconnect_count, 1);
    assert!(snapshot.late_packets > 0);
    assert_eq!(snapshot.output_failures, 0);
}
