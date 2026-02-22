//! Axum-based HTTP gateway with proper HTTP/1.1 compliance, body limits, and timeouts.
//!
//! This module replaces the raw TCP implementation with axum for:
//! - Proper HTTP/1.1 parsing and compliance
//! - Content-Length validation (handled by hyper)
//! - Request body size limits (64KB max)
//! - Request timeouts (30s) to prevent slow-loris attacks
//! - Header sanitization (handled by axum/hyper)

pub mod pair;

use crate::agent::agent::Agent;
use crate::channels::{Channel, SendMessage, WhatsAppChannel};
use crate::config::Config;
use serde::Deserialize as GatewayDeserialize;
use crate::memory::{self, Memory, MemoryCategory};
use crate::providers::{self, Provider};
use crate::security::pairing::{constant_time_eq, is_public_bind, PairingGuard};
use crate::util::truncate_with_ellipsis;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use uuid::Uuid;

/// Maximum request body size (64KB) — prevents memory exhaustion
pub const MAX_BODY_SIZE: usize = 65_536;
/// Request timeout (120s) — allows agentic tool execution while preventing abuse
pub const REQUEST_TIMEOUT_SECS: u64 = 120;
/// Sliding window used by gateway rate limiting.
pub const RATE_LIMIT_WINDOW_SECS: u64 = 60;

fn webhook_memory_key() -> String {
    format!("webhook_msg_{}", Uuid::new_v4())
}

fn whatsapp_memory_key(msg: &crate::channels::traits::ChannelMessage) -> String {
    format!("whatsapp_{}_{}", msg.sender, msg.id)
}

fn hash_webhook_secret(value: &str) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(value.as_bytes());
    hex::encode(digest)
}

/// How often the rate limiter sweeps stale IP entries from its map.
const RATE_LIMITER_SWEEP_INTERVAL_SECS: u64 = 300; // 5 minutes

#[derive(Debug)]
struct SlidingWindowRateLimiter {
    limit_per_window: u32,
    window: Duration,
    requests: Mutex<(HashMap<String, Vec<Instant>>, Instant)>,
}

impl SlidingWindowRateLimiter {
    fn new(limit_per_window: u32, window: Duration) -> Self {
        Self {
            limit_per_window,
            window,
            requests: Mutex::new((HashMap::new(), Instant::now())),
        }
    }

    fn allow(&self, key: &str) -> bool {
        if self.limit_per_window == 0 {
            return true;
        }

        let now = Instant::now();
        let cutoff = now.checked_sub(self.window).unwrap_or_else(Instant::now);

        let mut guard = self.requests.lock();
        let (requests, last_sweep) = &mut *guard;

        // Periodic sweep: remove IPs with no recent requests
        if last_sweep.elapsed() >= Duration::from_secs(RATE_LIMITER_SWEEP_INTERVAL_SECS) {
            requests.retain(|_, timestamps| {
                timestamps.retain(|t| *t > cutoff);
                !timestamps.is_empty()
            });
            *last_sweep = now;
        }

        let entry = requests.entry(key.to_owned()).or_default();
        entry.retain(|instant| *instant > cutoff);

        if entry.len() >= self.limit_per_window as usize {
            return false;
        }

        entry.push(now);
        true
    }
}

#[derive(Debug)]
pub struct GatewayRateLimiter {
    pair: SlidingWindowRateLimiter,
    webhook: SlidingWindowRateLimiter,
}

impl GatewayRateLimiter {
    fn new(pair_per_minute: u32, webhook_per_minute: u32) -> Self {
        let window = Duration::from_secs(RATE_LIMIT_WINDOW_SECS);
        Self {
            pair: SlidingWindowRateLimiter::new(pair_per_minute, window),
            webhook: SlidingWindowRateLimiter::new(webhook_per_minute, window),
        }
    }

    fn allow_pair(&self, key: &str) -> bool {
        self.pair.allow(key)
    }

    fn allow_webhook(&self, key: &str) -> bool {
        self.webhook.allow(key)
    }
}

#[derive(Debug)]
pub struct IdempotencyStore {
    ttl: Duration,
    keys: Mutex<HashMap<String, Instant>>,
}

impl IdempotencyStore {
    fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            keys: Mutex::new(HashMap::new()),
        }
    }

    /// Returns true if this key is new and is now recorded.
    fn record_if_new(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut keys = self.keys.lock();

        keys.retain(|_, seen_at| now.duration_since(*seen_at) < self.ttl);

        if keys.contains_key(key) {
            return false;
        }

        keys.insert(key.to_owned(), now);
        true
    }
}

fn client_key_from_headers(headers: &HeaderMap) -> String {
    for header_name in ["X-Forwarded-For", "X-Real-IP"] {
        if let Some(value) = headers.get(header_name).and_then(|v| v.to_str().ok()) {
            let first = value.split(',').next().unwrap_or("").trim();
            if !first.is_empty() {
                return first.to_owned();
            }
        }
    }
    "unknown".into()
}

/// Shared state for all axum handlers
#[derive(Clone)]
pub struct AppState {
    pub provider: Arc<dyn Provider>,
    pub model: String,
    pub temperature: f64,
    pub mem: Arc<dyn Memory>,
    pub auto_save: bool,
    /// SHA-256 hash of `X-Webhook-Secret` (hex-encoded), never plaintext.
    pub webhook_secret_hash: Option<Arc<str>>,
    pub pairing: Arc<PairingGuard>,
    pub rate_limiter: Arc<GatewayRateLimiter>,
    pub idempotency_store: Arc<IdempotencyStore>,
    pub whatsapp: Option<Arc<WhatsAppChannel>>,
    /// LINE channel (if configured) for webhook-based message handling.
    pub line: Option<Arc<crate::channels::line::LineChannel>>,
    /// `WhatsApp` app secret for webhook signature verification (`X-Hub-Signature-256`)
    pub whatsapp_app_secret: Option<Arc<str>>,
    /// SLM gatekeeper for local intent classification + simple response.
    pub gatekeeper: Option<Arc<tokio::sync::Mutex<crate::gatekeeper::GatekeeperRouter>>>,
    /// Agent with tools for agentic webhook handling.
    /// When `Some`, `/webhook` uses `agent.turn()` (tools enabled).
    /// When `None`, falls back to `provider.simple_chat()` (no tools).
    pub agent: Option<Arc<tokio::sync::Mutex<Agent>>>,
    /// Admin telemetry store for usage analytics.
    pub telemetry: Option<Arc<crate::telemetry::TelemetryStore>>,
    /// Admin token hash for telemetry API authentication.
    pub telemetry_admin_token_hash: Option<Arc<str>>,
    /// Multi-user auth store (when auth.enabled = true).
    pub auth_store: Option<Arc<crate::auth::AuthStore>>,
    /// Whether new user registration is allowed.
    pub auth_allow_registration: bool,
    /// Maximum registered users (0 = unlimited).
    pub auth_max_users: u64,
    /// Maximum devices per user.
    pub auth_max_devices_per_user: u32,
    /// Temporary relay for sync Layer 1 (TTL-based in-memory storage).
    pub sync_relay: Option<Arc<crate::sync::SyncRelay>>,
    /// Broadcast channel for sync WebSocket messages.
    pub sync_broadcast: Option<tokio::sync::broadcast::Sender<String>>,
    /// Shared pairing store for channel connect flow.
    pub channel_pairing: Option<Arc<crate::channels::pairing::ChannelPairingStore>>,
    /// Base URL for this gateway (e.g. "http://127.0.0.1:3000").
    pub gateway_base_url: String,
}

