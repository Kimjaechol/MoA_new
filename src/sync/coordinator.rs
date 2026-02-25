//! Sync Coordinator — end-to-end orchestration for Layer 2 + Layer 3 sync.
//!
//! This module bridges the gap between the sync protocol definitions
//! (message types, order buffer, manifest comparison) and the actual
//! memory backend.  It handles incoming `BroadcastMessage` variants,
//! dispatches them to the correct handlers, and drives outbound sync
//! flows (delta catch-up, full sync).
//!
//! ## Responsibilities
//!
//! - Deserialize incoming WebSocket JSON → `BroadcastMessage`
//! - **Layer 2**: Handle `SyncRequest` → respond with missing deltas
//! - **Layer 2**: Handle `SyncResponse` → apply ordered deltas via `OrderBuffer`
//! - **Layer 3**: Handle `FullSyncRequest` → compare manifests → respond
//! - **Layer 3**: Handle `FullSyncManifestResponse` → export & send missing entries
//! - **Layer 3**: Handle `FullSyncData` → decrypt & import into memory
//! - **Layer 3**: Handle `FullSyncComplete` → log completion
//! - **Layer 3**: Timeout detection for unresponsive peers
//! - Trigger outbound sync (delta catch-up, full sync request)

use crate::memory::sync::{DeltaEntry, VersionVector};
use crate::memory::synced::SyncedMemory;
use crate::sync::protocol::{BroadcastMessage, FullSyncManifest, FullSyncPlan, OrderBuffer};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// Timeout for full sync peer response (60 seconds as per patent spec).
const FULL_SYNC_TIMEOUT_SECS: u64 = 60;

// ── Full Sync State Machine ─────────────────────────────────────

/// Tracks the state of an outbound Layer 3 full sync operation.
#[derive(Debug)]
pub enum FullSyncState {
    /// No full sync in progress.
    Idle,
    /// Waiting for the peer's manifest response.
    WaitingForManifest {
        started_at: Instant,
        peer_device_id: Option<String>,
    },
    /// Receiving data from the peer.
    Receiving {
        started_at: Instant,
        peer_device_id: String,
        expected_count: Option<u64>,
        received_count: u64,
        last_activity: Instant,
    },
    /// The full sync completed successfully.
    Completed {
        peer_device_id: String,
        total_received: u64,
    },
    /// The peer did not respond within the timeout window.
    TimedOut { peer_device_id: Option<String> },
}

/// Progress snapshot for UI display during full sync.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FullSyncProgress {
    /// Current phase of the full sync.
    pub phase: FullSyncPhase,
    /// Peer device ID (if known).
    pub peer_device_id: Option<String>,
    /// Number of entities received so far.
    pub received_count: u64,
    /// Total expected entities (if known from peer's manifest).
    pub expected_count: Option<u64>,
    /// Seconds elapsed since the operation started.
    pub elapsed_secs: u64,
}

/// Phases of a full sync operation (for progress reporting).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum FullSyncPhase {
    /// Comparing manifests with the peer.
    Comparing,
    /// Receiving entities from the peer.
    Receiving,
    /// Sending our entities to the peer.
    Sending,
    /// Full sync completed successfully.
    Completed,
    /// Peer did not respond in time.
    TimedOut,
    /// No full sync in progress.
    Idle,
}

/// Orchestrates multi-device sync for a single device.
///
/// Holds references to the synced memory and sync engine, manages an
/// [`OrderBuffer`] for Layer 2 sequence guarantees, and provides
/// methods for both inbound message handling and outbound sync initiation.
pub struct SyncCoordinator {
    /// The synced memory backend (decorator around inner Memory).
    memory: Arc<SyncedMemory>,
    /// Order buffer for Layer 2 delta sequencing.
    order_buffer: Mutex<OrderBuffer>,
    /// Batch size for sync responses.
    batch_size: usize,
    /// Layer 3 full sync state machine.
    full_sync_state: Mutex<FullSyncState>,
}

impl SyncCoordinator {
    /// Create a new coordinator for the given synced memory.
    pub fn new(memory: Arc<SyncedMemory>, batch_size: usize) -> Self {
        let mut order_buffer = OrderBuffer::new();
        order_buffer.init_from_version_vector(&memory.version());

        Self {
            memory,
            order_buffer: Mutex::new(order_buffer),
            batch_size,
            full_sync_state: Mutex::new(FullSyncState::Idle),
        }
    }

