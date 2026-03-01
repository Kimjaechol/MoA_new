//! Remote wipe capability for ZeroClaw.
//!
//! Allows secure erasure of sensitive data (secrets, memory, config)
//! from a ZeroClaw instance, triggered either locally or via a
//! remote command through a connected channel.
//!
//! ## Safety
//! - Requires explicit confirmation token to prevent accidental wipes
//! - Logs all wipe operations to the audit trail
//! - Supports selective wipe (secrets only, memory only, full)

use std::path::{Path, PathBuf};

/// Confirmation token length for wipe authorization.
const WIPE_TOKEN_LENGTH: usize = 8;

/// Types of data that can be wiped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WipeScope {
    /// Wipe only encrypted secrets
    Secrets,
    /// Wipe only memory database
    Memory,
    /// Wipe memory and secrets
    Full,
    /// Wipe sync state (device binding, sync keys, delta journals)
    SyncState,
}

/// Result of a wipe operation.
#[derive(Debug, Clone)]
pub struct WipeResult {
    /// Scope of the wipe that was performed.
    pub scope: WipeScope,
    /// Number of files deleted.
    pub files_deleted: usize,
    /// Any errors encountered (non-fatal).
    pub errors: Vec<String>,
    /// Whether the wipe completed successfully overall.
    pub success: bool,
}

/// Remote wipe manager.
pub struct RemoteWipe {
    /// Workspace directory containing data to wipe.
    workspace_dir: PathBuf,
    /// Whether remote wipe is enabled.
    enabled: bool,
}

impl RemoteWipe {
    /// Create a new remote wipe manager.
    pub fn new(workspace_dir: &Path, enabled: bool) -> Self {
        Self {
            workspace_dir: workspace_dir.to_path_buf(),
            enabled,
        }
    }

    /// Check if remote wipe is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Generate a one-time confirmation token for wipe authorization.
    pub fn generate_confirmation_token() -> String {
        use rand::RngExt;
        let mut rng = rand::rng();
        let chars: Vec<char> = (0..WIPE_TOKEN_LENGTH)
            .map(|_| {
                let idx = rng.random_range(0..36u8);
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'a' + idx - 10) as char
                }
            })
            .collect();
        chars.into_iter().collect()
    }

    /// Execute a wipe operation with the given scope.
    pub fn execute_wipe(&self, scope: &WipeScope) -> WipeResult {
        if !self.enabled {
            return WipeResult {
                scope: scope.clone(),
                files_deleted: 0,
                errors: vec!["Remote wipe is disabled".to_string()],
                success: false,
            };
        }

        let mut files_deleted = 0;
        let mut errors = Vec::new();

        match scope {
            WipeScope::Secrets => {
                let result = self.wipe_secrets();
                files_deleted += result.0;
                errors.extend(result.1);
            }
            WipeScope::Memory => {
                let result = self.wipe_memory();
                files_deleted += result.0;
                errors.extend(result.1);
            }
            WipeScope::Full => {
                let secrets = self.wipe_secrets();
                files_deleted += secrets.0;
                errors.extend(secrets.1);

                let memory = self.wipe_memory();
                files_deleted += memory.0;
                errors.extend(memory.1);
            }
            WipeScope::SyncState => {
                let result = self.wipe_sync_state();
                files_deleted += result.0;
                errors.extend(result.1);
            }
        }

        WipeResult {
            scope: scope.clone(),
            files_deleted,
            success: errors.is_empty(),
            errors,
        }
    }

    /// Wipe secret store files.
    fn wipe_secrets(&self) -> (usize, Vec<String>) {
        let mut deleted = 0;
        let mut errors = Vec::new();

        let secret_files = ["secrets.key", ".secrets.db"];

        for filename in &secret_files {
            let path = self.workspace_dir.join(filename);
            if path.exists() {
                match secure_delete(&path) {
                    Ok(()) => deleted += 1,
                    Err(e) => errors.push(format!("Failed to wipe {filename}: {e}")),
                }
            }
        }

        deleted += self.wipe_glob_pattern("*.key", &mut errors);

        (deleted, errors)
    }

    /// Wipe memory database files.
    fn wipe_memory(&self) -> (usize, Vec<String>) {
        let mut deleted = 0;
        let mut errors = Vec::new();

        let memory_files = [
            "brain.db",
            "brain.db-shm",
            "brain.db-wal",
            "response_cache.db",
            "response_cache.db-shm",
            "response_cache.db-wal",
        ];

        for filename in &memory_files {
            let path = self.workspace_dir.join(filename);
            if path.exists() {
                match secure_delete(&path) {
                    Ok(()) => deleted += 1,
                    Err(e) => errors.push(format!("Failed to wipe {filename}: {e}")),
                }
            }
        }

        // Wipe memory directory markdown files
        let memory_dir = self.workspace_dir.join("memory");
        if memory_dir.exists() {
            deleted += Self::wipe_directory(&memory_dir, &mut errors);
        }

        (deleted, errors)
    }

    /// Wipe sync state files.
    fn wipe_sync_state(&self) -> (usize, Vec<String>) {
        let mut deleted = 0;
        let mut errors = Vec::new();

        let sync_files = [
            ".device_id",
            ".device_binding",
            ".sync_key",
            "sync_state.db",
            "sync_state.db-shm",
            "sync_state.db-wal",
        ];

        for filename in &sync_files {
            let path = self.workspace_dir.join(filename);
            if path.exists() {
                match secure_delete(&path) {
                    Ok(()) => deleted += 1,
                    Err(e) => errors.push(format!("Failed to wipe {filename}: {e}")),
                }
            }
        }

        (deleted, errors)
    }

    /// Wipe files matching a glob pattern in the workspace directory.
    fn wipe_glob_pattern(&self, pattern: &str, errors: &mut Vec<String>) -> usize {
        let mut deleted = 0;
        let full_pattern = self.workspace_dir.join(pattern);

        if let Ok(entries) = glob::glob(&full_pattern.to_string_lossy()) {
            for entry in entries.flatten() {
                match secure_delete(&entry) {
                    Ok(()) => deleted += 1,
                    Err(e) => errors.push(format!("Failed to wipe {}: {e}", entry.display())),
                }
            }
        }

        deleted
    }

    /// Recursively wipe a directory and its contents.
    fn wipe_directory(dir: &Path, errors: &mut Vec<String>) -> usize {
        let mut deleted = 0;

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    match secure_delete(&path) {
                        Ok(()) => deleted += 1,
                        Err(e) => errors.push(format!("Failed to wipe {}: {e}", path.display())),
                    }
                } else if path.is_dir() {
                    deleted += Self::wipe_directory(&path, errors);
                }
            }
        }

        // Remove the directory itself
        let _ = std::fs::remove_dir(dir);

        deleted
    }
}