/// Run the HTTP gateway using axum with proper HTTP/1.1 compliance.
#[allow(clippy::too_many_lines)]
pub async fn run_gateway(host: &str, port: u16, config: Config) -> Result<()> {
    // ── Security: refuse public bind without tunnel or explicit opt-in ──
    if is_public_bind(host) && config.tunnel.provider == "none" && !config.gateway.allow_public_bind
    {
        anyhow::bail!(
            "🛑 Refusing to bind to {host} — gateway would be exposed to the internet.\n\
             Fix: use --host 127.0.0.1 (default), configure a tunnel, or set\n\
             [gateway] allow_public_bind = true in config.toml (NOT recommended)."
        );
    }

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let actual_port = listener.local_addr()?.port();
    let display_addr = format!("{host}:{actual_port}");

    let provider: Arc<dyn Provider> = Arc::from(providers::create_resilient_provider(
        config.default_provider.as_deref().unwrap_or("openrouter"),
        config.api_key.as_deref(),
        config.api_url.as_deref(),
        &config.reliability,
    )?);
    let model = config
        .default_model
        .clone()
        .unwrap_or_else(|| "google/gemini-3.1-pro-preview".into());
    let temperature = config.default_temperature;
    let mem: Arc<dyn Memory> = Arc::from(memory::create_memory(
        &config.memory,
        &config.workspace_dir,
        config.api_key.as_deref(),
    )?);
    // Note: runtime, security, tools, and composio are created internally
    // by Agent::from_config() — no need to create them here.
    // Extract webhook secret for authentication
    let webhook_secret_hash: Option<Arc<str>> =
        config.channels_config.webhook.as_ref().and_then(|webhook| {
            webhook.secret.as_ref().and_then(|raw_secret| {
                let trimmed_secret = raw_secret.trim();
                (!trimmed_secret.is_empty())
                    .then(|| Arc::<str>::from(hash_webhook_secret(trimmed_secret)))
            })
        });

    // ── Channel pairing store (shared SQLite for gateway+channels) ──
    let pairing_db_path = config.workspace_dir.join("pairing.db");
    let channel_pairing: Option<Arc<crate::channels::pairing::ChannelPairingStore>> =
        match crate::channels::pairing::ChannelPairingStore::open(&pairing_db_path) {
            Ok(store) => {
                tracing::info!("Channel pairing store initialized at {}", pairing_db_path.display());
                Some(Arc::new(store))
            }
            Err(e) => {
                tracing::warn!("Failed to open channel pairing store: {e} — pairing disabled");
                None
            }
        };

    // LINE channel (if configured)
    let line_channel: Option<Arc<crate::channels::line::LineChannel>> =
        config.channels_config.line.as_ref().map(|line| {
            Arc::new(crate::channels::line::LineChannel::new(
                line.channel_access_token.clone(),
                line.channel_secret.clone(),
                line.allowed_users.clone(),
                channel_pairing.clone(),
                Some(format!("http://{}:{}", config.gateway.host, config.gateway.port)),
            ))
        });

    // WhatsApp channel (if configured)
    let whatsapp_channel: Option<Arc<WhatsAppChannel>> =
        config.channels_config.whatsapp.as_ref().map(|wa| {
            Arc::new(WhatsAppChannel::new(
                wa.access_token.clone(),
                wa.phone_number_id.clone(),
                wa.verify_token.clone(),
                wa.allowed_numbers.clone(),
                channel_pairing.clone(),
                Some(format!("http://{}:{}", config.gateway.host, config.gateway.port)),
            ))
        });

    // WhatsApp app secret for webhook signature verification
    // Priority: environment variable > config file
    let whatsapp_app_secret: Option<Arc<str>> = std::env::var("ZEROCLAW_WHATSAPP_APP_SECRET")
        .ok()
        .and_then(|secret| {
            let secret = secret.trim();
            (!secret.is_empty()).then(|| secret.to_owned())
        })
        .or_else(|| {
            config.channels_config.whatsapp.as_ref().and_then(|wa| {
                wa.app_secret
                    .as_deref()
                    .map(str::trim)
                    .filter(|secret| !secret.is_empty())
                    .map(ToOwned::to_owned)
            })
        })
        .map(Arc::from);

    // ── SLM Gatekeeper ────────────────────────────────────
    let gatekeeper = if config.gatekeeper.enabled {
        let mut router = crate::gatekeeper::GatekeeperRouter::from_config(&config.gatekeeper);
        let healthy = router.check_slm_health().await;
        if healthy {
            tracing::info!(
                model = config.gatekeeper.model,
                "SLM gatekeeper active — local routing enabled"
            );
        } else {
            tracing::warn!(
                "SLM gatekeeper enabled but Ollama not reachable — will fall back to cloud"
            );
        }
        Some(Arc::new(tokio::sync::Mutex::new(router)))
    } else {
        None
    };

    // ── Telemetry ─────────────────────────────────────────
    let (telemetry, telemetry_admin_token_hash) = if config.telemetry.enabled {
        match crate::telemetry::TelemetryStore::new(&config.workspace_dir, config.telemetry.clone())
        {
            Ok(store) => {
                tracing::info!("Telemetry store initialized");
                let hash = config.telemetry.admin_token.as_deref().map(|token| {
                    use sha2::{Digest, Sha256};
                    let h = format!("{:x}", Sha256::digest(token.as_bytes()));
                    Arc::from(h.as_str())
                });
                (Some(Arc::new(store)), hash)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize telemetry store: {e}");
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    // ── Auth store (multi-user) ─────────────────────────────
    let auth_store = if config.auth.enabled {
        let auth_db = config.workspace_dir.join("auth.db");
        match crate::auth::AuthStore::new(&auth_db, Some(config.auth.session_ttl_secs)) {
            Ok(store) => {
                tracing::info!("Multi-user auth store initialized");
                Some(Arc::new(store))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize auth store: {e}");
                None
            }
        }
    } else {
        None
    };
    let auth_allow_registration = config.auth.allow_registration;
    let auth_max_users = config.auth.max_users;
    let auth_max_devices_per_user = config.auth.max_devices_per_user;

    // ── Sync relay + broadcast ───────────────────────────────
    let (sync_relay, sync_broadcast) = if config.sync.enabled {
        let relay = Arc::new(crate::sync::SyncRelay::with_ttl(config.sync.relay_ttl_secs));
        let (tx, _rx) = tokio::sync::broadcast::channel::<String>(256);
        tracing::info!(
            ttl = config.sync.relay_ttl_secs,
            "Sync relay + broadcast channel initialized"
        );

        // Periodic relay sweep (every 60 seconds)
        let relay_for_sweep = Arc::clone(&relay);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let removed = relay_for_sweep.sweep_expired();
                if removed > 0 {
                    tracing::debug!(removed, "Swept expired relay entries");
                }
            }
        });

        (Some(relay), Some(tx))
    } else {
        (None, None)
    };

    // ── Pairing guard ──────────────────────────────────────
    let pairing = Arc::new(PairingGuard::new(
        config.gateway.require_pairing,
        &config.gateway.paired_tokens,
        config.gateway.owner_username.as_deref(),
        config.gateway.owner_password.as_deref(),
    ));
    let rate_limiter = Arc::new(GatewayRateLimiter::new(
        config.gateway.pair_rate_limit_per_minute,
        config.gateway.webhook_rate_limit_per_minute,
    ));
    let idempotency_store = Arc::new(IdempotencyStore::new(Duration::from_secs(
        config.gateway.idempotency_ttl_secs.max(1),
    )));

    // ── Tunnel ────────────────────────────────────────────────
    let tunnel = crate::tunnel::create_tunnel(&config.tunnel)?;
    let mut tunnel_url: Option<String> = None;

    if let Some(ref tun) = tunnel {
        println!("🔗 Starting {} tunnel...", tun.name());
        match tun.start(host, actual_port).await {
            Ok(url) => {
                println!("🌐 Tunnel active: {url}");
                tunnel_url = Some(url);
            }
            Err(e) => {
                println!("⚠️  Tunnel failed to start: {e}");
                println!("   Falling back to local-only mode.");
            }
        }
    }

    println!("🦀 ZeroClaw Gateway listening on http://{display_addr}");
    if let Some(ref url) = tunnel_url {
        println!("  🌐 Public URL: {url}");
    }
    println!("  POST /pair      — pair a new client (X-Pairing-Code header)");
    println!("  POST /webhook   — {{\"message\": \"your prompt\"}}");
    if whatsapp_channel.is_some() {
        println!("  GET  /whatsapp  — Meta webhook verification");
        println!("  POST /whatsapp  — WhatsApp message webhook");
    }
    if line_channel.is_some() {
        println!("  POST /line      — LINE message webhook");
    }
    println!("  GET  /pair/connect/{{channel}} — channel pairing login page");
    println!("  GET  /pair/signup            — channel pairing signup page");
    println!("  GET  /health    — health check");
    if config.sync.enabled {
        println!("  WS   /sync      — WebSocket sync broadcast channel");
        println!("  POST /api/sync/relay  — upload encrypted data to relay");
        println!("  GET  /api/sync/relay  — pickup pending relay entries");
    }
    if config.auth.enabled {
        println!("  POST /api/auth/register            — create new user account");
        println!("  POST /api/auth/login               — authenticate and get session token");
        println!("  POST /api/auth/logout              — revoke current session");
        println!("  GET  /api/auth/me                  — current user info");
        println!("  GET  /api/auth/devices             — list user devices");
        println!("  POST /api/auth/devices             — register a device");
        println!("  DELETE /api/auth/devices/:id       — remove a device");
    }
    if config.telemetry.enabled {
        println!("  POST /api/telemetry/events        — ingest telemetry events");
        println!("  GET  /api/admin/telemetry/events   — query events (admin)");
        println!("  GET  /api/admin/telemetry/summary  — usage summary (admin)");
        println!("  GET  /api/admin/telemetry/alerts   — suspicious alerts (admin)");
    }
    if let Some(code) = pairing.pairing_code() {
        println!();
        if pairing.requires_credentials() {
            println!("  🔐 PAIRING REQUIRED — credentials + code:");
        } else {
            println!("  🔐 PAIRING REQUIRED — use this code:");
        }
        println!("     ┌──────────────┐");
        println!("     │  {code}  │");
        println!("     └──────────────┘");
        if pairing.requires_credentials() {
            println!("     Send: POST /pair with X-Pairing-Code header + {{username, password}} body");
        } else {
            println!("     Send: POST /pair with header X-Pairing-Code: {code}");
        }
    } else if pairing.require_pairing() {
        println!("  🔒 Pairing: ACTIVE (bearer token required)");
    } else {
        println!("  ⚠️  Pairing: DISABLED (all requests accepted)");
    }
    println!("  Press Ctrl+C to stop.\n");

    crate::health::mark_component_ok("gateway");

    // ── Agent with tools ─────────────────────────────────────
    let agent = match Agent::from_config(&config) {
        Ok(a) => {
            tracing::info!("Gateway agent initialized with tools");
            Some(Arc::new(tokio::sync::Mutex::new(a)))
        }
        Err(e) => {
            tracing::warn!("Failed to create gateway agent: {e} — falling back to simple chat");
            None
        }
    };

    // Build shared state
    let state = AppState {
        provider,
        model,
        temperature,
        mem,
        auto_save: config.memory.auto_save,
        webhook_secret_hash,
        pairing,
        rate_limiter,
        idempotency_store,
        whatsapp: whatsapp_channel,
        line: line_channel,
        whatsapp_app_secret,
        gatekeeper,
        agent,
        telemetry,
        telemetry_admin_token_hash,
        auth_store,
        auth_allow_registration,
        auth_max_users,
        auth_max_devices_per_user,
        sync_relay,
        sync_broadcast,
        channel_pairing,
        gateway_base_url: format!("http://{}:{}", config.gateway.host, config.gateway.port),
    };

    // Ensure channel_links table exists if auth is enabled
    if let Some(ref auth) = state.auth_store {
        if let Err(e) = auth.ensure_channel_links_table() {
            tracing::warn!("Failed to create channel_links table: {e}");
        }
    }

    // ── CORS — allow web/Tauri clients to connect from any origin ──
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::HeaderName::from_static("x-webhook-secret"),
            header::HeaderName::from_static("x-pairing-code"),
            header::HeaderName::from_static("x-idempotency-key"),
        ])
        .max_age(Duration::from_secs(3600));

    // Build router with middleware
    let app = Router::new()
        .route("/health", get(handle_health))
        .route("/api/navigation", get(handle_navigation))
        .route("/api/coding/layout", get(handle_coding_layout))
        .route("/api/coding/layout/mobile", get(handle_coding_layout_mobile))
        .route("/pair", post(handle_pair))
        .route("/webhook", post(handle_webhook))
        .route("/whatsapp", get(handle_whatsapp_verify))
        .route("/whatsapp", post(handle_whatsapp_message))
        .route("/line", post(handle_line_message))
        .route("/api/auth/register", post(handle_auth_register))
        .route("/api/auth/login", post(handle_auth_login))
        .route("/api/auth/logout", post(handle_auth_logout))
        .route("/api/auth/me", get(handle_auth_me))
        .route("/api/auth/devices", get(handle_auth_devices_list))
        .route("/api/auth/devices", post(handle_auth_device_register))
        .route("/api/auth/devices/{device_id}", axum::routing::delete(handle_auth_device_remove))
        .route("/sync", get(handle_sync_ws))
        .route("/api/sync/relay", post(handle_sync_relay_upload))
        .route("/api/sync/relay", get(handle_sync_relay_pickup))
        .route("/pair/auto/{token}", get(pair::handle_auto_pair_page))
        .route("/pair/auto/{token}", post(pair::handle_auto_pair_login))
        .route("/pair/signup", get(pair::handle_pair_signup_page))
        .route("/pair/signup", post(pair::handle_pair_signup_submit))
        .route("/api/telemetry/events", post(handle_telemetry_ingest))
        .route("/api/admin/telemetry/events", get(handle_admin_telemetry_events))
        .route("/api/admin/telemetry/summary", get(handle_admin_telemetry_summary))
        .route("/api/admin/telemetry/alerts", get(handle_admin_telemetry_alerts))
        .with_state(state)
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(MAX_BODY_SIZE))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(REQUEST_TIMEOUT_SECS),
        ));

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// AXUM HANDLERS
// ══════════════════════════════════════════════════════════════════════════════

