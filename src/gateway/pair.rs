//! Gateway channel-pairing web flow.
//!
//! These endpoints serve a minimal web UI that lets users authenticate
//! and receive a 6-digit pairing code to enter in their messaging app.
//!
//! ## Flow
//!
//! 1. User clicks "Connect" button in their chat → opens browser
//! 2. `GET /pair/connect/{channel}` → login page (or auto-pair if already linked)
//! 3. User submits credentials → `POST /pair/connect/{channel}`
//! 4. On success → display 6-digit code
//! 5. User types code in chat → channel validates and auto-pairs
//!
//! For unregistered users:
//! 6. `GET /pair/signup` → signup form + app store links
//! 7. `POST /pair/signup` → create account → redirect to login

use super::AppState;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    Form,
};

/// Query parameters for the connect page.
#[derive(Debug, serde::Deserialize)]
pub struct ConnectQuery {
    /// Platform user ID (e.g., Kakao user ID, Telegram user ID)
    pub uid: Option<String>,
}

/// Form data for the login submission.
#[derive(Debug, serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub channel: String,
    pub uid: String,
}

/// Form data for the signup submission.
#[derive(Debug, serde::Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
    pub password_confirm: String,
    pub channel: Option<String>,
    pub uid: Option<String>,
}

/// GET /pair/connect/{channel}?uid={platform_uid}
/// Renders the login page for channel pairing.
pub async fn handle_pair_page(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    Query(query): Query<ConnectQuery>,
) -> impl IntoResponse {
    let uid = query.uid.as_deref().unwrap_or("");

    // Check if already linked
    if !uid.is_empty() {
        if let Some(ref auth_store) = state.auth_store {
            if let Ok(Some(_user)) = auth_store.find_channel_link(&channel, uid) {
                return Html(render_already_connected(&channel)).into_response();
            }
        }
    }

    Html(render_login_page(&channel, uid, None)).into_response()
}

/// POST /pair/connect/{channel}
/// Processes login form and generates pairing code on success.
pub async fn handle_pair_login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let auth_store = match state.auth_store {
        Some(ref s) => s,
        None => {
            return Html(render_login_page(
                &form.channel,
                &form.uid,
                Some("Authentication service is not enabled."),
            ))
            .into_response();
        }
    };

    // Authenticate
    let user = match auth_store.authenticate(&form.username, &form.password) {
        Ok(u) => u,
        Err(_) => {
            return Html(render_login_page(
                &form.channel,
                &form.uid,
                Some("Invalid username or password."),
            ))
            .into_response();
        }
    };

    // Link channel identity to user
    if let Err(e) = auth_store.link_channel(&form.channel, &form.uid, &user.id) {
        tracing::warn!("Failed to link channel: {e}");
    }

    // Generate pairing code
    let pairing_store = state.channel_pairing.as_ref();
    match pairing_store {
        Some(store) => {
            let code = store.create_pairing(&form.channel, &form.uid, &user.id);
            Html(render_success_page(&code, &form.channel)).into_response()
        }
        None => Html(render_login_page(
            &form.channel,
            &form.uid,
            Some("Pairing service is not available."),
        ))
        .into_response(),
    }
}

/// GET /pair/signup?channel={channel}&uid={uid}
/// Renders the signup page.
pub async fn handle_pair_signup_page(
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let channel = query.get("channel").map(|s| s.as_str()).unwrap_or("");
    let uid = query.get("uid").map(|s| s.as_str()).unwrap_or("");
    Html(render_signup_page(channel, uid, None))
}

