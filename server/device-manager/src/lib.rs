//! Bounded audio device capability and recovery state machine.
//!
//! Discovery is supplied by a control-plane adapter. This crate owns only
//! validated snapshots and transitions; it performs no device I/O and never
//! blocks the audio path.

#![deny(missing_docs)]
#![deny(unsafe_code)]

use topology::TopologyMode;

/// Maximum number of devices retained in one discovery snapshot.
pub const MAX_DEVICES: usize = 16;
/// Maximum UTF-8 byte length of backend device identifiers.
pub const MAX_DEVICE_ID_BYTES: usize = 256;
/// Maximum UTF-8 byte length of human-readable device names.
pub const MAX_DEVICE_NAME_BYTES: usize = 256;
/// Maximum number of sample rates advertised by one device.
pub const MAX_SAMPLE_RATES: usize = 16;
/// Maximum number of topology modes advertised by one device.
pub const MAX_SUPPORTED_MODES: usize = 8;
/// Maximum number of output channels a device may advertise.
pub const MAX_OUTPUT_CHANNELS: usize = 64;

/// Device lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceState {
    /// Device is present and usable.
    Available,
    /// Device disappeared or failed; output must be muted.
    Recovering,
    /// Device returned and awaits successful capability validation.
    Reconnected,
}

/// Validated capabilities reported by a device adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceCapabilities {
    /// Stable device identifier supplied by backend.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Supported sample rates in Hz.
    pub sample_rates_hz: Vec<u32>,
    /// Maximum number of output channels.
    pub max_output_channels: usize,
    /// Supported topology modes.
    pub supported_modes: Vec<TopologyMode>,
}

/// Device entry and current recovery state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Device {
    /// Validated device capabilities.
    pub capabilities: DeviceCapabilities,
    /// Current lifecycle state.
    pub state: DeviceState,
}

/// Errors from discovery snapshot validation or state transitions.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DeviceManagerError {
    /// Snapshot exceeds bounded capacity.
    #[error("device snapshot exceeds maximum of {MAX_DEVICES} devices")]
    TooManyDevices,
    /// Device identifier is empty or duplicated.
    #[error("invalid or duplicate device id: {0}")]
    InvalidDeviceId(String),
    /// Device identifier exceeds bounded UTF-8 byte length.
    #[error("device ID exceeds maximum of {MAX_DEVICE_ID_BYTES} bytes")]
    DeviceIdTooLong,
    /// Device name exceeds bounded UTF-8 byte length.
    #[error("device name exceeds maximum of {MAX_DEVICE_NAME_BYTES} bytes")]
    DeviceNameTooLong,
    /// Device advertises too many sample rates.
    #[error("device {0} advertises more than {MAX_SAMPLE_RATES} sample rates")]
    TooManySampleRates(String),
    /// Device advertises too many topology modes.
    #[error("device {0} advertises more than {MAX_SUPPORTED_MODES} supported modes")]
    TooManySupportedModes(String),
    /// Device reports no usable output channels.
    #[error("device {0} reports no output channels")]
    NoOutputChannels(String),
    /// Device reports more output channels than the bounded registry supports.
    #[error("device {0} reports more than {MAX_OUTPUT_CHANNELS} output channels")]
    TooManyOutputChannels(String),
    /// Device has no sample rates.
    #[error("device {0} reports no sample rates")]
    NoSampleRates(String),
    /// Transition references unknown device.
    #[error("unknown device: {0}")]
    UnknownDevice(String),
    /// Device is not ready for the requested transition.
    #[error("device {0} is not reconnected")]
    NotReconnected(String),
}

/// Bounded device registry updated by control-plane discovery.
#[derive(Clone, Debug, Default)]
pub struct DeviceManager {
    devices: Vec<Device>,
}