/// GET /health — always public (no secrets leaked)
async fn handle_health(State(state): State<AppState>) -> impl IntoResponse {
    let body = serde_json::json!({
        "status": "ok",
        "paired": state.pairing.is_paired(),
        "runtime": crate::health::snapshot_json(),
    });
    Json(body)
}

/// GET /api/navigation — returns the navigation manifest for the web chat UI.
async fn handle_navigation() -> impl IntoResponse {
    Json(serde_json::to_value(crate::task_category::NavigationManifest::build()).unwrap())
}

/// GET /api/coding/layout — returns the default split-screen coding layout.
async fn handle_coding_layout() -> impl IntoResponse {
    Json(serde_json::to_value(crate::sandbox::layout::CodingLayout::default()).unwrap())
}

/// GET /api/coding/layout/mobile — returns the mobile-optimized coding layout.
async fn handle_coding_layout_mobile() -> impl IntoResponse {
    Json(serde_json::to_value(crate::sandbox::layout::CodingLayout::mobile()).unwrap())
}

/// Optional JSON body for pairing with credentials.
#[derive(Debug, Default, GatewayDeserialize)]
struct PairBody {
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

/// POST /pair — exchange pairing code (+ optional credentials) for bearer token.
///
/// The pairing code is read from the `X-Pairing-Code` header.
/// If owner credentials are configured on the server, `username` and `password`
/// must also be provided in the JSON request body.
async fn handle_pair(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<PairBody>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let client_key = client_key_from_headers(&headers);
    if !state.rate_limiter.allow_pair(&client_key) {
        tracing::warn!("/pair rate limit exceeded for key: {client_key}");
        let err = serde_json::json!({
            "error": "Too many pairing requests. Please retry later.",
            "retry_after": RATE_LIMIT_WINDOW_SECS,
        });
        return (StatusCode::TOO_MANY_REQUESTS, Json(err));
    }

    let code = headers
        .get("X-Pairing-Code")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let pair_body = body.map(|Json(b)| b).unwrap_or_default();
    let username = pair_body.username;
    let password = pair_body.password;

    let request = crate::security::pairing::PairRequest {
        code,
        username: username.as_deref(),
        password: password.as_deref(),
    };

    match state.pairing.try_pair(&request) {
        Ok(Some(token)) => {
            tracing::info!("🔐 New device paired successfully");
            let body = serde_json::json!({
                "paired": true,
                "token": token,
                "message": "Device paired. Save this token — use it as Authorization: Bearer <token>"
            });
            (StatusCode::OK, Json(body))
        }
        Ok(None) => {
            let msg = if state.pairing.requires_credentials() {
                "Invalid credentials or pairing code"
            } else {
                "Invalid pairing code"
            };
            tracing::warn!("🔐 Pairing attempt failed");
            let err = serde_json::json!({"error": msg});
            (StatusCode::FORBIDDEN, Json(err))
        }
        Err(lockout_secs) => {
            tracing::warn!(
                "🔐 Pairing locked out — too many failed attempts ({lockout_secs}s remaining)"
            );
            let err = serde_json::json!({
                "error": format!("Too many failed attempts. Try again in {lockout_secs}s."),
                "retry_after": lockout_secs
            });
            (StatusCode::TOO_MANY_REQUESTS, Json(err))
        }
    }
}

/// Webhook request body
#[derive(serde::Deserialize)]
pub struct WebhookBody {
    pub message: String,
    /// Optional task category — when set, the agent selects tools for this mode.
    /// Values: "web_general", "document", "coding", "image", "music", "video", "translation".
    pub task_category: Option<String>,
}

/// POST /webhook — main webhook endpoint
async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<WebhookBody>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let client_key = client_key_from_headers(&headers);
    if !state.rate_limiter.allow_webhook(&client_key) {
        tracing::warn!("/webhook rate limit exceeded for key: {client_key}");
        let err = serde_json::json!({
            "error": "Too many webhook requests. Please retry later.",
            "retry_after": RATE_LIMIT_WINDOW_SECS,
        });
        return (StatusCode::TOO_MANY_REQUESTS, Json(err));
    }

