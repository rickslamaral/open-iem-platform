//! Versioned control-plane messages shared by backend and clients.

#![deny(missing_docs)]
#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Current wire protocol version.
pub const PROTOCOL_VERSION: u16 = 1;

/// Maximum UTF-8 byte length for a request correlation identifier.
pub const MAX_REQUEST_ID_BYTES: usize = 128;
/// Maximum encoded client message size accepted by protocol decoder.
pub const MAX_MESSAGE_BYTES: usize = 16 * 1024;

/// Supported authorization roles.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    /// Full administrative access.
    Admin,
    /// Engine configuration access.
    Engineer,
    /// Own mix control access.
    Musician,
}

/// Stable message envelope for WebSocket and future transports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Envelope<T> {
    /// Protocol version.
    pub version: u16,
    /// Correlation identifier chosen by sender.
    pub request_id: String,
    /// Message payload.
    pub payload: T,
}

impl<T> Envelope<T> {
    /// Build envelope using current protocol version.
    #[must_use]
    pub fn new(request_id: String, payload: T) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            request_id,
            payload,
        }
    }
}

/// Control-plane message catalog — Phase 3 foundation + Phase 16 send mutations.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    /// Request current server state.
    GetState,
    /// Update one channel's gain.
    SetChannelGain {
        /// Channel index.
        channel: u8,
        /// Gain in dBFS.
        gain_db: f32,
    },
    /// Update one channel mute state.
    SetChannelMute {
        /// Channel index.
        channel: u8,
        /// Mute flag.
        muted: bool,
    },
    /// Update one send's gain. Musician may only target their assigned mix.
    SetSendGain {
        /// Mix slot index.
        mix_index: u8,
        /// Channel slot index.
        channel_index: u8,
        /// Gain in dBFS.
        gain_db: f32,
    },
    /// Update one send's pan. Musician may only target their assigned mix.
    SetSendPan {
        /// Mix slot index.
        mix_index: u8,
        /// Channel slot index.
        channel_index: u8,
        /// Pan from -1.0 (left) to 1.0 (right).
        pan: f32,
    },
    /// Update one send's mute state. Musician may only target their assigned mix.
    SetSendMuted {
        /// Mix slot index.
        mix_index: u8,
        /// Channel slot index.
        channel_index: u8,
        /// Mute flag.
        muted: bool,
    },
    /// Set master gain for a mix. Engineer/Admin only.
    SetMasterGain {
        /// Mix slot index.
        mix_index: u8,
        /// Master gain in dBFS.
        gain_db: f32,
    },
    /// Set master mute for a mix. Engineer/Admin only.
    SetMasterMute {
        /// Mix slot index.
        mix_index: u8,
        /// Mute state.
        muted: bool,
    },
    /// Configure one parametric EQ band for a mix. Engineer/Admin only.
    SetEqBand {
        /// Mix slot index.
        mix_index: u8,
        /// Band index (`0..MAX_EQ_BANDS`).
        band_index: u8,
        /// Centre frequency in Hz (20–20000).
        frequency_hz: f32,
        /// Gain in dB (−24.0–+24.0).
        gain_db: f32,
        /// Q factor (0.1–10.0).
        q: f32,
        /// Whether this band is active.
        enabled: bool,
    },
}

/// Server-to-client control-plane messages.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    /// Current state revision.
    State {
        /// Monotonic state revision.
        revision: u64,
    },
    /// Acknowledged send mutation — echoes current send parameters.
    SendAck {
        /// Mix slot.
        mix_index: u8,
        /// Channel slot.
        channel_index: u8,
        /// Current gain in dBFS.
        gain_db: f32,
        /// Current pan.
        pan: f32,
        /// Current mute state.
        muted: bool,
        /// Updated state revision.
        revision: u64,
    },
    /// Acknowledged master mutation.
    MasterAck {
        /// Mix slot.
        mix_index: u8,
        /// Current master gain in dBFS.
        master_gain_db: f32,
        /// Current master mute state.
        master_muted: bool,
        /// Updated mix revision.
        revision: u64,
    },
    /// Acknowledged EQ band mutation.
    EqBandAck {
        /// Mix slot.
        mix_index: u8,
        /// Band index.
        band_index: u8,
        /// Centre frequency in Hz.
        frequency_hz: f32,
        /// Gain in dB.
        gain_db: f32,
        /// Q factor.
        q: f32,
        /// Whether band is active.
        enabled: bool,
        /// Updated mix EQ revision.
        revision: u64,
    },
    /// Protocol or request error.
    Error {
        /// Machine-readable error code.
        code: String,
        /// Human-readable detail.
        message: String,
    },
}

