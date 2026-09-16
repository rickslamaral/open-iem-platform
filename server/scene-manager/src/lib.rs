//! Strict, immutable scene model for durable control-plane configuration.
#![deny(missing_docs)]
#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Current persisted scene schema.
pub const SCHEMA_VERSION: u32 = 1;
/// Maximum UTF-8 bytes accepted for scene names.
pub const MAX_SCENE_NAME_BYTES: usize = 64;
/// Maximum UTF-8 bytes accepted for IDs and channel/mix names.
pub const MAX_TEXT_BYTES: usize = 64;
/// Minimum accepted gain in dB.
pub const GAIN_DB_MIN: f32 = -144.0;
/// Maximum accepted gain in dB.
pub const GAIN_DB_MAX: f32 = 12.0;
/// Maximum serialized payload size.
pub const MAX_PAYLOAD_BYTES: usize = 512 * 1024;

/// Strict durable scene configuration. Runtime state has no representation here.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SceneConfig {
    /// Persisted channel configuration.
    pub channels: Vec<ChannelConfig>,
    /// Persisted mix configuration.
    pub mixes: Vec<MixConfig>,
}

/// Persisted channel settings.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelConfig {
    /// Channel slot.
    pub slot: u8,
    /// Stable channel ID.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Input gain in dB.
    pub gain_db: f32,
    /// Global mute.
    pub muted: bool,
    /// Engineer lock.
    pub locked: bool,
    /// Enabled state.
    pub enabled: bool,
}

/// Persisted mix settings.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MixConfig {
    /// Mix slot.
    pub slot: u8,
    /// Stable mix ID.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Master gain in dB.
    pub master_gain_db: f32,
    /// Master mute.
    pub master_muted: bool,
}

/// Scene metadata and current immutable revision.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    /// Stable scene ID.
    pub id: String,
    /// Bounded non-empty name.
    pub name: String,
    /// Schema version.
    pub schema_version: u32,
    /// Monotonic immutable revision number.
    pub revision: u64,
    /// Durable configuration.
    pub config: SceneConfig,
}

/// Scene validation failures.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SceneError {
    /// Scene name is empty or too large.
    #[error("scene name must contain 1..={max} bytes")]
    InvalidName {
        /// Maximum accepted name length.
        max: usize,
    },
    /// Schema version is unsupported.
    #[error("unsupported scene schema version {0}")]
    UnsupportedVersion(u32),
    /// Payload exceeds bounded input size.
    #[error("scene payload exceeds {MAX_PAYLOAD_BYTES} bytes")]
    PayloadTooLarge,
    /// JSON is malformed or contains unknown fields.
    #[error("invalid scene payload: {0}")]
    InvalidPayload(String),
    /// Slot exceeds MVP bounds.
    #[error("scene slot out of range")]
    InvalidSlot,
    /// Scene contains duplicate slots.
    #[error("scene contains duplicate slots")]
    DuplicateSlot,
    /// Scene contains duplicate stable IDs.
    #[error("scene contains duplicate stable IDs")]
    DuplicateId,
    /// A text field is empty, oversized, or contains invalid data.
    #[error("invalid scene text field")]
    InvalidText,
    /// A numeric field is non-finite or outside its safe range.
    #[error("invalid scene numeric field")]
    InvalidNumber,
}

/// Validate scene metadata and control-plane bounds.
///
/// # Errors
/// Returns a [`SceneError`] when metadata or bounds are invalid.
pub fn validate(scene: &Scene) -> Result<(), SceneError> {
    if scene.id.is_empty() || scene.id.len() > MAX_TEXT_BYTES {
        return Err(SceneError::InvalidText);
    }
    if scene.name.is_empty() || scene.name.len() > MAX_SCENE_NAME_BYTES {
        return Err(SceneError::InvalidName {
            max: MAX_SCENE_NAME_BYTES,
        });
    }
    if scene.schema_version != SCHEMA_VERSION {
        return Err(SceneError::UnsupportedVersion(scene.schema_version));
    }
    let mut channel_slots = HashSet::new();
    let mut channel_ids = HashSet::new();
    for channel in &scene.config.channels {
        if channel.slot >= 8 {
            return Err(SceneError::InvalidSlot);
        }
        if !channel_slots.insert(channel.slot) {
            return Err(SceneError::DuplicateSlot);
        }
        if channel.id == 0 || !channel_ids.insert(channel.id) {
            return Err(SceneError::DuplicateId);
        }
        if channel.name.is_empty() || channel.name.len() > MAX_TEXT_BYTES {
            return Err(SceneError::InvalidText);
        }
        if !channel.gain_db.is_finite() || !(GAIN_DB_MIN..=GAIN_DB_MAX).contains(&channel.gain_db) {
            return Err(SceneError::InvalidNumber);
        }
    }
    let mut mix_slots = HashSet::new();
    let mut mix_ids = HashSet::new();
    for mix in &scene.config.mixes {
        if mix.slot >= 2 {
            return Err(SceneError::InvalidSlot);
        }
        if !mix_slots.insert(mix.slot) {
            return Err(SceneError::DuplicateSlot);
        }
        if mix.id == 0 || !mix_ids.insert(mix.id) {
            return Err(SceneError::DuplicateId);
        }
        if mix.name.is_empty() || mix.name.len() > MAX_TEXT_BYTES {
            return Err(SceneError::InvalidText);
        }
        if !mix.master_gain_db.is_finite()
            || !(GAIN_DB_MIN..=GAIN_DB_MAX).contains(&mix.master_gain_db)
        {
            return Err(SceneError::InvalidNumber);
        }
    }
    Ok(())
}

