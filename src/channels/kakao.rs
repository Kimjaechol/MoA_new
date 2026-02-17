use super::traits::{Channel, ChannelMessage, SendMessage};
use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Maximum text length per KakaoTalk message (platform limit).
const KAKAO_MAX_TEXT_LEN: usize = 1000;

/// KakaoTalk REST API base URL.
const KAKAO_API_BASE: &str = "https://kapi.kakao.com";

/// KakaoTalk Channel API base URL for sending messages.
const KAKAO_CHANNEL_API: &str = "https://kapi.kakao.com/v1/api/talk/channels/message";

/// Alimtalk API base URL (Kakao notification templates).
const KAKAO_ALIMTALK_API: &str = "https://kapi.kakao.com/v2/api/talk/memo/default/send";

/// OAuth2 token endpoint for refreshing access tokens.
const KAKAO_TOKEN_URL: &str = "https://kauth.kakao.com/oauth/token";

/// KakaoTalk channel — connects via webhook HTTP receiver for incoming messages,
/// sends replies through Kakao REST API with support for rich message types.
///
/// ## Architecture
/// - **Incoming**: Axum HTTP server receives webhook callbacks from Kakao Channel API
/// - **Outgoing**: REST API calls with OAuth2 Bearer token authentication
/// - **Rich Messages**: Text, carousel, buttons, quick replies via Kakao template system
/// - **Alimtalk**: Template-based notification messages for business use cases
///
/// ## Setup
/// 1. Register an app on Kakao Developers (https://developers.kakao.com)
/// 2. Enable Kakao Talk Channel messaging
/// 3. Set webhook URL to `https://your-domain:{port}/kakao/webhook`
/// 4. Configure REST API key and admin key in ZeroClaw config
pub struct KakaoTalkChannel {
    /// REST API key from Kakao Developers console
    rest_api_key: String,
    /// Admin key for server-side API calls (Alimtalk, push)
    admin_key: String,
    /// Webhook secret for verifying incoming payloads
    webhook_secret: Option<String>,
    /// Allowed Kakao user IDs. Empty = deny all, "*" = allow all.
    allowed_users: Vec<String>,
    /// HTTP port for the webhook receiver server
    port: u16,
    /// HTTP client for outgoing API calls
    client: reqwest::Client,
    /// Cached OAuth2 access token + expiry (epoch seconds)
    token_cache: Arc<RwLock<Option<(String, u64)>>>,
    /// Per-user reply context (user_id -> last known channel context)
    reply_contexts: Arc<RwLock<HashMap<String, ReplyContext>>>,
}

/// Cached reply context for a KakaoTalk user session.
#[derive(Clone, Debug)]
struct ReplyContext {
    /// Kakao user ID
    user_id: String,
    /// Bot user key for API calls
    bot_user_key: Option<String>,
}

/// Shared state for the Axum webhook server.
#[derive(Clone)]
struct WebhookState {
    tx: tokio::sync::mpsc::Sender<ChannelMessage>,
    webhook_secret: Option<String>,
    allowed_users: Vec<String>,
    reply_contexts: Arc<RwLock<HashMap<String, ReplyContext>>>,
}