/// POST /pair/signup
/// Creates a new account and redirects to success.
pub async fn handle_pair_signup_submit(
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> impl IntoResponse {
    let channel = form.channel.as_deref().unwrap_or("");
    let uid = form.uid.as_deref().unwrap_or("");

    let auth_store = match state.auth_store {
        Some(ref s) => s,
        None => {
            return Html(render_signup_page(channel, uid, Some("Authentication service is not enabled.")))
                .into_response();
        }
    };

    if !state.auth_allow_registration {
        return Html(render_signup_page(channel, uid, Some("Registration is currently disabled.")))
            .into_response();
    }

    if form.password != form.password_confirm {
        return Html(render_signup_page(channel, uid, Some("Passwords do not match.")))
            .into_response();
    }

    // Register
    let user_id = match auth_store.register(&form.username, &form.password) {
        Ok(id) => id,
        Err(e) => {
            let msg = e.to_string();
            return Html(render_signup_page(channel, uid, Some(&msg))).into_response();
        }
    };

    // Link channel identity if provided
    if !channel.is_empty() && !uid.is_empty() {
        if let Err(e) = auth_store.link_channel(channel, uid, &user_id) {
            tracing::warn!("Failed to link channel after signup: {e}");
        }

        // Generate pairing code
        if let Some(store) = state.channel_pairing.as_ref() {
            let code = store.create_pairing(channel, uid, &user_id);
            return Html(render_success_page(&code, channel)).into_response();
        }
    }

    // No channel context — just show account created
    Html(render_account_created()).into_response()
}

// ── HTML Templates ────────────────────────────────────────────────────

fn base_style() -> &'static str {
    r#"
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: #f5f5f5; color: #333;
        display: flex; justify-content: center; align-items: center;
        min-height: 100vh; padding: 20px;
    }
    .card {
        background: #fff; border-radius: 16px; padding: 32px;
        max-width: 400px; width: 100%; box-shadow: 0 4px 24px rgba(0,0,0,0.08);
    }
    .logo { text-align: center; margin-bottom: 24px; }
    .logo h1 { font-size: 28px; color: #1a1a2e; }
    .logo p { font-size: 14px; color: #666; margin-top: 4px; }
    .form-group { margin-bottom: 16px; }
    .form-group label { display: block; font-size: 14px; font-weight: 500; margin-bottom: 6px; color: #444; }
    .form-group input {
        width: 100%; padding: 12px 14px; border: 1.5px solid #ddd;
        border-radius: 10px; font-size: 16px; outline: none; transition: border-color 0.2s;
    }
    .form-group input:focus { border-color: #4a6cf7; }
    .btn {
        width: 100%; padding: 14px; border: none; border-radius: 10px;
        font-size: 16px; font-weight: 600; cursor: pointer; transition: background 0.2s;
    }
    .btn-primary { background: #4a6cf7; color: #fff; }
    .btn-primary:hover { background: #3b5de7; }
    .btn-secondary { background: #e8e8e8; color: #333; margin-top: 8px; }
    .btn-secondary:hover { background: #ddd; }
    .error { background: #fff0f0; color: #d32f2f; padding: 10px 14px; border-radius: 8px; font-size: 13px; margin-bottom: 16px; }
    .link { text-align: center; margin-top: 16px; font-size: 14px; color: #666; }
    .link a { color: #4a6cf7; text-decoration: none; }
    .link a:hover { text-decoration: underline; }
    .code-display {
        text-align: center; margin: 24px 0; padding: 20px;
        background: #f0f4ff; border-radius: 12px; border: 2px dashed #4a6cf7;
    }
    .code-display .code { font-size: 48px; font-weight: 700; letter-spacing: 8px; color: #1a1a2e; }
    .code-display p { font-size: 14px; color: #666; margin-top: 8px; }
    .success-icon { text-align: center; font-size: 64px; margin-bottom: 16px; }
    .steps { margin: 16px 0; padding: 0; }
    .steps li { list-style: none; padding: 8px 0; font-size: 14px; color: #555; }
    .steps li::before { content: '→ '; color: #4a6cf7; font-weight: bold; }
    .app-links { display: flex; gap: 12px; margin-top: 16px; }
    .app-links a {
        flex: 1; display: block; padding: 12px; text-align: center;
        border-radius: 10px; background: #1a1a2e; color: #fff;
        text-decoration: none; font-size: 14px; font-weight: 500;
    }
    .app-links a:hover { background: #2a2a4e; }
    "#
}

fn render_login_page(channel: &str, uid: &str, error: Option<&str>) -> String {
    let channel_display = channel_display_name(channel);
    let error_html = error
        .map(|e| format!(r#"<div class="error">{e}</div>"#))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="ko"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MoA - Connect {channel_display}</title>
<style>{style}</style>
</head><body>
<div class="card">
  <div class="logo"><h1>MoA</h1><p>{channel_display} 연결</p></div>
  {error_html}
  <form method="POST" action="/pair/connect/{channel}">
    <input type="hidden" name="channel" value="{channel}">
    <input type="hidden" name="uid" value="{uid}">
    <div class="form-group">
      <label>아이디 / Username</label>
      <input type="text" name="username" required autocomplete="username" placeholder="Enter username">
    </div>
    <div class="form-group">
      <label>비밀번호 / Password</label>
      <input type="password" name="password" required autocomplete="current-password" placeholder="Enter password">
    </div>
    <button type="submit" class="btn btn-primary">로그인 / Login</button>
  </form>
  <div class="link">
    계정이 없으신가요? / No account?<br>
    <a href="/pair/signup?channel={channel}&uid={uid}">회원가입 / Sign Up</a>
  </div>
</div>
</body></html>"#,
        style = base_style(),
    )
}

fn render_success_page(code: &str, channel: &str) -> String {
    let channel_display = channel_display_name(channel);

    format!(
        r#"<!DOCTYPE html>
<html lang="ko"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MoA - 연결 코드</title>
<style>{style}</style>
</head><body>
<div class="card">
  <div class="success-icon">✅</div>
  <div class="logo"><h1>인증 완료!</h1><p>Authentication Successful</p></div>
  <div class="code-display">
    <div class="code">{code}</div>
    <p>이 코드를 {channel_display}에서 보내주세요<br>
    Type this code in {channel_display}</p>
  </div>
  <ul class="steps">
    <li>{channel_display}(으)로 돌아가세요 / Return to {channel_display}</li>
    <li>위 6자리 코드를 메시지로 보내세요 / Send the 6-digit code</li>
    <li>연결이 자동으로 완료됩니다 / Connection completes automatically</li>
  </ul>
  <p style="text-align:center;font-size:12px;color:#999;margin-top:16px;">코드는 5분간 유효합니다 / Code expires in 5 minutes</p>
</div>
</body></html>"#,
        style = base_style(),
    )
}

fn render_signup_page(channel: &str, uid: &str, error: Option<&str>) -> String {
    let error_html = error
        .map(|e| format!(r#"<div class="error">{e}</div>"#))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="ko"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MoA - 회원가입</title>
<style>{style}</style>
</head><body>
<div class="card">
  <div class="logo"><h1>MoA</h1><p>회원가입 / Sign Up</p></div>
  {error_html}
  <form method="POST" action="/pair/signup">
    <input type="hidden" name="channel" value="{channel}">
    <input type="hidden" name="uid" value="{uid}">
    <div class="form-group">
      <label>아이디 / Username</label>
      <input type="text" name="username" required autocomplete="username" placeholder="Choose a username">
    </div>
    <div class="form-group">
      <label>비밀번호 / Password</label>
      <input type="password" name="password" required autocomplete="new-password" placeholder="Min 8 characters" minlength="8">
    </div>
    <div class="form-group">
      <label>비밀번호 확인 / Confirm Password</label>
      <input type="password" name="password_confirm" required autocomplete="new-password" placeholder="Re-enter password" minlength="8">
    </div>
    <button type="submit" class="btn btn-primary">가입하기 / Create Account</button>
  </form>
  <div class="link">
    이미 계정이 있으신가요? / Already have an account?<br>
    <a href="/pair/connect/{channel}?uid={uid}">로그인 / Login</a>
  </div>
</div>
</body></html>"#,
        style = base_style(),
    )
}

fn render_already_connected(channel: &str) -> String {
    let channel_display = channel_display_name(channel);
    format!(
        r#"<!DOCTYPE html>
<html lang="ko"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MoA - 이미 연결됨</title>
<style>{style}</style>
</head><body>
<div class="card">
  <div class="success-icon">✅</div>
  <div class="logo"><h1>이미 연결되어 있습니다</h1><p>Already Connected</p></div>
  <p style="text-align:center;font-size:14px;color:#666;margin-top:16px;">
    이 계정은 이미 {channel_display}에 연결되어 있습니다.<br>
    {channel_display}에서 바로 대화를 시작하세요!<br><br>
    This account is already connected to {channel_display}.<br>
    Start chatting in {channel_display}!
  </p>
</div>
</body></html>"#,
        style = base_style(),
    )
}

fn render_account_created() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ko"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>MoA - 가입 완료</title>
<style>{style}</style>
</head><body>
<div class="card">
  <div class="success-icon">🎉</div>
  <div class="logo"><h1>가입 완료!</h1><p>Account Created</p></div>
  <p style="text-align:center;font-size:14px;color:#666;margin-top:16px;">
    MoA 계정이 생성되었습니다.<br>
    메시징 앱에서 다시 연결하기 버튼을 눌러주세요.<br><br>
    Your MoA account has been created.<br>
    Tap the Connect button again in your messaging app.
  </p>
</div>
</body></html>"#,
        style = base_style(),
    )
}

fn channel_display_name(channel: &str) -> &str {
    match channel {
        "kakao" => "KakaoTalk",
        "telegram" => "Telegram",
        "whatsapp" => "WhatsApp",
        "discord" => "Discord",
        "slack" => "Slack",
        "imessage" => "iMessage",
        "signal" => "Signal",
        "matrix" => "Matrix",
        "email" => "Email",
        "irc" => "IRC",
        "lark" => "Lark",
        "dingtalk" => "DingTalk",
        "qq" => "QQ",
        _ => channel,
    }
}
