use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

// ── State ────────────────────────────────────────────────────────

/// Shared application state for Tauri commands.
struct AppState {
    server_url: std::sync::Mutex<String>,
    token: std::sync::Mutex<Option<String>>,
    /// Whether sync WebSocket is currently connected.
    sync_connected: AtomicBool,
    /// Flag to signal sync task to stop.
    sync_stop: AtomicBool,
    /// App data directory (platform-specific).
    data_dir: PathBuf,
}

// ── Types ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
struct ChatResponse {
    response: String,
    model: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct PairResponse {
    paired: bool,
    token: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SyncStatus {
    connected: bool,
    device_id: String,
    last_sync: Option<u64>,
}

// ── Tauri Commands ───────────────────────────────────────────────

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to MoA.", name)
}

/// Send a chat message to the MoA gateway and return the response.
#[tauri::command]
async fn chat(
    message: String,
    state: tauri::State<'_, AppState>,
) -> Result<ChatResponse, String> {
    let server_url = state.server_url.lock().map_err(|e| e.to_string())?.clone();
    let token = state
        .token
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "Not authenticated. Please pair first.".to_string())?;

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{server_url}/webhook"))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({ "message": message }))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if res.status() == 401 {
        *state.token.lock().map_err(|e| e.to_string())? = None;
        return Err("Authentication expired. Please re-pair.".to_string());
    }

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Chat failed ({status}): {text}"));
    }

    res.json::<ChatResponse>()
        .await
        .map_err(|e| format!("Invalid response: {e}"))
}

/// Pair with a MoA gateway server using credentials and/or pairing code.
///
/// Supports three modes:
/// - Credentials only (username + password) — auto-connect
/// - Pairing code only — legacy Bluetooth-style flow
/// - Both credentials and code
#[tauri::command]
async fn pair(
    username: Option<String>,
    password: Option<String>,
    code: Option<String>,
    server_url: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<PairResponse, String> {
    if let Some(url) = server_url {
        *state.server_url.lock().map_err(|e| e.to_string())? = url;
    }

    let url = state.server_url.lock().map_err(|e| e.to_string())?.clone();

    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{url}/pair"))
        .header("Content-Type", "application/json");

    if let Some(ref code) = code {
        req = req.header("X-Pairing-Code", code);
    }

    // Build body with credentials
    let mut body = serde_json::Map::new();
    if let Some(ref u) = username {
        body.insert("username".into(), serde_json::Value::String(u.clone()));
    }
    if let Some(ref p) = password {
        body.insert("password".into(), serde_json::Value::String(p.clone()));
    }

    if !body.is_empty() {
        req = req.json(&serde_json::Value::Object(body));
    }

    let res = req
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Pairing failed ({status}): {text}"));
    }

    let data: PairResponse = res
        .json()
        .await
        .map_err(|e| format!("Invalid response: {e}"))?;

    if data.paired {
        *state.token.lock().map_err(|e| e.to_string())? = Some(data.token.clone());
        // Persist token to data dir
        let token_path = state.data_dir.join("session_token");
        let _ = std::fs::write(&token_path, &data.token);
    }

    Ok(data)
}

/// Login via /api/auth/login — the new multi-user auth flow.
#[tauri::command]
async fn auth_login(
    username: String,
    password: String,
    device_id: Option<String>,
    device_name: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let url = state.server_url.lock().map_err(|e| e.to_string())?.clone();

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{url}/api/auth/login"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "username": username,
            "password": password,
            "device_id": device_id,
            "device_name": device_name,
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Login failed: {text}"));
    }

    let data: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Invalid response: {e}"))?;

    if let Some(token) = data.get("token").and_then(|t| t.as_str()) {
        *state.token.lock().map_err(|e| e.to_string())? = Some(token.to_string());
        let token_path = state.data_dir.join("session_token");
        let _ = std::fs::write(&token_path, token);
    }

    Ok(data)
}

