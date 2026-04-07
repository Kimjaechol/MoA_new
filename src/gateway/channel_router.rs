//! Common channel routing framework for messaging platforms.
//!
//! Routes messages from external channels (KakaoTalk, Telegram, Slack, etc.)
//! through the Railway relay to the user's specific MoA device.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐          ┌──────────────────────┐          ┌──────────────┐
//! │  KakaoTalk   │─webhook─►│  Railway Server      │─device──►│  MoA Device  │
//! │  Telegram    │─webhook─►│  ChannelRouter       │ router   │  (user's PC  │
//! │  Slack       │─webhook─►│  ├ find_channel_link  │          │   or phone)  │
//! │  etc.        │          │  ├ route_to_device    │◄─resp────│              │
//! └──────────────┘          │  └ reply_via_channel  │          └──────────────┘
//!                           └──────────────────────┘
//! ```
//!
//! ## Onboarding Flow (common for all channels)
//!
//! 1. User sends first message from channel (e.g., KakaoTalk)
//! 2. Railway checks `channel_links` table → not found
//! 3. Reply: "MoA에 연결하려면 아래 링크를 눌러주세요" + auth link
//! 4. User clicks link → web login → MoA account identified
//! 5. If single device: auto-link. If multiple: select device.
//! 6. channel_links record created: (kakao, user_kakao_id) → (moa_user, device)
//! 7. Subsequent messages auto-routed to device.
//!
//! ## Autonomy Modes
//!
//! - `read_only` (default): Agent can search web, recall memory, but cannot
//!   write files, execute shell commands, etc.
//! - `full`: Agent has full access — user explicitly opted in via `/모드 전체`.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::remote::{DeviceRouter, RoutedMessage, REMOTE_RESPONSE_CHANNELS};
use crate::auth::store::{AuthStore, ChannelLink};

/// Result of routing a channel message to a device.
#[derive(Debug)]
pub enum RouteResult {
    /// Message successfully delivered; response will arrive asynchronously.
    Delivered {
        /// Correlation ID for matching the response.
        msg_id: String,
        /// Receiver for device response chunks.
        response_rx: mpsc::Receiver<RoutedMessage>,
    },
    /// User not linked — needs onboarding.
    NotLinked,
    /// User linked but no device selected yet.
    NoDeviceSelected {
        link: ChannelLink,
    },
    /// Device is offline.
    DeviceOffline {
        device_id: String,
        device_name: Option<String>,
    },
}

/// A pending device response collector — gathers chunks and produces the final text.
pub struct ResponseCollector {
    pub rx: mpsc::Receiver<RoutedMessage>,
    pub msg_id: String,
}

impl ResponseCollector {
    /// Collect the full response, with a timeout.
    /// Returns accumulated text, or an informational message on timeout.
    pub async fn collect(mut self, timeout: Duration) -> String {
        let mut full_response = String::new();

        let result = tokio::time::timeout(timeout, async {
            while let Some(msg) = self.rx.recv().await {
                match msg.msg_type.as_str() {
                    "done" | "remote_response" => {
                        if !msg.content.is_empty() {
                            full_response = msg.content;
                        }
                        break;
                    }
                    "chunk" | "remote_chunk" => {
                        full_response.push_str(&msg.content);
                    }
                    "error" | "remote_error" => {
                        full_response = msg.content;
                        break;
                    }
                    _ => {
                        // Unknown type — append content
                        full_response.push_str(&msg.content);
                    }
                }
            }
        })
        .await;

        // Cleanup response channel
        REMOTE_RESPONSE_CHANNELS.lock().remove(&self.msg_id);

        if result.is_err() {
            if full_response.is_empty() {
                return "MoA 디바이스가 응답하는 데 시간이 오래 걸리고 있습니다. 잠시 후 다시 시도해 주세요.".into();
            }
            // Partial response — return what we have
            full_response.push_str("\n\n(응답이 길어 일부만 전달되었습니다)");
        }

        full_response
    }
}

