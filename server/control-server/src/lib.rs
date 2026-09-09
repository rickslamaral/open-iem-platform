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

    /// Read a mix, if configured.
    #[must_use]
    pub fn mix(&self, index: usize) -> Option<&mix_engine::Mix> {
        self.engine.mix(index)
    }

    /// Read a mutable mix, if configured.
    pub fn mix_mut(&mut self, index: usize) -> Option<&mut mix_engine::Mix> {
        self.engine.mix_mut(index)
    }

    /// Add or replace mix configuration.
    ///
    /// # Errors
    /// Returns `mix_engine::EngineError` when index is outside mix slots.
    pub fn set_mix(
        &mut self,
        index: usize,
        mix: mix_engine::Mix,
    ) -> Result<(), mix_engine::EngineError> {
        self.engine.set_mix(index, mix)
    }

    /// Read a channel, if configured.
    #[must_use]
    pub fn channel(&self, index: usize) -> Option<&Channel> {
        self.engine.channel(index)
    }

    /// Visit every configured channel with its stable slot index.
    pub fn for_each_channel(&self, mut visit: impl FnMut(usize, &Channel)) {
        for index in 0..MAX_CHANNELS {
            if let Some(channel) = self.engine.channel(index) {
                visit(index, channel);
            }
        }
    }

    /// Visit every configured mix with its stable slot index.
    pub fn for_each_mix(&self, mut visit: impl FnMut(usize, &mix_engine::Mix)) {
        for index in 0..mix_engine::MAX_MIXES {
            if let Some(mix) = self.engine.mix(index) {
                visit(index, mix);
            }
        }
    }

    /// Dispatch one validated client message.
    ///
    /// Mix ownership (Musician role) is enforced by the caller (`ws_handler`)
    /// before this method is called.  `dispatch` trusts the caller has already
    /// checked role and ownership.
    ///
    /// Invalid channel/mix indexes or out-of-range values produce an error
    /// response and do not mutate state.
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
                ClientMessage::SetSendGain {
                    mix_index,
                    channel_index,
                    gain_db,
                } => {
                    if !gain_db.is_finite() || !(GAIN_DB_MIN..=GAIN_DB_MAX).contains(&gain_db) {
                        return Envelope {
                            version: PROTOCOL_VERSION,
                            request_id: request.request_id,
                            payload: ServerMessage::Error {
                                code: "INVALID_GAIN".to_owned(),
                                message: format!(
                                    "gain_db must be finite and between {GAIN_DB_MIN} and {GAIN_DB_MAX} dB"
                                ),
                            },
                        };
                    }
                    self.update_send_gain(mix_index, channel_index, gain_db)
                }
                ClientMessage::SetSendPan {
                    mix_index,
                    channel_index,
                    pan,
                } => {
                    if !pan.is_finite() || !(-1.0..=1.0_f32).contains(&pan) {
                        return Envelope {
                            version: PROTOCOL_VERSION,
                            request_id: request.request_id,
                            payload: ServerMessage::Error {
                                code: "INVALID_PAN".to_owned(),
                                message: "pan must be finite and between -1.0 and 1.0".to_owned(),
                            },
                        };
                    }
                    self.update_send_pan(mix_index, channel_index, pan)
                }
                ClientMessage::SetSendMuted {
                    mix_index,
                    channel_index,
                    muted,
                } => self.update_send_muted(mix_index, channel_index, muted),
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

    fn update_send_gain(
        &mut self,
        mix_index: u8,
        channel_index: u8,
        gain_db: f32,
    ) -> ServerMessage {
        let mix_i = usize::from(mix_index);
        let ch_i = usize::from(channel_index);
        let Some(mix) = self.engine.mix_mut(mix_i) else {
            return ServerMessage::Error {
                code: "MIX_NOT_FOUND".to_owned(),
                message: format!("mix slot {mix_i} not configured"),
            };
        };
        if let Err(e) = mix.set_send_gain_db(ch_i, gain_db) {
            return ServerMessage::Error {
                code: "SEND_ERROR".to_owned(),
                message: e.to_string(),
            };
        }
        let send = mix.send(ch_i);
        ServerMessage::SendAck {
            mix_index,
            channel_index,
            gain_db: send.map_or(gain_db, mix_engine::MixSend::gain_db),
            pan: send.map_or(0.0, mix_engine::MixSend::pan),
            muted: send.is_some_and(|s| s.muted),
            revision: mix.revision(),
        }
    }

    fn update_send_pan(&mut self, mix_index: u8, channel_index: u8, pan: f32) -> ServerMessage {
        let mix_i = usize::from(mix_index);
        let ch_i = usize::from(channel_index);
        let Some(mix) = self.engine.mix_mut(mix_i) else {
            return ServerMessage::Error {
                code: "MIX_NOT_FOUND".to_owned(),
                message: format!("mix slot {mix_i} not configured"),
            };
        };
        if let Err(e) = mix.set_send_pan(ch_i, pan) {
            return ServerMessage::Error {
                code: "SEND_ERROR".to_owned(),
                message: e.to_string(),
            };
        }
        let send = mix.send(ch_i);
        ServerMessage::SendAck {
            mix_index,
            channel_index,
            gain_db: send.map_or(0.0, mix_engine::MixSend::gain_db),
            pan: send.map_or(pan, mix_engine::MixSend::pan),
            muted: send.is_some_and(|s| s.muted),
            revision: mix.revision(),
        }
    }

    fn update_send_muted(
        &mut self,
        mix_index: u8,
        channel_index: u8,
        muted: bool,
    ) -> ServerMessage {
        let mix_i = usize::from(mix_index);
        let ch_i = usize::from(channel_index);
        let Some(mix) = self.engine.mix_mut(mix_i) else {
            return ServerMessage::Error {
                code: "MIX_NOT_FOUND".to_owned(),
                message: format!("mix slot {mix_i} not configured"),
            };
        };
        if let Err(e) = mix.set_send_muted(ch_i, muted) {
            return ServerMessage::Error {
                code: "SEND_ERROR".to_owned(),
                message: e.to_string(),
            };
        }
        let send = mix.send(ch_i);
        ServerMessage::SendAck {
            mix_index,
            channel_index,
            gain_db: send.map_or(0.0, mix_engine::MixSend::gain_db),
            pan: send.map_or(0.0, mix_engine::MixSend::pan),
            muted: send.map_or(muted, |s| s.muted),
            revision: mix.revision(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use control_protocol::decode_client_message;
    use mix_engine::Mix;

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

    // --- Phase 16: send mutation dispatch ---

    fn state_with_mix() -> ControlState {
        let mut state = ControlState::new();
        let mix = Mix::new(0, "Monitor 1");
        state.engine.set_mix(0, mix).expect("set mix 0");
        state
    }

    #[test]
    fn set_send_gain_returns_send_ack() {
        let mut state = state_with_mix();
        let resp = state.dispatch(request(ClientMessage::SetSendGain {
            mix_index: 0,
            channel_index: 0,
            gain_db: -6.0,
        }));
        match resp.payload {
            ServerMessage::SendAck {
                mix_index,
                channel_index,
                gain_db,
                ..
            } => {
                assert_eq!(mix_index, 0);
                assert_eq!(channel_index, 0);
                assert!((gain_db - (-6.0)).abs() < 1e-5);
            }
            other => panic!("expected SendAck, got {other:?}"),
        }
    }

    #[test]
    fn set_send_pan_returns_send_ack() {
        let mut state = state_with_mix();
        let resp = state.dispatch(request(ClientMessage::SetSendPan {
            mix_index: 0,
            channel_index: 1,
            pan: 0.75,
        }));
        match resp.payload {
            ServerMessage::SendAck { pan, .. } => {
                assert!((pan - 0.75).abs() < 1e-5);
            }
            other => panic!("expected SendAck, got {other:?}"),
        }
    }

    #[test]
    fn set_send_muted_returns_send_ack() {
        let mut state = state_with_mix();
        let resp = state.dispatch(request(ClientMessage::SetSendMuted {
            mix_index: 0,
            channel_index: 2,
            muted: true,
        }));
        match resp.payload {
            ServerMessage::SendAck { muted, .. } => assert!(muted),
            other => panic!("expected SendAck, got {other:?}"),
        }
    }

    #[test]
    fn set_send_gain_missing_mix_returns_error() {
        let mut state = ControlState::new(); // no mixes configured
        let resp = state.dispatch(request(ClientMessage::SetSendGain {
            mix_index: 0,
            channel_index: 0,
            gain_db: 0.0,
        }));
        assert!(matches!(
            resp.payload,
            ServerMessage::Error { ref code, .. } if code == "MIX_NOT_FOUND"
        ));
    }

    #[test]
    fn set_send_gain_nan_returns_error() {
        let mut state = state_with_mix();
        let resp = state.dispatch(request(ClientMessage::SetSendGain {
            mix_index: 0,
            channel_index: 0,
            gain_db: f32::NAN,
        }));
        assert!(matches!(
            resp.payload,
            ServerMessage::Error { ref code, .. } if code == "INVALID_GAIN"
        ));
    }

    #[test]
    fn set_send_pan_out_of_range_returns_error() {
        let mut state = state_with_mix();
        let resp = state.dispatch(request(ClientMessage::SetSendPan {
            mix_index: 0,
            channel_index: 0,
            pan: 2.0, // out of range
        }));
        assert!(matches!(
            resp.payload,
            ServerMessage::Error { ref code, .. } if code == "INVALID_PAN"
        ));
    }
}
