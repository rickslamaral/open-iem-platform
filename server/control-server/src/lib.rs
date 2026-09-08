//! Control-plane state dispatch for the Open IEM backend.

#![deny(missing_docs)]
#![deny(unsafe_code)]

use control_protocol::{ClientMessage, Envelope, ServerMessage, PROTOCOL_VERSION};
use mix_engine::{Channel, MixEngine, GAIN_DB_MAX, GAIN_DB_MIN, MAX_CHANNELS};

/// Mutable control-plane state owned by the server task.
#[derive(Debug, Default)]
pub struct ControlState {
    engine: MixEngine,
}

impl ControlState {
    /// Create an empty control-plane state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            engine: MixEngine::new(),
        }
    }

    /// Return the current mix engine revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.engine.revision()
    }

    /// Read a channel, if configured.
    #[must_use]
    pub fn channel(&self, index: usize) -> Option<&Channel> {
        self.engine.channel(index)
    }

    /// Dispatch one validated client message.
    ///
    /// Invalid channel indexes produce an error response and do not mutate state.
    #[must_use]
    pub fn dispatch(&mut self, request: Envelope<ClientMessage>) -> Envelope<ServerMessage> {
        let response = if request.version == PROTOCOL_VERSION {
            match request.payload {
                ClientMessage::GetState => ServerMessage::State {
                    revision: self.revision(),
                },
                ClientMessage::SetChannelGain { channel, gain_db } => {
                    if gain_db.is_finite() && (GAIN_DB_MIN..=GAIN_DB_MAX).contains(&gain_db) {
                        self.update_channel(channel, |item| item.set_gain_db(gain_db))
                    } else {
                        ServerMessage::Error {
                            code: "INVALID_GAIN".to_owned(),
                            message: format!(
                                "gain_db must be finite and between {GAIN_DB_MIN} and {GAIN_DB_MAX} dB"
                            ),
                        }
                    }
                }
                ClientMessage::SetChannelMute { channel, muted } => {
                    self.update_channel(channel, |item| item.set_muted(muted))
                }
            }
        } else {
            ServerMessage::Error {
                code: "UNSUPPORTED_PROTOCOL_VERSION".to_owned(),
                message: format!("unsupported protocol version: {}", request.version),
            }
        };

        Envelope {
            version: PROTOCOL_VERSION,
            request_id: request.request_id,
            payload: response,
        }
    }

    fn update_channel<F>(&mut self, index: u8, update: F) -> ServerMessage
    where
        F: FnOnce(&mut Channel),
    {
        let index = usize::from(index);
        if index >= MAX_CHANNELS {
            return ServerMessage::Error {
                code: "CHANNEL_OUT_OF_RANGE".to_owned(),
                message: format!("channel index {index} exceeds maximum {MAX_CHANNELS}"),
            };
        }

        let mut channel = self
            .engine
            .channel(index)
            .cloned()
            .unwrap_or_else(|| Channel::new(index as u32, ""));
        update(&mut channel);
        if self.engine.set_channel(index, channel).is_err() {
            return ServerMessage::Error {
                code: "ENGINE_ERROR".to_owned(),
                message: "failed to update channel".to_owned(),
            };
        }
        ServerMessage::State {
            revision: self.revision(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use control_protocol::decode_client_message;

    fn request(payload: ClientMessage) -> Envelope<ClientMessage> {
        Envelope::new("req-1".to_owned(), payload)
    }

    #[test]
    fn get_state_returns_current_revision() {
        let mut state = ControlState::new();
        let response = state.dispatch(request(ClientMessage::GetState));
        assert_eq!(response.request_id, "req-1");
        assert_eq!(response.payload, ServerMessage::State { revision: 0 });
    }

    #[test]
    fn gain_update_changes_channel_and_revision() {
        let mut state = ControlState::new();
        let response = state.dispatch(request(ClientMessage::SetChannelGain {
            channel: 2,
            gain_db: 6.0,
        }));
        assert_eq!(response.payload, ServerMessage::State { revision: 1 });
        assert_eq!(state.channel(2).map(Channel::gain_db), Some(6.0));
    }

    #[test]
    fn mute_update_changes_channel_and_revision() {
        let mut state = ControlState::new();
        let response = state.dispatch(request(ClientMessage::SetChannelMute {
            channel: 1,
            muted: true,
        }));
        assert_eq!(response.payload, ServerMessage::State { revision: 1 });
        assert_eq!(state.channel(1).map(|channel| channel.muted), Some(true));
    }

    #[test]
    fn rejects_unsupported_protocol_version_without_mutation() {
        let mut state = ControlState::new();
        let mut request = request(ClientMessage::SetChannelMute {
            channel: 0,
            muted: true,
        });
        request.version = PROTOCOL_VERSION + 1;
        let response = state.dispatch(request);
        assert!(matches!(response.payload, ServerMessage::Error { .. }));
        assert_eq!(state.revision(), 0);
        assert!(state.channel(0).is_none());
    }

    #[test]
    fn invalid_channel_does_not_mutate_state() {
        let mut state = ControlState::new();
        let response = state.dispatch(request(ClientMessage::SetChannelMute {
            channel: MAX_CHANNELS as u8,
            muted: true,
        }));
        assert!(matches!(response.payload, ServerMessage::Error { .. }));
        assert_eq!(state.revision(), 0);
        assert!(state.channel(MAX_CHANNELS).is_none());
    }

    #[test]
    fn non_finite_gain_is_rejected_without_mutation() {
        let mut state = ControlState::new();
        let response = state.dispatch(request(ClientMessage::SetChannelGain {
            channel: 0,
            gain_db: f32::NAN,
        }));
        assert!(matches!(response.payload, ServerMessage::Error { .. }));
        assert_eq!(state.revision(), 0);
        assert!(state.channel(0).is_none());
    }

    #[test]
    fn decoded_json_dispatch_preserves_request_id_and_version() {
        let input = r#"{"version":1,"request_id":"json-1","payload":{"type":"SetChannelGain","data":{"channel":0,"gain_db":-3.0}}}"#;
        let message = decode_client_message(input).expect("valid protocol message");
        let mut state = ControlState::new();
        let response = state.dispatch(message);
        assert_eq!(response.version, PROTOCOL_VERSION);
        assert_eq!(response.request_id, "json-1");
        assert_eq!(response.payload, ServerMessage::State { revision: 1 });
    }
}