impl DeviceManager {
    /// Create an empty manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace discovery snapshot after validating all entries atomically.
    ///
    /// Devices absent from snapshot enter [`DeviceState::Recovering`] so
    /// callers can mute output while hardware recovery is attempted.
    ///
    /// # Errors
    ///
    /// Returns an error when snapshot exceeds capacity or contains invalid capabilities.
    pub fn discover(
        &mut self,
        capabilities: Vec<DeviceCapabilities>,
    ) -> Result<(), DeviceManagerError> {
        validate_snapshot(&capabilities)?;
        let absent = self
            .devices
            .iter()
            .filter(|old| {
                !capabilities
                    .iter()
                    .any(|caps| caps.id == old.capabilities.id)
            })
            .count();
        if capabilities.len() + absent > MAX_DEVICES {
            return Err(DeviceManagerError::TooManyDevices);
        }
        let mut next = Vec::with_capacity(capabilities.len() + absent);
        for caps in capabilities {
            let state = self
                .devices
                .iter()
                .find(|d| d.capabilities.id == caps.id)
                .map_or(DeviceState::Reconnected, |d| {
                    if d.state == DeviceState::Available && d.capabilities == caps {
                        DeviceState::Available
                    } else {
                        DeviceState::Reconnected
                    }
                });
            next.push(Device {
                capabilities: caps,
                state,
            });
        }
        for old in &self.devices {
            if !next
                .iter()
                .any(|d| d.capabilities.id == old.capabilities.id)
            {
                if next.len() == MAX_DEVICES {
                    break;
                }
                next.push(Device {
                    capabilities: old.capabilities.clone(),
                    state: DeviceState::Recovering,
                });
            }
        }
        self.devices = next;
        Ok(())
    }

    /// Mark known device as recovering after loss or backend failure.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceManagerError::UnknownDevice`] for an unknown ID.
    pub fn mark_failed(&mut self, id: &str) -> Result<(), DeviceManagerError> {
        validate_transition_id(id)?;
        let device = self.device_mut(id)?;
        device.state = DeviceState::Recovering;
        Ok(())
    }

    /// Mark a validated, present device available after recovery succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceManagerError::UnknownDevice`] for an unknown ID or
    /// [`DeviceManagerError::NotReconnected`] when rediscovery has not
    /// validated the device since it entered recovery.
    pub fn mark_available(&mut self, id: &str) -> Result<(), DeviceManagerError> {
        validate_transition_id(id)?;
        let device = self.device_mut(id)?;
        if device.state != DeviceState::Reconnected {
            return Err(DeviceManagerError::NotReconnected(id.to_owned()));
        }
        device.state = DeviceState::Available;
        Ok(())
    }

    /// Return immutable device snapshot.
    #[must_use]
    pub fn devices(&self) -> &[Device] {
        &self.devices
    }

    /// Return `true` when at least one device is available for output.
    #[must_use]
    pub fn has_available_device(&self) -> bool {
        self.devices
            .iter()
            .any(|d| d.state == DeviceState::Available)
    }

    fn device_mut(&mut self, id: &str) -> Result<&mut Device, DeviceManagerError> {
        self.devices
            .iter_mut()
            .find(|d| d.capabilities.id == id)
            .ok_or_else(|| DeviceManagerError::UnknownDevice(id.to_owned()))
    }
}

fn validate_transition_id(id: &str) -> Result<(), DeviceManagerError> {
    if id.len() > MAX_DEVICE_ID_BYTES {
        return Err(DeviceManagerError::DeviceIdTooLong);
    }
    if id.is_empty() {
        return Err(DeviceManagerError::InvalidDeviceId(String::new()));
    }
    Ok(())
}

