//! JACK/pipewire-jack backend — **hardware target only**.
//!
//! This module is compiled only when the `jack` Cargo feature is enabled.
//! On the target hardware (Raspberry Pi 5) `pipewire-jack` provides a
//! JACK-compatible library that routes JACK API calls through the PipeWire
//! graph transparently.
//!
//! # Prerequisites (hardware)
//!
//! ```bash
//! sudo apt install pipewire-jack libjack-dev
//! # Start PipeWire with JACK support:
//! systemctl --user start pipewire pipewire-pulse wireplumber
//! ```
//!
//! # Cargo build (hardware)
//!
//! ```bash
//! cargo build --features jack -p audio-engine
//! ```
//!
//! # Architecture
//!
//! ```text
//!   [PipeWire graph]
//!        │ pipewire-jack bridge
//!   libjack.so ←──── JACK client (this module)
//!        │
//!   JackBackend::process_callback()
//!        │
//!   mix_engine::MixEngine::process_frame()  (lock-free, stack-only)
//! ```
//!
//! # Realtime Rules (enforced in callback)
//!
//! The JACK process callback runs in a realtime OS thread managed by PipeWire.
//! All rules from [`mix_engine`] apply: no I/O, no alloc, no blocking.
//!
//! # SIMULATED status
//!
//! This module compiles with the `jack` feature but **has not been validated
//! on hardware**. It is a correctly-structured stub. Validation deferred to
//! Phase 5 (hardware integration on RPi 5 with PipeWire).

#![cfg(feature = "jack")]

use std::sync::{Arc, Mutex};

use jack::{AudioIn, AudioOut, Client, ClientOptions, Control, ProcessScope};
use mix_engine::{MixEngine, MAX_CHANNELS, SAMPLE_RATE};

use crate::{
    backend::{Backend, BackendResult, ProcessStats},
    error::AudioEngineError,
};

/// Shared DSP state accessed from the JACK callback.
///
/// The callback holds an `Arc<Mutex<JackState>>`. In production this would use
/// a lock-free ring buffer for the audio path, but for the stub the mutex is
/// acceptable since we validate the API surface, not RT performance.
///
/// **TODO Phase 5:** Replace with `crossbeam-channel` SPSC + double-buffer.
struct JackState {
    engine: MixEngine,
    xrun_count: u64,
    frames_processed: u64,
}

/// Handle to the active JACK client (opaque — keeps lifetime alive).
struct ActiveClient {
    // The async client wraps the JACK client after `activate_async`.
    // We keep it alive by holding the handle here.
    _handle: jack::AsyncClient<JackNotificationHandler, JackProcessHandler>,
}

/// JACK backend — connects to pipewire-jack and drives MixEngine from callback.
///
/// # SIMULATED
///
/// Not validated on hardware. Structure and API surface are correct.
pub struct JackBackend {
    client_name: String,
    sample_rate: u32,
    buffer_frames: u32,
    state: Arc<Mutex<JackState>>,
    active_client: Option<ActiveClient>,
}

impl JackBackend {
    /// Create a new JACK backend with the given client name and mix engine.
    #[must_use]
    pub fn new(engine: MixEngine, client_name: impl Into<String>, buffer_frames: u32) -> Self {
        let state = Arc::new(Mutex::new(JackState {
            engine,
            xrun_count: 0,
            frames_processed: 0,
        }));
        Self {
            client_name: client_name.into(),
            sample_rate: SAMPLE_RATE,
            buffer_frames,
            state,
            active_client: None,
        }
    }
}

