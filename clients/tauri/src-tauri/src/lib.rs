use serde::{Deserialize, Serialize};
use tauri::Manager;

// ── State ────────────────────────────────────────────────────────

/// Shared application state for Tauri commands.
struct AppState {
    server_url: std::sync::Mutex<String>,
    token: std::sync::Mutex<Option<String>>,
}

// ── Types ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct ChatResponse {
    response: String,
    model: String,
}

#[derive(Serialize, Deserialize)]
struct PairResponse {
    paired: bool,
    token: String,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
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

/// Pair with a MoA gateway server using a pairing code.
#[tauri::command]
async fn pair(
    code: String,
    server_url: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<PairResponse, String> {
    if let Some(url) = server_url {
        *state.server_url.lock().map_err(|e| e.to_string())? = url;
    }

    let url = state.server_url.lock().map_err(|e| e.to_string())?.clone();

    let client = reqwest::Client::new();
    let res = client
        .post(format!("{url}/pair"))
        .header("Content-Type", "application/json")
        .header("X-Pairing-Code", &code)
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
    }

    Ok(data)
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

// ── Entry Point ──────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            server_url: std::sync::Mutex::new(
                "https://moanew-production.up.railway.app".to_string(),
            ),
            token: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            chat,
            pair,
            health_check,
            get_server_url,
            set_server_url,
            is_authenticated,
            disconnect,
            get_platform_info,
        ])
        .setup(|app| {
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
