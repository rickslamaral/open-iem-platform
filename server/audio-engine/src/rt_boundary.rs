//! Bounded control/audio boundary for realtime backends.
//!
//! Producers run on control threads; the consumer runs on the audio callback.
//! `crossbeam_channel::bounded` provides a bounded lock-free queue with
//! non-blocking `try_recv`, so callback work has an explicit overflow policy.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use mix_engine::MixEngine;

/// Capacity reserved for control mutations. Full queue rejects newest command.
pub const CONTROL_QUEUE_CAPACITY: usize = 64;

/// Control mutation delivered to audio thread.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioControl {
    /// Set master gain for mix slot.
    MasterGain {
        /// Mix slot.
        mix_index: usize,
        /// Gain in dB.
        gain_db: f32,
    },
    /// Set master mute for mix slot.
    MasterMute {
        /// Mix slot.
        mix_index: usize,
        /// Mute state.
        muted: bool,
    },
}

/// Result of a non-blocking control enqueue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnqueueError {
    /// Queue is full; caller must retry or report rejection.
    Full,
    /// Consumer was stopped.
    Disconnected,
}

/// Producer handle. Safe to use from control-plane threads.
#[derive(Clone)]
pub struct ControlProducer {
    tx: Sender<AudioControl>,
    dropped: Arc<AtomicU64>,
}

impl ControlProducer {
    /// Enqueue without waiting. Newest command is rejected when queue is full.
    ///
    /// # Errors
    ///
    /// Returns [`EnqueueError::Full`] when capacity is exhausted or
    /// [`EnqueueError::Disconnected`] after processor shutdown.
    pub fn try_send(&self, command: AudioControl) -> Result<(), EnqueueError> {
        self.tx.try_send(command).map_err(|e| match e {
            TrySendError::Full(_) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                EnqueueError::Full
            }
            TrySendError::Disconnected(_) => EnqueueError::Disconnected,
        })
    }

    /// Number of control commands rejected because queue was full.
    #[must_use]
    pub fn dropped_commands(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// Realtime-owned processor. No mutex, I/O or blocking operation in `process`.
pub struct RealtimeProcessor {
    engine: MixEngine,
    rx: Receiver<AudioControl>,
    dropped: Arc<AtomicU64>,
}

impl RealtimeProcessor {
    /// Create processor and its bounded control producer.
    #[must_use]
    pub fn new(engine: MixEngine) -> (Self, ControlProducer) {
        let (tx, rx) = bounded(CONTROL_QUEUE_CAPACITY);
        let dropped = Arc::new(AtomicU64::new(0));
        (
            Self {
                engine,
                rx,
                dropped: Arc::clone(&dropped),
            },
            ControlProducer { tx, dropped },
        )
    }

    /// Apply pending controls and process one frame.
    #[must_use]
    pub fn process_frame(&mut self, input: &[f32]) -> mix_engine::FrameOutput {
        // Fixed budget keeps callback work bounded even while producer is busy.
        for _ in 0..CONTROL_QUEUE_CAPACITY {
            match self.rx.try_recv() {
                Ok(command) => self.apply(command),
                Err(_) => break,
            }
        }
        self.engine.process_frame(input)
    }

    /// Number of control commands rejected because queue was full.
    #[must_use]
    pub fn dropped_commands(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    fn apply(&mut self, command: AudioControl) {
        match command {
            AudioControl::MasterGain { mix_index, gain_db } => {
                if let Some(mix) = self.engine.mix_mut(mix_index) {
                    mix.set_master_gain_db(gain_db);
                }
            }
            AudioControl::MasterMute { mix_index, muted } => {
                if let Some(mix) = self.engine.mix_mut(mix_index) {
                    mix.set_master_muted(muted);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mix_engine::{Channel, Mix, MixSend};

    fn processor() -> (RealtimeProcessor, ControlProducer) {
        let mut engine = MixEngine::new();
        engine
            .set_channel(0, Channel::new(0, "input"))
            .expect("valid test engine configuration");
        let mut mix = Mix::new(1, "monitor");
        mix.set_send(0, MixSend::new(0, 1))
            .expect("valid test engine configuration");
        engine
            .set_mix(0, mix)
            .expect("valid test engine configuration");
        RealtimeProcessor::new(engine)
    }

    #[test]
    fn control_applies_without_callback_lock() {
        let (mut processor, producer) = processor();
        producer
            .try_send(AudioControl::MasterMute {
                mix_index: 0,
                muted: true,
            })
            .expect("valid test engine configuration");
        assert_eq!(processor.process_frame(&[1.0; 8]).mixes[0], (0.0, 0.0));
    }

    #[test]
    fn full_queue_rejects_newest_command() {
        let (_processor, producer) = processor();
        for _ in 0..CONTROL_QUEUE_CAPACITY {
            producer
                .try_send(AudioControl::MasterMute {
                    mix_index: 0,
                    muted: true,
                })
                .expect("valid test engine configuration");
        }
        assert_eq!(
            producer.try_send(AudioControl::MasterMute {
                mix_index: 0,
                muted: true
            }),
            Err(EnqueueError::Full)
        );
    }
}
