//! Encrypted backup vault for ZeroClaw (Time Machine-style).
//!
//! Provides automatic rotating encrypted backups with configurable retention:
//! - Daily backups: last 7 days
//! - Weekly backups: last 4 weeks
//! - Monthly backups: last 12 months
//! - Maximum 23 backup files total
//!
//! ## Design
//! - AES-256-GCM encryption using the workspace encryption key
//! - Backs up critical data: memory DB, secrets, sync state
//! - Rotation prunes old backups automatically
//! - Recovery via 12-word recovery code or key file

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use std::path::{Path, PathBuf};

/// AES-GCM nonce size.
const NONCE_SIZE: usize = 12;

/// Maximum daily backups.
const MAX_DAILY: usize = 7;

/// Maximum weekly backups.
const MAX_WEEKLY: usize = 4;

/// Maximum monthly backups.
const MAX_MONTHLY: usize = 12;

/// Seconds in a day.
const DAY_SECS: u64 = 86_400;

/// Seconds in a week.
const WEEK_SECS: u64 = 7 * DAY_SECS;

/// Backup tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupTier {
    Daily,
    Weekly,
    Monthly,
}

/// Metadata for a backup file.
#[derive(Debug, Clone)]
pub struct BackupMeta {
    /// Path to the encrypted backup file.
    pub path: PathBuf,
    /// Backup tier.
    pub tier: BackupTier,
    /// Creation timestamp (epoch seconds).
    pub created_at: u64,
    /// Size in bytes.
    pub size: u64,
}

/// Encrypted backup vault manager.
pub struct EncryptedVault {
    /// Root directory for backup storage.
    vault_dir: PathBuf,
    /// Encryption key (32 bytes, AES-256).
    key: [u8; 32],
    /// Whether the vault is enabled.
    enabled: bool,
}

impl EncryptedVault {
    /// Create a new encrypted vault.
    pub fn new(workspace_dir: &Path, key: [u8; 32], enabled: bool) -> anyhow::Result<Self> {
        let vault_dir = workspace_dir.join("vault");
        if enabled {
            std::fs::create_dir_all(vault_dir.join("daily"))?;
            std::fs::create_dir_all(vault_dir.join("weekly"))?;
            std::fs::create_dir_all(vault_dir.join("monthly"))?;
        }
        Ok(Self {
            vault_dir,
            key,
            enabled,
        })
    }

    /// Create a new backup of the specified data.
    /// Returns the path to the created backup file.
    pub fn backup(&self, data: &[u8]) -> anyhow::Result<PathBuf> {
        if !self.enabled {
            anyhow::bail!("Vault is disabled");
        }

        let now = current_epoch_secs();
        let encrypted = self.encrypt(data)?;

        // Use timestamp + random suffix for unique filenames within the same second
        let suffix: u32 = rand::random();
        let filename = format!("backup-{now}-{suffix:08x}.enc");
        let backup_path = self.vault_dir.join("daily").join(&filename);
        std::fs::write(&backup_path, &encrypted)?;

        // Rotate: promote old dailies to weekly/monthly
        self.rotate()?;

        Ok(backup_path)
    }

    /// Restore data from a backup file.
    pub fn restore(&self, backup_path: &Path) -> anyhow::Result<Vec<u8>> {
        if !self.enabled {
            anyhow::bail!("Vault is disabled");
        }

        let encrypted = std::fs::read(backup_path)?;
        self.decrypt(&encrypted)
    }

    /// List all backups across all tiers, sorted by creation time (newest first).
    pub fn list_backups(&self) -> anyhow::Result<Vec<BackupMeta>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for (tier, dir_name) in [
            (BackupTier::Daily, "daily"),
            (BackupTier::Weekly, "weekly"),
            (BackupTier::Monthly, "monthly"),
        ] {
            let dir = self.vault_dir.join(dir_name);
            if !dir.exists() {
                continue;
            }

            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("enc") {
                    let metadata = entry.metadata()?;
                    let created_at = Self::extract_timestamp_from_filename(&path);
                    backups.push(BackupMeta {
                        path,
                        tier,
                        created_at,
                        size: metadata.len(),
                    });
                }
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(backups)
    }