/// Register via /api/auth/register.
#[tauri::command]
async fn auth_register(
    username: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let url = state.server_url.lock().map_err(|e| e.to_string())?.clone();

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{url}/api/auth/register"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "username": username,
            "password": password,
        }))
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Registration failed: {text}"));
    }

    res.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Invalid response: {e}"))
}

/// Check gateway health.
#[tauri::command]
async fn health_check(state: tauri::State<'_, AppState>) -> Result<HealthResponse, String> {
    let url = state.server_url.lock().map_err(|e| e.to_string())?.clone();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .get(format!("{url}/health"))
        .send()
        .await
        .map_err(|e| format!("Health check failed: {e}"))?;

    if !res.status().is_success() {
        return Err(format!("Health check failed ({})", res.status()));
    }

    res.json::<HealthResponse>()
        .await
        .map_err(|e| format!("Invalid response: {e}"))
}

/// Get the current server URL.
#[tauri::command]
fn get_server_url(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.server_url.lock().map_err(|e| e.to_string())?.clone())
}

/// Set the server URL.
#[tauri::command]
fn set_server_url(url: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.server_url.lock().map_err(|e| e.to_string())? = url;
    Ok(())
}

/// Check if we have an active token.
#[tauri::command]
fn is_authenticated(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(state
        .token
        .lock()
        .map_err(|e| e.to_string())?
        .is_some())
}

/// Clear the current auth token.
#[tauri::command]
fn disconnect(state: tauri::State<'_, AppState>) -> Result<(), String> {
    *state.token.lock().map_err(|e| e.to_string())? = None;
    state.sync_stop.store(true, Ordering::SeqCst);
    state.sync_connected.store(false, Ordering::SeqCst);
    // Remove persisted token
    let token_path = state.data_dir.join("session_token");
    let _ = std::fs::remove_file(token_path);
    Ok(())
}

/// Get platform info for the frontend.
#[tauri::command]
fn get_platform_info() -> serde_json::Value {
    serde_json::json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "is_mobile": cfg!(target_os = "android") || cfg!(target_os = "ios"),
    })
}

// ── Sync Commands ────────────────────────────────────────────────

/// Get sync connection status.
#[tauri::command]
fn get_sync_status(state: tauri::State<'_, AppState>) -> SyncStatus {
    SyncStatus {
        connected: state.sync_connected.load(Ordering::SeqCst),
        device_id: get_device_id(&state.data_dir),
        last_sync: None,
    }
}

/// Trigger a full sync (Layer 3) with the server.
#[tauri::command]
async fn trigger_full_sync(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let server_url = state.server_url.lock().map_err(|e| e.to_string())?.clone();
    let token = state
        .token
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "Not authenticated".to_string())?;
    let device_id = get_device_id(&state.data_dir);

    // Upload a sync request via HTTP relay as a trigger
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "type": "full_sync_request",
        "from_device_id": device_id,
        "manifest": {
            "memory_chunk_ids": [],
            "conversation_ids": [],
            "setting_keys": [],
            "generated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        }
    });

    let res = client
        .post(format!("{server_url}/api/sync/relay"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "encrypted_payload": payload.to_string(),
            "nonce": "full_sync_trigger"
        }))
        .send()
        .await
        .map_err(|e| format!("Full sync trigger failed: {e}"))?;

    if res.status().is_success() {
        Ok("Full sync triggered".to_string())
    } else {
        let text = res.text().await.unwrap_or_default();
        Err(format!("Full sync failed: {text}"))
    }
}

// ── Lifecycle Commands ───────────────────────────────────────────

/// Called when the app goes to background (mobile).
/// Saves state and reduces activity.
#[tauri::command]
fn on_app_pause(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Persist current token to disk for recovery
    if let Ok(guard) = state.token.lock() {
        if let Some(token) = guard.as_ref() {
            let token_path = state.data_dir.join("session_token");
            let _ = std::fs::write(token_path, token);
        }
    }

    // Persist server URL
    if let Ok(url) = state.server_url.lock() {
        let url_path = state.data_dir.join("server_url");
        let _ = std::fs::write(url_path, url.as_str());
    }

    Ok(())
}

