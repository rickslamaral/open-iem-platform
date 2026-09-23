//! Pairing and receiver identity lifecycle (CODE/SIMULATED).
//!
//! Stores only Argon2id credential digests. DTLS-SRTP key negotiation remains
//! owned by the WebRTC session; this registry is its authorization boundary.

use argon2::Argon2;
use rand::random;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

use crate::canonicalize_dtls_fingerprint;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub musician_id: String,
    pub mix_index: usize,
    pub revoked: bool,
    pub dtls_fingerprint: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PairingError {
    #[error("device identity is invalid")]
    InvalidIdentity,
    #[error("device is already paired")]
    AlreadyPaired,
    #[error("device or credential is invalid")]
    InvalidCredential,
    #[error("device is revoked")]
    Revoked,
    #[error("pairing registry capacity reached")]
    CapacityReached,
    #[error("device is not paired")]
    NotFound,
}

#[derive(Clone, Default)]
pub struct PairingRegistry {
    devices: Arc<Mutex<HashMap<String, StoredDevice>>>,
}

#[derive(Clone)]
struct StoredDevice {
    identity: DeviceIdentity,
    credential_digest: [u8; 32],
    credential_salt: [u8; 16],
}

impl PairingRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Pair receiver with one musician/mix. Plain credential exists only at call boundary.
    /// # Errors
    /// Returns an error for invalid identity, weak credentials, or duplicate device IDs.
    pub async fn pair(
        &self,
        device_id: &str,
        musician_id: &str,
        mix_index: usize,
        credential: &[u8],
    ) -> Result<DeviceIdentity, PairingError> {
        self.pair_with_fingerprint(device_id, musician_id, mix_index, credential, None)
            .await
    }

    /// Pair receiver and optionally enroll its trusted DTLS-SRTP fingerprint.
    ///
    /// # Errors
    /// Returns [`PairingError::InvalidIdentity`] for malformed or unsupported fingerprints.
    pub async fn pair_with_fingerprint(
        &self,
        device_id: &str,
        musician_id: &str,
        mix_index: usize,
        credential: &[u8],
        dtls_fingerprint: Option<String>,
    ) -> Result<DeviceIdentity, PairingError> {
        let dtls_fingerprint = dtls_fingerprint
            .map(|value| {
                canonicalize_dtls_fingerprint(&value).map_err(|_| PairingError::InvalidIdentity)
            })
            .transpose()?;
        if !valid_id(device_id)
            || !valid_id(musician_id)
            || mix_index >= 16
            || credential.len() < 16
            || credential.len() > MAX_CREDENTIAL_BYTES
        {
            return Err(PairingError::InvalidIdentity);
        }
        {
            let devices = self.devices.lock().await;
            if devices.contains_key(device_id) {
                return Err(PairingError::AlreadyPaired);
            }
            if devices.len() >= MAX_DEVICES {
                return Err(PairingError::CapacityReached);
            }
        }
        let credential_salt = random::<[u8; 16]>();
        let credential_digest = derive_digest(credential, credential_salt).await?;
        let mut devices = self.devices.lock().await;
        if devices.contains_key(device_id) {
            return Err(PairingError::AlreadyPaired);
        }
        if devices.len() >= MAX_DEVICES {
            return Err(PairingError::CapacityReached);
        }
        let identity = DeviceIdentity {
            device_id: device_id.to_owned(),
            musician_id: musician_id.to_owned(),
            mix_index,
            revoked: false,
            dtls_fingerprint,
        };
        devices.insert(
            device_id.to_owned(),
            StoredDevice {
                identity: identity.clone(),
                credential_digest,
                credential_salt,
            },
        );
        Ok(identity)
    }

    /// Authenticate receiver and return authoritative binding. Revoked devices fail closed.
    /// # Errors
    /// Returns an error when device is unknown, revoked, or credential does not match.
    pub async fn authenticate(
        &self,
        device_id: &str,
        credential: &[u8],
    ) -> Result<DeviceIdentity, PairingError> {
        if credential.len() < 16 || credential.len() > MAX_CREDENTIAL_BYTES {
            return Err(PairingError::InvalidCredential);
        }
        let device = self
            .devices
            .lock()
            .await
            .get(device_id)
            .cloned()
            .ok_or(PairingError::InvalidCredential)?;
        if device.identity.revoked {
            return Err(PairingError::Revoked);
        }
        let candidate_digest = derive_digest(credential, device.credential_salt).await?;
        if !constant_time_eq(&device.credential_digest, &candidate_digest) {
            return Err(PairingError::InvalidCredential);
        }
        // Argon2 runs outside registry lock. Re-read authoritative state before
        // returning so revoke racing with password verification cannot authorize
        // a session after revocation completed.
        let devices = self.devices.lock().await;
        let current = devices
            .get(device_id)
            .ok_or(PairingError::InvalidCredential)?;
        if current.identity.revoked
            || current.credential_salt != device.credential_salt
            || current.credential_digest != device.credential_digest
        {
            return Err(if current.identity.revoked {
                PairingError::Revoked
            } else {
                PairingError::InvalidCredential
            });
        }
        Ok(current.identity.clone())
    }

    /// Revoke identity. Existing and future media sessions must be closed by caller.
    /// # Errors
    /// Returns [`PairingError::NotFound`] when device is not registered.
    pub async fn revoke(&self, device_id: &str) -> Result<(), PairingError> {
        let mut devices = self.devices.lock().await;
        let device = devices.get_mut(device_id).ok_or(PairingError::NotFound)?;
        device.identity.revoked = true;
        Ok(())
    }

    /// Re-pair requires explicit revocation first; prevents silent credential replacement.
    /// # Errors
    /// Returns an error when device is unknown, active, or credential is weak.
    pub async fn replace_revoked(
        &self,
        device_id: &str,
        old_credential: &[u8],
        new_credential: &[u8],
    ) -> Result<(), PairingError> {
        if old_credential.len() < 16
            || old_credential.len() > MAX_CREDENTIAL_BYTES
            || new_credential.len() < 16
            || new_credential.len() > MAX_CREDENTIAL_BYTES
        {
            return Err(PairingError::InvalidCredential);
        }
        let current = self
            .devices
            .lock()
            .await
            .get(device_id)
            .cloned()
            .ok_or(PairingError::NotFound)?;
        if !current.identity.revoked {
            return Err(PairingError::AlreadyPaired);
        }
        let old_digest = derive_digest(old_credential, current.credential_salt).await?;
        if !constant_time_eq(&current.credential_digest, &old_digest) {
            return Err(PairingError::InvalidCredential);
        }
        let new_salt = random::<[u8; 16]>();
        let new_digest = derive_digest(new_credential, new_salt).await?;
        let mut devices = self.devices.lock().await;
        let device = devices.get_mut(device_id).ok_or(PairingError::NotFound)?;
        if !device.identity.revoked || device.credential_digest != current.credential_digest {
            return Err(PairingError::InvalidCredential);
        }
        device.credential_salt = new_salt;
        device.credential_digest = new_digest;
        device.identity.revoked = false;
        Ok(())
    }

    pub async fn len(&self) -> usize {
        self.devices.lock().await.len()
    }

    #[must_use]
    pub async fn is_empty(&self) -> bool {
        self.devices.lock().await.is_empty()
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

const MAX_DEVICES: usize = 1024;
const MAX_CREDENTIAL_BYTES: usize = 4096;

async fn derive_digest(value: &[u8], salt: [u8; 16]) -> Result<[u8; 32], PairingError> {
    let value = value.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut output = [0u8; 32];
        Argon2::default()
            .hash_password_into(&value, &salt, &mut output)
            .map_err(|_| PairingError::InvalidCredential)?;
        Ok(output)
    })
    .await
    .map_err(|_| PairingError::InvalidCredential)?
}

fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    const CREDENTIAL: &[u8] = b"pairing-secret-1234";

    #[tokio::test]
    async fn unknown_and_wrong_credentials_fail_closed() {
        let registry = PairingRegistry::new();
        assert_eq!(
            registry.authenticate("unknown", CREDENTIAL).await,
            Err(PairingError::InvalidCredential)
        );
        registry
            .pair("rx-1", "musician-1", 0, CREDENTIAL)
            .await
            .unwrap();
        assert_eq!(
            registry.authenticate("rx-1", b"wrong-credential").await,
            Err(PairingError::InvalidCredential)
        );
    }

    #[tokio::test]
    async fn pairing_binds_device_to_mix() {
        let registry = PairingRegistry::new();
        let identity = registry
            .pair("rx-1", "musician-1", 1, CREDENTIAL)
            .await
            .unwrap();
        assert_eq!(
            registry.authenticate("rx-1", CREDENTIAL).await.unwrap(),
            identity
        );
    }

    #[tokio::test]
    async fn revocation_blocks_reconnect_until_explicit_repair() {
        let registry = PairingRegistry::new();
        registry
            .pair("rx-1", "musician-1", 0, CREDENTIAL)
            .await
            .unwrap();
        registry.revoke("rx-1").await.unwrap();
        assert_eq!(
            registry.authenticate("rx-1", CREDENTIAL).await,
            Err(PairingError::Revoked)
        );
        assert_eq!(
            registry
                .replace_revoked("rx-1", CREDENTIAL, b"new-pairing-secret")
                .await,
            Ok(())
        );
        assert!(registry
            .authenticate("rx-1", b"new-pairing-secret")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn credential_at_maximum_length_is_accepted() {
        let registry = PairingRegistry::new();
        let credential = vec![b'c'; MAX_CREDENTIAL_BYTES];

        registry
            .pair("rx-max-credential", "musician-1", 0, &credential)
            .await
            .expect("maximum-length credential must be accepted");
        assert!(registry
            .authenticate("rx-max-credential", &credential)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn oversized_credential_is_rejected_without_registry_change() {
        let registry = PairingRegistry::new();
        let valid_credential = b"valid-pairing-secret";
        let credential = vec![b'c'; MAX_CREDENTIAL_BYTES + 1];
        registry
            .pair("rx-existing", "musician-1", 0, valid_credential)
            .await
            .unwrap();

        assert_eq!(
            registry
                .pair("rx-oversized-credential", "musician-1", 0, &credential)
                .await,
            Err(PairingError::InvalidIdentity)
        );
        assert_eq!(registry.len().await, 1);
        assert!(registry
            .authenticate("rx-existing", valid_credential)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn identity_ids_at_maximum_length_are_accepted() {
        let registry = PairingRegistry::new();
        let device_id = "d".repeat(128);
        let musician_id = "m".repeat(128);

        registry
            .pair(&device_id, &musician_id, 0, CREDENTIAL)
            .await
            .expect("maximum-length identity IDs must be accepted");
        assert_eq!(registry.len().await, 1);
    }

    #[tokio::test]
    async fn oversized_identity_ids_are_rejected_without_registry_change() {
        let registry = PairingRegistry::new();
        let valid_device = "d".repeat(128);
        let oversized_device = "d".repeat(129);
        let oversized_musician = "m".repeat(129);

        assert_eq!(
            registry
                .pair(&oversized_device, "musician-1", 0, CREDENTIAL)
                .await,
            Err(PairingError::InvalidIdentity)
        );
        assert_eq!(
            registry
                .pair(&valid_device, &oversized_musician, 0, CREDENTIAL)
                .await,
            Err(PairingError::InvalidIdentity)
        );
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn replacement_credential_at_maximum_length_is_accepted() {
        let registry = PairingRegistry::new();
        let replacement = vec![b'r'; MAX_CREDENTIAL_BYTES];

        registry
            .pair("rx-replacement", "musician-1", 0, CREDENTIAL)
            .await
            .unwrap();
        registry.revoke("rx-replacement").await.unwrap();
        registry
            .replace_revoked("rx-replacement", CREDENTIAL, &replacement)
            .await
            .expect("maximum-length replacement credential must be accepted");

        assert!(registry
            .authenticate("rx-replacement", &replacement)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn oversized_replacement_credential_is_rejected_without_mutation() {
        let registry = PairingRegistry::new();
        let oversized = vec![b'r'; MAX_CREDENTIAL_BYTES + 1];

        registry
            .pair("rx-replacement", "musician-1", 0, CREDENTIAL)
            .await
            .unwrap();
        registry.revoke("rx-replacement").await.unwrap();

        assert_eq!(
            registry
                .replace_revoked("rx-replacement", CREDENTIAL, &oversized)
                .await,
            Err(PairingError::InvalidCredential)
        );
        assert_eq!(
            registry.authenticate("rx-replacement", CREDENTIAL).await,
            Err(PairingError::Revoked)
        );
    }

    #[tokio::test]
    async fn duplicate_and_weak_pairing_rejected() {
        let registry = PairingRegistry::new();
        assert_eq!(
            registry.pair("rx", "m", 0, b"short").await,
            Err(PairingError::InvalidIdentity)
        );
        registry.pair("rx", "m", 0, CREDENTIAL).await.unwrap();
        assert_eq!(
            registry.pair("rx", "m", 0, CREDENTIAL).await,
            Err(PairingError::AlreadyPaired)
        );
    }
}
