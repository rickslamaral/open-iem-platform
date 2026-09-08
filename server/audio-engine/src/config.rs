//! Audio engine runtime configuration.

use mix_engine::SAMPLE_RATE;

/// Which backend to use for audio I/O.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BackendKind {
    /// JACK/pipewire-jack backend (hardware target).
    /// Requires `jack` Cargo feature and `libjack.so` on host.
    Jack,
    /// Simulated backend — runs DSP against synthetic signal.
    /// Used on VPS / CI where no audio hardware is available.
    /// **SIMULATED**
    #[default]
    Simulated,
}

// Default impl derived — see #[default] on Simulated variant above.

/// Audio engine configuration.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// JACK/`PipeWire` client name (ignored by simulated backend).
    pub client_name: String,
    /// Sample rate in Hz. Must match the `PipeWire` graph quantum.
    pub sample_rate: u32,
    /// Audio buffer size in frames (quantum).
    /// `PipeWire` default: 1024 frames @ 48 kHz → ~21.3 ms.
    /// For IEM monitoring target: 256 frames @ 48 kHz → ~5.3 ms.
    pub buffer_frames: u32,
    /// Which backend to instantiate.
    pub backend: BackendKind,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            client_name: "open-iem".into(),
            sample_rate: SAMPLE_RATE,
            buffer_frames: 256,
            backend: BackendKind::Simulated,
        }
    }
}

impl AudioConfig {
    /// Create a new config with the given client name and simulated backend.
    #[must_use]
    pub fn simulated(client_name: impl Into<String>) -> Self {
        Self {
            client_name: client_name.into(),
            backend: BackendKind::Simulated,
            ..Default::default()
        }
    }

    /// Create a new config targeting the JACK/`PipeWire` backend.
    /// Panics if the `jack` Cargo feature is not enabled.
    #[must_use]
    pub fn jack(client_name: impl Into<String>, buffer_frames: u32) -> Self {
        Self {
            client_name: client_name.into(),
            buffer_frames,
            backend: BackendKind::Jack,
            ..Default::default()
        }
    }

    /// Buffer duration in milliseconds (derived from `sample_rate` and `buffer_frames`).
    #[must_use]
    pub fn buffer_ms(&self) -> f64 {
        f64::from(self.buffer_frames) / f64::from(self.sample_rate) * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = AudioConfig::default();
        assert_eq!(cfg.sample_rate, 48_000);
        assert_eq!(cfg.buffer_frames, 256);
        assert_eq!(cfg.backend, BackendKind::Simulated);
    }

    #[test]
    fn test_buffer_ms_256_at_48k() {
        let cfg = AudioConfig::default(); // 256 frames @ 48 kHz
        let ms = cfg.buffer_ms();
        assert!((ms - 5.333_333).abs() < 0.001, "got {ms:.4} ms");
    }

    #[test]
    fn test_simulated_constructor() {
        let cfg = AudioConfig::simulated("test-client");
        assert_eq!(cfg.client_name, "test-client");
        assert_eq!(cfg.backend, BackendKind::Simulated);
    }
}