fn validate_snapshot(capabilities: &[DeviceCapabilities]) -> Result<(), DeviceManagerError> {
    if capabilities.len() > MAX_DEVICES {
        return Err(DeviceManagerError::TooManyDevices);
    }
    for (index, device) in capabilities.iter().enumerate() {
        // Check byte and collection bounds before any ID comparisons or
        // capability cloning, keeping validation work and allocations bounded.
        if device.id.len() > MAX_DEVICE_ID_BYTES {
            return Err(DeviceManagerError::DeviceIdTooLong);
        }
        if device.name.len() > MAX_DEVICE_NAME_BYTES {
            return Err(DeviceManagerError::DeviceNameTooLong);
        }
        if device.sample_rates_hz.len() > MAX_SAMPLE_RATES {
            return Err(DeviceManagerError::TooManySampleRates(device.id.clone()));
        }
        if device.supported_modes.len() > MAX_SUPPORTED_MODES {
            return Err(DeviceManagerError::TooManySupportedModes(device.id.clone()));
        }
        if device.id.is_empty() || capabilities[..index].iter().any(|old| old.id == device.id) {
            return Err(DeviceManagerError::InvalidDeviceId(device.id.clone()));
        }
        if device.max_output_channels == 0 {
            return Err(DeviceManagerError::NoOutputChannels(device.id.clone()));
        }
        if device.max_output_channels > MAX_OUTPUT_CHANNELS {
            return Err(DeviceManagerError::TooManyOutputChannels(device.id.clone()));
        }
        if device.sample_rates_hz.is_empty() || device.sample_rates_hz.contains(&0) {
            return Err(DeviceManagerError::NoSampleRates(device.id.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(id: &str) -> DeviceCapabilities {
        DeviceCapabilities {
            id: id.into(),
            name: format!("Device {id}"),
            sample_rates_hz: vec![48_000],
            max_output_channels: 2,
            supported_modes: vec![TopologyMode::ChannelMode],
        }
    }

    #[test]
    fn discovery_accepts_capabilities_and_starts_reconnected() {
        let mut manager = DeviceManager::new();
        manager
            .discover(vec![device("usb-1")])
            .expect("valid snapshot");
        assert_eq!(manager.devices()[0].state, DeviceState::Reconnected);
        assert!(!manager.has_available_device());
        manager.mark_available("usb-1").expect("known device");
        assert!(manager.has_available_device());
    }

    #[test]
    fn missing_device_enters_recovery_and_new_snapshot_is_atomic() {
        let mut manager = DeviceManager::new();
        manager
            .discover(vec![device("usb-1"), device("usb-2")])
            .expect("valid snapshot");
        manager.mark_available("usb-1").expect("known device");
        manager
            .discover(vec![device("usb-1")])
            .expect("valid snapshot");
        assert_eq!(
            manager
                .devices()
                .iter()
                .find(|d| d.capabilities.id == "usb-2")
                .map(|d| d.state),
            Some(DeviceState::Recovering)
        );
        let before = manager.devices().to_vec();
        assert_eq!(
            manager.discover(vec![device("usb-1"), device("usb-1")]),
            Err(DeviceManagerError::InvalidDeviceId("usb-1".into()))
        );
        assert_eq!(manager.devices(), before.as_slice());
    }

    #[test]
    fn failed_device_requires_explicit_recovery_after_rediscovery() {
        let mut manager = DeviceManager::new();
        manager
            .discover(vec![device("usb-1")])
            .expect("valid snapshot");
        manager.mark_available("usb-1").expect("known device");
        manager.mark_failed("usb-1").expect("known device");
        manager
            .discover(vec![device("usb-1")])
            .expect("valid snapshot");
        assert_eq!(manager.devices()[0].state, DeviceState::Reconnected);
        assert!(!manager.has_available_device());
        manager.mark_available("usb-1").expect("known device");
        assert!(manager.has_available_device());
    }

    #[test]
    fn absent_devices_are_not_dropped_when_retention_exceeds_capacity() {
        let mut manager = DeviceManager::new();
        let initial: Vec<_> = (0..MAX_DEVICES)
            .map(|i| device(&format!("usb-{i}")))
            .collect();
        manager.discover(initial).expect("valid snapshot");
        let before = manager.devices().to_vec();
        let replacement = vec![device("new")];
        assert_eq!(
            manager.discover(replacement),
            Err(DeviceManagerError::TooManyDevices)
        );
        assert_eq!(manager.devices(), before.as_slice());
    }

    #[test]
    fn mark_available_rejects_recovering_device_without_rediscovery() {
        let mut manager = DeviceManager::new();
        manager
            .discover(vec![device("usb-1")])
            .expect("valid snapshot");
        manager.mark_failed("usb-1").expect("known device");
        assert_eq!(
            manager.mark_available("usb-1"),
            Err(DeviceManagerError::NotReconnected("usb-1".into()))
        );
        assert_eq!(manager.devices()[0].state, DeviceState::Recovering);
    }

    #[test]
    fn registry_remains_bounded_when_snapshot_is_full() {
        let mut manager = DeviceManager::new();
        let initial: Vec<_> = (0..MAX_DEVICES)
            .map(|i| device(&format!("usb-{i}")))
            .collect();
        manager.discover(initial).expect("valid snapshot");
        let replacement: Vec<_> = (0..MAX_DEVICES)
            .map(|i| device(&format!("new-{i}")))
            .collect();
        let before = manager.devices().to_vec();
        assert_eq!(
            manager.discover(replacement),
            Err(DeviceManagerError::TooManyDevices)
        );
        assert_eq!(manager.devices(), before.as_slice());
    }

    #[test]
    fn capability_byte_and_collection_limits_are_inclusive() {
        let mut caps = device(&"i".repeat(MAX_DEVICE_ID_BYTES));
        caps.name = "n".repeat(MAX_DEVICE_NAME_BYTES);
        caps.sample_rates_hz = vec![48_000; MAX_SAMPLE_RATES];
        caps.supported_modes = vec![TopologyMode::ChannelMode; MAX_SUPPORTED_MODES];
        let mut manager = DeviceManager::new();
        manager.discover(vec![caps]).expect("boundary is valid");
    }

    #[test]
    fn oversized_id_is_rejected_atomically() {
        let mut manager = manager_with_device();
        let before = manager.devices().to_vec();
        let mut caps = device("new");
        caps.id = "i".repeat(MAX_DEVICE_ID_BYTES + 1);
        assert_eq!(
            manager.discover(vec![caps]),
            Err(DeviceManagerError::DeviceIdTooLong)
        );
        assert_eq!(manager.devices(), before.as_slice());
    }

    #[test]
    fn oversized_name_is_rejected_atomically() {
        let mut manager = manager_with_device();
        let before = manager.devices().to_vec();
        let mut caps = device("new");
        caps.name = "n".repeat(MAX_DEVICE_NAME_BYTES + 1);
        assert_eq!(
            manager.discover(vec![caps]),
            Err(DeviceManagerError::DeviceNameTooLong)
        );
        assert_eq!(manager.devices(), before.as_slice());
    }

    #[test]
    fn excessive_sample_rates_are_rejected_atomically() {
        let mut manager = manager_with_device();
        let before = manager.devices().to_vec();
        let mut caps = device("new");
        caps.sample_rates_hz = vec![48_000; MAX_SAMPLE_RATES + 1];
        assert_eq!(
            manager.discover(vec![caps]),
            Err(DeviceManagerError::TooManySampleRates("new".into()))
        );
        assert_eq!(manager.devices(), before.as_slice());
    }

    #[test]
    fn excessive_supported_modes_are_rejected_atomically() {
        let mut manager = manager_with_device();
        let before = manager.devices().to_vec();
        let mut caps = device("new");
        caps.supported_modes = vec![TopologyMode::ChannelMode; MAX_SUPPORTED_MODES + 1];
        assert_eq!(
            manager.discover(vec![caps]),
            Err(DeviceManagerError::TooManySupportedModes("new".into()))
        );
        assert_eq!(manager.devices(), before.as_slice());
    }

    fn manager_with_device() -> DeviceManager {
        let mut manager = DeviceManager::new();
        manager
            .discover(vec![device("existing")])
            .expect("valid snapshot");
        manager
    }

    #[test]
    fn zero_sample_rate_is_rejected() {
        let mut manager = DeviceManager::new();
        let mut invalid = device("bad");
        invalid.sample_rates_hz = vec![0];
        assert_eq!(
            manager.discover(vec![invalid]),
            Err(DeviceManagerError::NoSampleRates("bad".into()))
        );
    }

    #[test]
    fn excessive_output_channels_are_rejected() {
        let mut manager = DeviceManager::new();
        let mut invalid = device("bad");
        invalid.max_output_channels = MAX_OUTPUT_CHANNELS + 1;
        assert_eq!(
            manager.discover(vec![invalid]),
            Err(DeviceManagerError::TooManyOutputChannels("bad".into()))
        );
    }

    #[test]
    fn transition_id_limits_reject_unbounded_error_input() {
        let mut manager = DeviceManager::new();
        let oversized = "i".repeat(MAX_DEVICE_ID_BYTES + 1);
        assert_eq!(
            manager.mark_failed(&oversized),
            Err(DeviceManagerError::DeviceIdTooLong)
        );
        assert_eq!(
            manager.mark_available(""),
            Err(DeviceManagerError::InvalidDeviceId(String::new()))
        );
    }

    #[test]
    fn rediscovery_keeps_reconnected_until_explicit_availability() {
        let mut manager = DeviceManager::new();
        manager
            .discover(vec![device("usb-1")])
            .expect("valid snapshot");
        manager
            .discover(vec![device("usb-1")])
            .expect("valid snapshot");
        assert_eq!(manager.devices()[0].state, DeviceState::Reconnected);
        assert!(!manager.has_available_device());
    }

    #[test]
    fn invalid_capabilities_and_unknown_transition_rejected() {
        let mut manager = DeviceManager::new();
        let mut invalid = device("bad");
        invalid.sample_rates_hz.clear();
        assert_eq!(
            manager.discover(vec![invalid]),
            Err(DeviceManagerError::NoSampleRates("bad".into()))
        );
        assert_eq!(
            manager.mark_failed("missing"),
            Err(DeviceManagerError::UnknownDevice("missing".into()))
        );
    }
}
