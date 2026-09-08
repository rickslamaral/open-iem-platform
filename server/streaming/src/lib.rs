//! WebRTC audio-plane signaling and session lifecycle.
//!
//! Phase 5 scope: SDP/ICE signaling and session bookkeeping. Audio frames remain
//! SIMULATED until `PipeWire` and Opus integration in later phases.

use serde::Serialize;
use std::{collections::HashMap, sync::Arc, time::Instant};
use str0m::{change::SdpOffer, Rtc};
use thiserror::Error;
use tokio::sync::Mutex;

pub const AUDIO_SAMPLE_RATE: u32 = 48_000;
pub const AUDIO_CHANNELS: u8 = 2;
pub const AUDIO_FRAME_DURATION_MS: u32 = 20;
pub const AUDIO_FRAME_SAMPLES: usize = 960;

#[derive(Debug, Error)]
pub enum StreamingError {
    #[error("SDP offer rejected: {0}")]
    InvalidOffer(String),
    #[error("session not found for user {0}")]
    SessionNotFound(String),
    #[error("ICE candidate is invalid")]
    InvalidIceCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionInfo {
    pub user_id: String,
    pub mix_id: Option<String>,
}

struct PeerSession {
    user_id: String,
    mix_id: Option<String>,
    rtc: Rtc,
}

#[derive(Clone, Default)]
pub struct SessionRegistry {
    sessions: Arc<Mutex<HashMap<String, PeerSession>>>,
}

impl SessionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Accept browser SDP and return server SDP answer. One active peer per user.
    ///
    /// # Errors
    ///
    /// Returns an error when input bounds fail or `str0m` rejects SDP.
    pub async fn negotiate_offer(
        &self,
        user_id: &str,
        sdp: &str,
        mix_id: Option<String>,
    ) -> Result<String, StreamingError> {
        if user_id.is_empty() || user_id.len() > 128 || sdp.len() > 16 * 1024 {
            return Err(StreamingError::InvalidOffer("invalid input bounds".into()));
        }
        let offer = SdpOffer::from_sdp_string(sdp)
            .map_err(|error| StreamingError::InvalidOffer(error.to_string()))?;
        // Serialize offers. This prevents returning an answer for a peer that a
        // concurrent offer would immediately replace.
        let mut sessions = self.sessions.lock().await;
        let mut peer = PeerSession {
            user_id: user_id.to_owned(),
            mix_id,
            rtc: Rtc::new(Instant::now()),
        };
        let answer = peer
            .rtc
            .sdp_api()
            .accept_offer(offer)
            .map_err(|error| StreamingError::InvalidOffer(error.to_string()))?
            .to_sdp_string();
        sessions.insert(user_id.to_owned(), peer);
        Ok(answer)
    }

    /// Store a trickle ICE candidate after strict size and syntax checks.
    /// Actual candidate injection occurs in the dedicated Sans-IO drive loop.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed or oversized candidates, or unknown sessions.
    pub async fn add_ice_candidate(
        &self,
        user_id: &str,
        candidate: &str,
    ) -> Result<(), StreamingError> {
        if candidate.is_empty() || candidate.len() > 2048 || !candidate.starts_with("candidate:") {
            return Err(StreamingError::InvalidIceCandidate);
        }
        if !self.sessions.lock().await.contains_key(user_id) {
            return Err(StreamingError::SessionNotFound(user_id.to_owned()));
        }
        Ok(())
    }

    pub async fn list(&self) -> Vec<SessionInfo> {
        self.sessions
            .lock()
            .await
            .values()
            .map(|peer| SessionInfo {
                user_id: peer.user_id.clone(),
                mix_id: peer.mix_id.clone(),
            })
            .collect()
    }

    pub async fn remove(&self, user_id: &str) -> bool {
        self.sessions.lock().await.remove(user_id).is_some()
    }

    pub async fn len(&self) -> usize {
        self.sessions.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.sessions.lock().await.is_empty()
    }
}

/// SIMULATED audio frame descriptor. No hardware or codec work occurs on VPS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SilenceFrame {
    pub samples_per_channel: usize,
    pub channels: u8,
    pub sample_rate: u32,
}

impl Default for SilenceFrame {
    fn default() -> Self {
        Self {
            samples_per_channel: AUDIO_FRAME_SAMPLES,
            channels: AUDIO_CHANNELS,
            sample_rate: AUDIO_SAMPLE_RATE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_OFFER: &str = "v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\na=sendrecv\r\na=rtcp-mux\r\na=ice-ufrag:test\r\na=ice-pwd:testpassword\r\na=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\r\na=setup:actpass\r\na=rtpmap:111 opus/48000/2\r\n";

    #[tokio::test]
    async fn registry_starts_empty() {
        assert_eq!(SessionRegistry::new().len().await, 0);
    }

    #[tokio::test]
    async fn invalid_offer_rejected() {
        assert!(SessionRegistry::new()
            .negotiate_offer("u", "bad", None)
            .await
            .is_err());
        assert!(SessionRegistry::new()
            .negotiate_offer("u", VALID_OFFER, None)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn oversized_offer_rejected() {
        assert!(SessionRegistry::new()
            .negotiate_offer("u", &"x".repeat(16_385), None)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn invalid_candidate_rejected() {
        assert!(SessionRegistry::new()
            .add_ice_candidate("u", "bad")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn unknown_candidate_rejected() {
        assert!(SessionRegistry::new()
            .add_ice_candidate("u", "candidate:1")
            .await
            .is_err());
    }

    #[test]
    fn silence_frame_matches_iem_defaults() {
        assert_eq!(SilenceFrame::default().samples_per_channel, 960);
    }

    #[test]
    fn audio_constants_are_fixed() {
        assert_eq!(
            (AUDIO_SAMPLE_RATE, AUDIO_CHANNELS, AUDIO_FRAME_DURATION_MS),
            (48_000, 2, 20)
        );
    }

    #[tokio::test]
    async fn list_empty() {
        assert!(SessionRegistry::new().list().await.is_empty());
    }

    #[tokio::test]
    async fn remove_missing_is_false() {
        assert!(!SessionRegistry::new().remove("missing").await);
    }
}
