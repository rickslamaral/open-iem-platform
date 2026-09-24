//! Bounded MixEngine-to-media handoff.
//!
//! The producer-facing method is synchronous and uses `try_send`; it is safe
//! for the realtime boundary because it never waits. An async consumer drains
//! frames and fans them out through [`MediaPlane`]. Evidence remains
//! SIMULATED; this bridge does not drive WebRTC network I/O.

use crate::clock::SampleTimestamp;
use crate::media_plane::MediaPlane;
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use mix_engine::FrameOutput;
use thiserror::Error;

/// Maximum number of mix frames awaiting media fan-out.
pub const MEDIA_BRIDGE_CAPACITY: usize = 64;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum MediaBridgeError {
    #[error("media bridge queue is full")]
    Full,
    #[error("media bridge consumer is stopped")]
    Disconnected,
}

/// Non-blocking producer and async media-plane consumer.
pub struct MediaBridge {
    tx: Sender<(FrameOutput, u64, Option<SampleTimestamp>)>,
    rx: Receiver<(FrameOutput, u64, Option<SampleTimestamp>)>,
}

impl MediaBridge {
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = bounded(MEDIA_BRIDGE_CAPACITY);
        Self { tx, rx }
    }

    /// Enqueue one processed mix frame without waiting or allocating.
    ///
    /// # Errors
    ///
    /// Returns `Full` when backpressure is active, or `Disconnected` after the
    /// bridge has been dropped.
    pub fn try_send(
        &self,
        frame: FrameOutput,
        engine_revision: u64,
        capture_timestamp: Option<SampleTimestamp>,
    ) -> Result<(), MediaBridgeError> {
        self.tx
            .try_send((frame, engine_revision, capture_timestamp))
            .map_err(|error| match error {
                TrySendError::Full(_) => MediaBridgeError::Full,
                TrySendError::Disconnected(_) => MediaBridgeError::Disconnected,
            })
    }

    /// Drain at most `budget` queued frames and route them to registered media sessions.
    pub async fn drain_to_with_budget(&self, media_plane: &MediaPlane, budget: usize) -> usize {
        let mut routed = 0;
        while routed < budget {
            let Ok((frame, revision, capture_timestamp)) = self.rx.try_recv() else {
                break;
            };
            media_plane
                .push_frame_output(&frame, revision, capture_timestamp)
                .await;
            routed += 1;
        }
        routed
    }

    /// Drain all currently queued frames and route them to registered media sessions.
    pub async fn drain_to(&self, media_plane: &MediaPlane) -> usize {
        self.drain_to_with_budget(media_plane, usize::MAX).await
    }
}

