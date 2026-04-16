//! Factory helper — wire a UserProfiler against the shared brain.db.

use super::profiler::UserProfiler;
use crate::skills::brain_db;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub fn build_profiler(workspace_dir: &Path, device_id: &str) -> Result<Arc<UserProfiler>> {
    let conn = brain_db::open_brain_db(workspace_dir)?;
    let profiler = UserProfiler::new(conn, device_id.to_string());
    profiler.migrate()?;
    Ok(Arc::new(profiler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn build_profiler_creates_and_migrates() {
        let dir = TempDir::new().unwrap();
        let _ = build_profiler(dir.path(), "test-dev").unwrap();
    }
}
