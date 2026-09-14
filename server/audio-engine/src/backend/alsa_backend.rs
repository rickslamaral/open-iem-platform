//! ALSA explicit fallback backend.
//!
//! Opens an ALSA PCM device directly via the `alsa` crate. Intended as a
//! hardware fallback when PipeWire/JACK is unavailable. Not validated on
//! physical hardware — see [`AlsaBackend::name`].
//!
//! # Fail-safe behaviour
//!
//! If device open or `hw_params` configuration fails, [`AlsaBackend::activate`]
//! sets `muted = true` and returns `Err(BackendActivate)`. The backend does NOT
//! panic. This is the normal path in CI/VPS where no ALSA device exists.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
};

use alsa::{
    pcm::{Access, Format, HwParams, State},
    Direction, PCM,
};
use mix_engine::MixEngine;

use crate::{
    backend::{Backend, BackendResult},
    error::AudioEngineError,
    rt_boundary::{AudioControl, ControlProducer, EnqueueError, RealtimeProcessor},
};

/// Stack size for the ALSA audio thread (512 KiB — avoids OS default 8 MiB).
const AUDIO_THREAD_STACK: usize = 512 * 1024;

/// ALSA explicit fallback backend.
pub struct AlsaBackend {
    device_name: String,
    sample_rate: u32,
    buffer_frames: u32,

    /// Processor moved into audio thread on activate; `None` while running.
    processor: Option<RealtimeProcessor>,
    /// Template engine used to recreate a fresh processor on re-activate.
    engine_template: MixEngine,

    /// Producer end of the control queue; `None` before first activate.
    control_producer: Option<ControlProducer>,

    active: bool,

    /// Shared with audio thread.
    xrun_count: Arc<AtomicU64>,
    /// Shared with audio thread; `true` when device failed irrecoverably.
    muted: Arc<AtomicBool>,

    /// Stop signal sent to audio thread.
    stop_flag: Arc<AtomicBool>,
    /// Join handle for the audio thread.
    thread_handle: Option<thread::JoinHandle<RealtimeProcessor>>,
}

