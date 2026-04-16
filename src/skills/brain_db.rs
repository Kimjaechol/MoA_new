//! Shared helpers for brain.db-backed subsystems (procedural skills,
//! correction patterns, user profiling, session search).
//!
//! All of these subsystems co-locate tables in the same SQLite file
//! (`<workspace>/memory/brain.db`) and follow the same open-with-WAL +
//! busy-timeout pattern. Extracted here to avoid four near-identical
//! `factory.rs` copies.

use anyhow::Result;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Canonical brain.db path inside the workspace.
///
/// Mirrors `memory::sqlite::SqliteMemory`'s DB location so skills,
/// memories, and timeline entries share the same SQLite file when the
/// deployment uses the SQLite memory backend.
pub fn brain_db_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join("memory").join("brain.db")
}

/// Open (or create) brain.db with the standard WAL + busy-timeout tuning
/// used across all self-learning stores.
///
/// The returned connection is wrapped in `Arc<Mutex<...>>` so it can be
/// shared with the matching Store/Profiler type.
pub fn open_brain_db(workspace_dir: &Path) -> Result<Arc<Mutex<Connection>>> {
    let path = brain_db_path(workspace_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000i64)?;
    Ok(Arc::new(Mutex::new(conn)))
}

/// Seconds since UNIX epoch — shared helper used by every timestamped
/// self-learning table. Infallible: falls back to 0 if the system clock
/// predates the epoch (practically impossible).
#[inline]
pub fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn open_brain_db_creates_dir() {
        let dir = TempDir::new().unwrap();
        let conn = open_brain_db(dir.path()).unwrap();
        let conn = conn.lock();
        // WAL should be active
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn now_epoch_is_monotonic() {
        let t1 = now_epoch();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let t2 = now_epoch();
        assert!(t2 >= t1);
    }
}
