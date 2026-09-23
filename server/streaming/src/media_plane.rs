//! Media plane: routes [`FrameOutput`] stereo pairs from the mix engine to
//! per-user [`MediaSession`] queues using bounded, lock-free channels.
//!
//! # Design (ADR-007)
//!
//! - All queues are `crossbeam_channel::bounded(MEDIA_QUEUE_CAPACITY)` — no
//!   blocking calls on the RT path.
//! - Frame metadata carries a monotonic `sequence` and `engine_revision` so
//!   consumers can detect gaps and stale frames.
//! - Evidence level: **SIMULATED** (ADR-001/002, ADR-010 L1/L2).

use crate::clock::SampleTimestamp;
use crossbeam_channel::{bounded, TrySendError};
use mix_engine::{FrameOutput, MAX_MIXES};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::Mutex;

/// Bounded queue capacity for [`MediaSession`] frame queues (ADR-007).
pub const MEDIA_QUEUE_CAPACITY: usize = 32;

/// Maximum UTF-8 byte length for a media session user ID.
pub const MAX_MEDIA_USER_ID_BYTES: usize = 128;

/// Versioned stream descriptor attached to every [`MediaFrame`].
///
/// Allows consumers to detect gaps (`sequence` jumps) and engine resets
/// (`revision` bumps).
#[derive(Debug, Clone, PartialEq)]
pub struct StreamMetadata {
    /// Human-readable stream identifier, e.g. `"mix_0"`.
    pub stream_id: String,
    /// Mix slot this frame belongs to (0-indexed).
    pub mix_index: usize,
    /// Monotonic engine revision; bumped on each engine reconfiguration.
    pub revision: u64,
    /// Monotonic per-session frame counter; starts at 0.
    pub sequence: u64,
    /// Audio sample rate in Hz (always 48 000).
    pub sample_rate: u32,
    /// Number of interleaved channels (always 2).
    pub channels: u8,
    /// Nominal frame duration in milliseconds (always 20).
    pub frame_duration_ms: u32,
    /// Optional capture-timeline timestamp from the audio interface clock.
    pub capture_timestamp: Option<SampleTimestamp>,
}

/// A single audio frame as routed from the mix engine to a consumer session.
///
/// `samples` is the L/R stereo pair taken directly from
/// [`FrameOutput::mixes`]`[mix_index]`.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaFrame {
    /// Versioned stream metadata for this frame.
    pub metadata: StreamMetadata,
    /// Left/right stereo sample pair.
    pub samples: (f32, f32),
}

/// Errors returned by [`MediaSession::push_frame`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaSessionError {
    /// The internal bounded queue is full; the frame was dropped.
    QueueFull,
    /// No session exists for the requested user.
    NoSession,
}

/// Errors returned by [`MediaPlane`] operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaPlaneError {
    /// `mix_index >= MAX_MIXES`; the engine only supports [`MAX_MIXES`] slots.
    InvalidMixIndex,
    /// The user ID is empty or exceeds the bounded media-session limit.
    InvalidUserId,
    /// A session already exists for the requested user ID.
    SessionAlreadyExists,
}

/// Per-user audio routing session with a bounded frame queue.
///
/// Holds the `tx`/`rx` pair for a `crossbeam_channel::bounded` queue.\
/// `push_frame` is called from the media-plane (potentially RT context);\
/// `drain_frames` is called from the consumer (WebRTC send path).
#[derive(Debug)]
pub struct MediaSession {
    /// Owning user identifier.
    pub user_id: String,
    /// Mix slot this session is subscribed to.
    pub mix_index: usize,
    /// Monotonic per-session frame counter.
    pub frame_sequence: u64,
    /// Number of frames dropped because the queue was full.
    pub drop_count: u64,
    /// Send-half of the bounded frame queue.
    pub tx: crossbeam_channel::Sender<MediaFrame>,
    /// Receive-half of the bounded frame queue.
    pub rx: crossbeam_channel::Receiver<MediaFrame>,
}

impl MediaSession {
    /// Create a new session subscribed to `mix_index`.
    #[must_use]
    pub fn new(user_id: String, mix_index: usize) -> Self {
        let (tx, rx) = bounded(MEDIA_QUEUE_CAPACITY);
        Self {
            user_id,
            mix_index,
            frame_sequence: 0,
            drop_count: 0,
            tx,
            rx,
        }
    }

