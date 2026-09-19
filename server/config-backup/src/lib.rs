//! Config backup/restore for the Open IEM Platform.
//!
//! Captures channel + mix + send state without any secrets or transient
//! DSP state (biquad coefficients, runtime revisions, filter delay lines).

#![deny(missing_docs)]
#![deny(unsafe_code)]

use control_server::ControlState;
use mix_engine::{Channel, EqBand, Mix, MixSend, MAX_CHANNELS, MAX_EQ_BANDS};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// Errors produced by backup serialization / deserialization.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    /// JSON serialization failed.
    #[error("serialization failed: {0}")]
    Serialize(String),
    /// JSON deserialization failed.
    #[error("deserialization failed: {0}")]
    Deserialize(String),
}

/// Errors produced by snapshot restore.
#[derive(Debug, thiserror::Error)]
pub enum RestoreError {
    /// The mix engine returned an error while rebuilding state.
    #[error("engine error during restore: {0}")]
    Engine(String),
    /// Snapshot version is not supported.
    #[error("invalid snapshot version {0}: expected 1")]
    UnsupportedVersion(u32),
}

// ---------------------------------------------------------------------------
// Snapshot types
// ---------------------------------------------------------------------------

/// Serializable snapshot of an input channel (no secrets, no revision).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSnapshot {
    /// Slot index in the engine.
    pub slot: usize,
    /// Stable channel identifier.
    pub id: u32,
    /// Human-readable channel name.
    pub name: String,
    /// Gain in dB.
    pub gain_db: f32,
    /// Mute state.
    pub muted: bool,
    /// Lock state.
    pub locked: bool,
    /// Enable state.
    pub enabled: bool,
}

/// Serializable snapshot of one EQ band.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EqBandSnapshot {
    /// Band index (0-based).
    pub index: usize,
    /// Centre frequency in Hz.
    pub frequency_hz: f32,
    /// Gain in dB.
    pub gain_db: f32,
    /// Quality factor.
    pub q: f32,
    /// Whether this band is active.
    pub enabled: bool,
}

/// Serializable snapshot of a parametric EQ.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EqSnapshot {
    /// All bands (always [`MAX_EQ_BANDS`] entries for full restore fidelity).
    pub bands: Vec<EqBandSnapshot>,
}

/// Serializable snapshot of the compressor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompressorSnapshot {
    /// Threshold in dB.
    pub threshold_db: f32,
    /// Compression ratio.
    pub ratio: f32,
    /// Attack time in milliseconds.
    pub attack_ms: f32,
    /// Release time in milliseconds.
    pub release_ms: f32,
    /// Whether the compressor is active.
    pub enabled: bool,
}

/// Serializable snapshot of the output limiter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LimiterSnapshot {
    /// Threshold in dB.
    pub threshold_db: f32,
    /// Whether the limiter is active.
    pub enabled: bool,
}

/// Serializable snapshot of one channel send into a mix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct SendSnapshot {
    /// Channel slot index (0-based).
    pub channel_index: usize,
    /// Stable channel identifier.
    pub channel_id: u32,
    /// Stable mix identifier.
    pub mix_id: u32,
    /// Send gain in dB.
    pub gain_db: f32,
    /// Pan position (−1.0 = hard left, 0.0 = centre, 1.0 = hard right).
    pub pan: f32,
    /// Send mute state.
    pub muted: bool,
    /// Send solo state.
    pub solo: bool,
    /// Send enable state.
    pub enabled: bool,
    /// Send lock state.
    pub locked: bool,
}

/// Serializable snapshot of one output mix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MixSnapshot {
    /// Slot index in the engine.
    pub slot: usize,
    /// Stable mix identifier.
    pub id: u32,
    /// Human-readable mix name.
    pub name: String,
    /// Master output gain in dB.
    pub master_gain_db: f32,
    /// Master mute state.
    pub master_muted: bool,
    /// Parametric EQ state.
    pub eq: EqSnapshot,
    /// Compressor state.
    pub compressor: CompressorSnapshot,
    /// Limiter state.
    pub limiter: LimiterSnapshot,
    /// Active sends into this mix.
    pub sends: Vec<SendSnapshot>,
}

