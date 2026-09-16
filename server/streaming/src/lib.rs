//! WebRTC audio-plane signaling and session lifecycle.
//!
//! Phase 5 scope: SDP/ICE signaling and session bookkeeping. Audio frames remain
//! SIMULATED until `PipeWire` and Opus integration in later phases.
//!
//! Phase 6 adds:
//! - Real trickle-ICE candidate injection via `Candidate::from_sdp_string` +
//!   `Rtc::add_remote_candidate` in the Sans-IO session.
//! - Oversized candidate rejection (>2048 bytes).

pub mod clock;
pub mod media_bridge;
pub mod media_plane;
pub mod opus_receiver;
pub mod pairing;
pub use media_bridge::{MediaBridge, MediaBridgeError, MEDIA_BRIDGE_CAPACITY};
pub use media_plane::{
    MediaFrame, MediaPlane, MediaPlaneError, MediaSession, MediaSessionError, StreamMetadata,
    MEDIA_QUEUE_CAPACITY,
};
pub use opus_receiver::{
    AudioOutput, JitterBuffer, OpusReceiver, ReceiverError, ReceiverState, RECEIVER_QUEUE_CAPACITY,
};
pub use pairing::{DeviceIdentity, PairingError, PairingRegistry};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc, time::Instant};
use str0m::{change::SdpOffer, Candidate, Output, Rtc};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::debug;

pub const AUDIO_SAMPLE_RATE: u32 = 48_000;
pub const AUDIO_CHANNELS: u8 = 2;
pub const AUDIO_FRAME_DURATION_MS: u32 = 20;
pub const AUDIO_FRAME_SAMPLES: usize = 960;

/// Maximum SDP body size accepted (16 KiB).
const MAX_SDP_BYTES: usize = 16 * 1024;
/// Maximum ICE candidate string size accepted.
const MAX_CANDIDATE_BYTES: usize = 2048;
/// Maximum user ID length.
const MAX_USER_ID_BYTES: usize = 128;

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

/// Bounded result of one simulated Sans-IO drive pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DriveReport {
    pub frames_drained: usize,
    pub outputs_polled: usize,
    pub transmitted_bytes: usize,
    pub budget_exhausted: bool,
}

