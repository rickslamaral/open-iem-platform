//! Open IEM Platform — Topology capability model and Channel Mode validation.
//!
//! Pure data + pure validation crate. No I/O, no async, no heap allocation in
//! hot paths. All configuration heap allocation happens at the control-plane
//! level, never in the audio path.
//!
//! # Usage
//!
//! ```rust
//! use topology::{MVP_CAPABILITIES, ChannelModeConfig, validate};
//!
//! let config = ChannelModeConfig {
//!     channel_count: 4,
//!     mix_count: 2,
//!     receiver_count: 2,
//!     source_names: vec![
//!         "Kick".to_string(),
//!         "Snare".to_string(),
//!         "Bass".to_string(),
//!         "Guitar".to_string(),
//!     ],
//! };
//! assert!(validate(&MVP_CAPABILITIES, &config).is_ok());
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]

use thiserror::Error;

/// Maximum byte length allowed for a source name.
pub const SOURCE_NAME_MAX_BYTES: usize = 64;

/// Supported topology operating modes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TopologyMode {
    /// Channel Mode: each input channel is routed to mixes via per-channel send
    /// controls. MVP-supported mode.
    ChannelMode,
    // AuxMono, AuxStereoPair, PlaybackStereo, Hybrid — deferred
}

/// System-level capability descriptor.
///
/// Describes the hard limits and supported modes for a given deployment.
/// The [`MVP_CAPABILITIES`] constant encodes the Phase 1 MVP constraints.
pub struct TopologyCapabilities {
    /// Maximum number of active input channels supported.
    pub max_channels: usize,
    /// Maximum number of active mixes supported.
    pub max_mixes: usize,
    /// Maximum number of wireless receivers supported.
    pub max_receivers: usize,
    /// Audio sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Codec frame duration in milliseconds.
    pub frame_duration_ms: u32,
    /// Topology modes enabled in this configuration.
    pub supported_modes: &'static [TopologyMode],
}

/// Channel Mode configuration to validate against [`TopologyCapabilities`].
pub struct ChannelModeConfig {
    /// Number of active input channels. Must be `1..=max_channels`.
    pub channel_count: usize,
    /// Number of active mixes. Must be `1..=max_mixes`.
    pub mix_count: usize,
    /// Number of receivers. Must be `1..=max_receivers` and equal to `mix_count`.
    pub receiver_count: usize,
    /// Human-readable name for each input channel slot (≤64 bytes each).
    /// Length must equal `channel_count`.
    pub source_names: Vec<String>,
}

/// Errors produced by topology validation.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum TopologyError {
    /// `channel_count` is outside `1..=max_channels`.
    #[error("channel_count {actual} out of range 1..={max}")]
    ChannelCountOutOfRange {
        /// The supplied value.
        actual: usize,
        /// The maximum allowed value.
        max: usize,
    },

    /// `mix_count` is outside `1..=max_mixes`.
    #[error("mix_count {actual} out of range 1..={max}")]
    MixCountOutOfRange {
        /// The supplied value.
        actual: usize,
        /// The maximum allowed value.
        max: usize,
    },

    /// `receiver_count` is outside `1..=max_receivers`.
    #[error("receiver_count {actual} out of range 1..={max}")]
    ReceiverCountOutOfRange {
        /// The supplied value.
        actual: usize,
        /// The maximum allowed value.
        max: usize,
    },

    /// `receiver_count` and `mix_count` must be equal (one receiver per mix).
    #[error("receiver_count {receivers} must equal mix_count {mixes} (one receiver per mix)")]
    ReceiverMixMismatch {
        /// The supplied `receiver_count`.
        receivers: usize,
        /// The supplied `mix_count`.
        mixes: usize,
    },

    /// `source_names.len()` does not equal `channel_count`.
    #[error("source_names length {actual} must equal channel_count {expected}")]
    SourceNamesMismatch {
        /// Actual length of `source_names`.
        actual: usize,
        /// Expected length (`channel_count`).
        expected: usize,
    },

    /// A source name exceeds [`SOURCE_NAME_MAX_BYTES`] bytes.
    #[error("source name at index {index} exceeds {max} bytes: {len} bytes")]
    SourceNameTooLong {
        /// Zero-based index of the offending name.
        index: usize,
        /// Byte length of the offending name.
        len: usize,
        /// Maximum allowed byte length.
        max: usize,
    },

    /// The requested [`TopologyMode`] is not in [`TopologyCapabilities::supported_modes`].
    #[error("topology mode {mode:?} is not supported by this configuration")]
    UnsupportedMode {
        /// The mode that was requested but not supported.
        mode: TopologyMode,
    },
}