    // ── Bearer token auth (pairing) ──
    if state.pairing.require_pairing() {
        let auth = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = auth.strip_prefix("Bearer ").unwrap_or("");
        if !state.pairing.is_authenticated(token) {
            tracing::warn!("Webhook: rejected — not paired / invalid bearer token");
            let err = serde_json::json!({
                "error": "Unauthorized — pair first via POST /pair, then send Authorization: Bearer <token>"
            });
            return (StatusCode::UNAUTHORIZED, Json(err));
        }
    }

    // ── Webhook secret auth (optional, additional layer) ──
    if let Some(ref secret_hash) = state.webhook_secret_hash {
        let header_hash = headers
            .get("X-Webhook-Secret")
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(hash_webhook_secret);
        match header_hash {
            Some(val) if constant_time_eq(&val, secret_hash.as_ref()) => {}
            _ => {
                tracing::warn!("Webhook: rejected request — invalid or missing X-Webhook-Secret");
                let err = serde_json::json!({"error": "Unauthorized — invalid or missing X-Webhook-Secret header"});
                return (StatusCode::UNAUTHORIZED, Json(err));
            }
        }
    }

    // ── Parse body ──
    let Json(webhook_body) = match body {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!("Webhook JSON parse error: {e}");
            let err = serde_json::json!({
                "error": "Invalid JSON body. Expected: {\"message\": \"...\"}"
            });
            return (StatusCode::BAD_REQUEST, Json(err));
        }
    };

    // ── Idempotency (optional) ──
    if let Some(idempotency_key) = headers
        .get("X-Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !state.idempotency_store.record_if_new(idempotency_key) {
            tracing::info!("Webhook duplicate ignored (idempotency key: {idempotency_key})");
            let body = serde_json::json!({
                "status": "duplicate",
                "idempotent": true,
                "message": "Request already processed for this idempotency key"
            });
            return (StatusCode::OK, Json(body));
        }
    }

    let message = &webhook_body.message;

    // ── Telemetry: record webhook interaction ──
    if let Some(ref store) = state.telemetry {
        let event = crate::telemetry::TelemetryEvent {
            id: 0,
            user_id: client_key.clone(),
            country: String::new(),
            ip_address: headers
                .get("x-forwarded-for")
                .or_else(|| headers.get("x-real-ip"))
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string(),
            channel: "webhook".into(),
            action: "message".into(),
            target_url: String::new(),
            details: crate::util::truncate_with_ellipsis(message, 200),
            tool_name: String::new(),
            alert_level: crate::telemetry::AlertLevel::None,
            alert_reason: String::new(),
            timestamp: chrono::Utc::now(),
        };
        if let Err(e) = store.record(event) {
            tracing::debug!("Telemetry record failed: {e}");
        }
    }

    if state.auto_save {
        let key = webhook_memory_key();
        let _ = state
            .mem
            .store(&key, message, MemoryCategory::Conversation, None)
            .await;
    }

    // ── SLM gatekeeper: try local handling first ──
    if let Some(ref gatekeeper) = state.gatekeeper {
        let gk: tokio::sync::MutexGuard<'_, crate::gatekeeper::GatekeeperRouter> =
            gatekeeper.lock().await;
        let result = gk.process_message(message).await;
        if let Some(local_response) = result.local_response {
            let body = serde_json::json!({
                "response": local_response,
                "model": gk.model(),
                "routed": "local",
                "category": format!("{:?}", result.decision.category),
            });
            return (StatusCode::OK, Json(body));
        }
        // Fall through to cloud LLM if gatekeeper didn't handle it.
        tracing::debug!(
            category = ?result.decision.category,
            reason = result.decision.reason,
            "Gatekeeper routed to cloud"
        );
    }

    // ── Agentic handling: use Agent with tools when available ──
    if let Some(ref agent) = state.agent {
        let mut agent = agent.lock().await;
        match agent.turn(message).await {
            Ok(response) => {
                let body = serde_json::json!({"response": response, "model": state.model});
                return (StatusCode::OK, Json(body));
            }
            Err(e) => {
                tracing::error!(
                    "Webhook agent error: {}",
                    providers::sanitize_api_error(&e.to_string())
                );
                let err = serde_json::json!({"error": "LLM request failed"});
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
            }
        }
    }

    // Fallback: simple chat without tools
    match state
        .provider
        .simple_chat(message, &state.model, state.temperature)
        .await
    {
        Ok(response) => {
            let body = serde_json::json!({"response": response, "model": state.model});
            (StatusCode::OK, Json(body))
        }
        Err(e) => {
            tracing::error!(
                "Webhook provider error: {}",
                providers::sanitize_api_error(&e.to_string())
            );
            let err = serde_json::json!({"error": "LLM request failed"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err))
        }
    }
}

/// `WhatsApp` verification query params
#[derive(serde::Deserialize)]
pub struct WhatsAppVerifyQuery {
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
}

/// GET /whatsapp — Meta webhook verification
async fn handle_whatsapp_verify(
    State(state): State<AppState>,
    Query(params): Query<WhatsAppVerifyQuery>,
) -> impl IntoResponse {
    let Some(ref wa) = state.whatsapp else {
        return (StatusCode::NOT_FOUND, "WhatsApp not configured".to_string());
    };

    // Verify the token matches (constant-time comparison to prevent timing attacks)
    let token_matches = params
        .verify_token
        .as_deref()
        .is_some_and(|t| constant_time_eq(t, wa.verify_token()));
    if params.mode.as_deref() == Some("subscribe") && token_matches {
        if let Some(ch) = params.challenge {
            tracing::info!("WhatsApp webhook verified successfully");
            return (StatusCode::OK, ch);
        }
        return (StatusCode::BAD_REQUEST, "Missing hub.challenge".to_string());
    }

    tracing::warn!("WhatsApp webhook verification failed — token mismatch");
    (StatusCode::FORBIDDEN, "Forbidden".to_string())
}

/// Verify `WhatsApp` webhook signature (`X-Hub-Signature-256`).
/// Returns true if the signature is valid, false otherwise.
/// See: <https://developers.facebook.com/docs/graph-api/webhooks/getting-started#verification-requests>
pub fn verify_whatsapp_signature(app_secret: &str, body: &[u8], signature_header: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    // Signature format: "sha256=<hex_signature>"
    let Some(hex_sig) = signature_header.strip_prefix("sha256=") else {
        return false;
    };

    // Decode hex signature
    let Ok(expected) = hex::decode(hex_sig) else {
        return false;
    };

    // Compute HMAC-SHA256
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(app_secret.as_bytes()) else {
        return false;
    };
    mac.update(body);

    // Constant-time comparison
    mac.verify_slice(&expected).is_ok()
}

