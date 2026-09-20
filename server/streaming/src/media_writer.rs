//! Bounded Opus frame writer for one negotiated media session.
//!
//! Produces encoded 48 kHz stereo Opus packets from `MediaFrame` samples.
//! Transport ownership stays with WebRTC session drive; this module performs
//! no socket or filesystem I/O and remains CODE/SIMULATED until connected to a
//! negotiated `str0m::media::Writer`.

use crate::clock::SampleTimestamp;
use crate::media_plane::MediaFrame;
use opus_pure::{Application, OpusEncoder};
use thiserror::Error;

pub const OPUS_MAX_PACKET_BYTES: usize = 4_000;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MediaWriterError {
    #[error("media frame format is not 48 kHz stereo")]
    InvalidFormat,
    #[error("media frame sample count is not one 20 ms frame")]
    InvalidFrameLength,
    #[error("Opus encoder failed")]
    Encode,
}

/// Stateful Opus encoder for one stereo media session.
pub struct MediaWriter {
    encoder: OpusEncoder,
    packet: Vec<u8>,
    next_rtp_timestamp: u32,
}

impl MediaWriter {
    /// # Errors
    ///
    /// Returns [`MediaWriterError::Encode`] when encoder initialization fails.
    pub fn new() -> Result<Self, MediaWriterError> {
        let encoder = OpusEncoder::new(48_000, 2, Application::Audio)
            .map_err(|_| MediaWriterError::Encode)?;
        Ok(Self {
            encoder,
            packet: vec![0; OPUS_MAX_PACKET_BYTES],
            next_rtp_timestamp: 0,
        })
    }

    /// Encode one 20 ms stereo frame. Current `MediaFrame` carries one stereo
    /// pair, so this CODE/SIMULATED boundary repeats that pair across 960
    /// samples. It is not a capture-buffer or runtime media writer yet.
    /// # Errors
    ///
    /// Returns a format error for unsupported frames or an encode error when
    /// Opus rejects input or cannot fit output in its bounded packet buffer.
    pub fn encode(&mut self, frame: &MediaFrame) -> Result<MediaPacket, MediaWriterError> {
        if frame.metadata.sample_rate != 48_000 || frame.metadata.channels != 2 {
            return Err(MediaWriterError::InvalidFormat);
        }
        if frame.metadata.frame_duration_ms != 20 {
            return Err(MediaWriterError::InvalidFrameLength);
        }
        let pcm = [frame.samples.0, frame.samples.1]
            .into_iter()
            .cycle()
            .take(960 * 2)
            .collect::<Vec<_>>();
        let len = self
            .encoder
            .encode(&pcm, 960, &mut self.packet)
            .map_err(|_| MediaWriterError::Encode)?;
        if len == 0 || len > OPUS_MAX_PACKET_BYTES {
            return Err(MediaWriterError::Encode);
        }
        let rtp_timestamp = self.next_rtp_timestamp;
        self.next_rtp_timestamp = self.next_rtp_timestamp.wrapping_add(960);
        Ok(MediaPacket {
            sequence: frame.metadata.sequence,
            revision: frame.metadata.revision,
            sample_rate: frame.metadata.sample_rate,
            channels: frame.metadata.channels,
            frame_duration_ms: frame.metadata.frame_duration_ms,
            rtp_timestamp,
            payload: self.packet[..len].to_vec(),
            capture_timestamp: frame.metadata.capture_timestamp,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPacket {
    pub sequence: u64,
    pub revision: u64,
    pub sample_rate: u32,
    pub channels: u8,
    pub frame_duration_ms: u32,
    /// RTP audio clock timestamp; increments by 960 samples per 20 ms frame.
    pub rtp_timestamp: u32,
    pub payload: Vec<u8>,
    pub capture_timestamp: Option<SampleTimestamp>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_plane::{MediaFrame, StreamMetadata};

    fn frame() -> MediaFrame {
        MediaFrame {
            metadata: StreamMetadata {
                stream_id: "mix_0".into(),
                mix_index: 0,
                revision: 9,
                sequence: 3,
                sample_rate: 48_000,
                channels: 2,
                frame_duration_ms: 20,
                capture_timestamp: None,
            },
            samples: (0.1, -0.1),
        }
    }

    #[test]
    fn encodes_bounded_opus_packet_with_metadata() {
        let packet = MediaWriter::new().unwrap().encode(&frame()).unwrap();
        assert_eq!((packet.sequence, packet.revision), (3, 9));
        assert_eq!(
            (
                packet.sample_rate,
                packet.channels,
                packet.frame_duration_ms
            ),
            (48_000, 2, 20)
        );
        assert_eq!(packet.rtp_timestamp, 0);
        assert!(!packet.payload.is_empty());
        assert!(packet.payload.len() <= OPUS_MAX_PACKET_BYTES);
    }

    #[test]
    fn rejects_wrong_sample_rate() {
        let mut f = frame();
        f.metadata.sample_rate = 44_100;
        assert_eq!(
            MediaWriter::new().unwrap().encode(&f),
            Err(MediaWriterError::InvalidFormat)
        );
    }

    #[test]
    fn consecutive_packets_advance_rtp_timestamp() {
        let mut writer = MediaWriter::new().unwrap();
        let first = writer.encode(&frame()).unwrap();
        let second = writer.encode(&frame()).unwrap();
        assert_eq!(second.rtp_timestamp.wrapping_sub(first.rtp_timestamp), 960);
    }

    #[test]
    fn rejects_wrong_frame_duration() {
        let mut f = frame();
        f.metadata.frame_duration_ms = 10;
        assert_eq!(
            MediaWriter::new().unwrap().encode(&f),
            Err(MediaWriterError::InvalidFrameLength)
        );
    }
}