/// Complete serializable snapshot of the mix engine control-plane state.
///
/// Does **not** contain any credentials, tokens, passwords, or transient DSP
/// state (biquad coefficients, revision counters, filter delay lines).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSnapshot {
    /// Snapshot format version — always 1 for this crate.
    pub version: u32,
    /// Creation timestamp (UTC seconds since Unix epoch). Set to 0 when
    /// `std::time` is unavailable.
    pub created_at_utc_secs: u64,
    /// Configured channels (only slots that are set).
    pub channels: Vec<ChannelSnapshot>,
    /// Configured mixes (only slots that are set).
    pub mixes: Vec<MixSnapshot>,
}

// ---------------------------------------------------------------------------
// backup
// ---------------------------------------------------------------------------

/// Build a [`ConfigSnapshot`] from the current [`ControlState`].
///
/// Reads every configured channel and mix. Transient runtime state
/// (revision counters, biquad coefficients / delay lines) is intentionally
/// omitted.
#[must_use]
pub fn backup(state: &ControlState) -> ConfigSnapshot {
    let created_at_utc_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    let mut channels = Vec::new();
    state.for_each_channel(|slot, ch| {
        channels.push(ChannelSnapshot {
            slot,
            id: ch.id,
            name: ch.name().to_owned(),
            gain_db: ch.gain_db(),
            muted: ch.muted,
            locked: ch.locked,
            enabled: ch.enabled,
        });
    });

    let mut mixes = Vec::new();
    state.for_each_mix(|slot, mix| {
        let eq_bands: Vec<EqBandSnapshot> = (0..MAX_EQ_BANDS)
            .map(|i| {
                let band = mix.eq.bands[i];
                EqBandSnapshot {
                    index: i,
                    frequency_hz: band.frequency_hz,
                    gain_db: band.gain_db,
                    q: band.q,
                    enabled: band.enabled,
                }
            })
            .collect();

        let compressor = CompressorSnapshot {
            threshold_db: mix.compressor.threshold_db,
            ratio: mix.compressor.ratio,
            attack_ms: mix.compressor.attack_ms,
            release_ms: mix.compressor.release_ms,
            enabled: mix.compressor.enabled,
        };

        let limiter = LimiterSnapshot {
            threshold_db: mix.limiter.threshold_db(),
            enabled: mix.limiter.enabled,
        };

        let sends: Vec<SendSnapshot> = (0..MAX_CHANNELS)
            .filter_map(|ch_idx| {
                mix.send(ch_idx).map(|send| SendSnapshot {
                    channel_index: ch_idx,
                    channel_id: send.channel_id,
                    mix_id: send.mix_id,
                    gain_db: send.gain_db(),
                    pan: send.pan(),
                    muted: send.muted,
                    solo: send.solo,
                    enabled: send.enabled,
                    locked: send.locked,
                })
            })
            .collect();

        mixes.push(MixSnapshot {
            slot,
            id: mix.id,
            name: mix.name().to_owned(),
            master_gain_db: mix.master_gain_db(),
            master_muted: mix.master_muted,
            eq: EqSnapshot { bands: eq_bands },
            compressor,
            limiter,
            sends,
        });
    });

    ConfigSnapshot {
        version: 1,
        created_at_utc_secs,
        channels,
        mixes,
    }
}

// ---------------------------------------------------------------------------
// restore
// ---------------------------------------------------------------------------

