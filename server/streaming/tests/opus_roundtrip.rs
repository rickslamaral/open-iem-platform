use streaming::{
    AudioOutput, MediaFrame, MediaWriter, OpusReceiver, OutputError, ReceiverState, StreamMetadata,
};

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
