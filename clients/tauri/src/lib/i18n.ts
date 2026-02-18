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
  | "server_url"
  | "pairing_code"
  | "pair"
  | "pairing"
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
  | "not_connected_hint";

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
    server_url: "Server URL",
    pairing_code: "Pairing Code",
    pair: "Pair",
    pairing: "Pairing...",
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
    not_connected_hint: "Connect to a server in Settings to start chatting",
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
    server_url: "\uC11C\uBC84 URL",
    pairing_code: "\uD398\uC5B4\uB9C1 \uCF54\uB4DC",
    pair: "\uD398\uC5B4\uB9C1",
    pairing: "\uD398\uC5B4\uB9C1 \uC911...",
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
    not_connected_hint: "\uCC44\uD305\uC744 \uC2DC\uC791\uD558\uB824\uBA74 \uC124\uC815\uC5D0\uC11C \uC11C\uBC84\uC5D0 \uC5F0\uACB0\uD558\uC138\uC694",
  },
};

export function t(key: TranslationKey, locale: Locale): string {
  return translations[locale]?.[key] ?? translations.en[key] ?? key;
}
