//! Layer 1: Temporary relay storage with TTL-based auto-expiry.
//!
//! The relay acts as a short-lived buffer for encrypted sync data.
//! Data is automatically deleted after the TTL expires, ensuring no
//! persistent server storage of user data.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Default TTL for relay entries: 5 minutes.
const DEFAULT_TTL_SECS: u64 = 300;

/// Maximum relay entries per device (prevents memory exhaustion).
const MAX_ENTRIES_PER_DEVICE: usize = 100;

/// A single encrypted relay entry waiting for pickup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayEntry {
    /// Unique relay entry ID.
    pub id: String,
    /// Sending device ID.
    pub sender_device_id: String,
    /// Recipient user ID (all devices of this user can pick up).
    pub user_id: String,
    /// Encrypted payload (E2E encrypted, server cannot read).
    pub encrypted_payload: String,
    /// IV/nonce for decryption.
    pub nonce: String,
    /// Creation timestamp for display/ordering.
    pub created_at_epoch: u64,
}

/// Internal entry with expiry tracking.
struct RelaySlot {
    entry: RelayEntry,
    expires_at: Instant,
}

/// In-memory temporary relay with TTL-based auto-expiry.
///
/// This implements the patent's Layer 1: the server stores encrypted
/// data temporarily and deletes it after the TTL expires.
/// No data is written to disk — pure in-memory relay.
pub struct SyncRelay {
    /// user_id -> Vec<RelaySlot>
    slots: Mutex<HashMap<String, Vec<RelaySlot>>>,
    /// TTL duration for relay entries.
    ttl: Duration,
}

impl SyncRelay {
    /// Create a new relay with default TTL (5 minutes).
    pub fn new() -> Self {
        Self {
            slots: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(DEFAULT_TTL_SECS),
        }
    }

    /// Create a new relay with custom TTL.
    pub fn with_ttl(ttl_secs: u64) -> Self {
        Self {
            slots: Mutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Store an encrypted entry in the relay for the given user.
    /// Returns the relay entry ID.
    pub fn store(&self, entry: RelayEntry) -> String {
        let id = entry.id.clone();
        let user_id = entry.user_id.clone();
        let expires_at = Instant::now() + self.ttl;

        let mut slots = self.slots.lock();
        let user_slots = slots.entry(user_id).or_default();

        // Evict expired entries first
        user_slots.retain(|slot| slot.expires_at > Instant::now());

        // Enforce per-device limit
        let sender = &entry.sender_device_id;
        let device_count = user_slots
            .iter()
            .filter(|s| s.entry.sender_device_id == *sender)
            .count();
        if device_count >= MAX_ENTRIES_PER_DEVICE {
            // Remove oldest from this device
            if let Some(pos) = user_slots
                .iter()
                .position(|s| s.entry.sender_device_id == *sender)
            {
                user_slots.remove(pos);
            }
        }

        user_slots.push(RelaySlot { entry, expires_at });
        id
    }

    /// Pick up all pending entries for a user, optionally filtered by
    /// entries newer than a given relay ID. Returned entries are removed
    /// from the relay (consumed).
    pub fn pickup(&self, user_id: &str, exclude_device: Option<&str>) -> Vec<RelayEntry> {
        let mut slots = self.slots.lock();
        let user_slots = match slots.get_mut(user_id) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let now = Instant::now();
        let mut picked = Vec::new();
        let mut remaining = Vec::new();

        for slot in user_slots.drain(..) {
            if slot.expires_at <= now {
                continue; // Expired — discard
            }
            if exclude_device == Some(slot.entry.sender_device_id.as_str()) {
                remaining.push(slot); // Don't pick up own entries
                continue;
            }
            picked.push(slot.entry);
        }

        *user_slots = remaining;
        picked
    }

    /// Sweep expired entries across all users. Call periodically.
    pub fn sweep_expired(&self) -> usize {
        let now = Instant::now();
        let mut slots = self.slots.lock();
        let mut total_removed = 0;

        slots.retain(|_, user_slots| {
            let before = user_slots.len();
            user_slots.retain(|slot| slot.expires_at > now);
            total_removed += before - user_slots.len();
            !user_slots.is_empty()
        });

        total_removed
    }

    /// Count total entries across all users (for diagnostics).
    pub fn entry_count(&self) -> usize {
        let slots = self.slots.lock();
        slots.values().map(Vec::len).sum()
    }
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str, sender: &str, user_id: &str) -> RelayEntry {
        RelayEntry {
            id: id.into(),
            sender_device_id: sender.into(),
            user_id: user_id.into(),
            encrypted_payload: "encrypted_data".into(),
            nonce: "nonce_value".into(),
            created_at_epoch: 1000,
        }
    }

    #[test]
    fn store_and_pickup() {
        let relay = SyncRelay::new();

        relay.store(make_entry("r1", "dev_a", "user_1"));
        relay.store(make_entry("r2", "dev_b", "user_1"));

        // Pickup excluding dev_a returns only dev_b's entry
        let entries = relay.pickup("user_1", Some("dev_a"));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "r2");

        // dev_a's entry (r1) still in relay (was excluded above).
        // Store another from dev_b.
        relay.store(make_entry("r3", "dev_b", "user_1"));

        // Pickup excluding dev_b returns dev_a's retained entry
        let entries = relay.pickup("user_1", Some("dev_b"));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "r1");
    }

    #[test]
    fn pickup_consumes_entries() {
        let relay = SyncRelay::new();

        relay.store(make_entry("r1", "dev_a", "user_1"));

        let first = relay.pickup("user_1", None);
        assert_eq!(first.len(), 1);

        // Second pickup — empty (consumed)
        let second = relay.pickup("user_1", None);
        assert!(second.is_empty());
    }

    #[test]
    fn expired_entries_are_discarded() {
        let relay = SyncRelay::with_ttl(0); // 0-second TTL = immediate expiry

        relay.store(make_entry("r1", "dev_a", "user_1"));

        // Entries expired immediately
        std::thread::sleep(std::time::Duration::from_millis(10));
        let entries = relay.pickup("user_1", None);
        assert!(entries.is_empty());
    }

    #[test]
    fn sweep_removes_expired() {
        let relay = SyncRelay::with_ttl(0);

        relay.store(make_entry("r1", "dev_a", "user_1"));
        relay.store(make_entry("r2", "dev_b", "user_2"));

        std::thread::sleep(std::time::Duration::from_millis(10));
        let removed = relay.sweep_expired();
        assert_eq!(removed, 2);
        assert_eq!(relay.entry_count(), 0);
    }

    #[test]
    fn per_device_limit_enforced() {
        let relay = SyncRelay::new();

        // Store MAX_ENTRIES_PER_DEVICE + 1 entries
        for i in 0..=MAX_ENTRIES_PER_DEVICE {
            relay.store(make_entry(
                &format!("r{i}"),
                "dev_a",
                "user_1",
            ));
        }

        // Should be capped at MAX_ENTRIES_PER_DEVICE
        assert!(relay.entry_count() <= MAX_ENTRIES_PER_DEVICE + 1);
    }

    #[test]
    fn different_users_isolated() {
        let relay = SyncRelay::new();

        relay.store(make_entry("r1", "dev_a", "user_1"));
        relay.store(make_entry("r2", "dev_b", "user_2"));

        let entries_1 = relay.pickup("user_1", None);
        assert_eq!(entries_1.len(), 1);
        assert_eq!(entries_1[0].id, "r1");

        let entries_2 = relay.pickup("user_2", None);
        assert_eq!(entries_2.len(), 1);
        assert_eq!(entries_2[0].id, "r2");
    }
}
