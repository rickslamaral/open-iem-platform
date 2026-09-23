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
pub mod media_writer;
pub mod opus_receiver;
pub mod pairing;
pub mod transport;
pub use media_bridge::{MediaBridge, MediaBridgeError, MEDIA_BRIDGE_CAPACITY};
pub use media_plane::{
    MediaFrame, MediaPlane, MediaPlaneError, MediaSession, MediaSessionError, StreamMetadata,
    MEDIA_QUEUE_CAPACITY,
};
pub use media_writer::{MediaPacket, MediaWriter, MediaWriterError, OPUS_MAX_PACKET_BYTES};
pub use opus_receiver::{
    AudioOutput, JitterBuffer, OpusReceiver, OutputError, ReceiverError, ReceiverState,
    RECEIVER_QUEUE_CAPACITY,
};
pub use pairing::{DeviceIdentity, PairingError, PairingRegistry};
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Instant,
};
use str0m::{change::SdpOffer, Candidate, Event, Output, Rtc};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::debug;
pub use transport::{TransportAdapter, TransportSendReport, TRANSPORT_SEND_BUDGET};

pub const AUDIO_SAMPLE_RATE: u32 = 48_000;
pub const AUDIO_CHANNELS: u8 = 2;
pub const AUDIO_FRAME_DURATION_MS: u32 = 20;
pub const AUDIO_FRAME_SAMPLES: usize = 960;
/// Maximum number of Sans-IO datagrams retained for the transport adapter.
pub const TRANSPORT_OUTPUT_CAPACITY: usize = 128;

/// Maximum SDP body size accepted (16 KiB).
const MAX_SDP_BYTES: usize = 16 * 1024;
/// Maximum ICE candidate string size accepted.
const MAX_CANDIDATE_BYTES: usize = 2048;
/// Maximum user ID length.
const MAX_USER_ID_BYTES: usize = 128;
/// Maximum persisted mix identifier length.
const MAX_MIX_ID_BYTES: usize = 128;

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
    pub device_id: Option<String>,
}

/// Bounded result of one simulated Sans-IO drive pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DriveReport {
    pub frames_drained: usize,
    pub outputs_polled: usize,
    pub transmitted_bytes: usize,
    pub budget_exhausted: bool,
    pub packets_encoded: usize,
    pub transport_outputs_dropped: usize,
    pub poll_errors: usize,
}

struct PeerSession {
    user_id: String,
    mix_id: Option<String>,
    device_id: Option<String>,
    /// Sans-IO WebRTC peer. Holds ICE/DTLS/SRTP state.
    ///
    /// SIMULATED on VPS: no real network I/O occurs; candidates are stored for
    /// future Raspberry Pi use and `poll_output` output stays in the bounded queue.
    rtc: Rtc,
    media_mid: Option<str0m::media::Mid>,
    writer: MediaWriter,
}

#[derive(Clone, Default)]
pub struct SessionRegistry {
    sessions: Arc<Mutex<HashMap<String, PeerSession>>>,
    transport_outputs: Arc<Mutex<VecDeque<str0m::net::Transmit>>>,
}

pub(crate) fn canonicalize_dtls_fingerprint(value: &str) -> Result<String, StreamingError> {
    let canonical = value.trim().to_ascii_lowercase();
    let (algorithm, digest) = canonical
        .split_once(' ')
        .ok_or_else(|| StreamingError::InvalidOffer("invalid DTLS fingerprint".into()))?;
    if algorithm != "sha-256"
        || digest.len() != 95
        || digest
            .bytes()
            .enumerate()
            .any(|(i, b)| (i % 3 == 2 && b != b':') || (i % 3 != 2 && !b.is_ascii_hexdigit()))
    {
        return Err(StreamingError::InvalidOffer(
            "invalid DTLS fingerprint".into(),
        ));
    }
    Ok(format!("{algorithm} {digest}"))
}

fn extract_dtls_fingerprint(sdp: &str) -> Result<String, StreamingError> {
    let fingerprints = sdp
        .lines()
        .filter_map(|line| line.strip_prefix("a=fingerprint:"))
        .map(canonicalize_dtls_fingerprint)
        .collect::<Result<Vec<_>, _>>()?;
    let fingerprint = fingerprints
        .first()
        .ok_or_else(|| StreamingError::InvalidOffer("missing a=fingerprint".into()))?;
    if fingerprints.iter().any(|value| value != fingerprint) {
        return Err(StreamingError::InvalidOffer(
            "conflicting DTLS fingerprints".into(),
        ));
    }
    Ok(fingerprint.clone())
}

