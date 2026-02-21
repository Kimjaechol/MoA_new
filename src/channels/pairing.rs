//! Channel Pairing Store — shared pairing code management for all channels.
//!
//! After a user authenticates via the gateway web flow (`/pair/{channel}`),
//! a 6-digit numeric code is generated and stored here.  The user then
//! types this code in their messaging app, and the channel validates it
//! against this store to complete the pairing.
//!
//! ## Storage
//! Uses SQLite for persistence, enabling code sharing between gateway and
//! channel processes (e.g., in daemon mode where both run as separate tasks).
//!
//! ## Security
//! - Codes expire after 5 minutes
//! - Maximum 20 active codes at any time (prevents flooding)
//! - Codes are single-use (consumed on redemption)
//! - Brute-force resistant: 6-digit numeric = 1M combinations

use parking_lot::Mutex;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// How long a pairing code remains valid (seconds).
const CODE_TTL_SECS: u64 = 300; // 5 minutes

/// Maximum number of active (unexpired) codes.
const MAX_ACTIVE_CODES: usize = 20;

/// A pending pairing entry, created after successful web authentication.
#[derive(Debug, Clone)]
pub struct PairingEntry {
    /// The 6-digit numeric code the user must type in their chat.
    pub code: String,
    /// Which channel this code is for ("kakao", "telegram", "whatsapp", etc.).
    pub channel: String,
    /// The platform-specific user identifier (Kakao user ID, Telegram user ID, phone number, etc.).
    pub platform_uid: String,
    /// The authenticated MoA user ID (from auth store).
    pub user_id: String,
    /// Unix timestamp when this code expires.
    pub expires_at: u64,
}

/// Thread-safe store for pending channel pairing codes.
/// Backed by SQLite for cross-process sharing (gateway + channels).
#[derive(Debug)]
pub struct ChannelPairingStore {
    conn: Mutex<rusqlite::Connection>,
}

impl ChannelPairingStore {
    /// Create an in-memory store (for tests).
    pub fn new() -> Self {
        let conn = rusqlite::Connection::open_in_memory()
            .expect("Failed to open in-memory SQLite for pairing store");
        Self::init_tables(&conn);
        Self {
            conn: Mutex::new(conn),
        }
    }

    /// Open a file-backed store for production use.
    /// Both gateway and channels should point to the same file.
    pub fn open(db_path: &Path) -> anyhow::Result<Self> {
        let conn = rusqlite::Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")?;
        Self::init_tables(&conn);
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_tables(conn: &rusqlite::Connection) {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS pairing_codes (
                code TEXT PRIMARY KEY,
                channel TEXT NOT NULL,
                platform_uid TEXT NOT NULL,
                user_id TEXT NOT NULL,
                expires_at INTEGER NOT NULL
            );",
        )
        .expect("Failed to initialize pairing_codes table");

        // Cleanup stale codes on startup
        let now = epoch_secs() as i64;
        let _ = conn.execute(
            "DELETE FROM pairing_codes WHERE expires_at <= ?1",
            rusqlite::params![now],
        );
    }

    /// Create a pairing code for a user who just authenticated via the web flow.
    /// Returns the 6-digit code to display on the success page.
    pub fn create_pairing(&self, channel: &str, platform_uid: &str, user_id: &str) -> String {
        let conn = self.conn.lock();
        let now = epoch_secs() as i64;

        // Cleanup expired entries
        let _ = conn.execute(
            "DELETE FROM pairing_codes WHERE expires_at <= ?1",
            rusqlite::params![now],
        );

        // Remove existing code for same channel+uid (prevent duplicates)
        let _ = conn.execute(
            "DELETE FROM pairing_codes WHERE channel = ?1 AND platform_uid = ?2",
            rusqlite::params![channel, platform_uid],
        );

        // Enforce max active codes — remove oldest if at limit
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM pairing_codes", [], |row| row.get(0))
            .unwrap_or(0);
        if count >= MAX_ACTIVE_CODES as i64 {
            let _ = conn.execute(
                "DELETE FROM pairing_codes WHERE code = \
                 (SELECT code FROM pairing_codes ORDER BY expires_at ASC LIMIT 1)",
                [],
            );
        }

        let code = generate_code_unique(&conn);
        let expires_at = now + CODE_TTL_SECS as i64;

        let _ = conn.execute(
            "INSERT INTO pairing_codes (code, channel, platform_uid, user_id, expires_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![code, channel, platform_uid, user_id, expires_at],
        );

        tracing::info!(
            channel = channel,
            platform_uid = platform_uid,
            "Channel pairing code created (expires in {}s)",
            CODE_TTL_SECS
        );

        code
    }