/// Serialize validated scene as bounded JSON.
///
/// # Errors
/// Returns a [`SceneError`] when validation fails or payload exceeds limit.
pub fn encode(scene: &Scene) -> Result<String, SceneError> {
    validate(scene)?;
    let json =
        serde_json::to_string(scene).map_err(|e| SceneError::InvalidPayload(e.to_string()))?;
    if json.len() > MAX_PAYLOAD_BYTES {
        return Err(SceneError::PayloadTooLarge);
    }
    Ok(json)
}

/// Decode and validate strict scene JSON.
///
/// # Errors
/// Returns a [`SceneError`] when JSON is invalid, unknown fields exist, or bounds fail.
pub fn decode(json: &str) -> Result<Scene, SceneError> {
    if json.len() > MAX_PAYLOAD_BYTES {
        return Err(SceneError::PayloadTooLarge);
    }
    let scene: Scene =
        serde_json::from_str(json).map_err(|e| SceneError::InvalidPayload(e.to_string()))?;
    validate(&scene)?;
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scene() -> Scene {
        Scene {
            id: "scene-1".into(),
            name: "Show".into(),
            schema_version: 1,
            revision: 1,
            config: SceneConfig {
                channels: vec![],
                mixes: vec![],
            },
        }
    }
    #[test]
    fn round_trip_preserves_durable_data() {
        let s = scene();
        assert_eq!(decode(&encode(&s).unwrap()).unwrap(), s);
    }
    #[test]
    fn unknown_fields_fail_closed() {
        let j = r#"{"id":"x","name":"x","schema_version":1,"revision":1,"config":{"channels":[],"mixes":[]},"runtime":{}}"#;
        assert!(matches!(decode(j), Err(SceneError::InvalidPayload(_))));
    }
    #[test]
    fn unknown_nested_fields_fail_closed() {
        let j = r#"{"id":"x","name":"x","schema_version":1,"revision":1,"config":{"channels":[{"slot":0,"id":1,"name":"x","gain_db":0,"muted":false,"locked":false,"enabled":true,"token":"no"}],"mixes":[]}}"#;
        assert!(matches!(decode(j), Err(SceneError::InvalidPayload(_))));
    }
    #[test]
    fn invalid_version_and_slot_rejected() {
        let mut s = scene();
        s.schema_version = 2;
        assert_eq!(validate(&s), Err(SceneError::UnsupportedVersion(2)));
        s.schema_version = 1;
        s.config.mixes.push(MixConfig {
            slot: 2,
            id: 1,
            name: "x".into(),
            master_gain_db: 0.0,
            master_muted: false,
        });
        assert_eq!(validate(&s), Err(SceneError::InvalidSlot));
    }
    #[test]
    fn oversized_name_rejected() {
        let mut s = scene();
        s.name = "x".repeat(MAX_SCENE_NAME_BYTES + 1);
        assert_eq!(
            validate(&s),
            Err(SceneError::InvalidName {
                max: MAX_SCENE_NAME_BYTES
            })
        );
    }

    #[test]
    fn duplicate_slots_and_nonfinite_gain_rejected() {
        let mut s = scene();
        s.config.channels = vec![
            ChannelConfig {
                slot: 0,
                id: 1,
                name: "A".into(),
                gain_db: 0.0,
                muted: false,
                locked: false,
                enabled: true,
            },
            ChannelConfig {
                slot: 0,
                id: 2,
                name: "B".into(),
                gain_db: 0.0,
                muted: false,
                locked: false,
                enabled: true,
            },
        ];
        assert_eq!(validate(&s), Err(SceneError::DuplicateSlot));
        s.config.channels.truncate(1);
        s.config.channels[0].gain_db = f32::NAN;
        assert_eq!(validate(&s), Err(SceneError::InvalidNumber));
    }

    #[test]
    fn zero_and_duplicate_ids_rejected() {
        let mut s = scene();
        s.config.channels = vec![ChannelConfig {
            slot: 0,
            id: 0,
            name: "A".into(),
            gain_db: 0.0,
            muted: false,
            locked: false,
            enabled: true,
        }];
        assert_eq!(validate(&s), Err(SceneError::DuplicateId));
        s.config.channels[0].id = 1;
        s.config.channels.push(ChannelConfig {
            slot: 1,
            id: 1,
            name: "B".into(),
            gain_db: 0.0,
            muted: false,
            locked: false,
            enabled: true,
        });
        assert_eq!(validate(&s), Err(SceneError::DuplicateId));
    }
}