/// POST /whatsapp — incoming message webhook
async fn handle_whatsapp_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let Some(ref wa) = state.whatsapp else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "WhatsApp not configured"})),
        );
    };

    // ── Security: Verify X-Hub-Signature-256 if app_secret is configured ──
    if let Some(ref app_secret) = state.whatsapp_app_secret {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !verify_whatsapp_signature(app_secret, &body, signature) {
            tracing::warn!(
                "WhatsApp webhook signature verification failed (signature: {})",
                if signature.is_empty() {
                    "missing"
                } else {
                    "invalid"
                }
            );
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid signature"})),
            );
        }
    }

    // Parse JSON body
    let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid JSON payload"})),
        );
    };

    // Send one-click connect links to unauthorized, unpaired senders
    let unpaired = wa.extract_unpaired_senders(&payload);
    if let Some(ref cp) = state.channel_pairing {
        for sender in &unpaired {
            let token = cp.create_token("whatsapp", sender);
            let auto_url =
                crate::channels::pairing::ChannelPairingStore::auto_pair_url(&state.gateway_base_url, &token);
            let _ = wa
                .send(&SendMessage::new(
                    &format!(
                        "🔗 MoA에 연결하려면 아래 링크를 클릭하세요.\nTap the link below to connect to MoA.\n\n{auto_url}"
                    ),
                    sender,
                ))
                .await;
        }
    }

    // Parse messages from the webhook payload
    let messages = wa.parse_webhook_payload(&payload);

    if messages.is_empty() {
        // Acknowledge the webhook even if no messages (could be status updates)
        return (StatusCode::OK, Json(serde_json::json!({"status": "ok"})));
    }

    // Process each message
    for msg in &messages {
        tracing::info!(
            "WhatsApp message from {}: {}",
            msg.sender,
            truncate_with_ellipsis(&msg.content, 50)
        );

        // Auto-save to memory
        if state.auto_save {
            let key = whatsapp_memory_key(msg);
            let _ = state
                .mem
                .store(&key, &msg.content, MemoryCategory::Conversation, None)
                .await;
        }

        // Call the LLM
        match state
            .provider
            .simple_chat(&msg.content, &state.model, state.temperature)
            .await
        {
            Ok(response) => {
                // Send reply via WhatsApp
                if let Err(e) = wa
                    .send(&SendMessage::new(response, &msg.reply_target))
                    .await
                {
                    tracing::error!("Failed to send WhatsApp reply: {e}");
                }
            }
            Err(e) => {
                tracing::error!("LLM error for WhatsApp message: {e:#}");
                let _ = wa
                    .send(&SendMessage::new(
                        "Sorry, I couldn't process your message right now.",
                        &msg.reply_target,
                    ))
                    .await;
            }
        }
    }

    // Acknowledge the webhook
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

/// POST /line — incoming LINE webhook
async fn handle_line_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let Some(ref line) = state.line else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "LINE not configured"})),
        );
    };

    // Verify X-Line-Signature
    let signature = headers
        .get("X-Line-Signature")
        .or_else(|| headers.get("x-line-signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !line.verify_signature(&body, signature) {
        tracing::warn!(
            "LINE webhook signature verification failed (signature: {})",
            if signature.is_empty() { "missing" } else { "invalid" }
        );
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid signature"})),
        );
    }

    // Parse JSON body
    let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&body) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid JSON payload"})),
        );
    };

    // Parse messages and unpaired senders
    let (messages, unpaired) = line.parse_webhook_payload(&payload);

    // Send one-click connect links to unpaired senders
    if let Some(ref cp) = state.channel_pairing {
        for sender in &unpaired {
            let token = cp.create_token("line", sender);
            let auto_url = crate::channels::pairing::ChannelPairingStore::auto_pair_url(
                &state.gateway_base_url,
                &token,
            );
            let _ = line
                .send(&SendMessage::new(
                    &format!(
                        "🔗 MoA에 연결하려면 아래 링크를 클릭하세요.\nTap the link below to connect to MoA.\n\n{auto_url}"
                    ),
                    sender,
                ))
                .await;
        }
    }

    if messages.is_empty() {
        return (StatusCode::OK, Json(serde_json::json!({"status": "ok"})));
    }

    // Process each message
    for msg in &messages {
        tracing::info!(
            "LINE message from {}: {}",
            msg.sender,
            truncate_with_ellipsis(&msg.content, 50)
        );

        // Auto-save to memory
        if state.auto_save {
            let key = format!(
                "line_msg_{}_{}",
                msg.sender,
                msg.timestamp
            );
            let _ = state
                .mem
                .store(&key, &msg.content, MemoryCategory::Conversation, None)
                .await;
        }

        // Call the LLM
        match state
            .provider
            .simple_chat(&msg.content, &state.model, state.temperature)
            .await
        {
            Ok(response) => {
                if let Err(e) = line
                    .send(&SendMessage::new(response, &msg.reply_target))
                    .await
                {
                    tracing::error!("Failed to send LINE reply: {e}");
                }
            }
            Err(e) => {
                tracing::error!("LLM error for LINE message: {e:#}");
                let _ = line
                    .send(&SendMessage::new(
                        "Sorry, I couldn't process your message right now.",
                        &msg.reply_target,
                    ))
                    .await;
            }
        }
    }

    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

// ══════════════════════════════════════════════════════════════════════════════
// TELEMETRY HANDLERS (admin-only)
// ══════════════════════════════════════════════════════════════════════════════

/// Authenticate admin requests by checking the `Authorization: Bearer <token>` header
/// against the SHA-256-hashed admin token stored in state.
fn authenticate_admin(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(ref expected_hash) = state.telemetry_admin_token_hash else {
        return false;
    };
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .strip_prefix("Bearer ")
        .unwrap_or("");

    if token.is_empty() {
        return false;
    }

    use sha2::{Digest, Sha256};
    let provided_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
    crate::security::pairing::constant_time_eq(&provided_hash, expected_hash.as_ref())
}

// ══════════════════════════════════════════════════════════════════════════════
// AUTH HANDLERS
// ══════════════════════════════════════════════════════════════════════════════

/// Concrete return type for auth handlers (avoids `impl IntoResponse` inference issues).
type AuthResponse = (StatusCode, Json<serde_json::Value>);

/// Request body for user registration.
#[derive(GatewayDeserialize)]
struct AuthRegisterBody {
    username: String,
    password: String,
}

/// Request body for login.
#[derive(GatewayDeserialize)]
struct AuthLoginBody {
    username: String,
    password: String,
    device_id: Option<String>,
    device_name: Option<String>,
}

/// Request body for device registration.
#[derive(GatewayDeserialize)]
struct AuthDeviceRegisterBody {
    device_id: String,
    device_name: String,
    platform: Option<String>,
}

/// Extract bearer token from Authorization header.
fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

/// Validate a session token and return the session. Returns error response if invalid.
fn require_auth_session(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::auth::Session, (StatusCode, Json<serde_json::Value>)> {
    let auth_store = state.auth_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "Auth not enabled"})),
        )
    })?;

    let token = extract_bearer_token(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization header"})),
        )
    })?;

    auth_store.validate_session(token).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired session token"})),
        )
    })
}

/// POST /api/auth/register — create a new user account.
async fn handle_auth_register(
    State(state): State<AppState>,
    body: Result<Json<AuthRegisterBody>, axum::extract::rejection::JsonRejection>,
) -> AuthResponse {
    let auth_store = match state.auth_store.as_ref() {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Auth not enabled"})),
            );
        }
    };

    if !state.auth_allow_registration {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Registration is disabled"})),
        );
    }

    // Enforce max_users limit (0 = unlimited)
    if state.auth_max_users > 0 {
        if let Ok(count) = auth_store.user_count() {
            if count >= state.auth_max_users {
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({"error": "Maximum user limit reached"})),
                );
            }
        }
    }

    let body = match body {
        Ok(Json(b)) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid request: {e}")})),
            );
        }
    };

    match auth_store.register(&body.username, &body.password) {
        Ok(user_id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "status": "registered",
                "user_id": user_id,
            })),
        ),
        Err(e) => {
            let msg = e.to_string();
            let status = if msg.contains("already taken") {
                StatusCode::CONFLICT
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(serde_json::json!({"error": msg})))
        }
    }
}

/// POST /api/auth/login — authenticate and get a session token.
async fn handle_auth_login(
    State(state): State<AppState>,
    body: Result<Json<AuthLoginBody>, axum::extract::rejection::JsonRejection>,
) -> AuthResponse {
    let auth_store = match state.auth_store.as_ref() {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Auth not enabled"})),
            );
        }
    };

    let body = match body {
        Ok(Json(b)) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid request: {e}")})),
            );
        }
    };

    let user = match auth_store.authenticate(&body.username, &body.password) {
        Ok(u) => u,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid username or password"})),
            );
        }
    };

    match auth_store.create_session(
        &user.id,
        body.device_id.as_deref(),
        body.device_name.as_deref(),
    ) {
        Ok(token) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "authenticated",
                "token": token,
                "user_id": user.id,
                "username": user.username,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Session creation failed: {e}")})),
        ),
    }
}

/// POST /api/auth/logout — revoke current session.
async fn handle_auth_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AuthResponse {
    let auth_store = match state.auth_store.as_ref() {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Auth not enabled"})),
            );
        }
    };

    let token = match extract_bearer_token(&headers) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing Authorization header"})),
            );
        }
    };

    match auth_store.revoke_session(token) {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "logged_out"})),
        ),
        Ok(false) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid session"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Logout failed: {e}")})),
        ),
    }
}

/// GET /api/auth/me — get current user info from session token.
async fn handle_auth_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AuthResponse {
    let session = match require_auth_session(&state, &headers) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let auth_store = state.auth_store.as_ref().unwrap();
    match auth_store.get_user(&session.user_id) {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "user_id": user.id,
                "username": user.username,
                "device_id": session.device_id,
                "device_name": session.device_name,
            })),
        ),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        ),
    }
}

