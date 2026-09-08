//! Error types for the audio engine layer.

use std::fmt;

/// Errors produced by the audio engine.
#[derive(Debug)]
pub enum AudioEngineError {
    /// Backend failed to initialise (JACK client creation, etc.).
    BackendInit(String),
    /// Backend failed to activate (start processing).
    BackendActivate(String),
    /// Backend is not in a state that allows the requested operation.
    InvalidState {
        /// Human-readable description.
        reason: String,
    },
    /// The mix-engine reported an error during configuration.
    MixEngine(mix_engine::mix_engine::EngineError),
}

impl fmt::Display for AudioEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BackendInit(msg) => write!(f, "audio backend init failed: {msg}"),
            Self::BackendActivate(msg) => write!(f, "audio backend activate failed: {msg}"),
            Self::InvalidState { reason } => {
                write!(f, "invalid audio engine state: {reason}")
            }
            Self::MixEngine(e) => write!(f, "mix engine error: {e}"),
        }
    }
}

impl std::error::Error for AudioEngineError {}

impl From<mix_engine::mix_engine::EngineError> for AudioEngineError {
    fn from(e: mix_engine::mix_engine::EngineError) -> Self {
        Self::MixEngine(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_backend_init() {
        let e = AudioEngineError::BackendInit("no JACK".into());
        assert!(e.to_string().contains("no JACK"));
    }

    #[test]
    fn test_display_invalid_state() {
        let e = AudioEngineError::InvalidState {
            reason: "already running".into(),
        };
        assert!(e.to_string().contains("already running"));
    }
}