impl SessionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            transport_outputs: Arc::new(Mutex::new(VecDeque::with_capacity(
                TRANSPORT_OUTPUT_CAPACITY,
            ))),
        }
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
        self.negotiate_offer_bound(user_id, sdp, mix_id, None).await
    }

    /// # Errors
    ///
    /// Returns [`StreamingError::InvalidOffer`] when bounds, identity binding, or SDP validation fails.
    pub async fn negotiate_offer_bound(
        &self,
        user_id: &str,
        sdp: &str,
        mix_id: Option<String>,
        identity: Option<&DeviceIdentity>,
    ) -> Result<String, StreamingError> {
        if user_id.is_empty()
            || user_id.len() > MAX_USER_ID_BYTES
            || sdp.len() > MAX_SDP_BYTES
            || mix_id
                .as_ref()
                .is_some_and(|id| id.len() > MAX_MIX_ID_BYTES)
        {
            return Err(StreamingError::InvalidOffer("invalid input bounds".into()));
        }
        if let Some(identity) = identity {
            if identity.revoked
                || identity.musician_id != user_id
                || mix_id
                    .as_deref()
                    .is_some_and(|mix| mix.parse::<usize>().ok() != Some(identity.mix_index))
            {
                return Err(StreamingError::InvalidOffer(
                    "device identity does not match session".into(),
                ));
            }
        }
        let offer = SdpOffer::from_sdp_string(sdp)
            .map_err(|error| StreamingError::InvalidOffer(error.to_string()))?;
        if let Some(identity) = identity {
            if let Some(expected_fingerprint) = identity.dtls_fingerprint.as_deref() {
                let offered_fingerprint = extract_dtls_fingerprint(sdp)?;
                if expected_fingerprint != offered_fingerprint {
                    return Err(StreamingError::InvalidOffer(
                        "DTLS fingerprint does not match paired device".into(),
                    ));
                }
            }
        }
        // Serialize offers. This prevents returning an answer for a peer that a
        // concurrent offer would immediately replace.
        let mut sessions = self.sessions.lock().await;
        let mut peer = PeerSession {
            user_id: user_id.to_owned(),
            mix_id,
            device_id: identity.map(|value| value.device_id.clone()),
            rtc: Rtc::new(Instant::now()),
            media_mid: None,
            writer: MediaWriter::new().map_err(|_| {
                StreamingError::InvalidOffer("media writer initialization failed".into())
            })?,
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
        if user_id.is_empty()
            || user_id.len() > MAX_USER_ID_BYTES
            || candidate.is_empty()
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

    /// Drain bounded engine frames, then consume bounded session frames and poll
    /// each Sans-IO peer without network I/O. `frame_budget` applies separately
    /// to bridge input and session output stages.
    ///
    /// `Transmit` output is retained in a bounded queue for an external transport
    /// adapter; this method does not perform network I/O or claim runtime support.
    #[allow(clippy::too_many_lines)]
    pub async fn drive_once(
        &self,
        bridge: &crate::media_bridge::MediaBridge,
        media_plane: &crate::media_plane::MediaPlane,
        frame_budget: usize,
        output_budget: usize,
    ) -> DriveReport {
        if output_budget == 0 {
            return DriveReport::default();
        }
        let frames_drained = bridge.drain_to_with_budget(media_plane, frame_budget).await;
        let session_ids = self
            .sessions
            .lock()
            .await
            .values()
            .filter(|peer| peer.media_mid.is_some())
            .map(|peer| peer.user_id.clone())
            .collect::<Vec<_>>();
        let mut drained = HashMap::new();
        for user_id in session_ids {
            if let Ok(frames) = media_plane
                .drain_session_frames_with_budget(&user_id, frame_budget)
                .await
            {
                drained.insert(user_id, frames);
            }
        }

        let mut sessions = self.sessions.lock().await;
        let mut outputs_polled = 0;
        let mut transmitted_bytes = 0;
        let mut packets_encoded = 0;
        let mut budget_exhausted = false;
        let mut transport_outputs_dropped = 0;
        let mut poll_errors = 0;

        for peer in sessions.values_mut() {
            while outputs_polled < output_budget {
                match peer.rtc.poll_output() {
                    Ok(Output::Timeout(_)) => break,
                    Err(_) => {
                        poll_errors += 1;
                        break;
                    }
                    Ok(Output::Transmit(transmit)) => {
                        let mut outputs = self.transport_outputs.lock().await;
                        if outputs.len() < TRANSPORT_OUTPUT_CAPACITY {
                            transmitted_bytes += transmit.contents.len();
                            outputs.push_back(transmit);
                        } else {
                            transport_outputs_dropped += 1;
                        }
                        outputs_polled += 1;
                    }
                    Ok(Output::Event(event)) => {
                        if let Event::MediaAdded(media) = event {
                            if media.kind.is_audio() {
                                peer.media_mid = Some(media.mid);
                            }
                        }
                        outputs_polled += 1;
                    }
                }
            }
            if let Some(mid) = peer.media_mid {
                if let Some(frames) = drained.remove(&peer.user_id) {
                    for frame in frames {
                        if packets_encoded >= output_budget {
                            budget_exhausted = true;
                            break;
                        }
                        let Ok(packet) = peer.writer.encode(&frame) else {
                            continue;
                        };
                        let Some(media_writer) = peer.rtc.writer(mid) else {
                            continue;
                        };
                        let Some(pt) = media_writer
                            .payload_params()
                            .find(|params| params.spec().codec == str0m::format::Codec::Opus)
                            .map(str0m::format::PayloadParams::pt)
                        else {
                            continue;
                        };
                        if media_writer
                            .write(
                                pt,
                                Instant::now(),
                                str0m::media::MediaTime::new(
                                    u64::from(packet.rtp_timestamp),
                                    str0m::media::Frequency::FORTY_EIGHT_KHZ,
                                ),
                                packet.payload,
                            )
                            .is_ok()
                        {
                            packets_encoded += 1;
                        }
                        if outputs_polled < output_budget {
                            match peer.rtc.poll_output() {
                                Ok(Output::Transmit(transmit)) => {
                                    let mut outputs = self.transport_outputs.lock().await;
                                    if outputs.len() < TRANSPORT_OUTPUT_CAPACITY {
                                        transmitted_bytes += transmit.contents.len();
                                        outputs.push_back(transmit);
                                    } else {
                                        transport_outputs_dropped += 1;
                                    }
                                    outputs_polled += 1;
                                }
                                Ok(Output::Event(event)) => {
                                    if let Event::MediaAdded(media) = event {
                                        if media.kind.is_audio() {
                                            peer.media_mid = Some(media.mid);
                                        }
                                    }
                                    outputs_polled += 1;
                                }
                                Ok(Output::Timeout(_)) => {}
                                Err(_) => {
                                    poll_errors += 1;
                                }
                            }
                        }
                    }
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
            packets_encoded,
            transport_outputs_dropped,
            poll_errors,
        }
    }

    /// Transfer bounded Sans-IO datagrams to owner of real socket I/O.
    pub async fn drain_transport_outputs(&self, budget: usize) -> Vec<str0m::net::Transmit> {
        let mut outputs = self.transport_outputs.lock().await;
        let count = outputs
            .len()
            .min(budget.min(crate::transport::TRANSPORT_SEND_BUDGET));
        outputs.drain(..count).collect()
    }

    /// Return unsent datagrams to the front of the bounded transport queue.
    pub async fn requeue_transport_outputs(&self, mut outputs: Vec<str0m::net::Transmit>) -> usize {
        let mut queue = self.transport_outputs.lock().await;
        let mut dropped = 0;
        while let Some(output) = outputs.pop() {
            if queue.len() < TRANSPORT_OUTPUT_CAPACITY {
                queue.push_front(output);
            } else {
                dropped += 1;
            }
        }
        dropped
    }

    pub async fn list(&self) -> Vec<SessionInfo> {
        self.sessions
            .lock()
            .await
            .values()
            .map(|peer| SessionInfo {
                user_id: peer.user_id.clone(),
                mix_id: peer.mix_id.clone(),
                device_id: peer.device_id.clone(),
            })
            .collect()
    }

    pub async fn remove(&self, user_id: &str) -> bool {
        self.sessions.lock().await.remove(user_id).is_some()
    }

    pub async fn remove_by_device_id(&self, device_id: &str) -> usize {
        let mut sessions = self.sessions.lock().await;
        let before = sessions.len();
        sessions.retain(|_, peer| peer.device_id.as_deref() != Some(device_id));
        before - sessions.len()
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
    use std::net::SocketAddr;
    use str0m::net::Protocol;

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
                None,
            )
            .unwrap();

        let report = registry.drive_once(&bridge, &plane, 2, 1).await;
        assert_eq!(report.frames_drained, 1);
        assert_eq!(report.outputs_polled, 0);
        assert!(!report.budget_exhausted);
        let sessions = plane.sessions.lock().await;
        assert_eq!(sessions["alice"].drain_frames().len(), 1);
    }

    #[tokio::test]
    async fn drive_once_keeps_frames_until_audio_media_is_ready() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("alice", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        let plane = crate::media_plane::MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        let bridge = crate::media_bridge::MediaBridge::new();
        bridge
            .try_send(
                mix_engine::FrameOutput {
                    mixes: [(0.5, -0.25), (0.0, 0.0)],
                },
                11,
                None,
            )
            .unwrap();

        registry.drive_once(&bridge, &plane, 1, 1).await;

        let sessions = plane.sessions.lock().await;
        assert_eq!(sessions["alice"].drain_frames().len(), 1);
    }

    #[tokio::test]
    async fn drive_once_negotiated_session_applies_stage_budgets() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("alice", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        let plane = crate::media_plane::MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        let bridge = crate::media_bridge::MediaBridge::new();
        for revision in [11, 12] {
            bridge
                .try_send(
                    mix_engine::FrameOutput {
                        mixes: [(0.5, -0.25), (0.0, 0.0)],
                    },
                    revision,
                    None,
                )
                .unwrap();
        }

        // Each drive pass consumes at most one bridge frame and polls at most
        // one output. No network I/O is required for this Sans-IO boundary.
        let first = registry.drive_once(&bridge, &plane, 1, 1).await;
        let second = registry.drive_once(&bridge, &plane, 1, 1).await;

        assert_eq!(first.frames_drained, 1);
        assert_eq!(second.frames_drained, 1);
        assert!(first.outputs_polled <= 1);
        assert!(second.outputs_polled <= 1);
        assert!(registry.drain_transport_outputs(8).await.len() <= 8);
    }

    #[tokio::test]
    async fn transport_output_drain_respects_budget() {
        let registry = SessionRegistry::new();
        assert!(registry.drain_transport_outputs(1).await.is_empty());
    }

    #[tokio::test]
    async fn transport_adapter_zero_budget_preserves_registry_output() {
        let registry = SessionRegistry::new();
        registry
            .transport_outputs
            .lock()
            .await
            .push_back(str0m::net::Transmit {
                proto: Protocol::Udp,
                source: "127.0.0.1:0".parse().unwrap(),
                destination: "127.0.0.1:9".parse().unwrap(),
                contents: b"must-remain-queued".to_vec().into(),
            });
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();

        let report = adapter.send_from_registry(&registry, 0).await.unwrap();

        assert_eq!(report.attempted, 0);
        assert_eq!(report.sent, 0);
        assert_eq!(report.bytes, 0);
        assert_eq!(report.dropped, 0);
        let queued = registry.drain_transport_outputs(1).await;
        assert_eq!(queued.len(), 1);
        assert_eq!(&queued[0].contents[..], b"must-remain-queued");
    }

    #[tokio::test]
    async fn transport_adapter_caps_oversized_registry_budget() {
        let registry = SessionRegistry::new();
        let receiver = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let destination = receiver.local_addr().unwrap();
        let transmit = |index: usize| str0m::net::Transmit {
            proto: Protocol::Udp,
            source: "127.0.0.1:0".parse().unwrap(),
            destination,
            contents: format!("packet-{index}").into_bytes().into(),
        };
        let mut outputs = registry.transport_outputs.lock().await;
        for index in 0..=TRANSPORT_SEND_BUDGET {
            outputs.push_back(transmit(index));
        }
        drop(outputs);

        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let report = adapter
            .send_from_registry(&registry, usize::MAX)
            .await
            .unwrap();

        assert_eq!(report.attempted, TRANSPORT_SEND_BUDGET);
        assert_eq!(report.sent, TRANSPORT_SEND_BUDGET);
        assert_eq!(report.dropped, 0);
        let queued = registry.drain_transport_outputs(usize::MAX).await;
        assert_eq!(queued.len(), 1);
        assert_eq!(&queued[0].contents[..], b"packet-32");

        let mut payload = [0_u8; 32];
        for index in 0..TRANSPORT_SEND_BUDGET {
            let (length, _) = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                receiver.recv_from(&mut payload),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(&payload[..length], format!("packet-{index}").as_bytes());
        }
    }

    #[tokio::test]
    async fn transport_adapter_sends_registry_output_and_reports_delivery() {
        let registry = SessionRegistry::new();
        let receiver = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let destination: SocketAddr = receiver.local_addr().unwrap();
        registry
            .transport_outputs
            .lock()
            .await
            .push_back(str0m::net::Transmit {
                proto: Protocol::Udp,
                source: "127.0.0.1:0".parse().unwrap(),
                destination,
                contents: b"registry-output".to_vec().into(),
            });
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();

        let report = adapter.send_from_registry(&registry, 1).await.unwrap();
        assert_eq!(report.attempted, 1);
        assert_eq!(report.sent, 1);
        assert_eq!(report.bytes, 15);
        assert_eq!(report.dropped, 0);
        assert!(registry.drain_transport_outputs(1).await.is_empty());

        let mut payload = [0_u8; 32];
        let (length, _) = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            receiver.recv_from(&mut payload),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(&payload[..length], b"registry-output");
    }

    #[tokio::test]
    async fn transport_adapter_requeues_failed_registry_output() {
        let registry = SessionRegistry::new();
        let destination: SocketAddr = "[::1]:9".parse().unwrap();
        let transmit = str0m::net::Transmit {
            proto: Protocol::Udp,
            source: "127.0.0.1:0".parse().unwrap(),
            destination,
            contents: b"retry-me".to_vec().into(),
        };
        registry.transport_outputs.lock().await.push_back(transmit);
        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();

        let result = adapter.send_from_registry(&registry, 1).await;
        assert!(result.is_err());
        let queued = registry.drain_transport_outputs(1).await;
        assert_eq!(queued.len(), 1);
        assert_eq!(&queued[0].contents[..], b"retry-me");
    }

    #[tokio::test]
    async fn transport_adapter_requeues_failed_suffix_in_order() {
        let registry = SessionRegistry::new();
        let receiver = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let valid_destination = receiver.local_addr().unwrap();
        let invalid_destination: SocketAddr = "[::1]:9".parse().unwrap();
        let transmit = |destination, payload: &str| str0m::net::Transmit {
            proto: Protocol::Udp,
            source: "127.0.0.1:0".parse().unwrap(),
            destination,
            contents: payload.as_bytes().to_vec().into(),
        };
        let mut outputs = registry.transport_outputs.lock().await;
        outputs.push_back(transmit(valid_destination, "first"));
        outputs.push_back(transmit(invalid_destination, "second"));
        outputs.push_back(transmit(valid_destination, "third"));
        drop(outputs);

        let adapter = TransportAdapter::bind("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        assert!(adapter.send_from_registry(&registry, 3).await.is_err());

        let mut payload = [0_u8; 16];
        let (length, _) = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            receiver.recv_from(&mut payload),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(&payload[..length], b"first");

        let queued = registry.drain_transport_outputs(3).await;
        assert_eq!(queued.len(), 2);
        assert_eq!(&queued[0].contents[..], b"second");
        assert_eq!(&queued[1].contents[..], b"third");
    }

    #[tokio::test]
    async fn requeue_transport_outputs_drops_when_queue_is_full() {
        let registry = SessionRegistry::new();
        let transmit = |payload: &[u8]| str0m::net::Transmit {
            proto: Protocol::Udp,
            source: "127.0.0.1:0".parse().unwrap(),
            destination: "127.0.0.1:9".parse().unwrap(),
            contents: payload.to_vec().into(),
        };

        let mut outputs = registry.transport_outputs.lock().await;
        for _ in 0..TRANSPORT_OUTPUT_CAPACITY {
            outputs.push_back(transmit(b"queued"));
        }
        drop(outputs);

        let dropped = registry
            .requeue_transport_outputs(vec![transmit(b"retry")])
            .await;

        assert_eq!(dropped, 1);
        assert_eq!(
            registry.transport_outputs.lock().await.len(),
            TRANSPORT_OUTPUT_CAPACITY
        );
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
    async fn offer_at_maximum_sdp_length_is_accepted() {
        let padding_len = MAX_SDP_BYTES - VALID_OFFER.len();
        let padding = "\r\n".repeat(padding_len / 2);
        let padding = if padding_len.is_multiple_of(2) {
            padding
        } else {
            format!("{padding} ")
        };
        let offer = format!("{VALID_OFFER}{padding}");
        assert_eq!(offer.len(), MAX_SDP_BYTES);

        assert!(SessionRegistry::new()
            .negotiate_offer("u", &offer, None)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn user_id_above_maximum_length_is_rejected() {
        let registry = SessionRegistry::new();
        let user_id = "u".repeat(MAX_USER_ID_BYTES + 1);

        assert!(registry
            .negotiate_offer(&user_id, VALID_OFFER, None)
            .await
            .is_err());
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn mix_id_above_maximum_length_is_rejected() {
        let registry = SessionRegistry::new();
        let mix_id = "m".repeat(MAX_MIX_ID_BYTES + 1);

        assert!(registry
            .negotiate_offer("alice", VALID_OFFER, Some(mix_id))
            .await
            .is_err());
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn mix_id_at_maximum_length_is_accepted() {
        let registry = SessionRegistry::new();
        let mix_id = "m".repeat(MAX_MIX_ID_BYTES);

        registry
            .negotiate_offer("alice", VALID_OFFER, Some(mix_id.clone()))
            .await
            .expect("maximum-length mix ID must be accepted");

        let session = registry
            .list()
            .await
            .into_iter()
            .next()
            .expect("negotiated session must be listed");
        assert_eq!(session.mix_id.as_deref(), Some(mix_id.as_str()));
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
    async fn oversized_candidate_user_id_rejected_without_registry_change() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("alice", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        let user_id = "u".repeat(MAX_USER_ID_BYTES + 1);

        let err = registry.add_ice_candidate(&user_id, VALID_CANDIDATE).await;

        assert!(matches!(err, Err(StreamingError::InvalidIceCandidate)));
        assert_eq!(registry.len().await, 1);
    }

    #[tokio::test]
    async fn maximum_length_candidate_user_id_is_accepted() {
        let registry = SessionRegistry::new();
        let user_id = "u".repeat(MAX_USER_ID_BYTES);

        registry
            .negotiate_offer(&user_id, VALID_OFFER, None)
            .await
            .expect("maximum-length user ID must be accepted");
        registry
            .add_ice_candidate(&user_id, VALID_CANDIDATE)
            .await
            .expect("maximum-length user ID must accept valid candidate");

        assert_eq!(registry.len().await, 1);
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
    fn conflicting_dtls_fingerprints_are_rejected() {
        let first = extract_dtls_fingerprint(VALID_OFFER).unwrap();
        let second = format!("{VALID_OFFER}a=fingerprint:{first}\r\na=fingerprint:sha-256 FF:EE:DD:CC:BB:AA:99:88:77:66:55:44:33:22:11:00:FF:EE:DD:CC:BB:AA:99:88:77:66:55:44:33:22:11:00\r\n");
        assert!(extract_dtls_fingerprint(&second).is_err());
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
    async fn legacy_bound_identity_without_fingerprint_remains_compatible() {
        let registry = SessionRegistry::new();
        let identity = DeviceIdentity {
            device_id: "rx-legacy".into(),
            musician_id: "alice".into(),
            mix_index: 0,
            revoked: false,
            dtls_fingerprint: None,
        };
        registry
            .negotiate_offer_bound("alice", VALID_OFFER, Some("0".into()), Some(&identity))
            .await
            .expect("legacy paired identity must remain compatible");
    }

    #[tokio::test]
    async fn bound_session_keeps_device_identity_and_revoke_removes_it() {
        let registry = SessionRegistry::new();
        let identity = DeviceIdentity {
            device_id: "rx-1".into(),
            musician_id: "alice".into(),
            mix_index: 0,
            revoked: false,
            dtls_fingerprint: extract_dtls_fingerprint(VALID_OFFER).ok(),
        };
        registry
            .negotiate_offer_bound("alice", VALID_OFFER, Some("0".into()), Some(&identity))
            .await
            .unwrap();
        let sessions = registry.list().await;
        assert_eq!(sessions[0].device_id.as_deref(), Some("rx-1"));
        assert_eq!(registry.remove_by_device_id("rx-1").await, 1);
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn bound_session_rejects_revoked_identity_without_mutating_registry() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("existing", VALID_OFFER, None)
            .await
            .unwrap();
        let identity = DeviceIdentity {
            device_id: "rx-1".into(),
            musician_id: "alice".into(),
            mix_index: 0,
            revoked: true,
            dtls_fingerprint: None,
        };
        assert!(registry
            .negotiate_offer_bound("alice", VALID_OFFER, Some("0".into()), Some(&identity))
            .await
            .is_err());
        let sessions = registry.list().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_id, "existing");
    }

    #[tokio::test]
    async fn bound_session_rejects_wrong_musician_or_mix() {
        let registry = SessionRegistry::new();
        let identity = DeviceIdentity {
            device_id: "rx-1".into(),
            musician_id: "alice".into(),
            mix_index: 0,
            revoked: false,
            dtls_fingerprint: Some("sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00".into()),
        };
        assert!(registry
            .negotiate_offer_bound("bob", VALID_OFFER, Some("0".into()), Some(&identity))
            .await
            .is_err());
        assert!(registry
            .negotiate_offer_bound("alice", VALID_OFFER, Some("1".into()), Some(&identity))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn remove_missing_is_false() {
        assert!(!SessionRegistry::new().remove("missing").await);
    }

    #[tokio::test]
    async fn drive_once_distributes_bridge_frames_to_two_sessions() {
        let registry = SessionRegistry::new();
        let plane = crate::media_plane::MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        plane.register_session("bob", 1).await.unwrap();
        let bridge = crate::media_bridge::MediaBridge::new();
        bridge
            .try_send(
                mix_engine::FrameOutput {
                    mixes: [(0.1, 0.2), (0.3, 0.4)],
                },
                1,
                None,
            )
            .unwrap();

        let report = registry.drive_once(&bridge, &plane, 2, 1).await;
        assert_eq!(report.frames_drained, 1);
        assert_eq!(report.outputs_polled, 0);

        let sessions = plane.sessions.lock().await;
        assert_eq!(sessions["alice"].drain_frames().len(), 1);
        assert_eq!(sessions["bob"].drain_frames().len(), 1);
    }

    #[tokio::test]
    async fn drive_once_skips_unnegotiated_session_output_but_routes_frames() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("alice", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        let plane = crate::media_plane::MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        plane.register_session("bob", 1).await.unwrap();
        let bridge = crate::media_bridge::MediaBridge::new();
        bridge
            .try_send(
                mix_engine::FrameOutput {
                    mixes: [(0.5, 0.5), (0.0, 0.0)],
                },
                5,
                None,
            )
            .unwrap();

        let report = registry.drive_once(&bridge, &plane, 2, 2).await;
        assert_eq!(report.frames_drained, 1);
        assert!(report.outputs_polled <= 2);

        let sessions = plane.sessions.lock().await;
        assert_eq!(sessions["alice"].drain_frames().len(), 1);
        assert_eq!(sessions["bob"].drain_frames().len(), 1);
    }

    #[tokio::test]
    async fn drive_once_output_budget_shared_across_sessions() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("alice", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        registry
            .negotiate_offer("bob", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        let plane = crate::media_plane::MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        plane.register_session("bob", 1).await.unwrap();
        let bridge = crate::media_bridge::MediaBridge::new();
        for revision in [20, 21] {
            bridge
                .try_send(
                    mix_engine::FrameOutput {
                        mixes: [(0.1, 0.1), (0.2, 0.2)],
                    },
                    revision,
                    None,
                )
                .unwrap();
        }

        let report = registry.drive_once(&bridge, &plane, 2, 1).await;
        assert_eq!(report.frames_drained, 2);
        assert!(report.outputs_polled <= 1);
    }

    #[tokio::test]
    async fn drive_once_zero_frame_budget_drains_nothing() {
        let registry = SessionRegistry::new();
        let plane = crate::media_plane::MediaPlane::new();
        let bridge = crate::media_bridge::MediaBridge::new();
        bridge
            .try_send(
                mix_engine::FrameOutput {
                    mixes: [(0.1, 0.2), (0.3, 0.4)],
                },
                42,
                None,
            )
            .unwrap();
        let report = registry.drive_once(&bridge, &plane, 0, 1).await;
        assert_eq!(report.frames_drained, 0);
        assert_eq!(report.packets_encoded, 0);
        assert_eq!(report.outputs_polled, 0);
        assert!(!report.budget_exhausted);
        let follow_up = registry.drive_once(&bridge, &plane, 1, 1).await;
        assert_eq!(follow_up.frames_drained, 1);
    }

    #[tokio::test]
    async fn drive_once_packets_encoded_zero_without_negotiated_media() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("carol", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        let plane = crate::media_plane::MediaPlane::new();
        plane.register_session("carol", 0).await.unwrap();
        let bridge = crate::media_bridge::MediaBridge::new();
        for revision in [30, 31] {
            bridge
                .try_send(
                    mix_engine::FrameOutput {
                        mixes: [(0.5, 0.5), (0.0, 0.0)],
                    },
                    revision,
                    None,
                )
                .unwrap();
        }
        let report = registry.drive_once(&bridge, &plane, 2, 2).await;
        assert_eq!(report.frames_drained, 2);
        assert_eq!(report.packets_encoded, 0);
        assert!(!report.budget_exhausted);
    }

    #[tokio::test]
    async fn drive_once_budget_not_exhausted_when_no_sessions() {
        let registry = SessionRegistry::new();
        let plane = crate::media_plane::MediaPlane::new();
        let bridge = crate::media_bridge::MediaBridge::new();
        bridge
            .try_send(
                mix_engine::FrameOutput {
                    mixes: [(0.0, 0.0), (0.0, 0.0)],
                },
                99,
                None,
            )
            .unwrap();
        let report = registry.drive_once(&bridge, &plane, 1, 1).await;
        assert_eq!(report.frames_drained, 1);
        assert_eq!(report.outputs_polled, 0);
        assert_eq!(report.poll_errors, 0);
        assert!(!report.budget_exhausted);
    }

    #[tokio::test]
    async fn remove_returns_true_for_existing_session() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("dave", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        assert!(registry.remove("dave").await);
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn remove_returns_false_for_missing_session() {
        let registry = SessionRegistry::new();
        assert!(!registry.remove("missing").await);
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn list_returns_session_with_mix_id() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("eve", VALID_OFFER, Some("1".into()))
            .await
            .expect("offer must succeed");
        let sessions = registry.list().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_id, "eve");
        assert_eq!(sessions[0].mix_id.as_deref(), Some("1"));
        assert!(sessions[0].device_id.is_none());
    }

    #[tokio::test]
    async fn remove_by_device_id_removes_all_matching_sessions() {
        let registry = SessionRegistry::new();
        for (user_id, device_id) in [
            ("alice", "shared-device"),
            ("bob", "shared-device"),
            ("carol", "other-device"),
        ] {
            let identity = DeviceIdentity {
                device_id: device_id.into(),
                musician_id: user_id.into(),
                mix_index: 0,
                revoked: false,
                dtls_fingerprint: None,
            };
            registry
                .negotiate_offer_bound(user_id, VALID_OFFER, Some("0".into()), Some(&identity))
                .await
                .expect("bound offer must succeed");
        }

        assert_eq!(registry.remove_by_device_id("shared-device").await, 2);
        let sessions = registry.list().await;
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].user_id, "carol");
        assert_eq!(sessions[0].device_id.as_deref(), Some("other-device"));
    }

    #[tokio::test]
    async fn remove_by_device_id_zero_when_no_match() {
        let registry = SessionRegistry::new();
        registry
            .negotiate_offer("frank", VALID_OFFER, None)
            .await
            .expect("offer must succeed");
        assert_eq!(registry.remove_by_device_id("nonexistent-device").await, 0);
        assert_eq!(registry.len().await, 1);
    }
}