struct PeerSession {
    user_id: String,
    mix_id: Option<String>,
    /// Sans-IO WebRTC peer. Holds ICE/DTLS/SRTP state.
    ///
    /// SIMULATED on VPS: no real network I/O occurs; candidates are stored for
    /// future Raspberry Pi use but no `poll_output` loop runs in this phase.
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
        if user_id.is_empty() || user_id.len() > MAX_USER_ID_BYTES || sdp.len() > MAX_SDP_BYTES {
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

    /// Inject a trickle ICE candidate into the Sans-IO peer session.
    ///
    /// The candidate string must start with `candidate:` (no `a=` prefix, no trailing
    /// newline) and must not exceed 2048 bytes. Only valid RFC 5245 candidate strings
    /// are accepted; `str0m` validates the full structure.
    ///
    /// **SIMULATED on VPS:** the candidate is parsed and stored in the `Rtc` object.
    /// The `poll_output` drive loop that would actually perform network I/O is deferred
    /// to the Raspberry Pi / real hardware phase.
    ///
    /// # Errors
    ///
    /// Returns `StreamingError::InvalidIceCandidate` for malformed, oversized, or
    /// unparseable candidates, and `StreamingError::SessionNotFound` when no session
    /// exists for the given user.
    pub async fn add_ice_candidate(
        &self,
        user_id: &str,
        candidate: &str,
    ) -> Result<(), StreamingError> {
        // Fast structural checks before acquiring the lock.
        if candidate.is_empty()
            || candidate.len() > MAX_CANDIDATE_BYTES
            || !candidate.starts_with("candidate:")
        {
            return Err(StreamingError::InvalidIceCandidate);
        }

        let parsed = Candidate::from_sdp_string(candidate)
            .map_err(|_| StreamingError::InvalidIceCandidate)?;

        let mut sessions = self.sessions.lock().await;
        let peer = sessions
            .get_mut(user_id)
            .ok_or_else(|| StreamingError::SessionNotFound(user_id.to_owned()))?;

        peer.rtc.add_remote_candidate(parsed);
        debug!(user_id, "trickle ICE candidate added to peer session");
        Ok(())
    }

    /// Drain bounded engine frames, then poll each Sans-IO peer without network I/O.
    ///
    /// `Transmit` output is counted and discarded deliberately. A future UDP
    /// adapter owns that output; this method does not claim media runtime support.
    pub async fn drive_once(
        &self,
        bridge: &crate::media_bridge::MediaBridge,
        media_plane: &crate::media_plane::MediaPlane,
        frame_budget: usize,
        output_budget: usize,
    ) -> DriveReport {
        let frames_drained = bridge.drain_to_with_budget(media_plane, frame_budget).await;
        let mut sessions = self.sessions.lock().await;
        let mut outputs_polled = 0;
        let mut transmitted_bytes = 0;
        let mut budget_exhausted = false;

        for peer in sessions.values_mut() {
            while outputs_polled < output_budget {
                match peer.rtc.poll_output() {
                    Ok(Output::Timeout(_)) | Err(_) => break,
                    Ok(Output::Transmit(transmit)) => {
                        transmitted_bytes += transmit.contents.len();
                        outputs_polled += 1;
                    }
                    Ok(Output::Event(_)) => outputs_polled += 1,
                }
            }
            if output_budget > 0 && outputs_polled >= output_budget {
                budget_exhausted = true;
                break;
            }
        }

        DriveReport {
            frames_drained,
            outputs_polled,
            transmitted_bytes,
            budget_exhausted,
        }
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

pub use clock::{AdaptiveResampler, DriftEstimator, SampleTimestamp, NOMINAL_SAMPLE_RATE};

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_OFFER: &str = "v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 111\r\nc=IN IP4 0.0.0.0\r\na=mid:0\r\na=sendrecv\r\na=rtcp-mux\r\na=ice-ufrag:test\r\na=ice-pwd:testpassword\r\na=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\r\na=setup:actpass\r\na=rtpmap:111 opus/48000/2\r\n";

    /// Valid trickle ICE candidate in RFC 5245 / browser format (`candidate:` prefix, no `a=`).
    const VALID_CANDIDATE: &str =
        "candidate:1 1 udp 2113937151 192.168.1.100 49152 typ host generation 0";

    #[tokio::test]
    async fn drive_once_zero_output_budget_is_not_exhausted() {
        let report = SessionRegistry::new()
            .drive_once(
                &crate::media_bridge::MediaBridge::new(),
                &crate::media_plane::MediaPlane::new(),
                0,
                0,
            )
            .await;
        assert_eq!(report.outputs_polled, 0);
        assert!(!report.budget_exhausted);
    }

    #[tokio::test]
    async fn drive_once_routes_bounded_bridge_frames() {
        let registry = SessionRegistry::new();
        let plane = crate::media_plane::MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        let bridge = crate::media_bridge::MediaBridge::new();
        bridge
            .try_send(
                mix_engine::FrameOutput {
                    mixes: [(0.5, 0.25), (0.0, 0.0)],
                },
                3,
            )
            .unwrap();

        let report = registry.drive_once(&bridge, &plane, 1, 1).await;
        assert_eq!(report.frames_drained, 1);
        assert_eq!(report.outputs_polled, 0);
        assert!(!report.budget_exhausted);
        let sessions = plane.sessions.lock().await;
        assert_eq!(sessions["alice"].drain_frames().len(), 1);
    }

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
        // "candidate:1" starts with the prefix but no session exists — SessionNotFound
        let err = SessionRegistry::new()
            .add_ice_candidate("u", "candidate:1 1 udp 123 127.0.0.1 1234 typ host")
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn malformed_candidate_rejected_before_session_lookup() {
        // No session for "u", but candidate is malformed — should fail as InvalidIceCandidate
        // (fast-path check before lock).
        let registry = SessionRegistry::new();
        let err = registry.add_ice_candidate("u", "not-a-candidate").await;
        assert!(matches!(err, Err(StreamingError::InvalidIceCandidate)));
    }

    #[tokio::test]
    async fn oversized_candidate_rejected() {
        let registry = SessionRegistry::new();
        let big = format!("candidate:{}", "x".repeat(2049));
        let err = registry.add_ice_candidate("u", &big).await;
        assert!(matches!(err, Err(StreamingError::InvalidIceCandidate)));
    }

    #[tokio::test]
    async fn valid_candidate_injected_after_offer() {
        let registry = SessionRegistry::new();
        // Establish a session first.
        registry
            .negotiate_offer("alice", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        // Inject a valid trickle ICE candidate.
        registry
            .add_ice_candidate("alice", VALID_CANDIDATE)
            .await
            .expect("valid candidate must be accepted");
        // Session still exists.
        assert_eq!(registry.len().await, 1);
    }

    #[tokio::test]
    async fn candidate_rejected_when_no_session() {
        let registry = SessionRegistry::new();
        let err = registry
            .add_ice_candidate("no-such-user", VALID_CANDIDATE)
            .await;
        assert!(matches!(err, Err(StreamingError::SessionNotFound(_))));
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