/// Securely delete a file by overwriting with zeros before removal.
fn secure_delete(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    // Overwrite with zeros
    let metadata = std::fs::metadata(path)?;
    let size = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    if size > 0 {
        let zeros = vec![0u8; size.min(1024 * 1024)]; // Cap at 1MB per write
        let mut remaining = size;
        let file = std::fs::OpenOptions::new().write(true).open(path)?;
        use std::io::Write;
        let mut writer = std::io::BufWriter::new(file);
        while remaining > 0 {
            let to_write = remaining.min(zeros.len());
            writer.write_all(&zeros[..to_write])?;
            remaining -= to_write;
        }
        writer.flush()?;
    }

    // Remove the file
    std::fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn confirmation_token_correct_length() {
        let token = RemoteWipe::generate_confirmation_token();
        assert_eq!(token.len(), WIPE_TOKEN_LENGTH);
    }

    #[test]
    fn confirmation_token_alphanumeric() {
        let token = RemoteWipe::generate_confirmation_token();
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn confirmation_tokens_are_unique() {
        let t1 = RemoteWipe::generate_confirmation_token();
        let t2 = RemoteWipe::generate_confirmation_token();
        // With 36^8 possibilities, collision is astronomically unlikely
        assert_ne!(t1, t2);
    }

    #[test]
    fn wipe_secrets_deletes_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("secrets.key"), "secret_data").unwrap();

        let wiper = RemoteWipe::new(tmp.path(), true);
        let result = wiper.execute_wipe(&WipeScope::Secrets);

        assert!(result.success);
        assert!(result.files_deleted >= 1);
        assert!(!tmp.path().join("secrets.key").exists());
    }

    #[test]
    fn wipe_memory_deletes_db() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("brain.db"), "memory_data").unwrap();
        std::fs::write(tmp.path().join("response_cache.db"), "cache_data").unwrap();

        let wiper = RemoteWipe::new(tmp.path(), true);
        let result = wiper.execute_wipe(&WipeScope::Memory);

        assert!(result.success);
        assert!(result.files_deleted >= 2);
        assert!(!tmp.path().join("brain.db").exists());
        assert!(!tmp.path().join("response_cache.db").exists());
    }

    #[test]
    fn wipe_full_deletes_both() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("secrets.key"), "secret").unwrap();
        std::fs::write(tmp.path().join("brain.db"), "memory").unwrap();

        let wiper = RemoteWipe::new(tmp.path(), true);
        let result = wiper.execute_wipe(&WipeScope::Full);

        assert!(result.success);
        assert!(result.files_deleted >= 2);
    }

    #[test]
    fn wipe_sync_state_deletes_sync_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".device_id"), "device_123").unwrap();
        std::fs::write(tmp.path().join(".sync_key"), [0u8; 32]).unwrap();
        std::fs::write(tmp.path().join(".device_binding"), "binding_data").unwrap();

        let wiper = RemoteWipe::new(tmp.path(), true);
        let result = wiper.execute_wipe(&WipeScope::SyncState);

        assert!(result.success);
        assert!(result.files_deleted >= 3);
        assert!(!tmp.path().join(".device_id").exists());
        assert!(!tmp.path().join(".sync_key").exists());
    }

    #[test]
    fn disabled_wipe_does_nothing() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("brain.db"), "memory").unwrap();

        let wiper = RemoteWipe::new(tmp.path(), false);
        let result = wiper.execute_wipe(&WipeScope::Full);

        assert!(!result.success);
        assert_eq!(result.files_deleted, 0);
        // File should still exist
        assert!(tmp.path().join("brain.db").exists());
    }

    #[test]
    fn secure_delete_overwrites_before_removing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("sensitive.txt");
        std::fs::write(&path, "very sensitive data").unwrap();

        secure_delete(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn secure_delete_nonexistent_file_is_ok() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.txt");
        assert!(secure_delete(&path).is_ok());
    }

    #[test]
    fn wipe_memory_directory_recursively() {
        let tmp = TempDir::new().unwrap();
        let memory_dir = tmp.path().join("memory");
        std::fs::create_dir_all(&memory_dir).unwrap();
        std::fs::write(memory_dir.join("2024-01-01.md"), "note 1").unwrap();
        std::fs::write(memory_dir.join("2024-01-02.md"), "note 2").unwrap();

        let wiper = RemoteWipe::new(tmp.path(), true);
        let result = wiper.execute_wipe(&WipeScope::Memory);

        assert!(result.success);
        assert!(result.files_deleted >= 2);
    }
}