/// Called when the app returns to foreground (mobile).
/// Restores state and reconnects.
#[tauri::command]
async fn on_app_resume(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let mut restored_token = false;
    let mut restored_url = false;

    // Restore token if lost from memory
    if state.token.lock().map_err(|e| e.to_string())?.is_none() {
        let token_path = state.data_dir.join("session_token");
        if let Ok(token) = std::fs::read_to_string(&token_path) {
            let token = token.trim().to_string();
            if !token.is_empty() {
                *state.token.lock().map_err(|e| e.to_string())? = Some(token);
                restored_token = true;
            }
        }
    }

    // Restore server URL
    let url_path = state.data_dir.join("server_url");
    if let Ok(url) = std::fs::read_to_string(&url_path) {
        let url = url.trim().to_string();
        if !url.is_empty() {
            *state.server_url.lock().map_err(|e| e.to_string())? = url;
            restored_url = true;
        }
    }

    // Try health check to verify connection.
    // Clone the URL out of the lock so the MutexGuard is dropped before await.
    let health_url = state
        .server_url
        .lock()
        .map_err(|e| e.to_string())?
        .clone();
    let is_online = {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        client
            .get(format!("{}/health", health_url))
            .send()
            .await
            .is_ok()
    };

    state.sync_stop.store(false, Ordering::SeqCst);

    Ok(serde_json::json!({
        "restored_token": restored_token,
        "restored_url": restored_url,
        "is_online": is_online,
        "has_token": state.token.lock().map_err(|e| e.to_string())?.is_some(),
    }))
}

// ── Helpers ──────────────────────────────────────────────────────

/// Get or create a persistent device ID.
fn get_device_id(data_dir: &std::path::Path) -> String {
    let id_path = data_dir.join(".device_id");
    if let Ok(id) = std::fs::read_to_string(&id_path) {
        let id = id.trim().to_string();
        if !id.is_empty() {
            return id;
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    let _ = std::fs::create_dir_all(data_dir);
    let _ = std::fs::write(&id_path, &id);
    id
}

// ── Entry Point ──────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            server_url: std::sync::Mutex::new(
                std::env::var("MOA_SERVER_URL")
                    .unwrap_or_else(|_| "https://moanew-production.up.railway.app".to_string()),
            ),
            token: std::sync::Mutex::new(None),
            sync_connected: AtomicBool::new(false),
            sync_stop: AtomicBool::new(false),
            data_dir: {
                // Use platform-appropriate data directory
                let dir = if cfg!(target_os = "android") || cfg!(target_os = "ios") {
                    // On mobile, Tauri provides the app data path at runtime.
                    // We use a safe default that Tauri's setup will override.
                    PathBuf::from(".")
                } else {
                    dirs_next::data_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("com.moa.agent")
                };
                let _ = std::fs::create_dir_all(&dir);
                dir
            },
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            chat,
            pair,
            auth_login,
            auth_register,
            health_check,
            get_server_url,
            set_server_url,
            is_authenticated,
            disconnect,
            get_platform_info,
            get_sync_status,
            trigger_full_sync,
            on_app_pause,
            on_app_resume,
        ])
        .setup(|app| {
            // Override data_dir with Tauri's actual app data path
            let app_data_dir = app.path().app_data_dir().unwrap_or_default();
            let _ = std::fs::create_dir_all(&app_data_dir);
            if let Some(state) = app.try_state::<AppState>() {
                // Restore persisted token on startup
                let token_path = app_data_dir.join("session_token");
                if let Ok(token) = std::fs::read_to_string(&token_path) {
                    let token = token.trim().to_string();
                    if !token.is_empty() {
                        if let Ok(mut guard) = state.token.lock() {
                            *guard = Some(token);
                        }
                    }
                }
                // Restore persisted server URL
                let url_path = app_data_dir.join("server_url");
                if let Ok(url) = std::fs::read_to_string(&url_path) {
                    let url = url.trim().to_string();
                    if !url.is_empty() {
                        if let Ok(mut guard) = state.server_url.lock() {
                            *guard = url;
                        }
                    }
                }
            }

            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MoA application");
}