/// Decode a client envelope and reject unsupported protocol versions.
///
/// # Errors
///
/// Returns [`ProtocolError::InvalidJson`] for malformed payloads and
/// [`ProtocolError::UnsupportedVersion`] when version is not supported.
pub fn decode_client_message(input: &str) -> Result<Envelope<ClientMessage>, ProtocolError> {
    if input.len() > MAX_MESSAGE_BYTES {
        return Err(ProtocolError::MessageTooLarge);
    }
    let envelope: Envelope<ClientMessage> =
        serde_json::from_str(input).map_err(ProtocolError::InvalidJson)?;
    if envelope.request_id.is_empty() || envelope.request_id.len() > MAX_REQUEST_ID_BYTES {
        return Err(ProtocolError::InvalidRequestId);
    }
    if envelope.version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(envelope.version));
    }
    Ok(envelope)
}

/// Protocol decoding failure.
#[derive(Debug)]
pub enum ProtocolError {
    /// Payload is not valid JSON or schema.
    InvalidJson(serde_json::Error),
    /// Version is not supported.
    UnsupportedVersion(u16),
    /// Request correlation identifier is empty or too long.
    InvalidRequestId,
    /// Encoded client message exceeds the protocol limit.
    MessageTooLarge,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid protocol JSON: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported protocol version: {version}")
            }
            Self::InvalidRequestId => write!(formatter, "invalid request id"),
            Self::MessageTooLarge => write!(formatter, "message too large"),
        }
    }
}

impl std::error::Error for ProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_envelope() {
        let envelope = Envelope::new("req-1".to_owned(), ClientMessage::GetState);
        let encoded =
            serde_json::to_string(&envelope).expect("serialization is infallible for known value");
        let decoded = decode_client_message(&encoded).expect("encoded message must decode");
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn rejects_wrong_version() {
        let input = r#"{"version":99,"request_id":"x","payload":{"type":"GetState"}}"#;
        assert!(matches!(
            decode_client_message(input),
            Err(ProtocolError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn rejects_empty_or_oversized_request_id() {
        let empty = r#"{"version":1,"request_id":"","payload":{"type":"GetState"}}"#;
        assert!(matches!(
            decode_client_message(empty),
            Err(ProtocolError::InvalidRequestId)
        ));

        let oversized = format!(
            r#"{{"version":1,"request_id":"{}","payload":{{"type":"GetState"}}}}"#,
            "x".repeat(MAX_REQUEST_ID_BYTES + 1)
        );
        assert!(matches!(
            decode_client_message(&oversized),
            Err(ProtocolError::InvalidRequestId)
        ));
    }

    #[test]
    fn round_trip_set_send_gain() {
        let envelope = Envelope::new(
            "req-sg".to_owned(),
            ClientMessage::SetSendGain {
                mix_index: 0,
                channel_index: 2,
                gain_db: -6.0,
            },
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded = decode_client_message(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn round_trip_set_send_pan() {
        let envelope = Envelope::new(
            "req-sp".to_owned(),
            ClientMessage::SetSendPan {
                mix_index: 1,
                channel_index: 0,
                pan: 0.5,
            },
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded = decode_client_message(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn round_trip_set_send_muted() {
        let envelope = Envelope::new(
            "req-sm".to_owned(),
            ClientMessage::SetSendMuted {
                mix_index: 0,
                channel_index: 3,
                muted: true,
            },
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded = decode_client_message(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn round_trip_send_ack_server_message() {
        let ack = ServerMessage::SendAck {
            mix_index: 0,
            channel_index: 2,
            gain_db: -6.0,
            pan: 0.0,
            muted: false,
            revision: 42,
        };
        let json = serde_json::to_string(&ack).unwrap();
        let decoded: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ack);
    }

    #[test]
    fn round_trip_set_master_gain() {
        let envelope = Envelope::new(
            "req-mg".to_owned(),
            ClientMessage::SetMasterGain {
                mix_index: 2,
                gain_db: -3.0,
            },
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded = decode_client_message(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn round_trip_set_master_mute() {
        let envelope = Envelope::new(
            "req-mm".to_owned(),
            ClientMessage::SetMasterMute {
                mix_index: 1,
                muted: true,
            },
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded = decode_client_message(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn round_trip_master_ack_server_message() {
        let ack = ServerMessage::MasterAck {
            mix_index: 0,
            master_gain_db: -6.0,
            master_muted: true,
            revision: 7,
        };
        let json = serde_json::to_string(&ack).unwrap();
        let decoded: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ack);
    }

    #[test]
    fn round_trip_set_eq_band() {
        let envelope = Envelope::new(
            "req-eq".to_owned(),
            ClientMessage::SetEqBand {
                mix_index: 0,
                band_index: 2,
                frequency_hz: 1_000.0,
                gain_db: 6.0,
                q: 1.4,
                enabled: true,
            },
        );
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded = decode_client_message(&json).unwrap();
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn round_trip_eq_band_ack_server_message() {
        let ack = ServerMessage::EqBandAck {
            mix_index: 1,
            band_index: 0,
            frequency_hz: 2_000.0,
            gain_db: -3.0,
            q: 0.7,
            enabled: false,
            revision: 12,
        };
        let json = serde_json::to_string(&ack).unwrap();
        let decoded: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ack);
    }
}
