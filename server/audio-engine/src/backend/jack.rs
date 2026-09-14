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

use jack::{AudioIn, AudioOut, Client, ClientOptions, Control, ProcessScope};
use mix_engine::{MixEngine, MAX_CHANNELS, SAMPLE_RATE};
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use crate::rt_boundary::RealtimeProcessor;

use crate::{
    backend::{Backend, BackendResult, ProcessStats},
    error::AudioEngineError,
};

const LIFECYCLE_IDLE: u8 = 0;
const LIFECYCLE_ACTIVATING: u8 = 1;
const LIFECYCLE_ACTIVE: u8 = 2;

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
    processor: Option<RealtimeProcessor>,
    engine_template: MixEngine,
    control_producer: Option<crate::rt_boundary::ControlProducer>,
    active_client: Option<ActiveClient>,
    lifecycle: Arc<AtomicU8>,
}

impl JackBackend {
    /// Create a new JACK backend with the given client name and mix engine.
    ///
    /// Deactivation ends current realtime state. A later activation starts
    /// from this initial engine snapshot; control-plane callers must reapply
    /// desired state after restart.
    #[must_use]
    pub fn new(engine: MixEngine, client_name: impl Into<String>, buffer_frames: u32) -> Self {
        let engine_template = engine.clone();
        let (processor, control_producer) = RealtimeProcessor::new(engine);
        Self {
            client_name: client_name.into(),
            sample_rate: SAMPLE_RATE,
            buffer_frames,
            processor: Some(processor),
            engine_template,
            control_producer: Some(control_producer),
            active_client: None,
            lifecycle: Arc::new(AtomicU8::new(LIFECYCLE_IDLE)),
        }
    }

    /// Enqueue a control mutation without blocking the caller.
    ///
    /// # Errors
    ///
    /// Returns [`crate::rt_boundary::EnqueueError`] when queue is full or
    /// realtime processor is not connected.
    pub fn try_send_control(
        &self,
        command: crate::rt_boundary::AudioControl,
    ) -> Result<(), crate::rt_boundary::EnqueueError> {
        if self.lifecycle.load(Ordering::Acquire) != LIFECYCLE_ACTIVE {
            return Err(crate::rt_boundary::EnqueueError::Disconnected);
        }
        self.control_producer
            .as_ref()
            .ok_or(crate::rt_boundary::EnqueueError::Disconnected)?
            .try_send(command)?;
        // Recheck lifecycle after enqueue. Callers must still tolerate a
        // concurrent shutdown racing with this non-blocking operation.
        if self.lifecycle.load(Ordering::Acquire) != LIFECYCLE_ACTIVE {
            return Err(crate::rt_boundary::EnqueueError::Disconnected);
        }
        Ok(())
    }

    /// Number of control commands rejected by a full queue.
    #[must_use]
    pub fn dropped_control_commands(&self) -> u64 {
        self.control_producer
            .as_ref()
            .map_or(0, crate::rt_boundary::ControlProducer::dropped_commands)
    }
}

impl Backend for JackBackend {
    fn activate(&mut self) -> BackendResult<()> {
        if self.active_client.is_some() {
            if self.lifecycle.load(Ordering::Acquire) == LIFECYCLE_ACTIVE {
                return Err(AudioEngineError::InvalidState {
                    reason: "JACK client already active".into(),
                });
            }
            // JACK may have invoked shutdown without dropping AsyncClient.
            // Release stale handle before attempting a fresh activation.
            self.active_client = None;
        }

        let (client, _status) = Client::new(&self.client_name, ClientOptions::NO_START_SERVER)
            .map_err(|e| AudioEngineError::BackendInit(e.to_string()))?;

        // Update sample rate from what JACK negotiated.
        self.sample_rate = client.sample_rate() as u32;
        self.buffer_frames = client.buffer_size() as u32;

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

        let processor = match self.processor.take() {
            Some(processor) => processor,
            None => {
                let (processor, producer) = RealtimeProcessor::new(self.engine_template.clone());
                self.control_producer = Some(producer);
                processor
            }
        };

        let process = JackProcessHandler {
            in_ports,
            out_l,
            out_r,
            processor,
        };
        self.lifecycle
            .store(LIFECYCLE_ACTIVATING, Ordering::Release);
        let notification = JackNotificationHandler {
            lifecycle: Arc::clone(&self.lifecycle),
        };

        let async_client = match client.activate_async(notification, process) {
            Ok(client) => client,
            Err(error) => {
                self.lifecycle.store(LIFECYCLE_IDLE, Ordering::Release);
                return Err(AudioEngineError::BackendActivate(error.to_string()));
            }
        };

        self.active_client = Some(ActiveClient {
            _handle: async_client,
        });
        // Shutdown uses CAS against ACTIVATING, so it cannot be overwritten
        // by this publication if JACK already declared client dead.
        if self
            .lifecycle
            .compare_exchange(
                LIFECYCLE_ACTIVATING,
                LIFECYCLE_ACTIVE,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            self.active_client = None;
            return Err(AudioEngineError::BackendActivate(
                "JACK client shut down during activation".into(),
            ));
        }

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
        self.lifecycle.store(LIFECYCLE_IDLE, Ordering::Release);
        self.active_client = None;
        log::info!("[JACK] audio-engine deactivated");
    }

    fn is_active(&self) -> bool {
        self.active_client.is_some() && self.lifecycle.load(Ordering::Acquire) == LIFECYCLE_ACTIVE
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
    processor: RealtimeProcessor,
}

impl jack::ProcessHandler for JackProcessHandler {
    fn process(&mut self, _: &Client, ps: &ProcessScope) -> Control {
        let frames = self.out_l.as_slice(ps).len();
        for frame in 0..frames {
            let mut samples = [0.0_f32; MAX_CHANNELS];
            for (i, port) in self.in_ports.iter().enumerate() {
                if let Some(sample) = samples.get_mut(i) {
                    *sample = port.as_slice(ps).get(frame).copied().unwrap_or(0.0);
                }
            }
            let output = self.processor.process_frame(&samples);
            let (left, right) = output.mixes[0];
            self.out_l.as_mut_slice(ps)[frame] = left;
            self.out_r.as_mut_slice(ps)[frame] = right;
        }
        Control::Continue
    }
}

/// JACK notification handler — logs lifecycle events.
struct JackNotificationHandler {
    lifecycle: Arc<AtomicU8>,
}

impl jack::NotificationHandler for JackNotificationHandler {
    unsafe fn shutdown(&mut self, _: jack::ClientStatus, _: &str) {
        let _ = self.lifecycle.compare_exchange(
            LIFECYCLE_ACTIVATING,
            LIFECYCLE_IDLE,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        let _ = self.lifecycle.compare_exchange(
            LIFECYCLE_ACTIVE,
            LIFECYCLE_IDLE,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
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
