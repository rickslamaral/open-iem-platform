//! Versioned control-plane messages shared by backend and clients.

#![deny(missing_docs)]
#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Current wire protocol version.
pub const PROTOCOL_VERSION: u16 = 1;

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

/// Control-plane message catalog, Phase 3 foundation.
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
}

/// Server-to-client control-plane messages.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    /// Current state revision.
    State {
        /// Monotonic state revision.
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
    let envelope: Envelope<ClientMessage> =
        serde_json::from_str(input).map_err(ProtocolError::InvalidJson)?;
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
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson(error) => write!(formatter, "invalid protocol JSON: {error}"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported protocol version: {version}")
            }
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
}
