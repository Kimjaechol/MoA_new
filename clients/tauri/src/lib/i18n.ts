export type Locale = "en" | "ko";

const STORAGE_KEY_LOCALE = "moa_locale";

export function getStoredLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY_LOCALE);
  if (stored === "en" || stored === "ko") return stored;
  const browserLang = navigator.language.toLowerCase();
  if (browserLang.startsWith("ko")) return "ko";
  return "en";
}

export function setStoredLocale(locale: Locale): void {
  localStorage.setItem(STORAGE_KEY_LOCALE, locale);
}

type TranslationKey =
  | "app_title"
  | "new_chat"
  | "settings"
  | "chat"
  | "send"
  | "type_message"
  | "thinking"
  | "error_occurred"
  | "retry"
  | "advanced_settings"
  | "server_url"
  | "username"
  | "password"
  | "password_confirm"
  | "pairing_code"
  | "pairing_code_optional"
  | "pair"
  | "pairing"
  | "connect"
  | "connecting"
  | "disconnect"
  | "connection_status"
  | "connected"
  | "disconnected"
  | "token"
  | "health_check"
  | "checking"
  | "server_healthy"
  | "server_unreachable"
  | "pair_success"
  | "pair_failed"
  | "language"
  | "no_chats"
  | "welcome_title"
  | "welcome_subtitle"
  | "welcome_hint"
  | "delete_chat"
  | "delete_confirm"
  | "model"
  | "back_to_chat"
  | "not_connected_hint"
  | "sync_status"
  | "sync_connected"
  | "sync_disconnected"
  | "sync_device_id"
  | "sync_trigger"
  | "sync_triggering"
  | "sync_triggered"
  | "sync_failed"
  | "platform"
  // Auth flow
  | "login"
  | "login_title"
  | "login_subtitle"
  | "login_button"
  | "logging_in"
  | "login_failed"
  | "signup"
  | "signup_title"
  | "signup_subtitle"
  | "signup_button"
  | "signing_up"
  | "signup_success"
  | "signup_failed"
  | "no_account"
  | "have_account"
  | "logout"
  | "logout_confirm"
  // Device selection
  | "select_device"
  | "select_device_subtitle"
  | "device_online"
  | "device_offline"
  | "device_this"
  | "device_remote"
  | "device_select"
  | "device_no_devices"
  | "device_pairing_required"
  | "enter_pairing_code"
  | "verify_pairing"
  | "verifying"
  | "pairing_verified"
  | "pairing_invalid"
  | "auto_connecting"
  // Device management in settings
  | "my_devices"
  | "device_name"
  | "device_platform"
  | "device_last_seen"
  | "set_pairing_code"
  | "change_pairing_code"
  | "remove_pairing_code"
  | "pairing_code_set"
  | "pairing_code_removed"
  | "new_pairing_code"
  | "save_pairing_code"
  | "account_info"
  // Sidebar sections
  | "sidebar_devices"
  | "sidebar_channels"
  | "sidebar_tools"
  | "sidebar_no_devices"
  | "sidebar_no_channels"
  | "sidebar_no_tools"
  | "sidebar_chats";