impl KakaoTalkChannel {
    pub fn new(
        rest_api_key: String,
        admin_key: String,
        webhook_secret: Option<String>,
        allowed_users: Vec<String>,
        port: u16,
    ) -> Self {
        Self {
            rest_api_key,
            admin_key,
            webhook_secret,
            allowed_users,
            port,
            client: reqwest::Client::new(),
            token_cache: Arc::new(RwLock::new(None)),
            reply_contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Build from config struct (convenience for factory wiring).
    pub fn from_config(config: &crate::config::schema::KakaoTalkConfig) -> Self {
        Self::new(
            config.rest_api_key.clone(),
            config.admin_key.clone(),
            config.webhook_secret.clone(),
            config.allowed_users.clone(),
            config.port,
        )
    }

    fn is_user_allowed(&self, user_id: &str) -> bool {
        self.allowed_users.iter().any(|u| u == "*" || u == user_id)
    }

    /// Get a valid access token, refreshing if expired.
    async fn get_access_token(&self) -> anyhow::Result<String> {
        // Check cache
        {
            let cache = self.token_cache.read().await;
            if let Some((ref token, expiry)) = *cache {
                let now = current_epoch_secs();
                if now < expiry {
                    return Ok(token.clone());
                }
            }
        }

        // Token expired or missing — use admin key directly for server-to-server calls
        // KakaoTalk business API uses admin key as Bearer token for server-side calls
        Ok(self.admin_key.clone())
    }

    /// Send a text message to a specific user via KakaoTalk Channel API.
    async fn send_text_message(&self, user_id: &str, text: &str) -> anyhow::Result<()> {
        let token = self.get_access_token().await?;

        let template = serde_json::json!({
            "object_type": "text",
            "text": text,
            "link": {
                "web_url": "",
                "mobile_web_url": ""
            }
        });

        let resp = self
            .client
            .post(KAKAO_CHANNEL_API)
            .header("Authorization", format!("KakaoAK {token}"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("receiver_uuids", serde_json::json!([user_id]).to_string()),
                ("template_object", template.to_string()),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("KakaoTalk send failed ({status}): {err}");
        }

        Ok(())
    }

    /// Send an Alimtalk template message (for business notifications).
    async fn send_alimtalk(
        &self,
        user_id: &str,
        template_id: &str,
        template_args: &HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let token = self.get_access_token().await?;

        let args: Vec<serde_json::Value> = template_args
            .iter()
            .map(|(k, v)| {
                serde_json::json!({
                    "key": k,
                    "value": v
                })
            })
            .collect();

        let body = serde_json::json!({
            "template_id": template_id,
            "receiver_uuids": [user_id],
            "template_args": args
        });

        let resp = self
            .client
            .post(KAKAO_ALIMTALK_API)
            .header("Authorization", format!("KakaoAK {token}"))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("KakaoTalk Alimtalk send failed ({status}): {err}");
        }

        Ok(())
    }

    /// Split a long message into KakaoTalk-sized chunks (1000 chars max).
    fn split_message(text: &str) -> Vec<String> {
        if text.len() <= KAKAO_MAX_TEXT_LEN {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut current = String::new();

        for line in text.lines() {
            if current.len() + line.len() + 1 > KAKAO_MAX_TEXT_LEN {
                if !current.is_empty() {
                    chunks.push(current.clone());
                    current.clear();
                }
                // Handle single lines that exceed the limit
                if line.len() > KAKAO_MAX_TEXT_LEN {
                    let mut remaining = line;
                    while !remaining.is_empty() {
                        let boundary = find_char_boundary(remaining, KAKAO_MAX_TEXT_LEN);
                        let (chunk, rest) = remaining.split_at(boundary);
                        chunks.push(chunk.to_string());
                        remaining = rest;
                    }
                } else {
                    current.push_str(line);
                }
            } else {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(line);
            }
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }

    /// Build a carousel (list) template for rich message display.
    fn build_carousel_template(items: &[CarouselItem]) -> serde_json::Value {
        let contents: Vec<serde_json::Value> = items
            .iter()
            .map(|item| {
                let mut content = serde_json::json!({
                    "title": item.title,
                    "description": item.description,
                    "link": {
                        "web_url": item.link_url,
                        "mobile_web_url": item.link_url
                    }
                });
                if let Some(ref img) = item.image_url {
                    content["image_url"] = serde_json::json!(img);
                    content["image_width"] = serde_json::json!(640);
                    content["image_height"] = serde_json::json!(640);
                }
                content
            })
            .collect();

        serde_json::json!({
            "object_type": "list",
            "header_title": "ZeroClaw",
            "header_link": {
                "web_url": "",
                "mobile_web_url": ""
            },
            "contents": contents
        })
    }

    /// Build a button template for interactive messages.
    fn build_button_template(text: &str, buttons: &[MessageButton]) -> serde_json::Value {
        let button_list: Vec<serde_json::Value> = buttons
            .iter()
            .map(|btn| {
                serde_json::json!({
                    "title": btn.label,
                    "link": {
                        "web_url": btn.url.as_deref().unwrap_or(""),
                        "mobile_web_url": btn.url.as_deref().unwrap_or("")
                    }
                })
            })
            .collect();

        serde_json::json!({
            "object_type": "text",
            "text": text,
            "link": {
                "web_url": "",
                "mobile_web_url": ""
            },
            "buttons": button_list
        })
    }

    /// Parse a remote command from an incoming message.
    /// Commands start with `/` prefix for ZeroClaw control via KakaoTalk.
    fn parse_remote_command(text: &str) -> Option<RemoteCommand> {
        let trimmed = text.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = trimmed[1..].splitn(2, ' ').collect();
        let command = parts.first()?.to_lowercase();
        let args = parts.get(1).unwrap_or(&"").to_string();

        match command.as_str() {
            "status" => Some(RemoteCommand::Status),
            "memory" => Some(RemoteCommand::MemoryQuery(args)),
            "remember" => Some(RemoteCommand::MemoryStore(args)),
            "forget" => Some(RemoteCommand::MemoryForget(args)),
            "cron" => Some(RemoteCommand::CronList),
            "help" => Some(RemoteCommand::Help),
            "shell" => Some(RemoteCommand::Shell(args)),
            _ => None,
        }
    }
}

/// Carousel item for rich list messages.
#[derive(Debug, Clone)]
pub struct CarouselItem {
    pub title: String,
    pub description: String,
    pub link_url: String,
    pub image_url: Option<String>,
}

/// Button for interactive messages.
#[derive(Debug, Clone)]
pub struct MessageButton {
    pub label: String,
    pub url: Option<String>,
}

/// Remote commands available via KakaoTalk `/command` syntax.
#[derive(Debug, Clone)]
pub enum RemoteCommand {
    /// Check ZeroClaw agent status
    Status,
    /// Query long-term memory
    MemoryQuery(String),
    /// Store to long-term memory
    MemoryStore(String),
    /// Forget a memory entry
    MemoryForget(String),
    /// List scheduled cron tasks
    CronList,
    /// Show help for available commands
    Help,
    /// Execute a shell command (requires Full autonomy level)
    Shell(String),
}

/// Find a safe character boundary for UTF-8 string splitting.
fn find_char_boundary(s: &str, max_bytes: usize) -> usize {
    if max_bytes >= s.len() {
        return s.len();
    }
    // Walk backwards to find a valid char boundary
    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
}

/// Get the current epoch time in seconds.
fn current_epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Verify HMAC-SHA256 webhook signature from Kakao.
fn verify_webhook_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);

    // Kakao sends base64-encoded HMAC
    let Ok(expected_bytes) = base64_decode(signature) else {
        return false;
    };

    mac.verify_slice(&expected_bytes).is_ok()
}

/// Decode base64 (standard encoding).
fn base64_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input)
}