    /// Rotate backups: promote old dailies to weekly, weeklies to monthly,
    /// and prune excess backups in each tier.
    fn rotate(&self) -> anyhow::Result<()> {
        let now = current_epoch_secs();

        // Collect daily backups sorted oldest first
        let daily_dir = self.vault_dir.join("daily");
        let mut dailies = self.list_tier_backups(&daily_dir)?;
        dailies.sort_by_key(|b| b.created_at);

        // Promote dailies older than 7 days to weekly
        let week_cutoff = now.saturating_sub(MAX_DAILY as u64 * DAY_SECS);
        for backup in &dailies {
            if backup.created_at < week_cutoff {
                let dest = self
                    .vault_dir
                    .join("weekly")
                    .join(backup.path.file_name().unwrap_or_default());
                std::fs::rename(&backup.path, &dest)?;
            }
        }

        // Promote weeklies older than 4 weeks to monthly
        let weekly_dir = self.vault_dir.join("weekly");
        let mut weeklies = self.list_tier_backups(&weekly_dir)?;
        weeklies.sort_by_key(|b| b.created_at);

        let month_cutoff = now.saturating_sub(MAX_WEEKLY as u64 * WEEK_SECS);
        for backup in &weeklies {
            if backup.created_at < month_cutoff {
                let dest = self
                    .vault_dir
                    .join("monthly")
                    .join(backup.path.file_name().unwrap_or_default());
                std::fs::rename(&backup.path, &dest)?;
            }
        }

        // Prune excess in each tier
        self.prune_tier(&self.vault_dir.join("daily"), MAX_DAILY)?;
        self.prune_tier(&self.vault_dir.join("weekly"), MAX_WEEKLY)?;
        self.prune_tier(&self.vault_dir.join("monthly"), MAX_MONTHLY)?;

        Ok(())
    }

    /// List backup files in a tier directory.
    fn list_tier_backups(&self, dir: &Path) -> anyhow::Result<Vec<BackupMeta>> {
        let mut backups = Vec::new();
        if !dir.exists() {
            return Ok(backups);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("enc") {
                let metadata = entry.metadata()?;
                let created_at = Self::extract_timestamp_from_filename(&path);
                backups.push(BackupMeta {
                    path,
                    tier: BackupTier::Daily, // tier doesn't matter here
                    created_at,
                    size: metadata.len(),
                });
            }
        }

        Ok(backups)
    }

    /// Prune a tier directory to keep at most `max_count` files.
    fn prune_tier(&self, dir: &Path, max_count: usize) -> anyhow::Result<()> {
        let mut backups = self.list_tier_backups(dir)?;
        if backups.len() <= max_count {
            return Ok(());
        }

        // Sort oldest first, remove oldest
        backups.sort_by_key(|b| b.created_at);
        let to_remove = backups.len() - max_count;
        for backup in backups.iter().take(to_remove) {
            std::fs::remove_file(&backup.path)?;
        }

        Ok(())
    }

    /// Extract a timestamp from a backup filename like "backup-1234567890-abcd1234.enc".
    fn extract_timestamp_from_filename(path: &Path) -> u64 {
        path.file_stem()
            .and_then(|s| s.to_str())
            .and_then(|name| name.strip_prefix("backup-"))
            .and_then(|rest| {
                // Handle both "timestamp" and "timestamp-suffix" formats
                let ts_part = rest.split('-').next().unwrap_or(rest);
                ts_part.parse::<u64>().ok()
            })
            .unwrap_or(0)
    }

    /// Encrypt data using AES-256-GCM.
    fn encrypt(&self, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| anyhow::anyhow!("Cipher init failed: {e}"))?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

        // Format: [nonce (12 bytes)][ciphertext]
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM.
    fn decrypt(&self, encrypted: &[u8]) -> anyhow::Result<Vec<u8>> {
        if encrypted.len() < NONCE_SIZE {
            anyhow::bail!("Encrypted data too short");
        }

        let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| anyhow::anyhow!("Cipher init failed: {e}"))?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {e}"))
    }

    /// Get the total number of backup files across all tiers.
    pub fn backup_count(&self) -> anyhow::Result<usize> {
        Ok(self.list_backups()?.len())
    }
}

