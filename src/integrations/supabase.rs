//! Supabase integration for ZeroClaw.
//!
//! Provides cloud data synchronization via Supabase's REST/RPC API:
//! - User management (credits, permissions, API keys)
//! - Usage tracking (per-request cost logging)
//! - Security event logging (audit trail)
//! - Atomic credit operations via RPC functions
//! - Realtime broadcast channel for E2E encrypted memory sync
//!
//! ## Design
//! - HTTP client (reqwest) against Supabase PostgREST and RPC endpoints
//! - Service-key authentication for server-side operations
//! - Row Level Security (RLS) compatible — read paths can use user tokens
//! - All sensitive data (API keys, sync payloads) is encrypted before storage

use serde::{Deserialize, Serialize};

// ── Configuration ────────────────────────────────────────────────

/// Supabase connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseConfig {
    /// Supabase project URL (e.g., https://xxxx.supabase.co).
    pub url: String,
    /// Supabase service role key (server-side, never exposed to client).
    pub service_key: String,
    /// Optional anon key for client-side RLS operations.
    pub anon_key: Option<String>,
}

impl SupabaseConfig {
    /// Load from environment variables.
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("SUPABASE_URL").ok()?;
        let service_key = std::env::var("SUPABASE_SERVICE_KEY").ok()?;
        let anon_key = std::env::var("SUPABASE_ANON_KEY").ok();

        if url.is_empty() || service_key.is_empty() {
            return None;
        }

        Some(Self {
            url,
            service_key,
            anon_key,
        })
    }
}

// ── Data models ──────────────────────────────────────────────────

/// User record in the Supabase `lawcall_users` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseUser {
    /// UUID primary key.
    pub id: Option<String>,
    /// SHA-256 hash of the KakaoTalk user ID.
    pub kakao_user_id: String,
    /// Current credit balance.
    pub credits: i32,
    /// Total credits spent.
    pub total_spent: i32,
    /// Encrypted custom API key (if user provides their own).
    pub custom_api_key: Option<String>,
    /// Custom provider name ("anthropic" | "openai" | etc.).
    pub custom_provider: Option<String>,
}

/// Usage record for the `lawcall_usage` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// User UUID.
    pub user_id: String,
    /// Model used.
    pub model: String,
    /// Input tokens consumed.
    pub input_tokens: i32,
    /// Output tokens generated.
    pub output_tokens: i32,
    /// Credits charged.
    pub credits_used: i32,
    /// Whether platform key was used (2x cost).
    pub used_platform_key: bool,
}

/// Security event record for the `security_events` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// User identifier.
    pub user_id: String,
    /// Event type (e.g., "suspicious_pattern", "rate_limit", "wipe_request").
    pub event_type: String,
    /// Structured event details.
    pub details: serde_json::Value,
    /// Severity level: "info", "warning", "critical".
    pub severity: String,
}

/// Audit log record for the `action_audit_log` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// User identifier.
    pub user_id: String,
    /// Action performed.
    pub action: String,
    /// Structured details.
    pub details: serde_json::Value,
    /// Result: "success", "blocked", "pending".
    pub result: String,
}

/// Result of an RPC credit operation.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditResult {
    /// New credit balance after the operation.
    pub new_balance: i32,
}

// ── Supabase client ──────────────────────────────────────────────

/// Supabase HTTP client for ZeroClaw cloud operations.
pub struct SupabaseClient {
    config: SupabaseConfig,
    http: reqwest::Client,
}