const translations: Record<Locale, Record<TranslationKey, string>> = {
  en: {
    app_title: "MoA",
    new_chat: "New Chat",
    settings: "Settings",
    chat: "Chat",
    send: "Send",
    type_message: "Type a message...",
    thinking: "Thinking...",
    error_occurred: "An error occurred",
    retry: "Retry",
    advanced_settings: "Advanced Settings",
    server_url: "Server URL",
    username: "Username",
    password: "Password",
    password_confirm: "Confirm Password",
    pairing_code: "Pairing Code",
    pairing_code_optional: "Pairing Code (optional, for code-only mode)",
    pair: "Pair",
    pairing: "Pairing...",
    connect: "Connect",
    connecting: "Connecting...",
    disconnect: "Disconnect",
    connection_status: "Connection Status",
    connected: "Connected",
    disconnected: "Disconnected",
    token: "Token",
    health_check: "Health Check",
    checking: "Checking...",
    server_healthy: "Server is healthy",
    server_unreachable: "Server is unreachable",
    pair_success: "Successfully paired with server",
    pair_failed: "Pairing failed",
    language: "Language",
    no_chats: "No conversations yet",
    welcome_title: "Welcome to MoA",
    welcome_subtitle: "Master of AI - Powered by ZeroClaw",
    welcome_hint: "Start a conversation by typing a message below",
    delete_chat: "Delete",
    delete_confirm: "Delete this conversation?",
    model: "Model",
    back_to_chat: "Back to Chat",
    not_connected_hint: "Please login to start chatting",
    sync_status: "Sync Status",
    sync_connected: "Sync connected",
    sync_disconnected: "Sync not connected",
    sync_device_id: "Device ID",
    sync_trigger: "Full Sync",
    sync_triggering: "Syncing...",
    sync_triggered: "Full sync triggered successfully",
    sync_failed: "Sync failed",
    platform: "Platform",
    // Auth flow
    login: "Login",
    login_title: "Login to MoA",
    login_subtitle: "Master of AI",
    login_button: "Login",
    logging_in: "Logging in...",
    login_failed: "Login failed",
    signup: "Sign Up",
    signup_title: "Create Account",
    signup_subtitle: "Join MoA - Master of AI",
    signup_button: "Create Account",
    signing_up: "Creating account...",
    signup_success: "Account created! Please login.",
    signup_failed: "Registration failed",
    no_account: "Don't have an account?",
    have_account: "Already have an account?",
    logout: "Logout",
    logout_confirm: "Are you sure you want to logout?",
    // Device selection
    select_device: "Select Device",
    select_device_subtitle: "Choose which MoA device to connect to",
    device_online: "Online",
    device_offline: "Offline",
    device_this: "This device",
    device_remote: "Remote",
    device_select: "Connect",
    device_no_devices: "No devices registered yet. This device will be registered automatically.",
    device_pairing_required: "Pairing code required for remote device",
    enter_pairing_code: "Enter pairing code",
    verify_pairing: "Verify",
    verifying: "Verifying...",
    pairing_verified: "Pairing verified!",
    pairing_invalid: "Invalid pairing code",
    auto_connecting: "Auto-connecting...",
    // Device management in settings
    my_devices: "My Devices",
    device_name: "Device Name",
    device_platform: "Platform",
    device_last_seen: "Last Seen",
    set_pairing_code: "Set Pairing Code",
    change_pairing_code: "Change Pairing Code",
    remove_pairing_code: "Remove Pairing Code",
    pairing_code_set: "Pairing code updated",
    pairing_code_removed: "Pairing code removed",
    new_pairing_code: "New pairing code",
    save_pairing_code: "Save",
    account_info: "Account",
    // Sidebar sections
    sidebar_devices: "Devices",
    sidebar_channels: "Channels",
    sidebar_tools: "Tools",
    sidebar_no_devices: "No devices",
    sidebar_no_channels: "No channels",
    sidebar_no_tools: "No tools",
    sidebar_chats: "Chats",
  },
  ko: {
    app_title: "MoA",
    new_chat: "\uC0C8 \uB300\uD654",
    settings: "\uC124\uC815",
    chat: "\uCC44\uD305",
    send: "\uBCF4\uB0B4\uAE30",
    type_message: "\uBA54\uC2DC\uC9C0\uB97C \uC785\uB825\uD558\uC138\uC694...",
    thinking: "\uC0DD\uAC01 \uC911...",
    error_occurred: "\uC624\uB958\uAC00 \uBC1C\uC0DD\uD588\uC2B5\uB2C8\uB2E4",
    retry: "\uC7AC\uC2DC\uB3C4",
    advanced_settings: "\uACE0\uAE09 \uC124\uC815",
    server_url: "\uC11C\uBC84 URL",
    username: "\uC544\uC774\uB514",
    password: "\uBE44\uBC00\uBC88\uD638",
    password_confirm: "\uBE44\uBC00\uBC88\uD638 \uD655\uC778",
    pairing_code: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC",
    pairing_code_optional: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC (\uC120\uD0DD\uC0AC\uD56D, \uCF54\uB4DC \uC804\uC6A9 \uBAA8\uB4DC)",
    pair: "\uD398\uC5B4\uB9C1",
    pairing: "\uD398\uC5B4\uB9C1 \uC911...",
    connect: "\uC5F0\uACB0",
    connecting: "\uC5F0\uACB0 \uC911...",
    disconnect: "\uC5F0\uACB0 \uD574\uC81C",
    connection_status: "\uC5F0\uACB0 \uC0C1\uD0DC",
    connected: "\uC5F0\uACB0\uB428",
    disconnected: "\uC5F0\uACB0 \uC548 \uB428",
    token: "\uD1A0\uD070",
    health_check: "\uC0C1\uD0DC \uD655\uC778",
    checking: "\uD655\uC778 \uC911...",
    server_healthy: "\uC11C\uBC84\uAC00 \uC815\uC0C1\uC785\uB2C8\uB2E4",
    server_unreachable: "\uC11C\uBC84\uC5D0 \uC5F0\uACB0\uD560 \uC218 \uC5C6\uC2B5\uB2C8\uB2E4",
    pair_success: "\uC11C\uBC84\uC640 \uC131\uACF5\uC801\uC73C\uB85C \uD398\uC5B4\uB9C1\uB418\uC5C8\uC2B5\uB2C8\uB2E4",
    pair_failed: "\uD398\uC5B4\uB9C1 \uC2E4\uD328",
    language: "\uC5B8\uC5B4",
    no_chats: "\uC544\uC9C1 \uB300\uD654\uAC00 \uC5C6\uC2B5\uB2C8\uB2E4",
    welcome_title: "MoA\uC5D0 \uC624\uC2E0 \uAC83\uC744 \uD658\uC601\uD569\uB2C8\uB2E4",
    welcome_subtitle: "Master of AI - ZeroClaw \uAE30\uBC18",
    welcome_hint: "\uC544\uB798\uC5D0 \uBA54\uC2DC\uC9C0\uB97C \uC785\uB825\uD558\uC5EC \uB300\uD654\uB97C \uC2DC\uC791\uD558\uC138\uC694",
    delete_chat: "\uC0AD\uC81C",
    delete_confirm: "\uC774 \uB300\uD654\uB97C \uC0AD\uC81C\uD558\uC2DC\uACA0\uC2B5\uB2C8\uAE4C?",
    model: "\uBAA8\uB378",
    back_to_chat: "\uCC44\uD305\uC73C\uB85C \uB3CC\uC544\uAC00\uAE30",
    not_connected_hint: "\uCC44\uD305\uC744 \uC2DC\uC791\uD558\uB824\uBA74 \uB85C\uADF8\uC778\uD574\uC8FC\uC138\uC694",
    sync_status: "\uB3D9\uAE30\uD654 \uC0C1\uD0DC",
    sync_connected: "\uB3D9\uAE30\uD654 \uC5F0\uACB0\uB428",
    sync_disconnected: "\uB3D9\uAE30\uD654 \uC5F0\uACB0 \uC548 \uB428",
    sync_device_id: "\uB514\uBC14\uC774\uC2A4 ID",
    sync_trigger: "\uC804\uCCB4 \uB3D9\uAE30\uD654",
    sync_triggering: "\uB3D9\uAE30\uD654 \uC911...",
    sync_triggered: "\uC804\uCCB4 \uB3D9\uAE30\uD654\uAC00 \uC131\uACF5\uC801\uC73C\uB85C \uC2DC\uC791\uB418\uC5C8\uC2B5\uB2C8\uB2E4",
    sync_failed: "\uB3D9\uAE30\uD654 \uC2E4\uD328",
    platform: "\uD50C\uB7AB\uD3FC",
    // Auth flow
    login: "\uB85C\uADF8\uC778",
    login_title: "MoA \uB85C\uADF8\uC778",
    login_subtitle: "Master of AI",
    login_button: "\uB85C\uADF8\uC778",
    logging_in: "\uB85C\uADF8\uC778 \uC911...",
    login_failed: "\uB85C\uADF8\uC778 \uC2E4\uD328",
    signup: "\uD68C\uC6D0\uAC00\uC785",
    signup_title: "\uD68C\uC6D0\uAC00\uC785",
    signup_subtitle: "MoA - Master of AI",
    signup_button: "\uACC4\uC815 \uB9CC\uB4E4\uAE30",
    signing_up: "\uACC4\uC815 \uC0DD\uC131 \uC911...",
    signup_success: "\uACC4\uC815\uC774 \uC0DD\uC131\uB418\uC5C8\uC2B5\uB2C8\uB2E4! \uB85C\uADF8\uC778\uD574\uC8FC\uC138\uC694.",
    signup_failed: "\uD68C\uC6D0\uAC00\uC785 \uC2E4\uD328",
    no_account: "\uACC4\uC815\uC774 \uC5C6\uC73C\uC2E0\uAC00\uC694?",
    have_account: "\uC774\uBBF8 \uACC4\uC815\uC774 \uC788\uC73C\uC2E0\uAC00\uC694?",
    logout: "\uB85C\uADF8\uC544\uC6C3",
    logout_confirm: "\uB85C\uADF8\uC544\uC6C3 \uD558\uC2DC\uACA0\uC2B5\uB2C8\uAE4C?",
    // Device selection
    select_device: "\uB514\uBC14\uC774\uC2A4 \uC120\uD0DD",
    select_device_subtitle: "\uC5F0\uACB0\uD560 MoA \uB514\uBC14\uC774\uC2A4\uB97C \uC120\uD0DD\uD558\uC138\uC694",
    device_online: "\uC628\uB77C\uC778",
    device_offline: "\uC624\uD504\uB77C\uC778",
    device_this: "\uC774 \uB514\uBC14\uC774\uC2A4",
    device_remote: "\uC6D0\uACA9",
    device_select: "\uC5F0\uACB0",
    device_no_devices: "\uB4F1\uB85D\uB41C \uB514\uBC14\uC774\uC2A4\uAC00 \uC5C6\uC2B5\uB2C8\uB2E4. \uC774 \uB514\uBC14\uC774\uC2A4\uAC00 \uC790\uB3D9\uC73C\uB85C \uB4F1\uB85D\uB429\uB2C8\uB2E4.",
    device_pairing_required: "\uC6D0\uACA9 \uB514\uBC14\uC774\uC2A4 \uC5F0\uACB0\uC5D0\uB294 \uD398\uC5B4\uB9C1 \uCF54\uB4DC\uAC00 \uD544\uC694\uD569\uB2C8\uB2E4",
    enter_pairing_code: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC \uC785\uB825",
    verify_pairing: "\uD655\uC778",
    verifying: "\uD655\uC778 \uC911...",
    pairing_verified: "\uD398\uC5B4\uB9C1 \uD655\uC778 \uC644\uB8CC!",
    pairing_invalid: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC\uAC00 \uC62C\uBC14\uB974\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4",
    auto_connecting: "\uC790\uB3D9 \uC5F0\uACB0 \uC911...",
    // Device management in settings
    my_devices: "\uB0B4 \uB514\uBC14\uC774\uC2A4",
    device_name: "\uB514\uBC14\uC774\uC2A4 \uC774\uB984",
    device_platform: "\uD50C\uB7AB\uD3FC",
    device_last_seen: "\uB9C8\uC9C0\uB9C9 \uC811\uC18D",
    set_pairing_code: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC \uC124\uC815",
    change_pairing_code: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC \uBCC0\uACBD",
    remove_pairing_code: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC \uC81C\uAC70",
    pairing_code_set: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC\uAC00 \uC5C5\uB370\uC774\uD2B8\uB418\uC5C8\uC2B5\uB2C8\uB2E4",
    pairing_code_removed: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC\uAC00 \uC81C\uAC70\uB418\uC5C8\uC2B5\uB2C8\uB2E4",
    new_pairing_code: "\uC0C8 \uD398\uC5B4\uB9C1 \uCF54\uB4DC",
    save_pairing_code: "\uC800\uC7A5",
    account_info: "\uACC4\uC815 \uC815\uBCF4",
    // Sidebar sections
    sidebar_devices: "\uB514\uBC14\uC774\uC2A4",
    sidebar_channels: "\uCC44\uB110",
    sidebar_tools: "\uB3C4\uAD6C",
    sidebar_no_devices: "\uB514\uBC14\uC774\uC2A4 \uC5C6\uC74C",
    sidebar_no_channels: "\uCC44\uB110 \uC5C6\uC74C",
    sidebar_no_tools: "\uB3C4\uAD6C \uC5C6\uC74C",
    sidebar_chats: "\uB300\uD654",
  },
};

export function t(key: TranslationKey, locale: Locale): string {
  return translations[locale]?.[key] ?? translations.en[key] ?? key;
}