    /// This device's unique ID.
    pub fn device_id(&self) -> String {
        self.memory.device_id()
    }

    /// Current local version vector.
    pub fn version(&self) -> VersionVector {
        self.memory.version()
    }

    // ── Inbound Message Dispatch ────────────────────────────────

    /// Handle an incoming WebSocket message (JSON string).
    ///
    /// Returns zero or more response messages to broadcast back.
    pub async fn handle_message(&self, json: &str) -> Vec<String> {
        let msg: BroadcastMessage = match serde_json::from_str(json) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("SyncCoordinator: failed to parse message: {e}");
                return Vec::new();
            }
        };

        // Ignore messages from ourselves.
        let from_device = match &msg {
            BroadcastMessage::RelayNotify { from_device_id, .. }
            | BroadcastMessage::SyncRequest { from_device_id, .. }
            | BroadcastMessage::SyncResponse { from_device_id, .. }
            | BroadcastMessage::DeltaAck { from_device_id, .. }
            | BroadcastMessage::FullSyncRequest { from_device_id, .. }
            | BroadcastMessage::FullSyncManifestResponse { from_device_id, .. }
            | BroadcastMessage::FullSyncData { from_device_id, .. }
            | BroadcastMessage::FullSyncComplete { from_device_id, .. } => from_device_id.clone(),
        };

        if from_device == self.device_id() {
            return Vec::new();
        }

        match msg {
            BroadcastMessage::SyncRequest {
                from_device_id,
                version_vector,
            } => self.handle_sync_request(&from_device_id, &version_vector),

            BroadcastMessage::SyncResponse {
                from_device_id,
                deltas,
                has_more: _,
            } => self.handle_sync_response(&from_device_id, deltas).await,

            BroadcastMessage::DeltaAck { .. } => {
                // Acknowledgement — currently no-op (future: track peer state)
                tracing::debug!(from = %from_device, "Received delta ack");
                Vec::new()
            }

            BroadcastMessage::FullSyncRequest {
                from_device_id,
                manifest,
            } => {
                self.handle_full_sync_request(&from_device_id, &manifest)
                    .await
            }

            BroadcastMessage::FullSyncManifestResponse {
                from_device_id,
                manifest,
            } => {
                self.handle_full_sync_manifest_response(&from_device_id, &manifest)
                    .await
            }

            BroadcastMessage::FullSyncData {
                from_device_id,
                entity_type,
                entity_id,
                encrypted_payload,
                iv,
                auth_tag,
            } => {
                self.handle_full_sync_data(
                    &from_device_id,
                    &entity_type,
                    &entity_id,
                    &encrypted_payload,
                    &iv,
                    &auth_tag,
                )
                .await
            }

            BroadcastMessage::FullSyncComplete {
                from_device_id,
                sent_count,
            } => {
                self.handle_full_sync_complete(&from_device_id, sent_count);
                Vec::new()
            }

            BroadcastMessage::RelayNotify { .. } => {
                // Layer 1 relay notification — handled by HTTP relay pickup
                tracing::debug!(from = %from_device, "Received relay notify");
                Vec::new()
            }
        }
    }

    // ── Layer 2: Delta Journal Sync ─────────────────────────────

    /// Handle an incoming SyncRequest: respond with deltas the peer hasn't seen.
    fn handle_sync_request(
        &self,
        from_device_id: &str,
        remote_version: &VersionVector,
    ) -> Vec<String> {
        tracing::info!(
            from = %from_device_id,
            "Layer 2: Received sync request, computing missing deltas"
        );

        let engine = self.memory.sync_engine().lock();
        let all_deltas: Vec<DeltaEntry> = engine
            .get_deltas_since(remote_version)
            .into_iter()
            .cloned()
            .collect();
        drop(engine);

        if all_deltas.is_empty() {
            tracing::debug!(from = %from_device_id, "No missing deltas for peer");
            return Vec::new();
        }

        // Send in batches
        let mut responses = Vec::new();
        let chunks: Vec<&[DeltaEntry]> = all_deltas.chunks(self.batch_size).collect();
        let total_chunks = chunks.len();

        for (i, chunk) in chunks.into_iter().enumerate() {
            let has_more = i + 1 < total_chunks;
            let msg = BroadcastMessage::SyncResponse {
                from_device_id: self.device_id(),
                deltas: chunk.to_vec(),
                has_more,
            };
            if let Ok(json) = serde_json::to_string(&msg) {
                responses.push(json);
            }
        }

        tracing::info!(
            from = %from_device_id,
            deltas = all_deltas.len(),
            batches = responses.len(),
            "Layer 2: Sending sync response"
        );

        responses
    }

    /// Handle an incoming SyncResponse: apply deltas in order.
    async fn handle_sync_response(
        &self,
        from_device_id: &str,
        deltas: Vec<DeltaEntry>,
    ) -> Vec<String> {
        tracing::info!(
            from = %from_device_id,
            incoming = deltas.len(),
            "Layer 2: Received sync response, applying deltas"
        );

        // Feed through the order buffer for sequencing
        let mut ready_deltas = Vec::new();
        {
            let mut buffer = self.order_buffer.lock();
            for delta in deltas {
                let batch = buffer.insert(delta);
                ready_deltas.extend(batch);
            }
        }

        if ready_deltas.is_empty() {
            tracing::debug!(from = %from_device_id, "No ready deltas after ordering");
            return Vec::new();
        }

        let applied = self.memory.apply_remote_deltas(ready_deltas).await;

        // Send ack
        let last_seq = self.version().get(from_device_id);
        let ack = BroadcastMessage::DeltaAck {
            from_device_id: self.device_id(),
            source_device_id: from_device_id.to_string(),
            last_seq,
        };

        tracing::info!(
            from = %from_device_id,
            applied,
            ack_seq = last_seq,
            "Layer 2: Applied deltas, sending ack"
        );

        match serde_json::to_string(&ack) {
            Ok(json) => vec![json],
            Err(_) => Vec::new(),
        }
    }

    // ── Layer 3: Full Sync ──────────────────────────────────────

    /// Handle an incoming FullSyncRequest: compare manifests, respond with ours.
    async fn handle_full_sync_request(
        &self,
        from_device_id: &str,
        remote_manifest: &FullSyncManifest,
    ) -> Vec<String> {
        tracing::info!(
            from = %from_device_id,
            remote_entries = remote_manifest.total_count(),
            "Layer 3: Received full sync request"
        );

        let local_manifest = match self.memory.build_manifest().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Layer 3: Failed to build manifest: {e}");
                return Vec::new();
            }
        };

        let plan = FullSyncPlan::compute(&local_manifest, remote_manifest);

        tracing::info!(
            we_need = plan.we_need.total_count(),
            they_need = plan.they_need.total_count(),
            "Layer 3: Computed full sync plan"
        );

        let mut responses = Vec::new();

        // 1. Send our manifest so the requester knows what we have
        let manifest_response = BroadcastMessage::FullSyncManifestResponse {
            from_device_id: self.device_id(),
            manifest: local_manifest,
        };
        if let Ok(json) = serde_json::to_string(&manifest_response) {
            responses.push(json);
        }

        // 2. Send entries that the requester is missing
        let missing_keys = &plan.they_need.memory_chunk_ids;
        if !missing_keys.is_empty() {
            match self.memory.export_missing_entries(missing_keys).await {
                Ok(entries) => {
                    let sent_count = entries.len() as u64;
                    for entry in entries {
                        let data_msg = BroadcastMessage::FullSyncData {
                            from_device_id: self.device_id(),
                            entity_type: entry.entity_type,
                            entity_id: entry.entity_id,
                            encrypted_payload: entry.encrypted_payload,
                            iv: entry.iv,
                            auth_tag: entry.auth_tag,
                        };
                        if let Ok(json) = serde_json::to_string(&data_msg) {
                            responses.push(json);
                        }
                    }
                    let complete_msg = BroadcastMessage::FullSyncComplete {
                        from_device_id: self.device_id(),
                        sent_count,
                    };
                    if let Ok(json) = serde_json::to_string(&complete_msg) {
                        responses.push(json);
                    }

                    tracing::info!(
                        to = %from_device_id,
                        sent = sent_count,
                        "Layer 3: Sent missing entries to peer"
                    );
                }
                Err(e) => {
                    tracing::warn!("Layer 3: Failed to export entries: {e}");
                }
            }
        }

        responses
    }

    /// Handle FullSyncManifestResponse: compute plan and request what we need.
    ///
    /// If the peer has data we are missing, we already received (or will receive)
    /// FullSyncData messages.  This handler can optionally send back our own
    /// missing entries to the peer.
    async fn handle_full_sync_manifest_response(
        &self,
        from_device_id: &str,
        remote_manifest: &FullSyncManifest,
    ) -> Vec<String> {
        tracing::info!(
            from = %from_device_id,
            remote_entries = remote_manifest.total_count(),
            "Layer 3: Received manifest response"
        );

        let local_manifest = match self.memory.build_manifest().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Layer 3: Failed to build manifest: {e}");
                return Vec::new();
            }
        };

        let plan = FullSyncPlan::compute(&local_manifest, remote_manifest);

        let we_need_count = plan.we_need.total_count() as u64;
        let they_need_count = plan.they_need.total_count();

        tracing::info!(
            we_need = we_need_count,
            they_need = they_need_count,
            "Layer 3: Manifest comparison complete"
        );

        // Transition state: we now know how many entities to expect.
        {
            let mut state = self.full_sync_state.lock();
            if matches!(&*state, FullSyncState::WaitingForManifest { .. }) {
                let now = Instant::now();
                *state = FullSyncState::Receiving {
                    started_at: now,
                    peer_device_id: from_device_id.to_string(),
                    expected_count: if we_need_count > 0 {
                        Some(we_need_count)
                    } else {
                        Some(0)
                    },
                    received_count: 0,
                    last_activity: now,
                };
            }
        }

        // If we need nothing, transition straight to completed.
        if we_need_count == 0 && they_need_count == 0 {
            let mut state = self.full_sync_state.lock();
            *state = FullSyncState::Completed {
                peer_device_id: from_device_id.to_string(),
                total_received: 0,
            };
            tracing::info!(
                from = %from_device_id,
                "Layer 3: Both devices already in sync"
            );
        }

        // Send entries the peer is missing from us
        let mut responses = Vec::new();
        let missing_keys = &plan.they_need.memory_chunk_ids;
        if !missing_keys.is_empty() {
            if let Ok(entries) = self.memory.export_missing_entries(missing_keys).await {
                let sent_count = entries.len() as u64;
                for entry in entries {
                    let data_msg = BroadcastMessage::FullSyncData {
                        from_device_id: self.device_id(),
                        entity_type: entry.entity_type,
                        entity_id: entry.entity_id,
                        encrypted_payload: entry.encrypted_payload,
                        iv: entry.iv,
                        auth_tag: entry.auth_tag,
                    };
                    if let Ok(json) = serde_json::to_string(&data_msg) {
                        responses.push(json);
                    }
                }
                let complete = BroadcastMessage::FullSyncComplete {
                    from_device_id: self.device_id(),
                    sent_count,
                };
                if let Ok(json) = serde_json::to_string(&complete) {
                    responses.push(json);
                }
                tracing::info!(
                    sent = sent_count,
                    "Layer 3: Sent our missing entries to peer"
                );
            }
        }

        responses
    }

    /// Handle FullSyncData: decrypt and import a single entity.
    async fn handle_full_sync_data(
        &self,
        from_device_id: &str,
        entity_type: &str,
        entity_id: &str,
        encrypted_payload: &str,
        iv: &str,
        _auth_tag: &str,
    ) -> Vec<String> {
        tracing::debug!(
            from = %from_device_id,
            entity_type,
            entity_id,
            "Layer 3: Received full sync data"
        );

        // Update progress tracking
        {
            let mut state = self.full_sync_state.lock();
            if let FullSyncState::Receiving {
                received_count,
                last_activity,
                ..
            } = &mut *state
            {
                *received_count += 1;
                *last_activity = Instant::now();
            }
        }

        // Reconstruct the SyncPayload and decrypt
        let payload = crate::memory::sync::SyncPayload {
            nonce: iv.to_string(),
            ciphertext: encrypted_payload.to_string(),
            sender: from_device_id.to_string(),
            version: VersionVector::default(),
        };

        match self.memory.decrypt_payload(&payload) {
            Ok(deltas) => {
                let count = deltas.len();
                let applied = self.memory.apply_remote_deltas(deltas).await;
                tracing::info!(
                    from = %from_device_id,
                    entity_id,
                    applied,
                    total = count,
                    "Layer 3: Imported full sync entity"
                );
            }
            Err(e) => {
                tracing::warn!(
                    from = %from_device_id,
                    entity_id,
                    "Layer 3: Failed to decrypt full sync data: {e}"
                );
            }
        }

        Vec::new()
    }

    /// Handle FullSyncComplete: transition state to completed.
    fn handle_full_sync_complete(&self, from_device_id: &str, sent_count: u64) {
        let mut state = self.full_sync_state.lock();
        let received = match &*state {
            FullSyncState::Receiving { received_count, .. } => *received_count,
            _ => 0,
        };
        *state = FullSyncState::Completed {
            peer_device_id: from_device_id.to_string(),
            total_received: received,
        };
        tracing::info!(
            from = %from_device_id,
            sent_count,
            received,
            "Layer 3: Full sync transfer complete from peer"
        );
    }

    // ── Outbound Sync Triggers ──────────────────────────────────

    /// Initiate a Layer 2 delta catch-up: build a SyncRequest message.
    ///
    /// Call this when coming online or after reconnecting.
    pub fn build_sync_request(&self) -> String {
        let msg = BroadcastMessage::SyncRequest {
            from_device_id: self.device_id(),
            version_vector: self.version(),
        };
        tracing::info!("Layer 2: Initiating delta sync request");
        serde_json::to_string(&msg).unwrap_or_default()
    }

    /// Initiate a Layer 3 full sync: build a FullSyncRequest message.
    ///
    /// Call this for long-offline recovery or user-initiated full sync.
    /// Sets the internal state to `WaitingForManifest` and starts the
    /// timeout clock.
    pub async fn build_full_sync_request(&self) -> Option<String> {
        let manifest = match self.memory.build_manifest().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Failed to build manifest for full sync request: {e}");
                return None;
            }
        };

        tracing::info!(
            entries = manifest.total_count(),
            "Layer 3: Initiating full sync request"
        );

        // Set state to waiting — timeout clock starts now.
        {
            let mut state = self.full_sync_state.lock();
            *state = FullSyncState::WaitingForManifest {
                started_at: Instant::now(),
                peer_device_id: None,
            };
        }

        let msg = BroadcastMessage::FullSyncRequest {
            from_device_id: self.device_id(),
            manifest,
        };
        serde_json::to_string(&msg).ok()
    }

    // ── Full Sync Timeout & Progress ────────────────────────────

    /// Check if the current full sync has timed out.
    ///
    /// Call this periodically (e.g., from a timer task). If it returns
    /// `true`, the peer is considered offline and the full sync should
    /// be abandoned.
    pub fn check_full_sync_timeout(&self) -> bool {
        let mut state = self.full_sync_state.lock();
        let timed_out = match &*state {
            FullSyncState::WaitingForManifest {
                started_at,
                peer_device_id,
            } => {
                if started_at.elapsed().as_secs() >= FULL_SYNC_TIMEOUT_SECS {
                    let peer = peer_device_id.clone();
                    tracing::warn!(
                        elapsed_secs = started_at.elapsed().as_secs(),
                        timeout_secs = FULL_SYNC_TIMEOUT_SECS,
                        "Layer 3: Full sync timed out waiting for manifest response"
                    );
                    *state = FullSyncState::TimedOut {
                        peer_device_id: peer,
                    };
                    true
                } else {
                    false
                }
            }
            FullSyncState::Receiving {
                last_activity,
                peer_device_id,
                ..
            } => {
                if last_activity.elapsed().as_secs() >= FULL_SYNC_TIMEOUT_SECS {
                    let peer = peer_device_id.clone();
                    tracing::warn!(
                        since_last_activity_secs = last_activity.elapsed().as_secs(),
                        timeout_secs = FULL_SYNC_TIMEOUT_SECS,
                        peer = %peer,
                        "Layer 3: Full sync timed out waiting for data"
                    );
                    *state = FullSyncState::TimedOut {
                        peer_device_id: Some(peer),
                    };
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        timed_out
    }

    /// Get the current full sync progress for UI display.
    pub fn full_sync_progress(&self) -> FullSyncProgress {
        let state = self.full_sync_state.lock();
        match &*state {
            FullSyncState::Idle => FullSyncProgress {
                phase: FullSyncPhase::Idle,
                peer_device_id: None,
                received_count: 0,
                expected_count: None,
                elapsed_secs: 0,
            },
            FullSyncState::WaitingForManifest {
                started_at,
                peer_device_id,
            } => FullSyncProgress {
                phase: FullSyncPhase::Comparing,
                peer_device_id: peer_device_id.clone(),
                received_count: 0,
                expected_count: None,
                elapsed_secs: started_at.elapsed().as_secs(),
            },
            FullSyncState::Receiving {
                started_at,
                peer_device_id,
                expected_count,
                received_count,
                ..
            } => FullSyncProgress {
                phase: FullSyncPhase::Receiving,
                peer_device_id: Some(peer_device_id.clone()),
                received_count: *received_count,
                expected_count: *expected_count,
                elapsed_secs: started_at.elapsed().as_secs(),
            },
            FullSyncState::Completed {
                peer_device_id,
                total_received,
            } => FullSyncProgress {
                phase: FullSyncPhase::Completed,
                peer_device_id: Some(peer_device_id.clone()),
                received_count: *total_received,
                expected_count: Some(*total_received),
                elapsed_secs: 0,
            },
            FullSyncState::TimedOut { peer_device_id } => FullSyncProgress {
                phase: FullSyncPhase::TimedOut,
                peer_device_id: peer_device_id.clone(),
                received_count: 0,
                expected_count: None,
                elapsed_secs: FULL_SYNC_TIMEOUT_SECS,
            },
        }
    }

    /// Reset the full sync state to Idle (e.g., after user acknowledges
    /// a timeout or completion).
    pub fn reset_full_sync_state(&self) {
        let mut state = self.full_sync_state.lock();
        *state = FullSyncState::Idle;
    }

    /// Periodic maintenance: prune old journal entries.
    pub fn prune_journal(&self) {
        self.memory.prune_journal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::sync::{DeltaOperation, SyncEngine};
    use crate::memory::traits::MemoryCategory;
    use crate::memory::Memory;
    use crate::memory::SqliteMemory;
    use tempfile::TempDir;

    fn make_coordinator(tmp: &TempDir) -> Arc<SyncCoordinator> {
        let mem: Arc<dyn Memory> = Arc::new(SqliteMemory::new(tmp.path()).unwrap());
        let engine = Arc::new(Mutex::new(SyncEngine::new(tmp.path(), true, None).unwrap()));
        let synced = Arc::new(SyncedMemory::new(mem, engine));
        Arc::new(SyncCoordinator::new(synced, 50))
    }

    #[tokio::test]
    async fn build_sync_request_produces_valid_json() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        let json = coord.build_sync_request();
        let msg: BroadcastMessage = serde_json::from_str(&json).unwrap();

        match msg {
            BroadcastMessage::SyncRequest {
                from_device_id,
                version_vector,
            } => {
                assert_eq!(from_device_id, coord.device_id());
                let _ = &version_vector; // version vector may or may not have clocks
            }
            _ => panic!("Expected SyncRequest"),
        }
    }

    #[tokio::test]
    async fn build_full_sync_request_includes_manifest() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        // Store some data first
        coord
            .memory
            .store("fact_1", "value_1", MemoryCategory::Core, None)
            .await
            .unwrap();

        let json = coord.build_full_sync_request().await.unwrap();
        let msg: BroadcastMessage = serde_json::from_str(&json).unwrap();

        match msg {
            BroadcastMessage::FullSyncRequest {
                from_device_id,
                manifest,
            } => {
                assert_eq!(from_device_id, coord.device_id());
                assert!(manifest.memory_chunk_ids.contains("fact_1"));
            }
            _ => panic!("Expected FullSyncRequest"),
        }
    }

    #[tokio::test]
    async fn handle_sync_request_responds_with_deltas() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        // Store data to create deltas
        coord
            .memory
            .store("k1", "v1", MemoryCategory::Core, None)
            .await
            .unwrap();
        coord
            .memory
            .store("k2", "v2", MemoryCategory::Daily, None)
            .await
            .unwrap();

        // Simulate a peer requesting sync with empty version vector
        let request_msg = BroadcastMessage::SyncRequest {
            from_device_id: "peer_device".into(),
            version_vector: VersionVector::default(),
        };
        let json = serde_json::to_string(&request_msg).unwrap();

        let responses = coord.handle_message(&json).await;
        assert!(!responses.is_empty());

        // Parse first response — should be a SyncResponse
        let resp: BroadcastMessage = serde_json::from_str(&responses[0]).unwrap();
        match resp {
            BroadcastMessage::SyncResponse { deltas, .. } => {
                assert_eq!(deltas.len(), 2);
            }
            _ => panic!("Expected SyncResponse"),
        }
    }

    #[tokio::test]
    async fn handle_sync_response_applies_deltas() {
        let tmp_a = TempDir::new().unwrap();
        let tmp_b = TempDir::new().unwrap();
        let coord_b = make_coordinator(&tmp_b);

        // Create deltas from device A
        let engine_a = SyncEngine::new(tmp_a.path(), true, None).unwrap();
        let device_a_id = engine_a.device_id().0.clone();
        drop(engine_a);

        let mut vv = VersionVector::default();
        vv.increment(&device_a_id);

        let response_msg = BroadcastMessage::SyncResponse {
            from_device_id: device_a_id.clone(),
            deltas: vec![DeltaEntry {
                id: "d1".into(),
                device_id: device_a_id.clone(),
                version: vv,
                operation: DeltaOperation::Store {
                    key: "synced_key".into(),
                    content: "synced_value".into(),
                    category: "core".into(),
                },
                timestamp: 9999,
            }],
            has_more: false,
        };
        let json = serde_json::to_string(&response_msg).unwrap();

        let ack_responses = coord_b.handle_message(&json).await;

        // Should get a DeltaAck back
        assert!(!ack_responses.is_empty());
        let ack: BroadcastMessage = serde_json::from_str(&ack_responses[0]).unwrap();
        assert!(matches!(ack, BroadcastMessage::DeltaAck { .. }));

        // Data should be in device B's memory
        let entry = coord_b.memory.get("synced_key").await.unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().content, "synced_value");
    }

    #[tokio::test]
    async fn full_sync_request_response_cycle() {
        let tmp_a = TempDir::new().unwrap();
        let tmp_b = TempDir::new().unwrap();
        let coord_a = make_coordinator(&tmp_a);
        let coord_b = make_coordinator(&tmp_b);

        // Device A has data
        coord_a
            .memory
            .store("only_on_a", "secret_a", MemoryCategory::Core, None)
            .await
            .unwrap();

        // Device B has different data
        coord_b
            .memory
            .store("only_on_b", "secret_b", MemoryCategory::Core, None)
            .await
            .unwrap();

        // Device A initiates full sync
        let request_json = coord_a.build_full_sync_request().await.unwrap();

        // Device B handles the request
        let b_responses = coord_b.handle_message(&request_json).await;

        // B should respond with:
        // 1. FullSyncManifestResponse
        // 2. FullSyncData (for "only_on_b" which A doesn't have)
        // 3. FullSyncComplete
        assert!(
            b_responses.len() >= 2,
            "Expected at least manifest + complete, got {}",
            b_responses.len()
        );

        // Verify manifest response
        let first: BroadcastMessage = serde_json::from_str(&b_responses[0]).unwrap();
        assert!(
            matches!(first, BroadcastMessage::FullSyncManifestResponse { .. }),
            "First message should be FullSyncManifestResponse"
        );
    }

    #[tokio::test]
    async fn ignore_own_messages() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        // Send a message from ourselves
        let msg = BroadcastMessage::SyncRequest {
            from_device_id: coord.device_id(),
            version_vector: VersionVector::default(),
        };
        let json = serde_json::to_string(&msg).unwrap();

        let responses = coord.handle_message(&json).await;
        assert!(responses.is_empty(), "Should ignore own messages");
    }

    #[tokio::test]
    async fn handle_malformed_json() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        let responses = coord.handle_message("not valid json {{{").await;
        assert!(responses.is_empty());
    }

    #[tokio::test]
    async fn handle_full_sync_complete_logs() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        let msg = BroadcastMessage::FullSyncComplete {
            from_device_id: "peer_device".into(),
            sent_count: 42,
        };
        let json = serde_json::to_string(&msg).unwrap();

        let responses = coord.handle_message(&json).await;
        assert!(responses.is_empty()); // Complete is just logged
    }

    // ── Full Sync Timeout Tests ─────────────────────────────────

    #[tokio::test]
    async fn full_sync_request_sets_waiting_state() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        let progress = coord.full_sync_progress();
        assert_eq!(progress.phase, FullSyncPhase::Idle);

        let _json = coord.build_full_sync_request().await.unwrap();

        let progress = coord.full_sync_progress();
        assert_eq!(progress.phase, FullSyncPhase::Comparing);
    }

    #[tokio::test]
    async fn timeout_not_triggered_when_idle() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        assert!(!coord.check_full_sync_timeout());
    }

    #[tokio::test]
    async fn timeout_not_triggered_before_deadline() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        let _json = coord.build_full_sync_request().await.unwrap();

        // Immediately check — should not be timed out (only ~0 seconds elapsed)
        assert!(!coord.check_full_sync_timeout());
        assert_eq!(coord.full_sync_progress().phase, FullSyncPhase::Comparing);
    }

    #[tokio::test]
    async fn manifest_response_transitions_to_receiving() {
        let tmp_a = TempDir::new().unwrap();
        let tmp_b = TempDir::new().unwrap();
        let coord_a = make_coordinator(&tmp_a);
        let coord_b = make_coordinator(&tmp_b);

        coord_b
            .memory
            .store("data_b", "val_b", MemoryCategory::Core, None)
            .await
            .unwrap();

        // Initiate full sync from A
        let request_json = coord_a.build_full_sync_request().await.unwrap();
        assert_eq!(coord_a.full_sync_progress().phase, FullSyncPhase::Comparing);

        // B handles request and sends responses
        let b_responses = coord_b.handle_message(&request_json).await;
        assert!(!b_responses.is_empty());

        // A handles B's manifest response
        coord_a.handle_message(&b_responses[0]).await;

        let progress = coord_a.full_sync_progress();
        assert_eq!(progress.phase, FullSyncPhase::Receiving);
        assert!(progress.expected_count.is_some());
    }

    #[tokio::test]
    async fn full_sync_complete_transitions_to_completed() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        // Manually set to receiving state
        {
            let mut state = coord.full_sync_state.lock();
            *state = FullSyncState::Receiving {
                started_at: Instant::now(),
                peer_device_id: "peer".into(),
                expected_count: Some(5),
                received_count: 5,
                last_activity: Instant::now(),
            };
        }

        let msg = BroadcastMessage::FullSyncComplete {
            from_device_id: "peer".into(),
            sent_count: 5,
        };
        let json = serde_json::to_string(&msg).unwrap();
        coord.handle_message(&json).await;

        let progress = coord.full_sync_progress();
        assert_eq!(progress.phase, FullSyncPhase::Completed);
        assert_eq!(progress.received_count, 5);
    }

    #[tokio::test]
    async fn reset_returns_to_idle() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        let _json = coord.build_full_sync_request().await.unwrap();
        assert_eq!(coord.full_sync_progress().phase, FullSyncPhase::Comparing);

        coord.reset_full_sync_state();
        assert_eq!(coord.full_sync_progress().phase, FullSyncPhase::Idle);
    }

    #[tokio::test]
    async fn timeout_triggered_for_expired_waiting() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        // Set waiting state with an already-expired start time
        {
            let mut state = coord.full_sync_state.lock();
            *state = FullSyncState::WaitingForManifest {
                started_at: Instant::now()
                    .checked_sub(std::time::Duration::from_secs(61))
                    .unwrap(),
                peer_device_id: None,
            };
        }

        assert!(coord.check_full_sync_timeout());
        assert_eq!(coord.full_sync_progress().phase, FullSyncPhase::TimedOut);
    }

    #[tokio::test]
    async fn timeout_triggered_for_stale_receiving() {
        let tmp = TempDir::new().unwrap();
        let coord = make_coordinator(&tmp);

        // Set receiving state with stale last_activity
        {
            let mut state = coord.full_sync_state.lock();
            *state = FullSyncState::Receiving {
                started_at: Instant::now()
                    .checked_sub(std::time::Duration::from_secs(120))
                    .unwrap(),
                peer_device_id: "peer_dev".into(),
                expected_count: Some(10),
                received_count: 3,
                last_activity: Instant::now()
                    .checked_sub(std::time::Duration::from_secs(65))
                    .unwrap(),
            };
        }

        assert!(coord.check_full_sync_timeout());
        let progress = coord.full_sync_progress();
        assert_eq!(progress.phase, FullSyncPhase::TimedOut);
        assert_eq!(progress.peer_device_id.as_deref(), Some("peer_dev"));
    }
}