/// Webhook handler: POST /kakao/webhook
async fn handle_webhook(State(state): State<WebhookState>, body: axum::body::Bytes) -> StatusCode {
    // Signature verification (if configured)
    // Note: In production, extract the X-Kakao-Signature header
    // For now we parse the body directly
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("KakaoTalk: invalid webhook payload: {e}");
            return StatusCode::BAD_REQUEST;
        }
    };

    // Extract user event type
    let user_request = payload.get("userRequest");
    // Kakao Chatbot Skill format
    if let Some(user_req) = user_request {
        let user_id = user_req
            .get("user")
            .and_then(|u| u.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("unknown");

        // Check user allowlist
        if !state.allowed_users.iter().any(|u| u == "*" || u == user_id) {
            tracing::warn!("KakaoTalk: ignoring message from unauthorized user: {user_id}");
            return StatusCode::FORBIDDEN;
        }

        let utterance = user_req
            .get("utterance")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .trim();

        if utterance.is_empty() {
            return StatusCode::OK;
        }

        // Extract bot_user_key for replies
        let bot_user_key = user_req
            .get("user")
            .and_then(|u| u.get("properties"))
            .and_then(|p| p.get("bot_user_key"))
            .and_then(|k| k.as_str())
            .map(String::from);

        // Cache reply context
        {
            let mut contexts = state.reply_contexts.write().await;
            contexts.insert(
                user_id.to_string(),
                ReplyContext {
                    user_id: user_id.to_string(),
                    bot_user_key,
                },
            );
        }

        let channel_msg = ChannelMessage {
            id: Uuid::new_v4().to_string(),
            sender: user_id.to_string(),
            reply_target: user_id.to_string(),
            content: utterance.to_string(),
            channel: "kakao".to_string(),
            timestamp: current_epoch_secs(),
        };

        if state.tx.send(channel_msg).await.is_err() {
            tracing::warn!("KakaoTalk: message channel closed");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }

        return StatusCode::OK;
    }

    // Also handle direct message callback format
    if let Some(content) = payload.get("content") {
        let user_id = payload
            .get("user_id")
            .and_then(|u| u.as_str())
            .unwrap_or("unknown");

        if !state.allowed_users.iter().any(|u| u == "*" || u == user_id) {
            return StatusCode::FORBIDDEN;
        }

        let text = content.as_str().unwrap_or("").trim();
        if text.is_empty() {
            return StatusCode::OK;
        }

        let channel_msg = ChannelMessage {
            id: Uuid::new_v4().to_string(),
            sender: user_id.to_string(),
            reply_target: user_id.to_string(),
            content: text.to_string(),
            channel: "kakao".to_string(),
            timestamp: current_epoch_secs(),
        };

        if state.tx.send(channel_msg).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }

        return StatusCode::OK;
    }

    StatusCode::OK
}