    /// Attempt to redeem a pairing code. Returns the entry if valid.
    /// The code is consumed (single-use) on success.
    pub fn redeem_code(&self, code: &str, channel: &str) -> Option<PairingEntry> {
        let conn = self.conn.lock();
        let now = epoch_secs() as i64;

        // Cleanup expired
        let _ = conn.execute(
            "DELETE FROM pairing_codes WHERE expires_at <= ?1",
            rusqlite::params![now],
        );

        // Look up
        let entry = conn
            .query_row(
                "SELECT code, channel, platform_uid, user_id, expires_at \
                 FROM pairing_codes WHERE code = ?1 AND channel = ?2",
                rusqlite::params![code, channel],
                |row| {
                    Ok(PairingEntry {
                        code: row.get(0)?,
                        channel: row.get(1)?,
                        platform_uid: row.get(2)?,
                        user_id: row.get(3)?,
                        expires_at: row.get::<_, i64>(4)? as u64,
                    })
                },
            )
            .ok()?;

        // Single-use: delete after reading
        let _ = conn.execute(
            "DELETE FROM pairing_codes WHERE code = ?1",
            rusqlite::params![code],
        );

        tracing::info!(
            channel = channel,
            platform_uid = entry.platform_uid,
            "Channel pairing code redeemed successfully"
        );

        Some(entry)
    }

    /// Check if a string looks like a pairing code (6 digits).
    pub fn looks_like_code(text: &str) -> bool {
        let trimmed = text.trim();
        trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit())
    }

    /// Get the number of active (non-expired) codes.
    pub fn active_count(&self) -> usize {
        let conn = self.conn.lock();
        let now = epoch_secs() as i64;
        conn.query_row(
            "SELECT COUNT(*) FROM pairing_codes WHERE expires_at > ?1",
            rusqlite::params![now],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as usize
    }

    /// Build the URL for the pairing web page.
    pub fn connect_url(gateway_base: &str, channel: &str, platform_uid: &str) -> String {
        format!("{gateway_base}/pair/connect/{channel}?uid={platform_uid}")
    }
}

impl Default for ChannelPairingStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a unique 6-digit numeric code not already in the DB.
fn generate_code_unique(conn: &rusqlite::Connection) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    for _ in 0..100 {
        let id = uuid::Uuid::new_v4();
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let hash = hasher.finish();
        let code = format!("{:06}", hash % 1_000_000);

        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pairing_codes WHERE code = ?1",
                rusqlite::params![code],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if !exists {
            return code;
        }
    }

    // Fallback (extremely unlikely)
    format!("{:06}", epoch_secs() % 1_000_000)
}