    /// Push a stereo sample pair into the queue.
    ///
    /// Builds [`StreamMetadata`] with the current `frame_sequence` (then
    /// increments it) and the supplied `engine_revision`.
    ///
    /// On queue full: increments `drop_count` and returns
    /// `Err(MediaSessionError::QueueFull)`. Never blocks.
    ///
    /// # Errors
    ///
    /// Returns [`MediaSessionError::QueueFull`] when the internal bounded queue
    /// is at capacity. The frame is discarded and `drop_count` is incremented.
    pub fn push_frame(
        &mut self,
        samples: (f32, f32),
        engine_revision: u64,
        capture_timestamp: Option<SampleTimestamp>,
    ) -> Result<(), MediaSessionError> {
        let metadata = StreamMetadata {
            stream_id: format!("mix_{}", self.mix_index),
            mix_index: self.mix_index,
            revision: engine_revision,
            sequence: self.frame_sequence,
            sample_rate: 48_000,
            channels: 2,
            frame_duration_ms: 20,
            capture_timestamp,
        };
        self.frame_sequence += 1;
        let frame = MediaFrame { metadata, samples };
        match self.tx.try_send(frame) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => {
                self.drop_count += 1;
                Err(MediaSessionError::QueueFull)
            }
            Err(TrySendError::Disconnected(_)) => {
                // rx still held by self — cannot happen in normal use
                self.drop_count += 1;
                Err(MediaSessionError::QueueFull)
            }
        }
    }

    /// Drain at most `budget` frames from the queue without blocking.
    #[must_use]
    pub fn drain_frames_with_budget(&self, budget: usize) -> Vec<MediaFrame> {
        let mut frames = Vec::new();
        while frames.len() < budget {
            let Ok(frame) = self.rx.try_recv() else {
                break;
            };
            frames.push(frame);
        }
        frames
    }

    /// Drain all available frames from the queue without blocking.
    #[must_use]
    pub fn drain_frames(&self) -> Vec<MediaFrame> {
        self.drain_frames_with_budget(usize::MAX)
    }

    /// Total number of frames dropped on this session due to queue saturation.
    #[must_use]
    pub fn drop_count(&self) -> u64 {
        self.drop_count
    }
}

/// Central media routing plane.
///
/// `push_frame_output` fans out each [`FrameOutput`] to every registered
/// [`MediaSession`], selecting the correct mix slot per session.
///
/// Thread-safe: sessions map is guarded by a `tokio::sync::Mutex`.\
/// `dropped_total` is an `AtomicU64` for wait-free reads.
#[derive(Clone, Debug)]
pub struct MediaPlane {
    /// Active sessions keyed by user ID.
    pub sessions: Arc<Mutex<HashMap<String, MediaSession>>>,
    /// Aggregate drop counter across all sessions.
    pub dropped_total: Arc<AtomicU64>,
}

impl MediaPlane {
    /// Create a new empty [`MediaPlane`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            dropped_total: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Register a new user session subscribed to `mix_index`.
    ///
    /// # Errors
    ///
    /// Returns [`MediaPlaneError::InvalidMixIndex`] when `mix_index >= MAX_MIXES`.
    pub async fn register_session(
        &self,
        user_id: &str,
        mix_index: usize,
    ) -> Result<(), MediaPlaneError> {
        if mix_index >= MAX_MIXES {
            return Err(MediaPlaneError::InvalidMixIndex);
        }
        if user_id.trim().is_empty() || user_id.len() > MAX_MEDIA_USER_ID_BYTES {
            return Err(MediaPlaneError::InvalidUserId);
        }
        let mut sessions = self.sessions.lock().await;
        if sessions.contains_key(user_id) {
            return Err(MediaPlaneError::SessionAlreadyExists);
        }
        let session = MediaSession::new(user_id.to_owned(), mix_index);
        sessions.insert(user_id.to_owned(), session);
        Ok(())
    }

    /// Remove a session by user ID. Returns `true` if a session was removed.
    pub async fn remove_session(&self, user_id: &str) -> bool {
        self.sessions.lock().await.remove(user_id).is_some()
    }