/// Get current epoch seconds.
fn current_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    #[test]
    fn backup_and_restore_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let key = test_key();
        let vault = EncryptedVault::new(tmp.path(), key, true).unwrap();

        let data = b"ZeroClaw memory data for backup";
        let backup_path = vault.backup(data).unwrap();

        assert!(backup_path.exists());
        let restored = vault.restore(&backup_path).unwrap();
        assert_eq!(restored, data);
    }

    #[test]
    fn backup_encrypted_differs_from_plaintext() {
        let tmp = TempDir::new().unwrap();
        let key = test_key();
        let vault = EncryptedVault::new(tmp.path(), key, true).unwrap();

        let data = b"sensitive data";
        let backup_path = vault.backup(data).unwrap();

        let encrypted = std::fs::read(&backup_path).unwrap();
        assert_ne!(encrypted, data);
    }

    #[test]
    fn wrong_key_fails_restore() {
        let tmp = TempDir::new().unwrap();
        let key1 = test_key();
        let key2 = test_key();

        let vault1 = EncryptedVault::new(tmp.path(), key1, true).unwrap();
        let backup_path = vault1.backup(b"secret").unwrap();

        let vault2 = EncryptedVault::new(tmp.path(), key2, true).unwrap();
        assert!(vault2.restore(&backup_path).is_err());
    }

    #[test]
    fn list_backups_returns_all() {
        let tmp = TempDir::new().unwrap();
        let key = test_key();
        let vault = EncryptedVault::new(tmp.path(), key, true).unwrap();

        vault.backup(b"backup 1").unwrap();
        vault.backup(b"backup 2").unwrap();
        vault.backup(b"backup 3").unwrap();

        let backups = vault.list_backups().unwrap();
        assert_eq!(backups.len(), 3);
    }

    #[test]
    fn prune_removes_excess_daily() {
        let tmp = TempDir::new().unwrap();
        let key = test_key();
        let vault = EncryptedVault::new(tmp.path(), key, true).unwrap();

        // Create more than MAX_DAILY backups in the daily dir
        let daily_dir = tmp.path().join("vault").join("daily");
        for i in 0..(MAX_DAILY + 3) {
            let filename = format!("backup-{}.enc", 1000 + i);
            let encrypted = vault.encrypt(b"data").unwrap();
            std::fs::write(daily_dir.join(filename), encrypted).unwrap();
        }

        // Prune should remove 3 oldest
        vault.prune_tier(&daily_dir, MAX_DAILY).unwrap();

        let remaining = vault.list_tier_backups(&daily_dir).unwrap();
        assert_eq!(remaining.len(), MAX_DAILY);
    }

    #[test]
    fn disabled_vault_rejects_operations() {
        let tmp = TempDir::new().unwrap();
        let key = test_key();
        let vault = EncryptedVault::new(tmp.path(), key, false).unwrap();

        assert!(vault.backup(b"data").is_err());
        assert!(vault.list_backups().unwrap().is_empty());
    }

    #[test]
    fn extract_timestamp_from_filename_works() {
        let path = PathBuf::from("/tmp/vault/daily/backup-1708123456-abcd1234.enc");
        assert_eq!(
            EncryptedVault::extract_timestamp_from_filename(&path),
            1_708_123_456
        );
    }

    #[test]
    fn extract_timestamp_legacy_filename() {
        let path = PathBuf::from("/tmp/vault/daily/backup-1708123456.enc");
        assert_eq!(
            EncryptedVault::extract_timestamp_from_filename(&path),
            1_708_123_456
        );
    }

    #[test]
    fn extract_timestamp_invalid_filename() {
        let path = PathBuf::from("/tmp/vault/daily/random.enc");
        assert_eq!(EncryptedVault::extract_timestamp_from_filename(&path), 0);
    }

    #[test]
    fn backup_count() {
        let tmp = TempDir::new().unwrap();
        let key = test_key();
        let vault = EncryptedVault::new(tmp.path(), key, true).unwrap();

        assert_eq!(vault.backup_count().unwrap(), 0);
        vault.backup(b"data1").unwrap();
        assert_eq!(vault.backup_count().unwrap(), 1);
        vault.backup(b"data2").unwrap();
        assert_eq!(vault.backup_count().unwrap(), 2);
    }

    #[test]
    fn vault_directories_created() {
        let tmp = TempDir::new().unwrap();
        let key = test_key();
        let _vault = EncryptedVault::new(tmp.path(), key, true).unwrap();

        assert!(tmp.path().join("vault/daily").exists());
        assert!(tmp.path().join("vault/weekly").exists());
        assert!(tmp.path().join("vault/monthly").exists());
    }
}
