//! `MixSend` — per-channel, per-mix routing and level configuration.
//!
//! Each channel can send to each mix independently with its own
//! gain, pan, mute, solo, enabled, and locked settings.

use crate::{GAIN_DB_MAX, GAIN_DB_MIN};

/// Unique mix identifier.
pub type MixId = u32;

/// Per-channel routing into a specific mix.
///
/// A `MixSend` connects one channel to one mix. The engineer or musician
/// can set the level, pan, mute, and solo independently per send.
///
/// # Realtime Safety
///
/// `MixSend` is a pure data struct. Reading its fields in the audio callback
/// is safe. Mutations happen only on the control thread and must be communicated
/// to the audio thread via a lock-free channel (future `mix-engine-rt` crate).
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct MixSend {
    /// The channel this send originates from.
    pub channel_id: u32,

    /// The mix this send routes into.
    pub mix_id: MixId,

    /// Send level in dB, applied after channel trim.
    ///
    /// Bounded to `[GAIN_DB_MIN, GAIN_DB_MAX]`.
    gain_db: f32,

    /// Stereo pan position: -1.0 (hard left) to +1.0 (hard right).
    pan: f32,

    /// Whether this send is muted (silences this channel in this mix only).
    pub muted: bool,

    /// Whether this send is soloed (when any send in a mix is soloed,
    /// only soloed sends are audible in that mix).
    pub solo: bool,

    /// Whether this send is active. When `false`, the channel does not
    /// contribute to the mix at all.
    pub enabled: bool,

    /// Whether this send is locked (cannot be changed by musician role).
    pub locked: bool,

    /// Monotonic revision counter. Incremented on every state mutation.
    revision: u64,
}

impl MixSend {
    /// Create a new `MixSend` with unity gain, centre pan, unmuted, un-soloed,
    /// enabled, unlocked.
    #[must_use]
    pub fn new(channel_id: u32, mix_id: MixId) -> Self {
        Self {
            channel_id,
            mix_id,
            gain_db: 0.0,
            pan: 0.0,
            muted: false,
            solo: false,
            enabled: true,
            locked: false,
            revision: 0,
        }
    }

    /// Returns the send gain in dB.
    #[must_use]
    pub fn gain_db(&self) -> f32 {
        self.gain_db
    }

    /// Returns the pan position (-1.0 to +1.0).
    #[must_use]
    pub fn pan(&self) -> f32 {
        self.pan
    }

    /// Returns the current monotonic revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Set the send gain in dB.
    ///
    /// Clamped to `[GAIN_DB_MIN, GAIN_DB_MAX]`.
    /// Increments revision.
    pub fn set_gain_db(&mut self, db: f32) {
        self.gain_db = db.clamp(GAIN_DB_MIN, GAIN_DB_MAX);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the pan position.
    ///
    /// Clamped to `[-1.0, 1.0]`.
    /// Increments revision.
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the muted state. Increments revision.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the solo state. Increments revision.
    pub fn set_solo(&mut self, solo: bool) {
        self.solo = solo;
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the enabled state. Increments revision.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the locked state. Increments revision.
    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
        self.revision = self.revision.saturating_add(1);
    }

    /// Returns `true` if this send contributes audio to the mix.
    ///
    /// A send is silent when: muted OR disabled.
    /// (Solo logic is evaluated at the Mix level, not here.)
    #[must_use]
    #[inline]
    pub fn is_active(&self) -> bool {
        self.enabled && !self.muted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_send_defaults() {
        let send = MixSend::new(1, 10);
        assert_eq!(send.channel_id, 1);
        assert_eq!(send.mix_id, 10);
        assert!((send.gain_db() - 0.0).abs() < f32::EPSILON);
        assert!((send.pan() - 0.0).abs() < f32::EPSILON);
        assert!(!send.muted);
        assert!(!send.solo);
        assert!(send.enabled);
        assert!(!send.locked);
        assert_eq!(send.revision(), 0);
        assert!(send.is_active());
    }

    #[test]
    fn test_mix_send_gain_clamped_high() {
        let mut send = MixSend::new(1, 1);
        send.set_gain_db(999.0);
        assert!((send.gain_db() - GAIN_DB_MAX).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_send_gain_clamped_low() {
        let mut send = MixSend::new(1, 1);
        send.set_gain_db(-999.0);
        assert!((send.gain_db() - GAIN_DB_MIN).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_send_gain_valid() {
        let mut send = MixSend::new(1, 1);
        send.set_gain_db(-12.0);
        assert!((send.gain_db() - (-12.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_send_pan_clamped() {
        let mut send = MixSend::new(1, 1);
        send.set_pan(5.0);
        assert!((send.pan() - 1.0).abs() < f32::EPSILON);
        send.set_pan(-5.0);
        assert!((send.pan() - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_send_pan_valid() {
        let mut send = MixSend::new(1, 1);
        send.set_pan(0.5);
        assert!((send.pan() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mix_send_revision_increments() {
        let mut send = MixSend::new(1, 1);
        assert_eq!(send.revision(), 0);
        send.set_gain_db(0.0);
        assert_eq!(send.revision(), 1);
        send.set_pan(0.5);
        assert_eq!(send.revision(), 2);
        send.set_muted(true);
        assert_eq!(send.revision(), 3);
        send.set_solo(true);
        assert_eq!(send.revision(), 4);
        send.set_enabled(false);
        assert_eq!(send.revision(), 5);
        send.set_locked(true);
        assert_eq!(send.revision(), 6);
    }

    #[test]
    fn test_mix_send_inactive_when_muted() {
        let mut send = MixSend::new(1, 1);
        send.set_muted(true);
        assert!(!send.is_active());
    }

    #[test]
    fn test_mix_send_inactive_when_disabled() {
        let mut send = MixSend::new(1, 1);
        send.set_enabled(false);
        assert!(!send.is_active());
    }

    #[test]
    fn test_mix_send_active_when_solo_only() {
        // solo alone does NOT make it inactive — Mix evaluates solo logic
        let mut send = MixSend::new(1, 1);
        send.set_solo(true);
        assert!(send.is_active());
    }
}