    /// Fan out a [`FrameOutput`] to all registered sessions.
    ///
    /// Each session receives the stereo pair for its subscribed mix slot.\
    /// Dropped frames (queue full) are counted in `dropped_total`.
    pub async fn push_frame_output(
        &self,
        frame_output: &FrameOutput,
        engine_revision: u64,
        capture_timestamp: Option<SampleTimestamp>,
    ) {
        let mut sessions = self.sessions.lock().await;
        for session in sessions.values_mut() {
            let samples = frame_output.mixes[session.mix_index];
            if let Err(MediaSessionError::QueueFull) =
                session.push_frame(samples, engine_revision, capture_timestamp)
            {
                self.dropped_total.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Drain at most `budget` frames for one session without waiting.
    ///
    /// This is the bounded handoff boundary for a future WebRTC media writer;
    /// it does not encode, transmit, or perform network I/O.
    ///
    /// # Errors
    ///
    /// Returns [`MediaSessionError::NoSession`] when `user_id` is not registered.
    pub async fn drain_session_frames_with_budget(
        &self,
        user_id: &str,
        budget: usize,
    ) -> Result<Vec<MediaFrame>, MediaSessionError> {
        let sessions = self.sessions.lock().await;
        let session = sessions.get(user_id).ok_or(MediaSessionError::NoSession)?;
        Ok(session.drain_frames_with_budget(budget))
    }

    /// Total frames dropped across all sessions (wait-free read).
    #[must_use]
    pub fn total_dropped(&self) -> u64 {
        self.dropped_total.load(Ordering::Relaxed)
    }

    /// Snapshot of active sessions as `(user_id, mix_index)` pairs.
    pub async fn sessions(&self) -> Vec<(String, usize)> {
        self.sessions
            .lock()
            .await
            .values()
            .map(|s| (s.user_id.clone(), s.mix_index))
            .collect()
    }
}

impl Default for MediaPlane {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mix_engine::FrameOutput;

    fn make_frame(l0: f32, r0: f32, l1: f32, r1: f32) -> FrameOutput {
        FrameOutput {
            mixes: [(l0, r0), (l1, r1)],
        }
    }

    #[tokio::test]
    async fn media_plane_new_has_no_sessions() {
        assert!(MediaPlane::new().sessions().await.is_empty());
    }

    #[tokio::test]
    async fn register_session_ok() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        assert_eq!(mp.sessions().await.len(), 1);
    }

    #[tokio::test]
    async fn register_rejects_empty_user_id_without_mutation() {
        let mp = MediaPlane::new();

        assert_eq!(
            mp.register_session("", 0).await,
            Err(MediaPlaneError::InvalidUserId)
        );
        assert!(mp.sessions().await.is_empty());
    }

    #[tokio::test]
    async fn register_rejects_whitespace_only_user_id_without_mutation() {
        let mp = MediaPlane::new();

        assert_eq!(
            mp.register_session(" \t\n", 0).await,
            Err(MediaPlaneError::InvalidUserId)
        );
        assert!(mp.sessions().await.is_empty());
    }

    #[tokio::test]
    async fn register_rejects_unicode_whitespace_only_user_id_without_mutation() {
        let mp = MediaPlane::new();

        assert_eq!(
            mp.register_session("\u{00a0}\u{2003}\u{202f}", 0).await,
            Err(MediaPlaneError::InvalidUserId)
        );
        assert!(mp.sessions().await.is_empty());
    }

    #[tokio::test]
    async fn register_accepts_maximum_user_id_length() {
        let mp = MediaPlane::new();
        let user_id = "u".repeat(MAX_MEDIA_USER_ID_BYTES);

        assert_eq!(mp.register_session(&user_id, 0).await, Ok(()));
        assert_eq!(mp.sessions().await, vec![(user_id, 0)]);
    }

    #[tokio::test]
    async fn register_rejects_oversized_user_id_without_mutation() {
        let mp = MediaPlane::new();
        let user_id = "u".repeat(MAX_MEDIA_USER_ID_BYTES + 1);

        assert_eq!(
            mp.register_session(&user_id, 0).await,
            Err(MediaPlaneError::InvalidUserId)
        );
        assert!(mp.sessions().await.is_empty());
    }

    #[tokio::test]
    async fn remove_session_preserves_other_sessions() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        mp.register_session("bob", 1).await.unwrap();

        assert!(mp.remove_session("alice").await);
        assert_eq!(mp.sessions().await, vec![("bob".to_owned(), 1)]);

        assert!(!mp.remove_session("alice").await);
        assert_eq!(mp.sessions().await, vec![("bob".to_owned(), 1)]);
    }

    #[tokio::test]
    async fn register_duplicate_user_id_rejects_without_replacing_session() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();

        assert_eq!(
            mp.register_session("alice", 1).await,
            Err(MediaPlaneError::SessionAlreadyExists)
        );
        assert_eq!(mp.sessions().await, vec![("alice".to_owned(), 0)]);
    }