/// Rebuild [`ControlState`] from a [`ConfigSnapshot`].
///
/// Only slots present in the snapshot are written; existing slots not
/// mentioned in the snapshot are left untouched.
///
/// # Errors
/// Returns [`RestoreError::UnsupportedVersion`] when `snapshot.version != 1`.
/// Returns [`RestoreError::Engine`] when the engine rejects a slot index.
fn validate_snapshot(snapshot: &ConfigSnapshot) -> Result<(), RestoreError> {
    if snapshot.version != 1 {
        return Err(RestoreError::UnsupportedVersion(snapshot.version));
    }
    let mut channel_slots = HashSet::new();
    let mut channel_ids = HashSet::new();
    let mut channel_ids_by_slot = [None; MAX_CHANNELS];
    for channel in &snapshot.channels {
        if channel.slot >= MAX_CHANNELS
            || !channel_slots.insert(channel.slot)
            || !channel_ids.insert(channel.id)
            || channel.name.is_empty()
            || channel.name.len() > 64
            || channel.id == 0
            || !channel.gain_db.is_finite()
            || !(mix_engine::GAIN_DB_MIN..=mix_engine::GAIN_DB_MAX).contains(&channel.gain_db)
        {
            return Err(RestoreError::Engine("invalid channel snapshot".to_owned()));
        }
        channel_ids_by_slot[channel.slot] = Some(channel.id);
    }
    let mut mix_slots = HashSet::new();
    let mut mix_ids = HashSet::new();
    for mix in &snapshot.mixes {
        if mix.slot >= mix_engine::MAX_MIXES
            || !mix_slots.insert(mix.slot)
            || !mix_ids.insert(mix.id)
            || mix.name.is_empty()
            || mix.name.len() > 64
            || mix.id == 0
            || !mix.master_gain_db.is_finite()
            || !(mix_engine::GAIN_DB_MIN..=mix_engine::GAIN_DB_MAX).contains(&mix.master_gain_db)
            || !mix.compressor.threshold_db.is_finite()
            || !mix.compressor.ratio.is_finite()
            || !mix.compressor.attack_ms.is_finite()
            || !mix.compressor.release_ms.is_finite()
            || !mix.limiter.threshold_db.is_finite()
        {
            return Err(RestoreError::Engine("invalid mix snapshot".to_owned()));
        }
        let mut band_indices = HashSet::new();
        if mix.eq.bands.len() != MAX_EQ_BANDS {
            return Err(RestoreError::Engine("invalid EQ snapshot".to_owned()));
        }
        let mut send_indices = HashSet::new();
        for band in &mix.eq.bands {
            if band.index >= MAX_EQ_BANDS
                || !band_indices.insert(band.index)
                || !band.frequency_hz.is_finite()
                || !(20.0..=20_000.0).contains(&band.frequency_hz)
                || !band.gain_db.is_finite()
                || !(mix_engine::GAIN_DB_MIN..=mix_engine::GAIN_DB_MAX).contains(&band.gain_db)
                || !band.q.is_finite()
                || !(0.1..=10.0).contains(&band.q)
            {
                return Err(RestoreError::Engine("invalid EQ snapshot".to_owned()));
            }
        }
        for send in &mix.sends {
            if send.channel_index >= MAX_CHANNELS
                || !send_indices.insert(send.channel_index)
                || channel_ids_by_slot[send.channel_index] != Some(send.channel_id)
                || send.mix_id != mix.id
                || send.mix_id == 0
                || !send.gain_db.is_finite()
                || !(mix_engine::GAIN_DB_MIN..=mix_engine::GAIN_DB_MAX).contains(&send.gain_db)
                || !send.pan.is_finite()
                || !(-1.0..=1.0).contains(&send.pan)
            {
                return Err(RestoreError::Engine("invalid send snapshot".to_owned()));
            }
        }
    }
    Ok(())
}

/// Apply validated snapshot atomically to control state.
///
/// # Errors
/// Returns an error when snapshot version or values are unsupported.
pub fn restore(snapshot: &ConfigSnapshot, state: &mut ControlState) -> Result<(), RestoreError> {
    validate_snapshot(snapshot)?;
    let mut candidate = state.clone();
    restore_in_place(snapshot, &mut candidate)?;
    *state = candidate;
    Ok(())
}