impl Default for MediaBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media_plane::MEDIA_QUEUE_CAPACITY;
    use std::sync::atomic::Ordering;
    fn frame() -> FrameOutput {
        FrameOutput {
            mixes: [(0.25, 0.5), (0.75, 1.0)],
        }
    }

    #[test]
    fn full_queue_rejects_without_waiting() {
        let bridge = MediaBridge::new();
        for _ in 0..MEDIA_BRIDGE_CAPACITY {
            bridge.try_send(frame(), 1, None).unwrap();
        }
        assert_eq!(
            bridge.try_send(frame(), 1, None),
            Err(MediaBridgeError::Full)
        );
    }

    #[tokio::test]
    async fn rejected_frame_is_not_enqueued_after_queue_recovers() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();

        for revision in 0..MEDIA_BRIDGE_CAPACITY as u64 {
            bridge.try_send(frame(), revision, None).unwrap();
        }
        assert_eq!(
            bridge.try_send(frame(), 99, None),
            Err(MediaBridgeError::Full)
        );

        assert_eq!(bridge.drain_to_with_budget(&plane, 1).await, 1);
        assert_eq!(
            plane
                .drain_session_frames_with_budget("alice", usize::MAX)
                .await
                .unwrap()[0]
                .metadata
                .revision,
            0
        );
        bridge.try_send(frame(), 100, None).unwrap();

        assert_eq!(
            bridge
                .drain_to_with_budget(&plane, MEDIA_QUEUE_CAPACITY)
                .await,
            MEDIA_QUEUE_CAPACITY
        );
        let first_batch = plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap();
        assert_eq!(first_batch.len(), MEDIA_QUEUE_CAPACITY);
        assert_eq!(first_batch[0].metadata.revision, 1);
        assert_eq!(
            first_batch[MEDIA_QUEUE_CAPACITY - 1].metadata.revision,
            MEDIA_QUEUE_CAPACITY as u64
        );

        assert_eq!(
            bridge.drain_to(&plane).await,
            MEDIA_BRIDGE_CAPACITY - MEDIA_QUEUE_CAPACITY
        );
        let last_batch = plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            last_batch.len(),
            MEDIA_BRIDGE_CAPACITY - MEDIA_QUEUE_CAPACITY
        );
        assert_eq!(
            last_batch[0].metadata.revision,
            (MEDIA_QUEUE_CAPACITY + 1) as u64
        );
        assert_eq!(
            last_batch[MEDIA_BRIDGE_CAPACITY - MEDIA_QUEUE_CAPACITY - 1]
                .metadata
                .revision,
            100
        );
        assert!(!first_batch
            .iter()
            .chain(last_batch.iter())
            .any(|item| item.metadata.revision == 99));
    }

    #[tokio::test]
    async fn drain_with_budget_routes_only_requested_frames() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 1).await.unwrap();
        bridge.try_send(frame(), 7, None).unwrap();
        bridge.try_send(frame(), 8, None).unwrap();

        assert_eq!(bridge.drain_to_with_budget(&plane, 1).await, 1);
        let first = plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].metadata.revision, 7);

        assert_eq!(bridge.drain_to_with_budget(&plane, 1).await, 1);
        let second = plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].metadata.revision, 8);
    }

    #[tokio::test]
    async fn oversized_budget_routes_all_available_frames() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        bridge.try_send(frame(), 7, None).unwrap();
        bridge.try_send(frame(), 8, None).unwrap();

        assert_eq!(bridge.drain_to_with_budget(&plane, usize::MAX).await, 2);
        let frames = plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap();
        assert_eq!(
            frames
                .iter()
                .map(|item| item.metadata.revision)
                .collect::<Vec<_>>(),
            vec![7, 8]
        );
        assert_eq!(bridge.drain_to_with_budget(&plane, usize::MAX).await, 0);
    }

    #[tokio::test]
    async fn zero_budget_preserves_queued_frames() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 1).await.unwrap();
        bridge.try_send(frame(), 7, None).unwrap();

        assert_eq!(bridge.drain_to_with_budget(&plane, 0).await, 0);
        assert_eq!(
            plane
                .drain_session_frames_with_budget("alice", usize::MAX)
                .await
                .unwrap()
                .len(),
            0
        );

        assert_eq!(bridge.drain_to_with_budget(&plane, 1).await, 1);
        let frames = plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].metadata.revision, 7);
    }

    #[tokio::test]
    async fn empty_drain_returns_zero_without_touching_media_plane() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 1).await.unwrap();

        assert_eq!(bridge.drain_to(&plane).await, 0);
        assert!(plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn drain_preserves_capture_timestamp() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        let timestamp = Some(SampleTimestamp::new(42, 2_016_000));
        bridge.try_send(frame(), 7, timestamp).unwrap();

        assert_eq!(bridge.drain_to(&plane).await, 1);
        let frames = plane
            .drain_session_frames_with_budget("alice", 1)
            .await
            .unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].metadata.capture_timestamp, timestamp);
    }

    #[tokio::test]
    async fn drain_consumes_frames_when_destination_queue_is_full() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();

        for revision in 0..MEDIA_QUEUE_CAPACITY as u64 {
            assert_eq!(plane.push_frame_output(&frame(), revision, None).await, ());
        }
        assert_eq!(plane.dropped_total.load(Ordering::Relaxed), 0);

        bridge.try_send(frame(), 99, None).unwrap();
        assert_eq!(bridge.drain_to(&plane).await, 1);
        assert_eq!(plane.dropped_total.load(Ordering::Relaxed), 1);
        assert_eq!(plane.total_dropped(), 1);
        let sessions = plane.sessions.lock().await;
        assert_eq!(sessions["alice"].drop_count(), 1);
        drop(sessions);

        let frames = plane
            .drain_session_frames_with_budget("alice", usize::MAX)
            .await
            .unwrap();
        assert_eq!(frames.len(), MEDIA_QUEUE_CAPACITY);
        assert_eq!(
            frames
                .iter()
                .map(|item| item.metadata.revision)
                .collect::<Vec<_>>(),
            (0..MEDIA_QUEUE_CAPACITY as u64).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn successful_delivery_keeps_aggregate_drop_count_zero() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 0).await.unwrap();
        bridge.try_send(frame(), 7, None).unwrap();

        assert_eq!(bridge.drain_to(&plane).await, 1);
        assert_eq!(plane.total_dropped(), 0);
        assert_eq!(
            plane
                .drain_session_frames_with_budget("alice", usize::MAX)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn drain_consumes_frames_without_registered_sessions() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        bridge.try_send(frame(), 7, None).unwrap();
        bridge.try_send(frame(), 8, None).unwrap();

        assert_eq!(bridge.drain_to(&plane).await, 2);
        assert!(plane.sessions.lock().await.is_empty());
        assert_eq!(bridge.drain_to(&plane).await, 0);
    }

    #[tokio::test]
    async fn drain_routes_frames_to_subscribed_mix() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 1).await.unwrap();
        bridge.try_send(frame(), 7, None).unwrap();
        bridge.try_send(frame(), 8, None).unwrap();

        assert_eq!(bridge.drain_to(&plane).await, 2);
        let sessions = plane.sessions.lock().await;
        let frames = sessions["alice"].drain_frames();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].samples, (0.75, 1.0));
        assert_eq!(frames[0].metadata.revision, 7);
        assert_eq!(frames[1].metadata.revision, 8);
    }
}