impl Backend for JackBackend {
    fn activate(&mut self) -> BackendResult<()> {
        if self.active_client.is_some() {
            return Err(AudioEngineError::InvalidState {
                reason: "JACK client already active".into(),
            });
        }

        let (client, _status) = Client::new(&self.client_name, ClientOptions::NO_START_SERVER)
            .map_err(|e| AudioEngineError::BackendInit(e.to_string()))?;

        // Update sample rate from what JACK negotiated.
        self.sample_rate = client.sample_rate() as u32;

        // Register input ports (one per channel slot).
        let mut in_ports: Vec<jack::Port<AudioIn>> = Vec::with_capacity(MAX_CHANNELS);
        for i in 0..MAX_CHANNELS {
            let port = client
                .register_port(&format!("input_{}", i + 1), AudioIn::default())
                .map_err(|e| AudioEngineError::BackendInit(e.to_string()))?;
            in_ports.push(port);
        }

        // Register output ports (stereo pair per mix — stub: just mix 0).
        let out_l = client
            .register_port("output_1_L", AudioOut::default())
            .map_err(|e| AudioEngineError::BackendInit(e.to_string()))?;
        let out_r = client
            .register_port("output_1_R", AudioOut::default())
            .map_err(|e| AudioEngineError::BackendInit(e.to_string()))?;

        let state = Arc::clone(&self.state);

        let process = JackProcessHandler {
            in_ports,
            out_l,
            out_r,
            state,
        };

        let async_client = client
            .activate_async(JackNotificationHandler, process)
            .map_err(|e| AudioEngineError::BackendActivate(e.to_string()))?;

        self.active_client = Some(ActiveClient {
            _handle: async_client,
        });

        log::info!(
            "[JACK] audio-engine activated: client='{}' {}Hz {}frames",
            self.client_name,
            self.sample_rate,
            self.buffer_frames,
        );

        Ok(())
    }

    fn deactivate(&mut self) {
        // Dropping the async client deactivates the JACK client.
        self.active_client = None;
        log::info!("[JACK] audio-engine deactivated");
    }

    fn is_active(&self) -> bool {
        self.active_client.is_some()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn buffer_frames(&self) -> u32 {
        self.buffer_frames
    }

    fn name(&self) -> &'static str {
        "JackBackend (pipewire-jack) [HARDWARE — not validated on RPi5]"
    }
}

// ---------------------------------------------------------------------------
// JACK callbacks
// ---------------------------------------------------------------------------

/// JACK process callback — runs in realtime thread.
struct JackProcessHandler {
    in_ports: Vec<jack::Port<AudioIn>>,
    out_l: jack::Port<AudioOut>,
    out_r: jack::Port<AudioOut>,
    state: Arc<Mutex<JackState>>,
}

impl jack::ProcessHandler for JackProcessHandler {
    fn process(&mut self, _: &Client, ps: &ProcessScope) -> Control {
        // Collect one sample per channel (first sample of the period for stub).
        // Production: process full buffer_frames in a loop.
        let mut samples = [0.0_f32; MAX_CHANNELS];
        for (i, port) in self.in_ports.iter().enumerate() {
            if let Some(s) = samples.get_mut(i) {
                // Use first frame only for stub; full buffer in Phase 5.
                *s = port.as_slice(ps).first().copied().unwrap_or(0.0);
            }
        }

        // Acquire state — in production this would be a lock-free dequeue.
        // JACK realtime thread: mutex is acceptable for stub validation only.
        if let Ok(mut state) = self.state.try_lock() {
            let frame_out = state.engine.process_frame(&samples);
            let (l, r) = frame_out.mixes[0];

            // Write to output buffers.
            if let Some(out_l_slice) = self.out_l.as_mut_slice(ps).first_mut() {
                *out_l_slice = l;
            }
            if let Some(out_r_slice) = self.out_r.as_mut_slice(ps).first_mut() {
                *out_r_slice = r;
            }

            state.frames_processed = state.frames_processed.saturating_add(1);
        }
        // If try_lock fails, output silence (zero-initialized buffer).

        Control::Continue
    }
}

/// JACK notification handler — logs lifecycle events.
struct JackNotificationHandler;

impl jack::NotificationHandler for JackNotificationHandler {
    fn xrun(&mut self, _: &Client) -> Control {
        // In production: increment xrun counter, emit metric to Prometheus.
        log::warn!("[JACK] XRUN detected");
        Control::Continue
    }

    fn sample_rate(&mut self, _: &Client, srate: jack::Frames) -> Control {
        log::info!("[JACK] sample rate changed to {srate}");
        Control::Continue
    }
}
