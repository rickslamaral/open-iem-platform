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
    /// Device reports no usable output channels.
    #[error("device {0} reports no output channels")]
    NoOutputChannels(String),
    /// Device has no sample rates.
    #[error("device {0} reports no sample rates")]
    NoSampleRates(String),
    /// Transition references unknown device.
    #[error("unknown device: {0}")]
    UnknownDevice(String),
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
        let mut next = Vec::with_capacity(capabilities.len());
        for caps in capabilities {
            let state = self
                .devices
                .iter()
                .find(|d| d.capabilities.id == caps.id)
                .map_or(DeviceState::Reconnected, |d| {
                    if d.state == DeviceState::Recovering {
                        DeviceState::Reconnected
                    } else if d.capabilities == caps {
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
        let device = self.device_mut(id)?;
        device.state = DeviceState::Recovering;
        Ok(())
    }

    /// Mark a validated, present device available after recovery succeeds.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceManagerError::UnknownDevice`] for an unknown ID.
    pub fn mark_available(&mut self, id: &str) -> Result<(), DeviceManagerError> {
        let device = self.device_mut(id)?;
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

fn validate_snapshot(capabilities: &[DeviceCapabilities]) -> Result<(), DeviceManagerError> {
    if capabilities.len() > MAX_DEVICES {
        return Err(DeviceManagerError::TooManyDevices);
    }
    for (index, device) in capabilities.iter().enumerate() {
        if device.id.is_empty() || capabilities[..index].iter().any(|old| old.id == device.id) {
            return Err(DeviceManagerError::InvalidDeviceId(device.id.clone()));
        }
        if device.max_output_channels == 0 {
            return Err(DeviceManagerError::NoOutputChannels(device.id.clone()));
        }
        if device.sample_rates_hz.is_empty() {
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
    fn registry_remains_bounded_when_snapshot_is_full() {
        let mut manager = DeviceManager::new();
        let initial: Vec<_> = (0..MAX_DEVICES)
            .map(|i| device(&format!("usb-{i}")))
            .collect();
        manager.discover(initial).expect("valid snapshot");
        let replacement: Vec<_> = (0..MAX_DEVICES)
            .map(|i| device(&format!("new-{i}")))
            .collect();
        manager.discover(replacement).expect("valid snapshot");
        assert_eq!(manager.devices().len(), MAX_DEVICES);
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