impl AlsaBackend {
    /// Construct a new (inactive) ALSA backend.
    #[must_use]
    pub fn new(
        engine: MixEngine,
        device_name: impl Into<String>,
        sample_rate: u32,
        buffer_frames: u32,
    ) -> Self {
        let (processor, producer) = RealtimeProcessor::new(engine.clone());
        Self {
            device_name: device_name.into(),
            sample_rate,
            buffer_frames,
            processor: Some(processor),
            engine_template: engine,
            control_producer: Some(producer),
            active: false,
            xrun_count: Arc::new(AtomicU64::new(0)),
            muted: Arc::new(AtomicBool::new(false)),
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    /// Non-blocking enqueue of a control command.
    ///
    /// # Errors
    ///
    /// Returns [`EnqueueError`] if queue is full or backend is stopped.
    pub fn try_send_control(&self, command: AudioControl) -> Result<(), EnqueueError> {
        match &self.control_producer {
            Some(p) => p.try_send(command),
            None => Err(EnqueueError::Disconnected),
        }
    }

    /// Number of control commands dropped due to full queue.
    #[must_use]
    pub fn dropped_control_commands(&self) -> u64 {
        match &self.control_producer {
            Some(p) => p.dropped_commands(),
            None => 0,
        }
    }

    /// Total XRUN count since last activate.
    #[must_use]
    pub fn xrun_count(&self) -> u64 {
        self.xrun_count.load(Ordering::Relaxed)
    }

    /// `true` if backend muted itself after an irrecoverable device error.
    #[must_use]
    pub fn is_muted_by_device_failure(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    /// Open and configure the ALSA PCM device. Returns the ready PCM handle.
    fn open_pcm(&self) -> Result<PCM, AudioEngineError> {
        let pcm = PCM::new(&self.device_name, Direction::Playback, false).map_err(|e| {
            AudioEngineError::BackendActivate(format!(
                "ALSA PCM open '{}' failed: {e}",
                self.device_name
            ))
        })?;

        {
            let hwp = HwParams::any(&pcm).map_err(|e| {
                AudioEngineError::BackendActivate(format!("ALSA hw_params_any failed: {e}"))
            })?;

            hwp.set_channels(2).map_err(|e| {
                AudioEngineError::BackendActivate(format!("ALSA set_channels failed: {e}"))
            })?;

            hwp.set_format(Format::s16()).map_err(|e| {
                AudioEngineError::BackendActivate(format!("ALSA set_format failed: {e}"))
            })?;

            hwp.set_access(Access::RWInterleaved).map_err(|e| {
                AudioEngineError::BackendActivate(format!("ALSA set_access failed: {e}"))
            })?;

            hwp.set_rate_near(self.sample_rate, alsa::ValueOr::Nearest)
                .map_err(|e| {
                    AudioEngineError::BackendActivate(format!("ALSA set_rate_near failed: {e}"))
                })?;

            let period = alsa::pcm::Frames::from(i64::from(self.buffer_frames));
            hwp.set_period_size_near(period, alsa::ValueOr::Nearest)
                .map_err(|e| {
                    AudioEngineError::BackendActivate(format!(
                        "ALSA set_period_size_near failed: {e}"
                    ))
                })?;

            hwp.set_buffer_size_near(period * 4).map_err(|e| {
                AudioEngineError::BackendActivate(format!("ALSA set_buffer_size_near failed: {e}"))
            })?;

            pcm.hw_params(&hwp).map_err(|e| {
                AudioEngineError::BackendActivate(format!("ALSA hw_params apply failed: {e}"))
            })?;
        }

        Ok(pcm)
    }
}

impl Backend for AlsaBackend {
    fn activate(&mut self) -> BackendResult<()> {
        if self.active {
            return Err(AudioEngineError::InvalidState {
                reason: "AlsaBackend already active".into(),
            });
        }

        // Reset shared state.
        self.xrun_count.store(0, Ordering::Relaxed);
        self.muted.store(false, Ordering::Relaxed);
        self.stop_flag.store(false, Ordering::Release);

        // Open and configure PCM — fail-safe on error.
        let pcm = match self.open_pcm() {
            Ok(p) => p,
            Err(e) => {
                self.muted.store(true, Ordering::Relaxed);
                self.active = false;
                return Err(e);
            }
        };

        // Take processor; recreate from template if needed.
        let processor = if let Some(p) = self.processor.take() {
            p
        } else {
            let (p, prod) = RealtimeProcessor::new(self.engine_template.clone());
            self.control_producer = Some(prod);
            p
        };

        let frames = self.buffer_frames as usize;
        let xrun_count = Arc::clone(&self.xrun_count);
        let muted = Arc::clone(&self.muted);
        let stop_flag = Arc::clone(&self.stop_flag);

        let handle = thread::Builder::new()
            .name("alsa-audio".into())
            .stack_size(AUDIO_THREAD_STACK)
            .spawn(move || audio_thread_main(pcm, processor, frames, xrun_count, muted, stop_flag))
            .map_err(|e| {
                AudioEngineError::BackendActivate(format!("spawn audio thread failed: {e}"))
            })?;

        self.thread_handle = Some(handle);
        self.active = true;
        log::info!(
            "AlsaBackend activated: device='{}' rate={} frames={}",
            self.device_name,
            self.sample_rate,
            self.buffer_frames,
        );
        Ok(())
    }

    fn deactivate(&mut self) {
        if !self.active && self.thread_handle.is_none() {
            return;
        }
        // Mark inactive before joining — ensures is_active() returns false
        // even if the thread join takes time or panics.
        self.active = false;
        self.stop_flag.store(true, Ordering::Release);
        if let Some(h) = self.thread_handle.take() {
            match h.join() {
                Ok(processor) => {
                    self.processor = Some(processor);
                }
                Err(_) => {
                    log::error!("AlsaBackend audio thread panicked; processor not recovered");
                }
            }
        }
        log::info!("AlsaBackend deactivated");
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn buffer_frames(&self) -> u32 {
        self.buffer_frames
    }

    fn name(&self) -> &'static str {
        "AlsaBackend (ALSA explicit fallback) [HARDWARE \u{2014} not validated]"
    }
}

impl Drop for AlsaBackend {
    fn drop(&mut self) {
        self.deactivate();
    }
}

// ---------------------------------------------------------------------------
// Audio thread
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
fn audio_thread_main(
    pcm: PCM,
    mut processor: RealtimeProcessor,
    frames: usize,
    xrun_count: Arc<AtomicU64>,
    muted: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
) -> RealtimeProcessor {
    let buf_len = frames * 2;
    let mut buf: Vec<i16> = vec![0i16; buf_len];
    let silence: Vec<i16> = vec![0i16; buf_len];
    let input_zeros = vec![0.0f32; frames * 2];

    loop {
        if stop_flag.load(Ordering::Acquire) {
            break;
        }

        if muted.load(Ordering::Relaxed) {
            // Fail-safe: write silence then loop.
            // Break if PCM handle is completely dead (io_i16 or writei failure).
            let Ok(io) = pcm.io_i16() else {
                log::error!(
                    "AlsaBackend: PCM io_i16 failed in fail-safe mute path; stopping thread"
                );
                break;
            };
            if let Err(e) = io.writei(&silence) {
                log::warn!("AlsaBackend: silence writei failed in mute path: {e}");
            }
            continue;
        }

        let frame_output = processor.process_frame(&input_zeros);

        let (l, r) = frame_output.mixes[0];
        let l_i16 = (l.clamp(-1.0, 1.0) * 32767.0) as i16;
        let r_i16 = (r.clamp(-1.0, 1.0) * 32767.0) as i16;
        for i in 0..frames {
            buf[i * 2] = l_i16;
            buf[i * 2 + 1] = r_i16;
        }

        let io = match pcm.io_i16() {
            Ok(io) => io,
            Err(e) => {
                log::error!("AlsaBackend: io_i16 error: {e}");
                muted.store(true, Ordering::Relaxed);
                break;
            }
        };

        match io.writei(&buf) {
            Ok(_) => {}
            Err(e) => {
                let raw_err = e.errno();
                match pcm.recover(raw_err, false) {
                    Ok(()) => {
                        xrun_count.fetch_add(1, Ordering::Relaxed);
                        log::warn!(
                            "AlsaBackend: XRUN recovered (total={})",
                            xrun_count.load(Ordering::Relaxed)
                        );
                    }
                    Err(re) => {
                        log::error!("AlsaBackend: unrecoverable PCM error: {re}");
                        muted.store(true, Ordering::Relaxed);
                        break;
                    }
                }
            }
        }

        // After an XRUN recovery, ALSA may leave PCM in Setup state.
        // pcm.recover() above already calls prepare() in most cases, but this
        // catches edge cases where the driver resets without signalling an error.
        if pcm.state() == State::Setup {
            if let Err(e) = pcm.prepare() {
                log::warn!("AlsaBackend: pcm.prepare() after Setup state failed: {e}");
            }
        }
    }

    processor
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AudioEngineError;

    fn make_backend(device: &str) -> AlsaBackend {
        AlsaBackend::new(MixEngine::new(), device, 48_000, 256)
    }

    #[test]
    fn test_alsa_backend_new_sets_fields() {
        let b = make_backend("nonexistent_device");
        assert_eq!(b.sample_rate(), 48_000);
        assert_eq!(b.buffer_frames(), 256);
        assert!(!b.is_active());
        assert!(b.name().contains("ALSA"));
        assert!(b.name().contains("not validated"));
    }

    #[test]
    fn test_alsa_backend_activate_nonexistent_device_fails_gracefully() {
        let mut b = make_backend("nonexistent_device_that_does_not_exist");
        let result = b.activate();
        assert!(
            matches!(result, Err(AudioEngineError::BackendActivate(_))),
            "expected BackendActivate, got: {result:?}"
        );
        assert!(!b.is_active());
        assert!(b.is_muted_by_device_failure());
    }

    #[test]
    fn test_alsa_backend_deactivate_when_not_active_is_safe() {
        let mut b = make_backend("nonexistent_device");
        b.deactivate();
        assert!(!b.is_active());
    }

    #[test]
    fn test_alsa_backend_xrun_count_starts_zero() {
        let b = make_backend("nonexistent_device");
        assert_eq!(b.xrun_count(), 0);
    }
}