fn restore_in_place(
    snapshot: &ConfigSnapshot,
    state: &mut ControlState,
) -> Result<(), RestoreError> {
    if snapshot.version != 1 {
        return Err(RestoreError::UnsupportedVersion(snapshot.version));
    }

    for ch_snap in &snapshot.channels {
        let mut channel = Channel::new(ch_snap.id, &ch_snap.name);
        channel.set_gain_db(ch_snap.gain_db);
        channel.set_muted(ch_snap.muted);
        channel.set_locked(ch_snap.locked);
        channel.set_enabled(ch_snap.enabled);
        state
            .set_channel(ch_snap.slot, channel)
            .map_err(|e| RestoreError::Engine(e.to_string()))?;
    }

    for mix_snap in &snapshot.mixes {
        let mut mix = Mix::new(mix_snap.id, &mix_snap.name);
        mix.set_master_gain_db(mix_snap.master_gain_db);
        mix.set_master_muted(mix_snap.master_muted);

        for band_snap in &mix_snap.eq.bands {
            mix.eq.set_band(
                band_snap.index,
                EqBand {
                    frequency_hz: band_snap.frequency_hz,
                    gain_db: band_snap.gain_db,
                    q: band_snap.q,
                    enabled: band_snap.enabled,
                },
            );
        }

        let comp = &mix_snap.compressor;
        mix.compressor.set_threshold(comp.threshold_db);
        mix.compressor.set_ratio(comp.ratio);
        mix.compressor.set_attack_ms(comp.attack_ms);
        mix.compressor.set_release_ms(comp.release_ms);
        mix.compressor.set_enabled(comp.enabled);

        mix.limiter.set_threshold_db(mix_snap.limiter.threshold_db);
        mix.limiter.set_enabled(mix_snap.limiter.enabled);

        for send_snap in &mix_snap.sends {
            let mut send = MixSend::new(send_snap.channel_id, send_snap.mix_id);
            send.set_gain_db(send_snap.gain_db);
            send.set_pan(send_snap.pan);
            send.set_muted(send_snap.muted);
            send.set_solo(send_snap.solo);
            send.set_enabled(send_snap.enabled);
            send.set_locked(send_snap.locked);
            mix.set_send(send_snap.channel_index, send)
                .map_err(|e| RestoreError::Engine(e.to_string()))?;
        }

        state
            .set_mix(mix_snap.slot, mix)
            .map_err(|e| RestoreError::Engine(e.to_string()))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// serialize / deserialize
// ---------------------------------------------------------------------------

/// Serialize a [`ConfigSnapshot`] to pretty-printed JSON.
///
/// # Errors
/// Returns [`BackupError::Serialize`] on JSON encoding failure.
pub fn serialize(snapshot: &ConfigSnapshot) -> Result<String, BackupError> {
    serde_json::to_string_pretty(snapshot).map_err(|e| BackupError::Serialize(e.to_string()))
}

/// Deserialize a [`ConfigSnapshot`] from JSON.
///
/// # Errors
/// Returns [`BackupError::Deserialize`] on JSON decoding failure.
pub fn deserialize(json: &str) -> Result<ConfigSnapshot, BackupError> {
    serde_json::from_str(json).map_err(|e| BackupError::Deserialize(e.to_string()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> ControlState {
        ControlState::new()
    }

    #[test]
    fn test_backup_empty_state() {
        let state = fresh_state();
        let snap = backup(&state);
        assert_eq!(snap.version, 1);
        assert!(snap.channels.is_empty());
        assert!(snap.mixes.is_empty());
    }

    #[test]
    fn test_backup_restore_channel() {
        let mut state = fresh_state();
        let mut ch = Channel::new(42, "Vocals");
        ch.set_gain_db(-3.0);
        ch.set_muted(true);
        state.set_channel(0, ch).expect("set_channel");

        let snap = backup(&state);
        let mut restored = fresh_state();
        restore(&snap, &mut restored).expect("restore");

        let ch = restored.channel(0).expect("channel 0 restored");
        assert_eq!(ch.id, 42);
        assert_eq!(ch.name(), "Vocals");
        assert!((ch.gain_db() - (-3.0)).abs() < 1e-6);
        assert!(ch.muted);
    }

    #[test]
    fn test_backup_restore_mix() {
        let mut state = fresh_state();
        let mut mix = Mix::new(7, "Monitor A");
        mix.set_master_gain_db(-6.0);
        mix.set_master_muted(true);
        state.set_mix(0, mix).expect("set_mix");

        let snap = backup(&state);
        let mut restored = fresh_state();
        restore(&snap, &mut restored).expect("restore");

        let mix = restored.mix(0).expect("mix 0 restored");
        assert_eq!(mix.id, 7);
        assert_eq!(mix.name(), "Monitor A");
        assert!((mix.master_gain_db() - (-6.0)).abs() < 1e-6);
        assert!(mix.master_muted);
    }

    #[test]
    fn test_backup_restore_sends() {
        let mut state = fresh_state();
        let channel = Channel::new(42, "Input");
        state.set_channel(0, channel).expect("set_channel");
        let mut mix = Mix::new(1, "Mix1");
        let mut send = MixSend::new(42, 1);
        send.set_gain_db(-6.0);
        send.set_pan(0.5);
        mix.set_send(0, send).expect("set_send");
        state.set_mix(0, mix).expect("set_mix");

        let snap = backup(&state);
        let mut restored = fresh_state();
        restore(&snap, &mut restored).expect("restore");

        let mix = restored.mix(0).expect("mix 0");
        let send = mix.send(0).expect("send 0");
        assert!((send.gain_db() - (-6.0)).abs() < 1e-6);
        assert!((send.pan() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut state = fresh_state();
        let mut ch = Channel::new(1, "Guitar");
        ch.set_gain_db(2.0);
        state.set_channel(1, ch).expect("set_channel");

        let snap = backup(&state);
        let json = serialize(&snap).expect("serialize");
        let snap2 = deserialize(&json).expect("deserialize");

        assert_eq!(snap.version, snap2.version);
        assert_eq!(snap.channels.len(), snap2.channels.len());
        assert_eq!(snap.channels[0].name, snap2.channels[0].name);
        assert!((snap.channels[0].gain_db - snap2.channels[0].gain_db).abs() < 1e-6);
    }

    #[test]
    fn test_serialized_snapshot_restores_into_fresh_state() {
        let mut source = fresh_state();
        let mut channel = Channel::new(42, "Lead Vocal");
        channel.set_gain_db(-3.0);
        channel.set_muted(true);
        source.set_channel(0, channel).expect("set_channel");

        let mut mix = Mix::new(7, "Monitor A");
        mix.set_master_gain_db(-6.0);
        let mut send = MixSend::new(42, 7);
        send.set_gain_db(-9.0);
        send.set_pan(0.25);
        mix.set_send(0, send).expect("set_send");
        source.set_mix(0, mix).expect("set_mix");

        let snapshot =
            deserialize(&serialize(&backup(&source)).expect("serialize")).expect("deserialize");
        let mut fresh = fresh_state();
        restore(&snapshot, &mut fresh).expect("restore into fresh state");

        let restored_channel = fresh.channel(0).expect("channel 0");
        assert_eq!(restored_channel.name(), "Lead Vocal");
        assert!((restored_channel.gain_db() - (-3.0)).abs() < 1e-6);
        assert!(restored_channel.muted);
        let restored_mix = fresh.mix(0).expect("mix 0");
        assert_eq!(restored_mix.name(), "Monitor A");
        assert!((restored_mix.master_gain_db() - (-6.0)).abs() < 1e-6);
        let restored_send = restored_mix.send(0).expect("send 0");
        assert!((restored_send.gain_db() - (-9.0)).abs() < 1e-6);
        assert!((restored_send.pan() - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_restore_clears_nothing() {
        // State A: channel 0 = "OldName"
        let mut state_a = fresh_state();
        state_a
            .set_channel(0, Channel::new(1, "OldName"))
            .expect("set_channel");

        // Snapshot from state B: channel 0 = "NewName"
        let mut state_b = fresh_state();
        state_b
            .set_channel(0, Channel::new(1, "NewName"))
            .expect("set_channel");
        let snap = backup(&state_b);

        // Restore snap onto state_a — "NewName" should win
        restore(&snap, &mut state_a).expect("restore");
        let ch = state_a.channel(0).expect("channel 0");
        assert_eq!(ch.name(), "NewName");
    }

    #[test]
    fn test_unsupported_version() {
        let mut snap = backup(&fresh_state());
        snap.version = 99;
        let result = restore(&snap, &mut fresh_state());
        assert!(matches!(result, Err(RestoreError::UnsupportedVersion(99))));
    }

    #[test]
    fn test_no_secrets_in_snapshot() {
        let mut state = fresh_state();
        state
            .set_channel(0, Channel::new(0, "test-ch"))
            .expect("set_channel");
        let snap = backup(&state);
        let json = serialize(&snap).expect("serialize");

        // Check for credential-style JSON keys — the JSON key patterns we never want
        let lower = json.to_lowercase();
        assert!(
            !lower.contains("\"password\""),
            "JSON must not contain key 'password'"
        );
        assert!(
            !lower.contains("\"token\""),
            "JSON must not contain key 'token'"
        );
        assert!(
            !lower.contains("\"secret\""),
            "JSON must not contain key 'secret'"
        );
        assert!(
            !lower.contains("\"api_key\""),
            "JSON must not contain key 'api_key'"
        );
    }
}
