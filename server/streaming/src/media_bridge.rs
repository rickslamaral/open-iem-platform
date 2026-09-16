//! Bounded MixEngine-to-media handoff.
//!
//! The producer-facing method is synchronous and uses `try_send`; it is safe
//! for the realtime boundary because it never waits. An async consumer drains
//! frames and fans them out through [`MediaPlane`]. Evidence remains
//! SIMULATED; this bridge does not drive WebRTC network I/O.

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
    tx: Sender<(FrameOutput, u64)>,
    rx: Receiver<(FrameOutput, u64)>,
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
    ) -> Result<(), MediaBridgeError> {
        self.tx
            .try_send((frame, engine_revision))
            .map_err(|error| match error {
                TrySendError::Full(_) => MediaBridgeError::Full,
                TrySendError::Disconnected(_) => MediaBridgeError::Disconnected,
            })
    }

    /// Drain at most `budget` queued frames and route them to registered media sessions.
    pub async fn drain_to_with_budget(&self, media_plane: &MediaPlane, budget: usize) -> usize {
        let mut routed = 0;
        while routed < budget {
            let Ok((frame, revision)) = self.rx.try_recv() else {
                break;
            };
            media_plane.push_frame_output(&frame, revision).await;
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
    fn frame() -> FrameOutput {
        FrameOutput {
            mixes: [(0.25, 0.5), (0.75, 1.0)],
        }
    }

    #[test]
    fn full_queue_rejects_without_waiting() {
        let bridge = MediaBridge::new();
        for _ in 0..MEDIA_BRIDGE_CAPACITY {
            bridge.try_send(frame(), 1).unwrap();
        }
        assert_eq!(bridge.try_send(frame(), 1), Err(MediaBridgeError::Full));
    }

    #[tokio::test]
    async fn drain_routes_frames_to_subscribed_mix() {
        let bridge = MediaBridge::new();
        let plane = MediaPlane::new();
        plane.register_session("alice", 1).await.unwrap();
        bridge.try_send(frame(), 7).unwrap();
        bridge.try_send(frame(), 8).unwrap();

        assert_eq!(bridge.drain_to(&plane).await, 2);
        let sessions = plane.sessions.lock().await;
        let frames = sessions["alice"].drain_frames();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].samples, (0.75, 1.0));
        assert_eq!(frames[0].metadata.revision, 7);
        assert_eq!(frames[1].metadata.revision, 8);
    }
}