    #[tokio::test]
    async fn register_invalid_mix_index_fails() {
        let mp = MediaPlane::new();
        assert_eq!(
            mp.register_session("bob", MAX_MIXES).await,
            Err(MediaPlaneError::InvalidMixIndex)
        );
    }

    #[tokio::test]
    async fn push_frame_output_increments_sequence() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        let fo = make_frame(0.1, 0.2, 0.3, 0.4);
        for _ in 0..3 {
            mp.push_frame_output(&fo, 1, None).await;
        }
        let sessions = mp.sessions.lock().await;
        let frames = sessions["alice"].drain_frames();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].metadata.sequence, 0);
        assert_eq!(frames[1].metadata.sequence, 1);
        assert_eq!(frames[2].metadata.sequence, 2);
    }

    #[tokio::test]
    async fn push_frame_output_overflow_increments_drop_count() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        let fo = make_frame(0.1, 0.2, 0.3, 0.4);
        for _ in 0..(MEDIA_QUEUE_CAPACITY + 5) {
            mp.push_frame_output(&fo, 1, None).await;
        }
        let sessions = mp.sessions.lock().await;
        assert!(sessions["alice"].drop_count() >= 5);
    }

    #[tokio::test]
    async fn push_frame_to_correct_mix_slot() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 1).await.unwrap();
        let fo = make_frame(0.1, 0.2, 0.8, 0.9);
        mp.push_frame_output(&fo, 1, None).await;
        let sessions = mp.sessions.lock().await;
        let frames = sessions["alice"].drain_frames();
        assert_eq!(frames[0].samples, (0.8, 0.9));
    }

    #[tokio::test]
    async fn remove_session_works() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        mp.remove_session("alice").await;
        assert!(mp.sessions().await.is_empty());
    }

    #[tokio::test]
    async fn drain_session_frames_respects_budget_and_preserves_order() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        let fo = make_frame(0.1, 0.2, 0.3, 0.4);
        mp.push_frame_output(&fo, 7, None).await;
        mp.push_frame_output(&fo, 8, None).await;

        let first = mp
            .drain_session_frames_with_budget("alice", 1)
            .await
            .unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].metadata.sequence, 0);
        assert_eq!(first[0].metadata.revision, 7);

        let second = mp
            .drain_session_frames_with_budget("alice", 1)
            .await
            .unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].metadata.sequence, 1);
        assert_eq!(second[0].metadata.revision, 8);
    }

    #[tokio::test]
    async fn drain_session_frames_zero_budget_and_missing_session() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        let fo = make_frame(0.1, 0.2, 0.3, 0.4);
        mp.push_frame_output(&fo, 1, None).await;

        assert!(mp
            .drain_session_frames_with_budget("alice", 0)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            mp.drain_session_frames_with_budget("missing", 1).await,
            Err(MediaSessionError::NoSession)
        );
        assert_eq!(
            mp.drain_session_frames_with_budget("alice", 1)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn total_dropped_aggregates() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        let fo = make_frame(0.0, 0.0, 0.0, 0.0);
        for _ in 0..=MEDIA_QUEUE_CAPACITY {
            mp.push_frame_output(&fo, 1, None).await;
        }
        assert!(mp.total_dropped() > 0);
    }

    #[tokio::test]
    async fn stream_id_format() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        let fo = make_frame(0.1, 0.2, 0.3, 0.4);
        mp.push_frame_output(&fo, 1, None).await;
        let sessions = mp.sessions.lock().await;
        let frames = sessions["alice"].drain_frames();
        assert_eq!(frames[0].metadata.stream_id, "mix_0");
    }

    #[tokio::test]
    async fn frame_metadata_fields() {
        let mp = MediaPlane::new();
        mp.register_session("alice", 0).await.unwrap();
        let fo = make_frame(0.1, 0.2, 0.3, 0.4);
        mp.push_frame_output(&fo, 1, None).await;
        let sessions = mp.sessions.lock().await;
        let frames = sessions["alice"].drain_frames();
        let meta = &frames[0].metadata;
        assert_eq!(meta.sample_rate, 48_000);
        assert_eq!(meta.channels, 2);
        assert_eq!(meta.frame_duration_ms, 20);
    }
}
