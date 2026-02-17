//! Device binding and attestation for ZeroClaw.
//!
//! Binds a ZeroClaw instance to a specific device using hardware-derived
//! fingerprints and PBKDF2-based key derivation. Prevents unauthorized
//! cloning of configuration and secrets to a different machine.
//!
//! ## Design
//! - Collects hardware fingerprint (hostname, MAC, machine-id)
//! - Derives a binding key via PBKDF2-HMAC-SHA256
//! - Stores salted binding hash for future verification
//! - On startup, re-derive and compare to detect device change

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::path::{Path, PathBuf};

/// Number of PBKDF2 iterations for key derivation.
const PBKDF2_ITERATIONS: u32 = 100_000;

/// Salt size in bytes.
const SALT_SIZE: usize = 16;

/// Device binding manager.
pub struct DeviceBinding {
    /// Path to the binding state file.
    binding_path: PathBuf,
    /// Whether binding enforcement is enabled.
    enabled: bool,
}

/// Result of a device binding check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingStatus {
    /// Device matches the stored binding.
    Bound,
    /// No binding exists yet (first run).
    Unbound,
    /// Device does not match the stored binding.
    Mismatch,
    /// Binding check is disabled.
    Disabled,
}

impl DeviceBinding {
    /// Create a new device binding manager.
    pub fn new(workspace_dir: &Path, enabled: bool) -> Self {
        Self {
            binding_path: workspace_dir.join(".device_binding"),
            enabled,
        }
    }

    /// Check if device binding is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Collect a hardware fingerprint for the current device.
    pub fn collect_fingerprint() -> String {
        let mut parts = Vec::new();

        // Hostname
        if let Ok(hostname) = hostname::get() {
            parts.push(hostname.to_string_lossy().to_string());
        }

        // Machine ID (Linux)
        if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
            parts.push(machine_id.trim().to_string());
        }

        // Fallback: OS + arch
        parts.push(format!(
            "{}:{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));

        parts.join("|")
    }

    /// Derive a binding key from the fingerprint using PBKDF2-HMAC-SHA256.
    fn derive_binding_key(fingerprint: &str, salt: &[u8]) -> [u8; 32] {
        // Simple PBKDF2 using HMAC-SHA256
        type HmacSha256 = Hmac<Sha256>;

        let mut result = [0u8; 32];

        // PBKDF2 with single block (32 bytes output = single HMAC-SHA256 block)
        let mut block = [0u8; 4];
        block[3] = 1; // Block counter = 1

        let mut mac = HmacSha256::new_from_slice(fingerprint.as_bytes())
            .expect("HMAC can accept any key length");
        mac.update(salt);
        mac.update(&block);
        let u = mac.finalize().into_bytes();

        let mut accumulator = [0u8; 32];
        accumulator.copy_from_slice(&u);
        let mut prev = u.to_vec();

        for _ in 1..PBKDF2_ITERATIONS {
            let mut mac = HmacSha256::new_from_slice(fingerprint.as_bytes())
                .expect("HMAC can accept any key length");
            mac.update(&prev);
            let u = mac.finalize().into_bytes();

            for (acc, &byte) in accumulator.iter_mut().zip(u.iter()) {
                *acc ^= byte;
            }
            prev = u.to_vec();
        }

        result.copy_from_slice(&accumulator);
        result
    }

    /// Create and store a new device binding.
    pub fn bind(&self) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let fingerprint = Self::collect_fingerprint();

        let mut salt = [0u8; SALT_SIZE];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut salt);

        let binding_key = Self::derive_binding_key(&fingerprint, &salt);

        // Store salt + binding hash
        use base64::Engine;
        let salt_b64 = base64::engine::general_purpose::STANDARD.encode(salt);
        let key_b64 = base64::engine::general_purpose::STANDARD.encode(binding_key);

        let binding_data = format!("{salt_b64}\n{key_b64}");
        std::fs::write(&self.binding_path, binding_data)?;