/// Route a channel message to the user's MoA device.
///
/// This is the main entry point for all channel integrations.
///
/// # Returns
///
/// - `RouteResult::Delivered` — message sent to device, use `response_rx` to get reply.
/// - `RouteResult::NotLinked` — user not registered, needs onboarding link.
/// - `RouteResult::NoDeviceSelected` — user linked but no device chosen.
/// - `RouteResult::DeviceOffline` — device is not connected.
pub async fn route_channel_message(
    auth_store: &AuthStore,
    device_router: &DeviceRouter,
    channel: &str,
    platform_uid: &str,
    content: &str,
) -> RouteResult {
    // 1. Look up channel link
    let link = match auth_store.find_channel_link_full(channel, platform_uid) {
        Ok(Some(link)) => link,
        Ok(None) => return RouteResult::NotLinked,
        Err(e) => {
            tracing::error!(channel, platform_uid, "Channel link lookup failed: {e}");
            return RouteResult::NotLinked;
        }
    };

    // 2. Check device is selected
    let device_id = match &link.device_id {
        Some(id) if !id.is_empty() => id.clone(),
        _ => return RouteResult::NoDeviceSelected { link },
    };

    // 3. Check device is online
    if !device_router.is_device_online(&device_id) {
        // Try to get device name from auth store
        let device_name = auth_store
            .list_devices(&link.user_id)
            .ok()
            .and_then(|devices| {
                devices
                    .into_iter()
                    .find(|d| d.device_id == device_id)
                    .map(|d| d.device_name)
            });
        return RouteResult::DeviceOffline {
            device_id,
            device_name,
        };
    }

    // 4. Build routed message
    let msg_id = Uuid::new_v4().to_string();
    let msg_type = if link.autonomy_mode == "full" {
        "remote_message"
    } else {
        "remote_read_only"
    };

    let routed = RoutedMessage {
        id: msg_id.clone(),
        direction: "to_device".into(),
        content: content.to_string(),
        msg_type: msg_type.into(),
    };

    // 5. Create response channel
    let (resp_tx, resp_rx) = mpsc::channel::<RoutedMessage>(64);
    REMOTE_RESPONSE_CHANNELS
        .lock()
        .insert(msg_id.clone(), resp_tx);

    // 6. Send to device
    if let Err(err) = device_router.send_to_device(&device_id, routed).await {
        tracing::warn!(device_id, "Failed to send channel message to device: {err}");
        // Cleanup
        REMOTE_RESPONSE_CHANNELS.lock().remove(&msg_id);
        return RouteResult::DeviceOffline {
            device_id,
            device_name: None,
        };
    }

    RouteResult::Delivered {
        msg_id,
        response_rx: resp_rx,
    }
}

/// Generate the onboarding auth URL for a channel user.
///
/// The URL points to the MoA web auth page with channel info pre-filled,
/// so after login the system can auto-link the channel identity.
pub fn build_onboarding_url(
    gateway_url: &str,
    channel: &str,
    platform_uid: &str,
) -> String {
    let encoded_uid =
        urlencoding::encode(platform_uid);
    format!(
        "{gateway_url}/auth?channel_link={channel}&platform_uid={encoded_uid}"
    )
}

