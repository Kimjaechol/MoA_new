//! Mobile runtime adapter for Android/iOS.
//!
//! Constrains the agent runtime for mobile environments:
//! - No direct shell access (security + sandboxing)
//! - Filesystem limited to app data directory
//! - Reduced memory budget (256 MB default)
//! - Long-running processes supported (foreground service / background task)

use super::traits::RuntimeAdapter;
use std::path::{Path, PathBuf};

/// Default memory budget for mobile: 256 MB.
const DEFAULT_MOBILE_MEMORY_BUDGET: u64 = 256 * 1024 * 1024;

/// Runtime adapter for mobile platforms (Android/iOS via Tauri).
///
/// Mobile devices have stricter resource constraints and security
/// sandboxing.  This adapter reflects those limits so the agent
/// and tool subsystems respect mobile boundaries.
pub struct MobileRuntime {
    /// App-specific data directory (e.g. Android `getFilesDir()` or iOS `Documents/`).
    data_dir: PathBuf,
    /// Memory budget in bytes (default 256 MB).
    memory_budget: u64,
}

impl MobileRuntime {
    /// Create a new mobile runtime with the given app data directory.
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            memory_budget: DEFAULT_MOBILE_MEMORY_BUDGET,
        }
    }

    /// Create a mobile runtime with a custom memory budget.
    pub fn with_memory_budget(data_dir: PathBuf, memory_budget_bytes: u64) -> Self {
        Self {
            data_dir,
            memory_budget: memory_budget_bytes,
        }
    }
}

impl RuntimeAdapter for MobileRuntime {
    fn name(&self) -> &str {
        "mobile"
    }

    /// Mobile does not allow direct shell access for security.
    fn has_shell_access(&self) -> bool {
        false
    }

    /// Mobile has filesystem access within the app sandbox.
    fn has_filesystem_access(&self) -> bool {
        true
    }

    /// Returns the app-specific data directory.
    fn storage_path(&self) -> PathBuf {
        self.data_dir.clone()
    }

    /// Mobile supports long-running via foreground services / background tasks.
    fn supports_long_running(&self) -> bool {
        true
    }

    /// Returns the configured memory budget (default 256 MB).
    fn memory_budget(&self) -> u64 {
        self.memory_budget
    }

    /// Shell commands are not available on mobile — return an error.
    fn build_shell_command(
        &self,
        command: &str,
        _workspace_dir: &Path,
    ) -> anyhow::Result<tokio::process::Command> {
        anyhow::bail!(
            "Shell commands are not available on mobile runtime (attempted: '{}'). \
             Mobile agents should use tool APIs instead of shell execution.",
            command
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_name() {
        let rt = MobileRuntime::new(PathBuf::from("/data/com.moa.agent"));
        assert_eq!(rt.name(), "mobile");
    }

    #[test]
    fn mobile_no_shell_access() {
        let rt = MobileRuntime::new(PathBuf::from("/data"));
        assert!(!rt.has_shell_access());
    }

    #[test]
    fn mobile_has_filesystem_access() {
        let rt = MobileRuntime::new(PathBuf::from("/data"));
        assert!(rt.has_filesystem_access());
    }

    #[test]
    fn mobile_supports_long_running() {
        let rt = MobileRuntime::new(PathBuf::from("/data"));
        assert!(rt.supports_long_running());
    }

    #[test]
    fn mobile_default_memory_budget() {
        let rt = MobileRuntime::new(PathBuf::from("/data"));
        assert_eq!(rt.memory_budget(), 256 * 1024 * 1024);
    }

    #[test]
    fn mobile_custom_memory_budget() {
        let rt = MobileRuntime::with_memory_budget(PathBuf::from("/data"), 128 * 1024 * 1024);
        assert_eq!(rt.memory_budget(), 128 * 1024 * 1024);
    }

    #[test]
    fn mobile_storage_path() {
        let rt = MobileRuntime::new(PathBuf::from("/data/com.moa.agent/files"));
        assert_eq!(rt.storage_path(), PathBuf::from("/data/com.moa.agent/files"));
    }

    #[test]
    fn mobile_shell_command_errors() {
        let rt = MobileRuntime::new(PathBuf::from("/data"));
        let result = rt.build_shell_command("ls", Path::new("/"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not available on mobile"));
    }
}