/// MVP capability constant encoding Phase 1 hard limits.
///
/// - 8 mono logical input channels
/// - 2 independent stereo mixes
/// - 2 musicians / 2 receivers
/// - 48 kHz, Opus, 20 ms frames
/// - Channel Mode only
pub const MVP_CAPABILITIES: TopologyCapabilities = TopologyCapabilities {
    max_channels: mix_engine::MAX_CHANNELS,
    max_mixes: mix_engine::MAX_MIXES,
    max_receivers: 2,
    sample_rate_hz: mix_engine::SAMPLE_RATE,
    frame_duration_ms: 20,
    supported_modes: &[TopologyMode::ChannelMode],
};

/// Validate a [`ChannelModeConfig`] against [`TopologyCapabilities`].
///
/// Checks are applied in order; the first failure returns immediately.
///
/// # Errors
///
/// Returns [`TopologyError`] describing the first constraint violation found.
pub fn validate(
    caps: &TopologyCapabilities,
    config: &ChannelModeConfig,
) -> Result<(), TopologyError> {
    // 1. channel_count in 1..=caps.max_channels
    if config.channel_count == 0 || config.channel_count > caps.max_channels {
        return Err(TopologyError::ChannelCountOutOfRange {
            actual: config.channel_count,
            max: caps.max_channels,
        });
    }

    // 2. mix_count in 1..=caps.max_mixes
    if config.mix_count == 0 || config.mix_count > caps.max_mixes {
        return Err(TopologyError::MixCountOutOfRange {
            actual: config.mix_count,
            max: caps.max_mixes,
        });
    }

    // 3. receiver_count in 1..=caps.max_receivers
    if config.receiver_count == 0 || config.receiver_count > caps.max_receivers {
        return Err(TopologyError::ReceiverCountOutOfRange {
            actual: config.receiver_count,
            max: caps.max_receivers,
        });
    }

    // 4. receiver_count == mix_count
    if config.receiver_count != config.mix_count {
        return Err(TopologyError::ReceiverMixMismatch {
            receivers: config.receiver_count,
            mixes: config.mix_count,
        });
    }

    // 5. source_names.len() == channel_count
    if config.source_names.len() != config.channel_count {
        return Err(TopologyError::SourceNamesMismatch {
            actual: config.source_names.len(),
            expected: config.channel_count,
        });
    }

    // 6. each source name <= SOURCE_NAME_MAX_BYTES bytes
    for (index, name) in config.source_names.iter().enumerate() {
        let len = name.len();
        if len > SOURCE_NAME_MAX_BYTES {
            return Err(TopologyError::SourceNameTooLong {
                index,
                len,
                max: SOURCE_NAME_MAX_BYTES,
            });
        }
    }

    // 7. ChannelMode must be in supported_modes
    validate_mode(caps, TopologyMode::ChannelMode)?;

    Ok(())
}