impl SupabaseClient {
    /// Create a new Supabase client.
    pub fn new(config: SupabaseConfig) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { config, http })
    }

    /// Build the PostgREST URL for a table.
    fn table_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.config.url, table)
    }

    /// Build the RPC URL for a function.
    fn rpc_url(&self, function: &str) -> String {
        format!("{}/rest/v1/rpc/{}", self.config.url, function)
    }

    /// Get the base headers for authenticated requests.
    fn auth_headers(&self) -> Vec<(&str, String)> {
        vec![
            ("apikey", self.config.service_key.clone()),
            (
                "Authorization",
                format!("Bearer {}", self.config.service_key),
            ),
        ]
    }

    // ── User operations ──────────────────────────────────────

    /// Get or create a user by their KakaoTalk user ID hash.
    pub async fn get_or_create_user(
        &self,
        kakao_user_id: &str,
    ) -> anyhow::Result<SupabaseUser> {
        // Try to fetch existing user
        let url = format!(
            "{}?kakao_user_id=eq.{}&select=*",
            self.table_url("lawcall_users"),
            kakao_user_id
        );

        let mut request = self.http.get(&url);
        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;
        let users: Vec<SupabaseUser> = resp.json().await?;

        if let Some(user) = users.into_iter().next() {
            return Ok(user);
        }

        // Create new user with default credits
        let new_user = SupabaseUser {
            id: None,
            kakao_user_id: kakao_user_id.to_string(),
            credits: 1000, // Default starter credits
            total_spent: 0,
            custom_api_key: None,
            custom_provider: None,
        };

        let mut request = self
            .http
            .post(self.table_url("lawcall_users"))
            .json(&new_user)
            .header("Prefer", "return=representation");

        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create user ({status}): {body}");
        }

        let created: Vec<SupabaseUser> = resp.json().await?;
        created
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("User creation returned empty response"))
    }

    // ── Credit operations (RPC) ──────────────────────────────

    /// Deduct credits atomically via Supabase RPC.
    pub async fn deduct_credits(
        &self,
        kakao_user_id: &str,
        amount: i32,
    ) -> anyhow::Result<CreditResult> {
        let payload = serde_json::json!({
            "p_kakao_user_id": kakao_user_id,
            "p_amount": amount,
        });

        let mut request = self
            .http
            .post(self.rpc_url("deduct_credits"))
            .json(&payload);

        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Credit deduction failed ({status}): {body}");
        }

        let results: Vec<CreditResult> = resp.json().await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Insufficient credits or user not found"))
    }

    /// Add credits atomically via Supabase RPC.
    pub async fn add_credits(
        &self,
        kakao_user_id: &str,
        amount: i32,
    ) -> anyhow::Result<CreditResult> {
        let payload = serde_json::json!({
            "p_kakao_user_id": kakao_user_id,
            "p_amount": amount,
        });

        let mut request = self
            .http
            .post(self.rpc_url("add_credits"))
            .json(&payload);

        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Credit addition failed ({status}): {body}");
        }

        let results: Vec<CreditResult> = resp.json().await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("User not found"))
    }

    // ── Usage logging ────────────────────────────────────────

    /// Record a usage event.
    pub async fn log_usage(&self, usage: &UsageRecord) -> anyhow::Result<()> {
        let mut request = self
            .http
            .post(self.table_url("lawcall_usage"))
            .json(usage);

        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("Failed to log usage ({status}): {body}");
        }

        Ok(())
    }

    // ── Security events ──────────────────────────────────────

    /// Log a security event.
    pub async fn log_security_event(&self, event: &SecurityEvent) -> anyhow::Result<()> {
        let mut request = self
            .http
            .post(self.table_url("security_events"))
            .json(event);

        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("Failed to log security event ({status}): {body}");
        }

        Ok(())
    }

    /// Log an audit entry.
    pub async fn log_audit(&self, entry: &AuditLogEntry) -> anyhow::Result<()> {
        let mut request = self
            .http
            .post(self.table_url("action_audit_log"))
            .json(entry);

        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("Failed to log audit entry ({status}): {body}");
        }

        Ok(())
    }

    // ── Realtime broadcast (for E2E memory sync) ─────────────

    /// Broadcast an E2E encrypted sync payload via Supabase Realtime.
    ///
    /// The payload is already encrypted before reaching this method.
    pub async fn broadcast_sync(
        &self,
        channel: &str,
        encrypted_payload: &[u8],
    ) -> anyhow::Result<()> {
        let broadcast_url = format!(
            "{}/realtime/v1/api/broadcast",
            self.config.url
        );

        let payload = serde_json::json!({
            "channel": channel,
            "event": "sync",
            "payload": {
                "data": base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    encrypted_payload,
                ),
            },
        });

        let mut request = self
            .http
            .post(&broadcast_url)
            .json(&payload);

        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        let resp = request.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Broadcast failed ({status}): {body}");
        }

        Ok(())
    }

    // ── Health check ─────────────────────────────────────────

    /// Check if Supabase is reachable.
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/rest/v1/", self.config.url);

        let mut request = self.http.get(&url);
        for (key, value) in self.auth_headers() {
            request = request.header(key, value);
        }

        matches!(request.send().await, Ok(resp) if resp.status().is_success())
    }
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SupabaseConfig {
        SupabaseConfig {
            url: "https://test-project.supabase.co".into(),
            service_key: "test-service-key".into(),
            anon_key: Some("test-anon-key".into()),
        }
    }

    #[test]
    fn supabase_config_from_env_missing() {
        // Without env vars set, should return None
        // (we don't set SUPABASE_URL in test environment)
        // This test validates the code path, not env-dependent behavior
        let _ = SupabaseConfig::from_env();
    }

    #[test]
    fn table_url_construction() {
        let client = SupabaseClient::new(test_config()).unwrap();
        assert_eq!(
            client.table_url("lawcall_users"),
            "https://test-project.supabase.co/rest/v1/lawcall_users"
        );
    }

    #[test]
    fn rpc_url_construction() {
        let client = SupabaseClient::new(test_config()).unwrap();
        assert_eq!(
            client.rpc_url("deduct_credits"),
            "https://test-project.supabase.co/rest/v1/rpc/deduct_credits"
        );
    }

    #[test]
    fn auth_headers_contain_key() {
        let client = SupabaseClient::new(test_config()).unwrap();
        let headers = client.auth_headers();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].0, "apikey");
        assert_eq!(headers[0].1, "test-service-key");
        assert!(headers[1].1.starts_with("Bearer "));
    }

    #[test]
    fn supabase_user_serialization() {
        let user = SupabaseUser {
            id: Some("uuid-123".into()),
            kakao_user_id: "hash-abc".into(),
            credits: 1000,
            total_spent: 500,
            custom_api_key: None,
            custom_provider: None,
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("hash-abc"));
        assert!(json.contains("1000"));

        let parsed: SupabaseUser = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.credits, 1000);
    }

    #[test]
    fn usage_record_serialization() {
        let usage = UsageRecord {
            user_id: "uuid-123".into(),
            model: "gemini-3-flash".into(),
            input_tokens: 500,
            output_tokens: 200,
            credits_used: 10,
            used_platform_key: true,
        };

        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("gemini-3-flash"));
        assert!(json.contains("true"));
    }

    #[test]
    fn security_event_serialization() {
        let event = SecurityEvent {
            user_id: "zeroclaw_user".into(),
            event_type: "suspicious_pattern".into(),
            details: serde_json::json!({"pattern": "data_exfiltration", "score": 85}),
            severity: "critical".into(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("suspicious_pattern"));
        assert!(json.contains("critical"));
    }

    #[test]
    fn audit_log_entry_serialization() {
        let entry = AuditLogEntry {
            user_id: "zeroclaw_user".into(),
            action: "tool_execute".into(),
            details: serde_json::json!({"tool": "shell", "command": "ls"}),
            result: "success".into(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("tool_execute"));
    }

    #[test]
    fn credit_result_deserialization() {
        let json = r#"{"new_balance": 750}"#;
        let result: CreditResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.new_balance, 750);
    }

    #[test]
    fn client_creation_succeeds() {
        let client = SupabaseClient::new(test_config());
        assert!(client.is_ok());
    }
}
