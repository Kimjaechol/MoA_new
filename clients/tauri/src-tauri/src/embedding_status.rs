//! PR #1 — embedding model status + download progress for the Tauri UI.
//!
//! `fastembed` handles the actual HuggingFace download the first time
//! `TextEmbedding::try_new()` runs; it streams ~1.1 GB of BGE-M3 weights
//! into `~/.moa/embedding-models/models--BAAI--bge-m3/blobs/`. The
//! library doesn't expose a progress callback (5.8 uses an internal
//! `indicatif` bar on stderr), so we provide *visibility* rather than
//! drive the download ourselves: a cache-size probe that the frontend
//! polls while a download is in flight.
//!
//! Two commands:
//! * `check_embedding_model` — one-shot status (installed + size bytes)
//! * `monitor_embedding_download` — same payload, idempotent, intended
//!   for periodic polling from a React `useInterval`.
//!
//! The UI then renders:
//! * "Ready" when `installed` is true,
//! * "Downloading… xxx MB / ~1.1 GB" while bytes grow,
//! * "Not downloaded" when bytes == 0.

use std::fs;
use std::path::PathBuf;

use serde::Serialize;

/// Approximate on-disk footprint of a fully-downloaded BGE-M3 cache
/// (model.onnx + onnx_data + tokenizer + config files). Used as the
/// denominator for the progress bar. The true value fluctuates a few
/// dozen MB between revisions; 1.1 GB is close enough for UX.
const BGE_M3_TARGET_BYTES: u64 = 1_100_000_000;

/// The HuggingFace-style cache directory fastembed writes into.
fn model_cache_root() -> Option<PathBuf> {
    // Respect the MOA_EMBEDDING_CACHE override so test harnesses and
    // sandboxed builds see the same directory fastembed is writing to.
    if let Some(custom) = std::env::var_os("MOA_EMBEDDING_CACHE") {
        return Some(PathBuf::from(custom));
    }
    dirs_next::home_dir().map(|h| h.join(".moa").join("embedding-models"))
}

/// Sum the byte-size of every regular file under `dir`, recursively.
/// Symlinks and directories contribute 0. Best-effort: any IO error on
/// a sub-path is skipped so a partial read never poisons the whole sum.
fn dir_size_bytes(dir: &std::path::Path) -> u64 {
    fn walk(path: &std::path::Path, total: &mut u64) {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_file() {
                *total += meta.len();
            } else if meta.is_dir() {
                walk(&entry.path(), total);
            }
        }
    }
    let mut total = 0u64;
    walk(dir, &mut total);
    total
}

/// Status payload sent back to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingModelStatus {
    /// Absolute cache directory (or empty if home can't be resolved).
    pub cache_dir: String,
    /// Does the BGE-M3 subdirectory exist?
    pub model_present: bool,
    /// Bytes currently on disk for the BGE-M3 model.
    pub size_bytes: u64,
    /// Target bytes for the completed download (approximate — used for
    /// the progress-bar denominator only).
    pub target_bytes: u64,
    /// `true` when `size_bytes >= 95% of target_bytes` — a heuristic
    /// that holds even if the target shifts by ±50 MB between revisions.
    pub installed: bool,
    /// Progress fraction in [0.0, 1.0].
    pub progress: f64,
}

fn build_status() -> EmbeddingModelStatus {
    let Some(root) = model_cache_root() else {
        return EmbeddingModelStatus {
            cache_dir: String::new(),
            model_present: false,
            size_bytes: 0,
            target_bytes: BGE_M3_TARGET_BYTES,
            installed: false,
            progress: 0.0,
        };
    };
    let bge = root.join("models--BAAI--bge-m3");
    let model_present = bge.exists();
    let size_bytes = if model_present {
        dir_size_bytes(&bge)
    } else {
        0
    };
    let installed = size_bytes as f64 >= BGE_M3_TARGET_BYTES as f64 * 0.95;
    #[allow(clippy::cast_precision_loss)]
    let progress = (size_bytes as f64 / BGE_M3_TARGET_BYTES as f64).clamp(0.0, 1.0);

    EmbeddingModelStatus {
        cache_dir: root.display().to_string(),
        model_present,
        size_bytes,
        target_bytes: BGE_M3_TARGET_BYTES,
        installed,
        progress,
    }
}

#[tauri::command]
pub fn check_embedding_model() -> EmbeddingModelStatus {
    build_status()
}

#[tauri::command]
pub fn monitor_embedding_download() -> EmbeddingModelStatus {
    build_status()
}