/// Check whether a [`TopologyMode`] is supported by the given capabilities.
///
/// # Errors
///
/// Returns [`TopologyError::UnsupportedMode`] when `mode` is absent from
/// [`TopologyCapabilities::supported_modes`].
pub fn validate_mode(caps: &TopologyCapabilities, mode: TopologyMode) -> Result<(), TopologyError> {
    if caps.supported_modes.contains(&mode) {
        Ok(())
    } else {
        Err(TopologyError::UnsupportedMode { mode })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid config for MVP.
    fn valid_config() -> ChannelModeConfig {
        ChannelModeConfig {
            channel_count: 4,
            mix_count: 2,
            receiver_count: 2,
            source_names: vec![
                "Kick".to_string(),
                "Snare".to_string(),
                "Bass".to_string(),
                "Guitar".to_string(),
            ],
        }
    }

    #[test]
    fn mvp_caps_valid_config() {
        assert!(validate(&MVP_CAPABILITIES, &valid_config()).is_ok());
    }

    #[test]
    fn channel_count_zero_rejected() {
        let mut cfg = valid_config();
        cfg.channel_count = 0;
        cfg.source_names = vec![];
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::ChannelCountOutOfRange {
                actual: 0,
                max: mix_engine::MAX_CHANNELS,
            })
        );
    }

    #[test]
    fn channel_count_overflow_rejected() {
        let overflow = mix_engine::MAX_CHANNELS + 1;
        let mut cfg = valid_config();
        cfg.channel_count = overflow;
        cfg.source_names = vec!["x".to_string(); overflow];
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::ChannelCountOutOfRange {
                actual: overflow,
                max: mix_engine::MAX_CHANNELS,
            })
        );
    }

    #[test]
    fn mix_count_zero_rejected() {
        let mut cfg = valid_config();
        cfg.mix_count = 0;
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::MixCountOutOfRange {
                actual: 0,
                max: mix_engine::MAX_MIXES,
            })
        );
    }

    #[test]
    fn mix_count_overflow_rejected() {
        let overflow = mix_engine::MAX_MIXES + 1;
        let mut cfg = valid_config();
        cfg.mix_count = overflow;
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::MixCountOutOfRange {
                actual: overflow,
                max: mix_engine::MAX_MIXES,
            })
        );
    }

    #[test]
    fn receiver_count_zero_rejected() {
        let mut cfg = valid_config();
        cfg.receiver_count = 0;
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::ReceiverCountOutOfRange {
                actual: 0,
                max: MVP_CAPABILITIES.max_receivers,
            })
        );
    }

    #[test]
    fn receiver_count_overflow_rejected() {
        let overflow = MVP_CAPABILITIES.max_receivers + 1;
        let mut cfg = valid_config();
        // mix_count stays at valid_config() default (2 == max_mixes) so only receiver check fires
        cfg.receiver_count = overflow;
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::ReceiverCountOutOfRange {
                actual: overflow,
                max: MVP_CAPABILITIES.max_receivers,
            })
        );
    }

    #[test]
    fn receiver_mix_mismatch_rejected() {
        let mut cfg = valid_config();
        cfg.receiver_count = 1;
        cfg.mix_count = 2;
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::ReceiverMixMismatch {
                receivers: 1,
                mixes: 2,
            })
        );
    }

    #[test]
    fn source_names_length_mismatch() {
        let mut cfg = valid_config();
        // channel_count = 4, but give only 3 names
        cfg.source_names = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::SourceNamesMismatch {
                actual: 3,
                expected: 4,
            })
        );
    }

    #[test]
    fn source_name_too_long() {
        let mut cfg = valid_config();
        // Replace first name with a 65-byte string
        cfg.source_names[0] = "x".repeat(65);
        assert_eq!(
            validate(&MVP_CAPABILITIES, &cfg),
            Err(TopologyError::SourceNameTooLong {
                index: 0,
                len: 65,
                max: SOURCE_NAME_MAX_BYTES,
            })
        );
    }

    #[test]
    fn unsupported_mode_rejected() {
        const NO_MODES: TopologyCapabilities = TopologyCapabilities {
            max_channels: mix_engine::MAX_CHANNELS,
            max_mixes: mix_engine::MAX_MIXES,
            max_receivers: 2,
            sample_rate_hz: mix_engine::SAMPLE_RATE,
            frame_duration_ms: 20,
            supported_modes: &[],
        };
        assert_eq!(
            validate_mode(&NO_MODES, TopologyMode::ChannelMode),
            Err(TopologyError::UnsupportedMode {
                mode: TopologyMode::ChannelMode,
            })
        );
    }
}