/// GET /api/auth/devices — list devices for authenticated user.
async fn handle_auth_devices_list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AuthResponse {
    let session = match require_auth_session(&state, &headers) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let auth_store = state.auth_store.as_ref().unwrap();
    match auth_store.list_devices(&session.user_id) {
        Ok(devices) => {
            let list: Vec<_> = devices
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "device_id": d.device_id,
                        "device_name": d.device_name,
                        "last_seen": d.last_seen,
                    })
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({"devices": list})))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to list devices: {e}")})),
        ),
    }
}

/// POST /api/auth/devices — register a device for authenticated user.
async fn handle_auth_device_register(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<AuthDeviceRegisterBody>, axum::extract::rejection::JsonRejection>,
) -> AuthResponse {
    let session = match require_auth_session(&state, &headers) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let body = match body {
        Ok(Json(b)) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid request: {e}")})),
            );
        }
    };

    let auth_store = state.auth_store.as_ref().unwrap();

    // Enforce max_devices_per_user limit
    if let Ok(devices) = auth_store.list_devices(&session.user_id) {
        // Only enforce if this is a new device (not an update of an existing one)
        let is_existing = devices.iter().any(|d| d.device_id == body.device_id);
        if !is_existing && devices.len() >= state.auth_max_devices_per_user as usize {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": format!("Maximum devices per user ({}) reached", state.auth_max_devices_per_user),
                })),
            );
        }
    }

    match auth_store.register_device(
        &session.user_id,
        &body.device_id,
        &body.device_name,
        body.platform.as_deref(),
    ) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "status": "device_registered",
                "device_id": body.device_id,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Device registration failed: {e}")})),
        ),
    }
}

/// DELETE /api/auth/devices/:device_id — remove a device.
async fn handle_auth_device_remove(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(device_id): axum::extract::Path<String>,
) -> AuthResponse {
    let session = match require_auth_session(&state, &headers) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let auth_store = state.auth_store.as_ref().unwrap();
    match auth_store.remove_device(&session.user_id, &device_id) {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "device_removed"})),
        ),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Device not found"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Device removal failed: {e}")})),
        ),
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// SYNC HANDLERS
// ══════════════════════════════════════════════════════════════════════════════

/// Request body for relay upload.
#[derive(GatewayDeserialize)]
struct SyncRelayUploadBody {
    encrypted_payload: String,
    nonce: String,
}

/// Query params for relay pickup.
#[derive(GatewayDeserialize)]
struct SyncRelayPickupQuery {
    device_id: Option<String>,
}

/// GET /sync — WebSocket upgrade for sync broadcast channel.
///
/// Once connected, clients receive all broadcast messages from peers.
/// Clients can also send messages that get broadcast to all other
/// connected WebSocket clients. The server does NOT store any messages.
async fn handle_sync_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: axum::extract::WebSocketUpgrade,
) -> impl IntoResponse {
    // Authenticate via session token (query param or header)
    let session = match require_auth_session(&state, &headers) {
        Ok(s) => s,
        Err((status, json)) => {
            return (status, json.0.to_string()).into_response();
        }
    };

    let broadcast_tx = match state.sync_broadcast.as_ref() {
        Some(tx) => tx.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Sync not enabled",
            )
                .into_response();
        }
    };

    let device_id = session.device_id.clone().unwrap_or_default();
    let user_id = session.user_id.clone();

    ws.on_upgrade(move |socket| {
        handle_sync_ws_connection(socket, broadcast_tx, device_id, user_id)
    })
}

/// Handle a single WebSocket sync connection.
async fn handle_sync_ws_connection(
    socket: axum::extract::ws::WebSocket,
    broadcast_tx: tokio::sync::broadcast::Sender<String>,
    device_id: String,
    _user_id: String,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut broadcast_rx = broadcast_tx.subscribe();

    // Spawn task to forward broadcast messages to this client
    let device_id_clone = device_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            // Don't echo messages back to the sender.
            // Parse JSON to reliably check from_device_id regardless of field order.
            let is_own_message = serde_json::from_str::<serde_json::Value>(&msg)
                .ok()
                .and_then(|v| v.get("from_device_id")?.as_str().map(String::from))
                .is_some_and(|id| id == device_id_clone);
            if is_own_message {
                continue;
            }
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Receive messages from this client and broadcast to all peers
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                let _ = broadcast_tx.send(text.to_string());
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
    tracing::debug!(device_id, "Sync WebSocket disconnected");
}

/// POST /api/sync/relay — upload encrypted data to the temporary relay.
async fn handle_sync_relay_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<SyncRelayUploadBody>, axum::extract::rejection::JsonRejection>,
) -> AuthResponse {
    let session = match require_auth_session(&state, &headers) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let relay = match state.sync_relay.as_ref() {
        Some(r) => r,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Sync not enabled"})),
            );
        }
    };

    let body = match body {
        Ok(Json(b)) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("Invalid request: {e}")})),
            );
        }
    };

    let entry_id = uuid::Uuid::new_v4().to_string();
    let device_id = session.device_id.clone().unwrap_or_default();

    let entry = crate::sync::relay::RelayEntry {
        id: entry_id.clone(),
        sender_device_id: device_id.clone(),
        user_id: session.user_id.clone(),
        encrypted_payload: body.encrypted_payload,
        nonce: body.nonce,
        created_at_epoch: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };

    relay.store(entry);

    // Notify connected peers via broadcast
    if let Some(ref tx) = state.sync_broadcast {
        let notify = serde_json::json!({
            "type": "relay_notify",
            "from_device_id": device_id,
            "relay_ids": [entry_id],
        });
        let _ = tx.send(notify.to_string());
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "stored",
            "relay_id": entry_id,
        })),
    )
}

/// GET /api/sync/relay — pick up pending relay entries.
async fn handle_sync_relay_pickup(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: Query<SyncRelayPickupQuery>,
) -> AuthResponse {
    let session = match require_auth_session(&state, &headers) {
        Ok(s) => s,
        Err(resp) => return resp,
    };

    let relay = match state.sync_relay.as_ref() {
        Some(r) => r,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Sync not enabled"})),
            );
        }
    };

    let exclude_device = query.device_id.as_deref().or(session.device_id.as_deref());
    let entries = relay.pickup(&session.user_id, exclude_device);

    let list: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "sender_device_id": e.sender_device_id,
                "encrypted_payload": e.encrypted_payload,
                "nonce": e.nonce,
                "created_at_epoch": e.created_at_epoch,
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "entries": list,
            "count": list.len(),
        })),
    )
}

// ══════════════════════════════════════════════════════════════════════════════
// TELEMETRY HANDLERS
// ══════════════════════════════════════════════════════════════════════════════

/// POST /api/telemetry/events — ingest telemetry events from app clients.
/// Requires bearer token auth (pairing or admin token).
async fn handle_telemetry_ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(event): Json<crate::telemetry::TelemetryEvent>,
) -> impl IntoResponse {
    let Some(ref store) = state.telemetry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "telemetry not enabled"})),
        );
    };

    // Accept events from paired clients or admin
    let is_paired = {
        let auth = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = auth.strip_prefix("Bearer ").unwrap_or("");
        state.pairing.is_authenticated(token)
    };
    let is_admin = authenticate_admin(&state, &headers);

    if !is_paired && !is_admin {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "unauthorized"})),
        );
    }

    // Enrich event with IP-derived country if not already set
    let mut enriched = event;
    if enriched.ip_address.is_empty() {
        enriched.ip_address = headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
    }

    match store.record(enriched) {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "recorded"})),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("failed to record: {e}")})),
        ),
    }
}

/// GET /api/admin/telemetry/events — query telemetry events (admin only).
/// Query params: user_id, country, channel, action, alert_level, since, until, search, limit, offset
async fn handle_admin_telemetry_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(query): axum::extract::Query<crate::telemetry::TelemetryQuery>,
) -> impl IntoResponse {
    if !authenticate_admin(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "admin authentication required"})),
        );
    }

    let Some(ref store) = state.telemetry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "telemetry not enabled"})),
        );
    };

    match store.query(&query) {
        Ok(events) => (StatusCode::OK, Json(serde_json::json!({"events": events}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("query failed: {e}")})),
        ),
    }
}

