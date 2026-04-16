//! Factory helper — wire a CorrectionStore against the shared brain.db.

use super::store::CorrectionStore;
use crate::skills::brain_db;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub fn build_store(workspace_dir: &Path, device_id: &str) -> Result<Arc<CorrectionStore>> {
    let conn = brain_db::open_brain_db(workspace_dir)?;
    let store = CorrectionStore::new(conn, device_id.to_string());
    store.migrate()?;
    Ok(Arc::new(store))
}
