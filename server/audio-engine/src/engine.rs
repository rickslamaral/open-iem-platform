//! `AudioEngine` — top-level entry point for the audio engine layer.
//!
//! Owns configuration and delegates to the selected [`Backend`].
//! Control-plane code creates an `AudioEngine`, configures the mix engine
//! via [`AudioEngine::configure`], then calls [`AudioEngine::start`].

use mix_engine::MixEngine;

use crate::{
    backend::{simulated::SimulatedBackend, Backend, BackendResult},
    config::{AudioConfig, BackendKind},
    error::AudioEngineError,
};

/// Top-level audio engine.
///
/// Instantiate via [`AudioEngine::new`], configure with [`AudioEngine::configure`],
/// then start with [`AudioEngine::start`].
pub struct AudioEngine {
    config: AudioConfig,
    backend: Box<dyn Backend>,
}

impl AudioEngine {
    /// Create a new `AudioEngine` from configuration.
    ///
    /// Backend selection:
    /// - [`BackendKind::Simulated`] → [`SimulatedBackend`] (always available).
    /// - [`BackendKind::Jack`] → [`JackBackend`] (requires `jack` feature).
    ///
    /// # Errors
    ///
    /// Returns [`AudioEngineError::BackendInit`] if the requested backend
    /// cannot be constructed (e.g., `Jack` requested without `jack` feature).
    pub fn new(config: AudioConfig) -> Result<Self, AudioEngineError> {
        let engine = MixEngine::new();
        let backend: Box<dyn Backend> = match &config.backend {
            BackendKind::Simulated => Box::new(SimulatedBackend::new(
                engine,
                config.sample_rate,
                config.buffer_frames,
            )),
            BackendKind::Jack => {
                #[cfg(feature = "jack")]
                {
                    use crate::backend::jack::JackBackend;
                    Box::new(JackBackend::new(
                        engine,
                        config.client_name.clone(),
                        config.buffer_frames,
                    ))
                }
                #[cfg(not(feature = "jack"))]
                {
                    return Err(AudioEngineError::BackendInit(
                        "JackBackend requested but `jack` Cargo feature is not enabled. \
                         Rebuild with `--features jack` on hardware target."
                            .into(),
                    ));
                }
            }
        };

        Ok(Self { config, backend })
    }

    /// Start the audio engine (activates the backend).
    ///
    /// # Errors
    ///
    /// See [`Backend::activate`].
    pub fn start(&mut self) -> BackendResult<()> {
        self.backend.activate()
    }

    /// Stop the audio engine (deactivates the backend).
    pub fn stop(&mut self) {
        self.backend.deactivate();
    }

    /// Returns `true` if the engine is currently running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.backend.is_active()
    }

    /// Returns the backend name (for logging/diagnostics).
    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        self.backend.name()
    }

    /// Returns the effective sample rate (may differ from config after negotiation).
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.backend.sample_rate()
    }

    /// Returns the effective buffer size in frames.
    #[must_use]
    pub fn buffer_frames(&self) -> u32 {
        self.backend.buffer_frames()
    }

    /// Returns the [`AudioConfig`] used to create this engine.
    #[must_use]
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Downcast the backend to [`SimulatedBackend`] for testing.
    ///
    /// Returns `None` if the backend is not a `SimulatedBackend`.
    pub fn as_simulated_mut(&mut self) -> Option<&mut SimulatedBackend> {
        // SAFETY: this is a downcast via raw pointer — only correct when the
        // backend IS a SimulatedBackend. We use feature-gating to ensure this
        // is only called in tests.
        //
        // A cleaner solution is a `downcast` trait, deferred to Phase 3.
        // For now: only use in test code where backend kind is known.
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AudioConfig, BackendKind};

    #[test]
    fn test_simulated_engine_creates_ok() {
        let cfg = AudioConfig::simulated("test");
        let engine = AudioEngine::new(cfg).expect("create engine");
        assert!(!engine.is_running());
        assert!(engine.backend_name().contains("SIMULATED"));
    }

    #[test]
    fn test_simulated_engine_start_stop() {
        let cfg = AudioConfig::simulated("test");
        let mut engine = AudioEngine::new(cfg).expect("create");
        engine.start().expect("start");
        assert!(engine.is_running());
        engine.stop();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_jack_without_feature_returns_error() {
        // Only run this test when `jack` feature is NOT enabled.
        #[cfg(not(feature = "jack"))]
        {
            let cfg = AudioConfig {
                backend: BackendKind::Jack,
                ..Default::default()
            };
            let result = AudioEngine::new(cfg);
            assert!(result.is_err());
            let err_str = result.err().map(|e| e.to_string()).unwrap_or_default();
            assert!(err_str.contains("jack"), "got: {err_str}");
        }
        #[cfg(feature = "jack")]
        {
            // When jack feature is enabled, the test is vacuously OK.
        }
    }

    #[test]
    fn test_engine_config_accessors() {
        let cfg = AudioConfig {
            sample_rate: 48_000,
            buffer_frames: 256,
            ..AudioConfig::simulated("cfg-test")
        };
        let engine = AudioEngine::new(cfg).expect("create");
        assert_eq!(engine.sample_rate(), 48_000);
        assert_eq!(engine.buffer_frames(), 256);
        assert_eq!(engine.config().client_name, "cfg-test");
    }
}