/// Handle channel commands (e.g., /디바이스, /모드, /연결해제).
///
/// Returns `Some(reply_text)` if the message was a command, `None` otherwise.
pub fn handle_channel_command(
    auth_store: &AuthStore,
    device_router: &DeviceRouter,
    channel: &str,
    platform_uid: &str,
    message: &str,
) -> Option<String> {
    let trimmed = message.trim();

    // Device switch command
    if trimmed == "/디바이스" || trimmed == "/디바이스 변경" || trimmed.eq_ignore_ascii_case("/device") {
        let link = auth_store.find_channel_link_full(channel, platform_uid).ok()??;
        let devices = auth_store.list_devices(&link.user_id).ok()?;
        if devices.is_empty() {
            return Some("등록된 디바이스가 없습니다. MoA 앱을 설치해 주세요.".into());
        }
        if devices.len() == 1 {
            return Some(format!(
                "현재 연결된 디바이스: {}\n디바이스가 1대뿐이므로 변경할 수 없습니다.",
                devices[0].device_name
            ));
        }
        let mut text = "디바이스를 선택하세요:\n".to_string();
        for (i, d) in devices.iter().enumerate() {
            let online = if device_router.is_device_online(&d.device_id) {
                "🟢"
            } else {
                "⚪"
            };
            let current = if link.device_id.as_deref() == Some(&d.device_id) {
                " (현재)"
            } else {
                ""
            };
            text.push_str(&format!(
                "\n{} {} {}{} → /디바이스 {}",
                i + 1,
                online,
                d.device_name,
                current,
                i + 1
            ));
        }
        return Some(text);
    }

    // Device selection by number: /디바이스 1, /디바이스 2
    if let Some(num_str) = trimmed
        .strip_prefix("/디바이스 ")
        .or_else(|| trimmed.strip_prefix("/device "))
    {
        if let Ok(num) = num_str.trim().parse::<usize>() {
            let link = auth_store.find_channel_link_full(channel, platform_uid).ok()??;
            let devices = auth_store.list_devices(&link.user_id).ok()?;
            if num == 0 || num > devices.len() {
                return Some("올바른 번호를 입력해 주세요.".into());
            }
            let target = &devices[num - 1];
            let _ = auth_store.update_channel_device(channel, platform_uid, &target.device_id);
            return Some(format!(
                "✅ 디바이스가 '{}'(으)로 변경되었습니다.",
                target.device_name
            ));
        }
    }

    // Autonomy mode toggle
    if trimmed == "/모드 전체" || trimmed.eq_ignore_ascii_case("/mode full") {
        let _ = auth_store.set_channel_autonomy_mode(channel, platform_uid, "full");
        return Some(
            "🔓 전체 모드로 전환되었습니다. MoA가 파일 작성, 명령 실행 등 모든 기능을 사용할 수 있습니다.\n\n읽기 전용으로 되돌리려면: /모드 읽기전용".into(),
        );
    }
    if trimmed == "/모드 읽기전용" || trimmed.eq_ignore_ascii_case("/mode readonly") {
        let _ = auth_store.set_channel_autonomy_mode(channel, platform_uid, "read_only");
        return Some(
            "🔒 읽기 전용 모드로 전환되었습니다. MoA가 검색, 기억 조회만 수행합니다.\n\n전체 모드로 전환하려면: /모드 전체".into(),
        );
    }

    // Unlink
    if trimmed == "/연결해제" || trimmed.eq_ignore_ascii_case("/unlink") {
        let _ = auth_store.unlink_channel(channel, platform_uid);
        return Some(
            "연결이 해제되었습니다. 다시 연결하려면 아무 메시지를 보내주세요.".into(),
        );
    }

    // Help
    if trimmed == "/도움말" || trimmed.eq_ignore_ascii_case("/help") {
        return Some(
            "📋 채널 명령어:\n\n\
             /디바이스 — 연결된 디바이스 확인 및 변경\n\
             /모드 전체 — 전체 기능 모드 (파일 작성, 명령 실행 등)\n\
             /모드 읽기전용 — 읽기 전용 모드 (검색, 기억 조회만)\n\
             /연결해제 — 채널 연결 해제\n\
             /도움말 — 이 도움말 표시"
                .into(),
        );
    }

    None // Not a command
}

/// Friendly message when device is offline (no "오류" wording).
pub fn device_offline_message(device_name: Option<&str>) -> String {
    match device_name {
        Some(name) => format!(
            "'{name}' 디바이스에 연결할 수 없습니다.\n\n\
             디바이스가 꺼져 있거나 인터넷에 연결되어 있지 않을 수 있습니다.\n\
             MoA 앱이 실행 중인지 확인해 주세요.\n\n\
             다른 디바이스로 전환하려면: /디바이스"
        ),
        None => "디바이스에 연결할 수 없습니다.\n\n\
                 디바이스가 꺼져 있거나 인터넷에 연결되어 있지 않을 수 있습니다.\n\
                 MoA 앱이 실행 중인지 확인해 주세요."
            .into(),
    }
}

/// Message for first-time users who need to link their account.
pub fn onboarding_message(auth_url: &str) -> String {
    format!(
        "MoA에 오신 것을 환영합니다! 🎉\n\n\
         MoA 계정과 연결하려면 아래 링크를 눌러주세요.\n\
         로그인하면 자동으로 연결됩니다.\n\n\
         👉 {auth_url}"
    )
}

/// Message for when user has multiple devices and needs to select one.
pub fn device_selection_message(
    devices: &[crate::auth::store::Device],
    device_router: &DeviceRouter,
) -> String {
    let mut text = "MoA 앱이 여러 디바이스에 설치되어 있습니다.\n어떤 디바이스와 대화할까요?\n".to_string();
    for (i, d) in devices.iter().enumerate() {
        let online = if device_router.is_device_online(&d.device_id) {
            "🟢 온라인"
        } else {
            "⚪ 오프라인"
        };
        text.push_str(&format!(
            "\n{} {} — {} → /디바이스 {}",
            i + 1,
            d.device_name,
            online,
            i + 1
        ));
    }
    text.push_str("\n\n번호를 입력해 주세요 (예: /디바이스 1)");
    text
}