        Ok(())
    }

    /// Verify that the current device matches the stored binding.
    pub fn verify(&self) -> anyhow::Result<BindingStatus> {
        if !self.enabled {
            return Ok(BindingStatus::Disabled);
        }

        if !self.binding_path.exists() {
            return Ok(BindingStatus::Unbound);
        }

        let binding_data = std::fs::read_to_string(&self.binding_path)?;
        let mut lines = binding_data.lines();

        let salt_b64 = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid binding file: missing salt"))?;
        let stored_key_b64 = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid binding file: missing key"))?;

        use base64::Engine;
        let salt = base64::engine::general_purpose::STANDARD.decode(salt_b64)?;

        let fingerprint = Self::collect_fingerprint();
        let derived_key = Self::derive_binding_key(&fingerprint, &salt);
        let derived_b64 = base64::engine::general_purpose::STANDARD.encode(derived_key);

        if derived_b64 == stored_key_b64 {
            Ok(BindingStatus::Bound)
        } else {
            Ok(BindingStatus::Mismatch)
        }
    }

    /// Remove the device binding (e.g., for migration to a new device).
    pub fn unbind(&self) -> anyhow::Result<()> {
        if self.binding_path.exists() {
            std::fs::remove_file(&self.binding_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn fingerprint_is_non_empty() {
        let fp = DeviceBinding::collect_fingerprint();
        assert!(!fp.is_empty());
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let fp1 = DeviceBinding::collect_fingerprint();
        let fp2 = DeviceBinding::collect_fingerprint();
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn bind_and_verify_succeeds() {
        let tmp = TempDir::new().unwrap();
        let binding = DeviceBinding::new(tmp.path(), true);

        binding.bind().unwrap();
        let status = binding.verify().unwrap();
        assert_eq!(status, BindingStatus::Bound);
    }

    #[test]
    fn verify_unbound_returns_unbound() {
        let tmp = TempDir::new().unwrap();
        let binding = DeviceBinding::new(tmp.path(), true);

        let status = binding.verify().unwrap();
        assert_eq!(status, BindingStatus::Unbound);
    }

    #[test]
    fn disabled_binding_returns_disabled() {
        let tmp = TempDir::new().unwrap();
        let binding = DeviceBinding::new(tmp.path(), false);

        let status = binding.verify().unwrap();
        assert_eq!(status, BindingStatus::Disabled);
    }

    #[test]
    fn unbind_removes_file() {
        let tmp = TempDir::new().unwrap();
        let binding = DeviceBinding::new(tmp.path(), true);

        binding.bind().unwrap();
        assert!(tmp.path().join(".device_binding").exists());

        binding.unbind().unwrap();
        assert!(!tmp.path().join(".device_binding").exists());

        let status = binding.verify().unwrap();
        assert_eq!(status, BindingStatus::Unbound);
    }

    #[test]
    fn derive_binding_key_deterministic_with_same_salt() {
        let salt = [1u8; SALT_SIZE];
        let key1 = DeviceBinding::derive_binding_key("fingerprint_a", &salt);
        let key2 = DeviceBinding::derive_binding_key("fingerprint_a", &salt);
        assert_eq!(key1, key2);
    }

    #[test]
    fn derive_binding_key_different_with_different_fingerprint() {
        let salt = [1u8; SALT_SIZE];
        let key1 = DeviceBinding::derive_binding_key("fingerprint_a", &salt);
        let key2 = DeviceBinding::derive_binding_key("fingerprint_b", &salt);
        assert_ne!(key1, key2);
    }

    #[test]
    fn derive_binding_key_different_with_different_salt() {
        let salt1 = [1u8; SALT_SIZE];
        let salt2 = [2u8; SALT_SIZE];
        let key1 = DeviceBinding::derive_binding_key("fingerprint_a", &salt1);
        let key2 = DeviceBinding::derive_binding_key("fingerprint_a", &salt2);
        assert_ne!(key1, key2);
    }
}
