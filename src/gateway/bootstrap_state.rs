//! Shared bootstrap progress state for the first-launch local-LLM install.
//!
//! The gateway kicks off `local_llm::setup::ensure_ready` in a background
//! task immediately after binding its port (see `gateway::mod::run_gateway`),
//! so the HTTP surface is responsive *while* Ollama + Gemma 4 are being
//! downloaded. The desktop/mobile client polls
//! `GET /api/local-llm/bootstrap-status` (see `local_llm_api`) to render the
//! "기본 총 (Gemma 4) 준비 중…" screen.
//!
//! Keep this module tiny and panic-free: callbacks touch the state from
//! synchronous `FnMut` closures called inside an async task, so we use a
//! plain `std::sync::Mutex` (blocking writes take microseconds) rather than
//! `tokio::sync::Mutex` to avoid smuggling async runtime concerns into the
//! `SetupCallbacks` contract.
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;

/// Discriminated progress snapshot surfaced to the client.
///
/// Mirrors [`crate::local_llm::setup::SetupStage`] but adds the pull-progress
/// digest + fraction inline so the UI can render a download bar without a
/// second polling endpoint. `NotStarted` is emitted until the background
/// task actually begins work (covers the window between `/health` becoming
/// reachable and the first `Probing` transition).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum BootstrapStage {
    /// Gateway has bound its port but the background task has not yet run.
    NotStarted,
    /// Auto-bootstrap disabled via `ZEROCLAW_SKIP_LOCAL_LLM_BOOTSTRAP=1`.
    Skipped,
    /// Detecting RAM / GPU / disk / OS.
    Probing,
    /// Verifying free disk against the minimum threshold.
    CheckingDisk,
    /// Running the OS-matched Ollama installer.
    InstallingOllama,
    /// Polling the Ollama HTTP endpoint until it is reachable.
    WaitingForDaemon,
    /// `/api/pull` stream is active for the recommended Gemma 4 tag.
    PullingModel {
        /// 1-indexed retry attempt.
        attempt: u32,
        /// 0.0 .. 1.0 download completion. `None` while the NDJSON stream is
        /// emitting non-progress events (verifying digest, layer metadata).
        #[serde(skip_serializing_if = "Option::is_none")]
        fraction: Option<f32>,
        /// Free-form status line straight from Ollama (e.g.
        /// `"downloading digest sha256:…"`). Useful for the debug log but
        /// not required for the progress bar.
        #[serde(skip_serializing_if = "Option::is_none")]
        pull_status: Option<String>,
    },
    /// Writing hardware profile and local-LLM config.
    Persisting,
    /// Setup finished. The tag is what the gateway will route local requests
    /// to (matches `default_model` in [`crate::local_llm::LocalLlmConfig`]).
    Done {
        model: String,
    },
    /// Install or pull failed. Non-fatal for the gateway as a whole — the
    /// client can still BYOK a cloud provider.
    Error {
        message: String,
    },
}

static STATE: OnceLock<Arc<Mutex<BootstrapStage>>> = OnceLock::new();

fn global() -> &'static Arc<Mutex<BootstrapStage>> {
    STATE.get_or_init(|| Arc::new(Mutex::new(BootstrapStage::NotStarted)))
}

/// Overwrite the current snapshot with `stage`. Poisoned lock is ignored —
/// the callers are bulk-writers from `FnMut` closures and a poisoned mutex
/// here just means a previous writer panicked, which cannot corrupt the next
/// write because we always clobber rather than read-modify-write.
pub fn set(stage: BootstrapStage) {
    let mutex = global();
    match mutex.lock() {
        Ok(mut guard) => *guard = stage,
        Err(poisoned) => *poisoned.into_inner() = stage,
    }
}

/// Read-only snapshot for the HTTP handler.
pub fn snapshot() -> BootstrapStage {
    let mutex = global();
    match mutex.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}
