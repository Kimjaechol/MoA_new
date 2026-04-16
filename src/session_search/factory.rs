//! Factory helper — wire a SessionSearchStore against the shared brain.db.

use super::store::SessionSearchStore;
use crate::skills::brain_db;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub fn build_store(workspace_dir: &Path, device_id: &str) -> Result<Arc<SessionSearchStore>> {
    let conn = brain_db::open_brain_db(workspace_dir)?;
    let store = SessionSearchStore::new(conn, device_id.to_string());
    store.migrate()?;
    Ok(Arc::new(store))
}