fn epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Persist a newly-paired identity to the channel's allowed-user list in config.toml.
/// This is a generic helper usable by all channels.
pub fn persist_channel_allowlist(
    channel_name: &str,
    identity: &str,
) -> anyhow::Result<()> {
    use crate::config::Config;
    use directories::UserDirs;

    let home = UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let zeroclaw_dir = home.join(".zeroclaw");
    let config_path = zeroclaw_dir.join("config.toml");

    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config: {e}"))?;
    let mut config: Config =
        toml::from_str(&contents).map_err(|e| anyhow::anyhow!("Failed to parse config: {e}"))?;
    config.config_path = config_path;
    config.workspace_dir = zeroclaw_dir.join("workspace");

    let identity = identity.trim().to_string();
    if identity.is_empty() {
        anyhow::bail!("Cannot persist empty identity");
    }

    let added = match channel_name {
        "kakao" => {
            if let Some(ref mut kakao) = config.channels_config.kakao {
                if !kakao.allowed_users.iter().any(|u| u == &identity) {
                    kakao.allowed_users.push(identity.clone());
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        "telegram" => {
            if let Some(ref mut tg) = config.channels_config.telegram {
                if !tg.allowed_users.iter().any(|u| u == &identity) {
                    tg.allowed_users.push(identity.clone());
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        "discord" => {
            if let Some(ref mut dc) = config.channels_config.discord {
                if !dc.allowed_users.iter().any(|u| u == &identity) {
                    dc.allowed_users.push(identity.clone());
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        "slack" => {
            if let Some(ref mut sl) = config.channels_config.slack {
                if !sl.allowed_users.iter().any(|u| u == &identity) {
                    sl.allowed_users.push(identity.clone());
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        "whatsapp" => {
            if let Some(ref mut wa) = config.channels_config.whatsapp {
                if !wa.allowed_numbers.iter().any(|n| n == &identity) {
                    wa.allowed_numbers.push(identity.clone());
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        "imessage" => {
            if let Some(ref mut im) = config.channels_config.imessage {
                if !im.allowed_contacts.iter().any(|c| c == &identity) {
                    im.allowed_contacts.push(identity.clone());
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => {
            tracing::warn!(
                "persist_channel_allowlist: unsupported channel '{channel_name}'"
            );
            return Ok(());
        }
    };

    if added {
        config
            .save()
            .map_err(|e| anyhow::anyhow!("Failed to save config: {e}"))?;
        tracing::info!(
            channel = channel_name,
            "Persisted paired identity to config.toml"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_redeem_code() {
        let store = ChannelPairingStore::new();
        let code = store.create_pairing("kakao", "user_123", "auth_abc");

        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));

        let entry = store.redeem_code(&code, "kakao").unwrap();
        assert_eq!(entry.platform_uid, "user_123");
        assert_eq!(entry.user_id, "auth_abc");
        assert_eq!(entry.channel, "kakao");

        // Code should be consumed
        assert!(store.redeem_code(&code, "kakao").is_none());
    }

    #[test]
    fn wrong_channel_rejects() {
        let store = ChannelPairingStore::new();
        let code = store.create_pairing("kakao", "user_123", "auth_abc");

        assert!(store.redeem_code(&code, "telegram").is_none());
        // Original should still be valid
        assert!(store.redeem_code(&code, "kakao").is_some());
    }

    #[test]
    fn duplicate_uid_replaces() {
        let store = ChannelPairingStore::new();
        let code1 = store.create_pairing("kakao", "user_123", "auth_abc");
        let code2 = store.create_pairing("kakao", "user_123", "auth_abc");

        assert_ne!(code1, code2);
        // Old code should be gone
        assert!(store.redeem_code(&code1, "kakao").is_none());
        // New code should work
        assert!(store.redeem_code(&code2, "kakao").is_some());
    }

    #[test]
    fn looks_like_code_valid() {
        assert!(ChannelPairingStore::looks_like_code("123456"));
        assert!(ChannelPairingStore::looks_like_code("000000"));
        assert!(ChannelPairingStore::looks_like_code(" 482901 "));
    }

    #[test]
    fn looks_like_code_invalid() {
        assert!(!ChannelPairingStore::looks_like_code("12345")); // too short
        assert!(!ChannelPairingStore::looks_like_code("1234567")); // too long
        assert!(!ChannelPairingStore::looks_like_code("abcdef")); // not digits
        assert!(!ChannelPairingStore::looks_like_code("hello world"));
        assert!(!ChannelPairingStore::looks_like_code(""));
    }

    #[test]
    fn active_count_tracks() {
        let store = ChannelPairingStore::new();
        assert_eq!(store.active_count(), 0);

        store.create_pairing("kakao", "u1", "a1");
        store.create_pairing("telegram", "u2", "a2");
        assert_eq!(store.active_count(), 2);
    }

    #[test]
    fn max_codes_evicts_oldest() {
        let store = ChannelPairingStore::new();
        for i in 0..MAX_ACTIVE_CODES + 5 {
            store.create_pairing("kakao", &format!("user_{i}"), &format!("auth_{i}"));
        }
        assert!(store.active_count() <= MAX_ACTIVE_CODES);
    }

    #[test]
    fn connect_url_format() {
        let url = ChannelPairingStore::connect_url("http://localhost:3000", "kakao", "uid_123");
        assert_eq!(url, "http://localhost:3000/pair/connect/kakao?uid=uid_123");
    }
}