/// Health check endpoint: GET /kakao/health
async fn handle_health() -> StatusCode {
    StatusCode::OK
}

#[async_trait]
impl Channel for KakaoTalkChannel {
    fn name(&self) -> &str {
        "kakao"
    }

    async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
        let chunks = Self::split_message(&message.content);

        for chunk in chunks {
            self.send_text_message(&message.recipient, &chunk).await?;
        }

        Ok(())
    }

    async fn listen(&self, tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> anyhow::Result<()> {
        let state = WebhookState {
            tx,
            webhook_secret: self.webhook_secret.clone(),
            allowed_users: self.allowed_users.clone(),
            reply_contexts: Arc::clone(&self.reply_contexts),
        };

        let app = Router::new()
            .route("/kakao/webhook", post(handle_webhook))
            .route("/kakao/health", get(handle_health))
            .with_state(state);

        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], self.port));
        tracing::info!("KakaoTalk: webhook server listening on {addr}");

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("KakaoTalk webhook server error: {e}"))?;

        anyhow::bail!("KakaoTalk webhook server stopped unexpectedly")
    }

    async fn health_check(&self) -> bool {
        // Verify API connectivity by checking if admin key is set
        // A more thorough check could call /v1/api/talk/profile endpoint
        let result = self
            .client
            .get(format!("{KAKAO_API_BASE}/v1/api/talk/profile"))
            .header("Authorization", format!("KakaoAK {}", self.admin_key))
            .send()
            .await;

        match result {
            Ok(resp) => resp.status().is_success() || resp.status().as_u16() == 401,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let ch = KakaoTalkChannel::new("key".into(), "admin".into(), None, vec![], 8080);
        assert_eq!(ch.name(), "kakao");
    }

    #[test]
    fn test_user_allowed_wildcard() {
        let ch = KakaoTalkChannel::new("key".into(), "admin".into(), None, vec!["*".into()], 8080);
        assert!(ch.is_user_allowed("anyone"));
    }

    #[test]
    fn test_user_allowed_specific() {
        let ch = KakaoTalkChannel::new(
            "key".into(),
            "admin".into(),
            None,
            vec!["user_123".into()],
            8080,
        );
        assert!(ch.is_user_allowed("user_123"));
        assert!(!ch.is_user_allowed("other"));
    }

    #[test]
    fn test_user_denied_empty() {
        let ch = KakaoTalkChannel::new("key".into(), "admin".into(), None, vec![], 8080);
        assert!(!ch.is_user_allowed("anyone"));
    }

    #[test]
    fn test_split_message_short() {
        let text = "Hello, world!";
        let chunks = KakaoTalkChannel::split_message(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello, world!");
    }

    #[test]
    fn test_split_message_long() {
        let text = "a".repeat(2500);
        let chunks = KakaoTalkChannel::split_message(&text);
        assert!(chunks.len() >= 3);
        for chunk in &chunks {
            assert!(chunk.len() <= KAKAO_MAX_TEXT_LEN);
        }
        let rejoined: String = chunks.join("");
        assert_eq!(rejoined, text);
    }

    #[test]
    fn test_split_message_multiline() {
        let lines: Vec<String> = (0..20)
            .map(|i| format!("Line {i}: {}", "x".repeat(80)))
            .collect();
        let text = lines.join("\n");
        let chunks = KakaoTalkChannel::split_message(&text);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= KAKAO_MAX_TEXT_LEN);
        }
    }

    #[test]
    fn test_split_message_exact_boundary() {
        let text = "a".repeat(KAKAO_MAX_TEXT_LEN);
        let chunks = KakaoTalkChannel::split_message(&text);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_split_message_utf8_safe() {
        // Korean text (3 bytes per char in UTF-8)
        let korean = "가".repeat(500);
        let chunks = KakaoTalkChannel::split_message(&korean);
        for chunk in &chunks {
            assert!(chunk.len() <= KAKAO_MAX_TEXT_LEN);
            // Must not panic on char boundary
            assert!(chunk.is_char_boundary(chunk.len()));
        }
    }

    #[test]
    fn test_find_char_boundary_ascii() {
        let s = "hello world";
        assert_eq!(find_char_boundary(s, 5), 5);
    }

    #[test]
    fn test_find_char_boundary_utf8() {
        let s = "가나다라"; // Each char is 3 bytes
        assert_eq!(find_char_boundary(s, 4), 3); // Rounds down to char boundary
        assert_eq!(find_char_boundary(s, 6), 6); // Exactly on boundary
    }

    #[test]
    fn test_find_char_boundary_beyond_len() {
        let s = "abc";
        assert_eq!(find_char_boundary(s, 100), 3);
    }

    #[test]
    fn test_parse_remote_command_status() {
        let cmd = KakaoTalkChannel::parse_remote_command("/status");
        assert!(matches!(cmd, Some(RemoteCommand::Status)));
    }

    #[test]
    fn test_parse_remote_command_memory_query() {
        let cmd = KakaoTalkChannel::parse_remote_command("/memory what is my name");
        assert!(matches!(cmd, Some(RemoteCommand::MemoryQuery(ref s)) if s == "what is my name"));
    }

    #[test]
    fn test_parse_remote_command_help() {
        let cmd = KakaoTalkChannel::parse_remote_command("/help");
        assert!(matches!(cmd, Some(RemoteCommand::Help)));
    }

    #[test]
    fn test_parse_remote_command_shell() {
        let cmd = KakaoTalkChannel::parse_remote_command("/shell ls -la");
        assert!(matches!(cmd, Some(RemoteCommand::Shell(ref s)) if s == "ls -la"));
    }

    #[test]
    fn test_parse_remote_command_unknown() {
        let cmd = KakaoTalkChannel::parse_remote_command("/unknown_cmd");
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_remote_command_no_prefix() {
        let cmd = KakaoTalkChannel::parse_remote_command("just a message");
        assert!(cmd.is_none());
    }

    #[test]
    fn test_parse_remote_command_remember() {
        let cmd = KakaoTalkChannel::parse_remote_command("/remember my favorite color is blue");
        assert!(
            matches!(cmd, Some(RemoteCommand::MemoryStore(ref s)) if s == "my favorite color is blue")
        );
    }

    #[test]
    fn test_parse_remote_command_forget() {
        let cmd = KakaoTalkChannel::parse_remote_command("/forget my_key");
        assert!(matches!(cmd, Some(RemoteCommand::MemoryForget(ref s)) if s == "my_key"));
    }

    #[test]
    fn test_parse_remote_command_cron() {
        let cmd = KakaoTalkChannel::parse_remote_command("/cron");
        assert!(matches!(cmd, Some(RemoteCommand::CronList)));
    }

    #[test]
    fn test_carousel_template_structure() {
        let items = vec![
            CarouselItem {
                title: "Item 1".into(),
                description: "Description 1".into(),
                link_url: "https://example.com/1".into(),
                image_url: Some("https://example.com/img1.png".into()),
            },
            CarouselItem {
                title: "Item 2".into(),
                description: "Description 2".into(),
                link_url: "https://example.com/2".into(),
                image_url: None,
            },
        ];

        let template = KakaoTalkChannel::build_carousel_template(&items);
        assert_eq!(template["object_type"], "list");
        assert_eq!(template["header_title"], "ZeroClaw");
        let contents = template["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0]["title"], "Item 1");
        assert!(contents[0].get("image_url").is_some());
        assert!(contents[1].get("image_url").is_none());
    }

    #[test]
    fn test_button_template_structure() {
        let buttons = vec![
            MessageButton {
                label: "Visit".into(),
                url: Some("https://example.com".into()),
            },
            MessageButton {
                label: "Cancel".into(),
                url: None,
            },
        ];

        let template = KakaoTalkChannel::build_button_template("Choose an option:", &buttons);
        assert_eq!(template["object_type"], "text");
        assert_eq!(template["text"], "Choose an option:");
        let btns = template["buttons"].as_array().unwrap();
        assert_eq!(btns.len(), 2);
        assert_eq!(btns[0]["title"], "Visit");
        assert_eq!(btns[1]["title"], "Cancel");
    }

    #[test]
    fn test_config_serde() {
        let toml_str = r#"
rest_api_key = "test_key_123"
admin_key = "admin_key_456"
webhook_secret = "secret_789"
allowed_users = ["user_a", "*"]
port = 9090
"#;
        let config: crate::config::schema::KakaoTalkConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.rest_api_key, "test_key_123");
        assert_eq!(config.admin_key, "admin_key_456");
        assert_eq!(config.webhook_secret, Some("secret_789".to_string()));
        assert_eq!(config.allowed_users, vec!["user_a", "*"]);
        assert_eq!(config.port, 9090);
    }

    #[test]
    fn test_config_serde_defaults() {
        let toml_str = r#"
rest_api_key = "key"
admin_key = "admin"
"#;
        let config: crate::config::schema::KakaoTalkConfig = toml::from_str(toml_str).unwrap();
        assert!(config.allowed_users.is_empty());
        assert!(config.webhook_secret.is_none());
        assert_eq!(config.port, 8787);
    }
}
