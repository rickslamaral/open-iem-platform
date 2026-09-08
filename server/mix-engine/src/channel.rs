//! Channel — represents a single audio input source.

use crate::{GAIN_DB_MAX, GAIN_DB_MIN};

/// Unique channel identifier.
pub type ChannelId = u32;

/// A single audio input channel.
#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    /// Unique identifier for this channel.
    pub id: ChannelId,

    /// Human-readable display name (e.g. "VOC 1", "GTR R").
    name: heapless_str::HeaplessStr64,

    /// Input gain trim in dB, applied before routing to mixes.
    gain_db: f32,

    /// Whether this channel is globally muted (across all mixes).
    pub muted: bool,

    /// Whether this channel is locked (cannot be changed by musician role).
    pub locked: bool,

    /// Whether this channel is enabled/active.
    pub enabled: bool,

    /// Monotonic revision counter. Incremented on every state mutation.
    revision: u64,
}

impl Channel {
    /// Create a new channel with the given ID and name.
    #[must_use]
    pub fn new(id: ChannelId, name: &str) -> Self {
        Self {
            id,
            name: heapless_str::HeaplessStr64::from_str_truncate(name),
            gain_db: 0.0,
            muted: false,
            locked: false,
            enabled: true,
            revision: 0,
        }
    }

    /// Returns the channel display name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the current input gain trim in dB.
    #[must_use]
    pub fn gain_db(&self) -> f32 {
        self.gain_db
    }

    /// Returns the current monotonic revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Set the input gain trim in dB. Clamped to `[GAIN_DB_MIN, GAIN_DB_MAX]`.
    pub fn set_gain_db(&mut self, db: f32) {
        self.gain_db = db.clamp(GAIN_DB_MIN, GAIN_DB_MAX);
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the muted state. Increments the revision counter.
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the locked state. Increments the revision counter.
    pub fn set_locked(&mut self, locked: bool) {
        self.locked = locked;
        self.revision = self.revision.saturating_add(1);
    }

    /// Set the enabled state. Increments the revision counter.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.revision = self.revision.saturating_add(1);
    }

    /// Returns `true` if the channel is effectively silent (muted OR disabled).
    #[must_use]
    #[inline]
    pub fn is_silent(&self) -> bool {
        self.muted || !self.enabled
    }
}

// ---------------------------------------------------------------------------
// Minimal fixed-capacity string — no heap allocation, no external crate
// ---------------------------------------------------------------------------
mod heapless_str {
    const MAX: usize = 64;

    #[derive(Debug, Clone, PartialEq)]
    pub struct HeaplessStr64 {
        buf: [u8; MAX],
        len: usize,
    }

    impl HeaplessStr64 {
        pub fn from_str_truncate(s: &str) -> Self {
            let bytes = s.as_bytes();
            // Find a valid UTF-8 boundary at or before MAX bytes
            let len = if bytes.len() <= MAX {
                bytes.len()
            } else {
                // Walk back from MAX to find a char boundary
                let mut l = MAX;
                while l > 0 {
                    // Safe: l is within [0, MAX], bytes has at least MAX bytes
                    if let Some(&b) = bytes.get(l) {
                        if (b & 0b1100_0000) != 0b1000_0000 {
                            break;
                        }
                    } else {
                        break;
                    }
                    l -= 1;
                }
                l
            };

            let mut buf = [0u8; MAX];
            // Safe: len <= MAX and len <= bytes.len()
            if let (Some(dst), Some(src)) = (buf.get_mut(..len), bytes.get(..len)) {
                dst.copy_from_slice(src);
            }
            Self { buf, len }
        }

        pub fn as_str(&self) -> &str {
            // Safe: we only store valid UTF-8 bytes
            self.buf
                .get(..self.len)
                .and_then(|b| core::str::from_utf8(b).ok())
                .unwrap_or("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_new_defaults() {
        let ch = Channel::new(1, "VOC 1");
        assert_eq!(ch.id, 1);
        assert_eq!(ch.name(), "VOC 1");
        assert!((ch.gain_db() - 0.0).abs() < f32::EPSILON);
        assert!(!ch.muted);
        assert!(!ch.locked);
        assert!(ch.enabled);
        assert_eq!(ch.revision(), 0);
    }

    #[test]
    fn test_channel_gain_clamped() {
        let mut ch = Channel::new(1, "test");
        ch.set_gain_db(100.0);
        assert!((ch.gain_db() - GAIN_DB_MAX).abs() < f32::EPSILON);
        ch.set_gain_db(-999.0);
        assert!((ch.gain_db() - GAIN_DB_MIN).abs() < f32::EPSILON);
    }

    #[test]
    fn test_channel_gain_in_range() {
        let mut ch = Channel::new(1, "test");
        ch.set_gain_db(6.0);
        assert!((ch.gain_db() - 6.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_channel_revision_increments() {
        let mut ch = Channel::new(1, "test");
        assert_eq!(ch.revision(), 0);
        ch.set_gain_db(3.0);
        assert_eq!(ch.revision(), 1);
        ch.set_muted(true);
        assert_eq!(ch.revision(), 2);
        ch.set_locked(true);
        assert_eq!(ch.revision(), 3);
        ch.set_enabled(false);
        assert_eq!(ch.revision(), 4);
    }

    #[test]
    fn test_channel_is_silent_when_muted() {
        let mut ch = Channel::new(1, "test");
        assert!(!ch.is_silent());
        ch.set_muted(true);
        assert!(ch.is_silent());
    }

    #[test]
    fn test_channel_is_silent_when_disabled() {
        let mut ch = Channel::new(1, "test");
        ch.set_enabled(false);
        assert!(ch.is_silent());
    }

    #[test]
    fn test_channel_name_truncation() {
        let long_name = "A".repeat(200);
        let ch = Channel::new(99, &long_name);
        assert!(ch.name().len() <= 64);
    }
}