/// GET /api/admin/telemetry/summary — dashboard summary (admin only).
async fn handle_admin_telemetry_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authenticate_admin(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "admin authentication required"})),
        );
    }

    let Some(ref store) = state.telemetry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "telemetry not enabled"})),
        );
    };

    match store.summary() {
        Ok(summary) => (StatusCode::OK, Json(serde_json::json!(summary))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("summary failed: {e}")})),
        ),
    }
}

/// GET /api/admin/telemetry/alerts — recent suspicious activity alerts (admin only).
async fn handle_admin_telemetry_alerts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authenticate_admin(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "admin authentication required"})),
        );
    }

    let Some(ref store) = state.telemetry else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "telemetry not enabled"})),
        );
    };

    match store.pending_alerts() {
        Ok(alerts) => (StatusCode::OK, Json(serde_json::json!({"alerts": alerts}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("alerts query failed: {e}")})),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::traits::ChannelMessage;
    use crate::memory::{Memory, MemoryCategory, MemoryEntry};
    use crate::providers::Provider;
    use async_trait::async_trait;
    use axum::http::HeaderValue;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;
    use parking_lot::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn security_body_limit_is_64kb() {
        assert_eq!(MAX_BODY_SIZE, 65_536);
    }

    #[test]
    fn security_timeout_is_120_seconds() {
        assert_eq!(REQUEST_TIMEOUT_SECS, 120);
    }

    #[test]
    fn webhook_body_requires_message_field() {
        let valid = r#"{"message": "hello"}"#;
        let parsed: Result<WebhookBody, _> = serde_json::from_str(valid);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap().message, "hello");

        let missing = r#"{"other": "field"}"#;
        let parsed: Result<WebhookBody, _> = serde_json::from_str(missing);
        assert!(parsed.is_err());
    }

    #[test]
    fn whatsapp_query_fields_are_optional() {
        let q = WhatsAppVerifyQuery {
            mode: None,
            verify_token: None,
            challenge: None,
        };
        assert!(q.mode.is_none());
    }

    #[test]
    fn app_state_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<AppState>();
    }

    #[test]
    fn gateway_rate_limiter_blocks_after_limit() {
        let limiter = GatewayRateLimiter::new(2, 2);
        assert!(limiter.allow_pair("127.0.0.1"));
        assert!(limiter.allow_pair("127.0.0.1"));
        assert!(!limiter.allow_pair("127.0.0.1"));
    }

    #[test]
    fn rate_limiter_sweep_removes_stale_entries() {
        let limiter = SlidingWindowRateLimiter::new(10, Duration::from_secs(60));
        // Add entries for multiple IPs
        assert!(limiter.allow("ip-1"));
        assert!(limiter.allow("ip-2"));
        assert!(limiter.allow("ip-3"));

        {
            let guard = limiter.requests.lock();
            assert_eq!(guard.0.len(), 3);
        }

        // Force a sweep by backdating last_sweep
        {
            let mut guard = limiter.requests.lock();
            guard.1 = Instant::now()
                .checked_sub(Duration::from_secs(RATE_LIMITER_SWEEP_INTERVAL_SECS + 1))
                .unwrap();
            // Clear timestamps for ip-2 and ip-3 to simulate stale entries
            guard.0.get_mut("ip-2").unwrap().clear();
            guard.0.get_mut("ip-3").unwrap().clear();
        }

        // Next allow() call should trigger sweep and remove stale entries
        assert!(limiter.allow("ip-1"));

        {
            let guard = limiter.requests.lock();
            assert_eq!(guard.0.len(), 1, "Stale entries should have been swept");
            assert!(guard.0.contains_key("ip-1"));
        }
    }

    #[test]
    fn rate_limiter_zero_limit_always_allows() {
        let limiter = SlidingWindowRateLimiter::new(0, Duration::from_secs(60));
        for _ in 0..100 {
            assert!(limiter.allow("any-key"));
        }
    }

    #[test]
    fn idempotency_store_rejects_duplicate_key() {
        let store = IdempotencyStore::new(Duration::from_secs(30));
        assert!(store.record_if_new("req-1"));
        assert!(!store.record_if_new("req-1"));
        assert!(store.record_if_new("req-2"));
    }

    #[test]
    fn webhook_memory_key_is_unique() {
        let key1 = webhook_memory_key();
        let key2 = webhook_memory_key();

        assert!(key1.starts_with("webhook_msg_"));
        assert!(key2.starts_with("webhook_msg_"));
        assert_ne!(key1, key2);
    }

    #[test]
    fn whatsapp_memory_key_includes_sender_and_message_id() {
        let msg = ChannelMessage {
            id: "wamid-123".into(),
            sender: "+1234567890".into(),
            reply_target: "+1234567890".into(),
            content: "hello".into(),
            channel: "whatsapp".into(),
            timestamp: 1,
        };

        let key = whatsapp_memory_key(&msg);
        assert_eq!(key, "whatsapp_+1234567890_wamid-123");
    }

    #[derive(Default)]
    struct MockMemory;

    #[async_trait]
    impl Memory for MockMemory {
        fn name(&self) -> &str {
            "mock"
        }

        async fn store(
            &self,
            _key: &str,
            _content: &str,
            _category: MemoryCategory,
            _session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn recall(
            &self,
            _query: &str,
            _limit: usize,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(false)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(0)
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[derive(Default)]
    struct MockProvider {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok("ok".into())
        }
    }

    #[derive(Default)]
    struct TrackingMemory {
        keys: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl Memory for TrackingMemory {
        fn name(&self) -> &str {
            "tracking"
        }

        async fn store(
            &self,
            key: &str,
            _content: &str,
            _category: MemoryCategory,
            _session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            self.keys.lock().push(key.to_string());
            Ok(())
        }

        async fn recall(
            &self,
            _query: &str,
            _limit: usize,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(false)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            let size = self.keys.lock().len();
            Ok(size)
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn webhook_idempotency_skips_duplicate_provider_calls() {
        let provider_impl = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();
        let memory: Arc<dyn Memory> = Arc::new(MockMemory);

        let state = AppState {
            provider,
            model: "test-model".into(),
            temperature: 0.0,
            mem: memory,
            auto_save: false,
            webhook_secret_hash: None,
            pairing: Arc::new(PairingGuard::new(false, &[], None, None)),
            rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100)),
            idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300))),
            whatsapp: None,
            line: None,
            whatsapp_app_secret: None,
            gatekeeper: None,
            agent: None,
            telemetry: None,
            telemetry_admin_token_hash: None,
            auth_store: None,
            auth_allow_registration: false,
            auth_max_users: 0,
            auth_max_devices_per_user: 10,
            sync_relay: None,
            sync_broadcast: None,
            channel_pairing: None,
            gateway_base_url: "http://127.0.0.1:3000".into(),
        };

        let mut headers = HeaderMap::new();
        headers.insert("X-Idempotency-Key", HeaderValue::from_static("abc-123"));

        let body = Ok(Json(WebhookBody {
            task_category: None,
            message: "hello".into(),
        }));
        let first = handle_webhook(State(state.clone()), headers.clone(), body)
            .await
            .into_response();
        assert_eq!(first.status(), StatusCode::OK);

        let body = Ok(Json(WebhookBody {
            task_category: None,
            message: "hello".into(),
        }));
        let second = handle_webhook(State(state), headers, body)
            .await
            .into_response();
        assert_eq!(second.status(), StatusCode::OK);

        let payload = second.into_body().collect().await.unwrap().to_bytes();
        let parsed: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(parsed["status"], "duplicate");
        assert_eq!(parsed["idempotent"], true);
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn webhook_autosave_stores_distinct_keys_per_request() {
        let provider_impl = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();

        let tracking_impl = Arc::new(TrackingMemory::default());
        let memory: Arc<dyn Memory> = tracking_impl.clone();

        let state = AppState {
            provider,
            model: "test-model".into(),
            temperature: 0.0,
            mem: memory,
            auto_save: true,
            webhook_secret_hash: None,
            pairing: Arc::new(PairingGuard::new(false, &[], None, None)),
            rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100)),
            idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300))),
            whatsapp: None,
            line: None,
            whatsapp_app_secret: None,
            gatekeeper: None,
            agent: None,
            telemetry: None,
            telemetry_admin_token_hash: None,
            auth_store: None,
            auth_allow_registration: false,
            auth_max_users: 0,
            auth_max_devices_per_user: 10,
            sync_relay: None,
            sync_broadcast: None,
            channel_pairing: None,
            gateway_base_url: "http://127.0.0.1:3000".into(),
        };

        let headers = HeaderMap::new();

        let body1 = Ok(Json(WebhookBody {
            task_category: None,
            message: "hello one".into(),
        }));
        let first = handle_webhook(State(state.clone()), headers.clone(), body1)
            .await
            .into_response();
        assert_eq!(first.status(), StatusCode::OK);

        let body2 = Ok(Json(WebhookBody {
            task_category: None,
            message: "hello two".into(),
        }));
        let second = handle_webhook(State(state), headers, body2)
            .await
            .into_response();
        assert_eq!(second.status(), StatusCode::OK);

        let keys = tracking_impl.keys.lock().clone();
        assert_eq!(keys.len(), 2);
        assert_ne!(keys[0], keys[1]);
        assert!(keys[0].starts_with("webhook_msg_"));
        assert!(keys[1].starts_with("webhook_msg_"));
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn webhook_secret_hash_is_deterministic_and_nonempty() {
        let one = hash_webhook_secret("secret-value");
        let two = hash_webhook_secret("secret-value");
        let other = hash_webhook_secret("other-value");

        assert_eq!(one, two);
        assert_ne!(one, other);
        assert_eq!(one.len(), 64);
    }

    #[tokio::test]
    async fn webhook_secret_hash_rejects_missing_header() {
        let provider_impl = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();
        let memory: Arc<dyn Memory> = Arc::new(MockMemory);

        let state = AppState {
            provider,
            model: "test-model".into(),
            temperature: 0.0,
            mem: memory,
            auto_save: false,
            webhook_secret_hash: Some(Arc::from(hash_webhook_secret("super-secret"))),
            pairing: Arc::new(PairingGuard::new(false, &[], None, None)),
            rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100)),
            idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300))),
            whatsapp: None,
            line: None,
            whatsapp_app_secret: None,
            gatekeeper: None,
            agent: None,
            telemetry: None,
            telemetry_admin_token_hash: None,
            auth_store: None,
            auth_allow_registration: false,
            auth_max_users: 0,
            auth_max_devices_per_user: 10,
            sync_relay: None,
            sync_broadcast: None,
            channel_pairing: None,
            gateway_base_url: "http://127.0.0.1:3000".into(),
        };

        let response = handle_webhook(
            State(state),
            HeaderMap::new(),
            Ok(Json(WebhookBody {
                task_category: None,
                message: "hello".into(),
            })),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn webhook_secret_hash_rejects_invalid_header() {
        let provider_impl = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();
        let memory: Arc<dyn Memory> = Arc::new(MockMemory);

        let state = AppState {
            provider,
            model: "test-model".into(),
            temperature: 0.0,
            mem: memory,
            auto_save: false,
            webhook_secret_hash: Some(Arc::from(hash_webhook_secret("super-secret"))),
            pairing: Arc::new(PairingGuard::new(false, &[], None, None)),
            rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100)),
            idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300))),
            whatsapp: None,
            line: None,
            whatsapp_app_secret: None,
            gatekeeper: None,
            agent: None,
            telemetry: None,
            telemetry_admin_token_hash: None,
            auth_store: None,
            auth_allow_registration: false,
            auth_max_users: 0,
            auth_max_devices_per_user: 10,
            sync_relay: None,
            sync_broadcast: None,
            channel_pairing: None,
            gateway_base_url: "http://127.0.0.1:3000".into(),
        };

        let mut headers = HeaderMap::new();
        headers.insert("X-Webhook-Secret", HeaderValue::from_static("wrong-secret"));

        let response = handle_webhook(
            State(state),
            headers,
            Ok(Json(WebhookBody {
                task_category: None,
                message: "hello".into(),
            })),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn webhook_secret_hash_accepts_valid_header() {
        let provider_impl = Arc::new(MockProvider::default());
        let provider: Arc<dyn Provider> = provider_impl.clone();
        let memory: Arc<dyn Memory> = Arc::new(MockMemory);

        let state = AppState {
            provider,
            model: "test-model".into(),
            temperature: 0.0,
            mem: memory,
            auto_save: false,
            webhook_secret_hash: Some(Arc::from(hash_webhook_secret("super-secret"))),
            pairing: Arc::new(PairingGuard::new(false, &[], None, None)),
            rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100)),
            idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300))),
            whatsapp: None,
            line: None,
            whatsapp_app_secret: None,
            gatekeeper: None,
            agent: None,
            telemetry: None,
            telemetry_admin_token_hash: None,
            auth_store: None,
            auth_allow_registration: false,
            auth_max_users: 0,
            auth_max_devices_per_user: 10,
            sync_relay: None,
            sync_broadcast: None,
            channel_pairing: None,
            gateway_base_url: "http://127.0.0.1:3000".into(),
        };

        let mut headers = HeaderMap::new();
        headers.insert("X-Webhook-Secret", HeaderValue::from_static("super-secret"));

        let response = handle_webhook(
            State(state),
            headers,
            Ok(Json(WebhookBody {
                task_category: None,
                message: "hello".into(),
            })),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(provider_impl.calls.load(Ordering::SeqCst), 1);
    }

    // ══════════════════════════════════════════════════════════
    // WhatsApp Signature Verification Tests (CWE-345 Prevention)
    // ══════════════════════════════════════════════════════════

    fn compute_whatsapp_signature_hex(secret: &str, body: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        hex::encode(mac.finalize().into_bytes())
    }

    fn compute_whatsapp_signature_header(secret: &str, body: &[u8]) -> String {
        format!("sha256={}", compute_whatsapp_signature_hex(secret, body))
    }

    #[test]
    fn whatsapp_signature_valid() {
        // Test with known values
        let app_secret = "test_secret_key_12345";
        let body = b"test body content";

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_invalid_wrong_secret() {
        let app_secret = "correct_secret_key_abc";
        let wrong_secret = "wrong_secret_key_xyz";
        let body = b"test body content";

        let signature_header = compute_whatsapp_signature_header(wrong_secret, body);

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_invalid_wrong_body() {
        let app_secret = "test_secret_key_12345";
        let original_body = b"original body";
        let tampered_body = b"tampered body";

        let signature_header = compute_whatsapp_signature_header(app_secret, original_body);

        // Verify with tampered body should fail
        assert!(!verify_whatsapp_signature(
            app_secret,
            tampered_body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_missing_prefix() {
        let app_secret = "test_secret_key_12345";
        let body = b"test body";

        // Signature without "sha256=" prefix
        let signature_header = "abc123def456";

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_empty_header() {
        let app_secret = "test_secret_key_12345";
        let body = b"test body";

        assert!(!verify_whatsapp_signature(app_secret, body, ""));
    }

    #[test]
    fn whatsapp_signature_invalid_hex() {
        let app_secret = "test_secret_key_12345";
        let body = b"test body";

        // Invalid hex characters
        let signature_header = "sha256=not_valid_hex_zzz";

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_empty_body() {
        let app_secret = "test_secret_key_12345";
        let body = b"";

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_unicode_body() {
        let app_secret = "test_secret_key_12345";
        let body = "Hello 🦀 World".as_bytes();

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_json_payload() {
        let app_secret = "test_app_secret_key_xyz";
        let body = br#"{"entry":[{"changes":[{"value":{"messages":[{"from":"1234567890","text":{"body":"Hello"}}]}}]}]}"#;

        let signature_header = compute_whatsapp_signature_header(app_secret, body);

        assert!(verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_case_sensitive_prefix() {
        let app_secret = "test_secret_key_12345";
        let body = b"test body";

        let hex_sig = compute_whatsapp_signature_hex(app_secret, body);

        // Wrong case prefix should fail
        let wrong_prefix = format!("SHA256={hex_sig}");
        assert!(!verify_whatsapp_signature(app_secret, body, &wrong_prefix));

        // Correct prefix should pass
        let correct_prefix = format!("sha256={hex_sig}");
        assert!(verify_whatsapp_signature(app_secret, body, &correct_prefix));
    }

    #[test]
    fn whatsapp_signature_truncated_hex() {
        let app_secret = "test_secret_key_12345";
        let body = b"test body";

        let hex_sig = compute_whatsapp_signature_hex(app_secret, body);
        let truncated = &hex_sig[..32]; // Only half the signature
        let signature_header = format!("sha256={truncated}");

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }

    #[test]
    fn whatsapp_signature_extra_bytes() {
        let app_secret = "test_secret_key_12345";
        let body = b"test body";

        let hex_sig = compute_whatsapp_signature_hex(app_secret, body);
        let extended = format!("{hex_sig}deadbeef");
        let signature_header = format!("sha256={extended}");

        assert!(!verify_whatsapp_signature(
            app_secret,
            body,
            &signature_header
        ));
    }
}
