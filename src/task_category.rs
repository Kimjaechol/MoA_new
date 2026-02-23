//! Task category definitions and tool preset routing.
//!
//! Maps web-chat navigation categories to tool subsets so the agent
//! automatically prepares the right capabilities when a user selects
//! a top-bar or sidebar category.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ── Top-bar categories (작업 유형) ──────────────────────────────────

/// Primary task categories shown in the top navigation bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    /// 웹/일반 — general web search, Q&A, browsing
    WebGeneral,
    /// 문서 — document drafting, analysis, summarization
    Document,
    /// 코딩 — code generation, sandbox execution, auto-repair loop
    Coding,
    /// 이미지 — image generation, editing, analysis
    Image,
    /// 음악 — music generation, analysis
    Music,
    /// 비디오 — video generation, editing, analysis
    Video,
    /// 통역 — real-time translation / interpretation
    Translation,
}

impl TaskCategory {
    /// All top-bar categories in display order.
    pub const ALL: &'static [TaskCategory] = &[
        TaskCategory::WebGeneral,
        TaskCategory::Document,
        TaskCategory::Coding,
        TaskCategory::Image,
        TaskCategory::Music,
        TaskCategory::Video,
        TaskCategory::Translation,
    ];

    /// Human-readable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            TaskCategory::WebGeneral => "웹/일반",
            TaskCategory::Document => "문서",
            TaskCategory::Coding => "코딩",
            TaskCategory::Image => "이미지",
            TaskCategory::Music => "음악",
            TaskCategory::Video => "비디오",
            TaskCategory::Translation => "통역",
        }
    }

    /// English identifier used in API payloads and config.
    pub fn id(self) -> &'static str {
        match self {
            TaskCategory::WebGeneral => "web_general",
            TaskCategory::Document => "document",
            TaskCategory::Coding => "coding",
            TaskCategory::Image => "image",
            TaskCategory::Music => "music",
            TaskCategory::Video => "video",
            TaskCategory::Translation => "translation",
        }
    }

    /// Parse from the API id string.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "web_general" => Some(TaskCategory::WebGeneral),
            "document" => Some(TaskCategory::Document),
            "coding" => Some(TaskCategory::Coding),
            "image" => Some(TaskCategory::Image),
            "music" => Some(TaskCategory::Music),
            "video" => Some(TaskCategory::Video),
            "translation" => Some(TaskCategory::Translation),
            _ => None,
        }
    }

    /// Whether this category enables the coding sandbox loop.
    pub fn uses_sandbox(self) -> bool {
        matches!(self, TaskCategory::Coding)
    }
}

impl std::fmt::Display for TaskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

// ── Sidebar categories (탐색) ───────────────────────────────────────

/// Sidebar navigation items (non-task, navigation-only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidebarCategory {
    /// 채널 — channel management (Telegram, Discord, etc.)
    Channels,
    /// 결제 — billing / payment
    Billing,
    /// 마이페이지 — user profile / settings
    MyPage,
}

impl SidebarCategory {
    pub const ALL: &'static [SidebarCategory] = &[
        SidebarCategory::Channels,
        SidebarCategory::Billing,
        SidebarCategory::MyPage,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SidebarCategory::Channels => "채널",
            SidebarCategory::Billing => "결제",
            SidebarCategory::MyPage => "마이페이지",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            SidebarCategory::Channels => "channels",
            SidebarCategory::Billing => "billing",
            SidebarCategory::MyPage => "my_page",
        }
    }
}

// ── Navigation manifest (for web chat frontend) ────────────────────

/// Full navigation structure returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationManifest {
    pub top_bar: Vec<NavigationItem>,
    pub sidebar: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationItem {
    pub id: String,
    pub label: String,
    /// "task" for top-bar (triggers tool selection), "nav" for sidebar.
    pub kind: String,
    /// Icon hint for the frontend (e.g. Material icon name).
    pub icon: String,
    /// Optional UI mode hint. When set, the frontend should render a
    /// specialized UI panel instead of the default chat-only view.
    /// Values: "voice_interpret" for Translation, "sandbox" for Coding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_mode: Option<String>,
}

// ── Translation / Interpretation UI manifest ───────────────────────

/// Direction mode for voice interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterpretDirection {
    /// A → B only (unidirectional).
    Unidirectional,
    /// A ↔ B auto-detect (bidirectional).
    Bidirectional,
}

/// A single language option for the UI language selector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageOption {
    /// ISO 639-1 code (e.g. "ko", "ja", "en").
    pub code: String,
    /// Display name in the language's own script (e.g. "한국어").
    pub label: String,
    /// English name (e.g. "Korean").
    pub label_en: String,
    /// Unicode flag emoji for visual cue.
    pub flag: String,
}

/// A direction mode option for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionOption {
    pub id: String,
    pub label: String,
    /// Description shown as tooltip/subtitle.
    pub description: String,
    /// Icon hint (e.g. "arrow_forward" for A→B, "swap_horiz" for A↔B).
    pub icon: String,
}

/// A domain option for specialized interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainOption {
    pub id: String,
    pub label: String,
    pub description: String,
}

/// A formality option for interpretation output style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormalityOption {
    pub id: String,
    pub label: String,
}

/// Full UI manifest for the Translation / Interpretation panel.
///
/// Returned by `GET /api/voice/ui` so the frontend can render:
/// - Language A / Language B selectors
/// - Direction toggle (A→B or A↔B)
/// - Domain selector
/// - Formality selector
/// - Default values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUiManifest {
    /// All supported languages for selection.
    pub languages: Vec<LanguageOption>,
    /// Available direction modes.
    pub directions: Vec<DirectionOption>,
    /// Available domain specializations.
    pub domains: Vec<DomainOption>,
    /// Available formality levels.
    pub formality_levels: Vec<FormalityOption>,
    /// Default values for initial UI state.
    pub defaults: TranslationDefaults,
    /// API endpoints the frontend needs.
    pub endpoints: TranslationEndpoints,
}

/// Default values for the translation UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationDefaults {
    pub language_a: String,
    pub language_b: String,
    pub direction: String,
    pub domain: String,
    pub formality: String,
}

/// API endpoints for translation operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEndpoints {
    /// POST — create a session.
    pub create_session: String,
    /// GET — list active sessions.
    pub list_sessions: String,
    /// GET (WebSocket) — real-time audio stream. Append `?session_id=<id>`.
    pub interpret_ws: String,
}

impl TranslationUiManifest {
    /// Build the UI manifest with the given default languages from config.
    pub fn build(default_source: &str, default_target: &str) -> Self {
        Self {
            languages: Self::all_languages(),
            directions: vec![
                DirectionOption {
                    id: "bidirectional".to_string(),
                    label: "양방향 (A ↔ B)".to_string(),
                    description: "자동으로 언어를 감지하여 상대 언어로 통역합니다".to_string(),
                    icon: "swap_horiz".to_string(),
                },
                DirectionOption {
                    id: "unidirectional".to_string(),
                    label: "단방향 (A → B)".to_string(),
                    description: "A 언어만 B 언어로 통역합니다".to_string(),
                    icon: "arrow_forward".to_string(),
                },
            ],
            domains: vec![
                DomainOption {
                    id: "general".to_string(),
                    label: "일반".to_string(),
                    description: "일상 대화 및 일반 주제".to_string(),
                },
                DomainOption {
                    id: "business".to_string(),
                    label: "비즈니스".to_string(),
                    description: "회의, 프레젠테이션, 협상".to_string(),
                },
                DomainOption {
                    id: "medical".to_string(),
                    label: "의료".to_string(),
                    description: "진료, 의학 용어, 건강 상담".to_string(),
                },
                DomainOption {
                    id: "legal".to_string(),
                    label: "법률".to_string(),
                    description: "법률 문서, 계약, 소송".to_string(),
                },
                DomainOption {
                    id: "technical".to_string(),
                    label: "기술".to_string(),
                    description: "IT, 공학, 과학 기술".to_string(),
                },
            ],
            formality_levels: vec![
                FormalityOption {
                    id: "formal".to_string(),
                    label: "격식체".to_string(),
                },
                FormalityOption {
                    id: "neutral".to_string(),
                    label: "표준".to_string(),
                },
                FormalityOption {
                    id: "casual".to_string(),
                    label: "구어체".to_string(),
                },
            ],
            defaults: TranslationDefaults {
                language_a: default_source.to_string(),
                language_b: default_target.to_string(),
                direction: "bidirectional".to_string(),
                domain: "general".to_string(),
                formality: "neutral".to_string(),
            },
            endpoints: TranslationEndpoints {
                create_session: "/api/voice/sessions".to_string(),
                list_sessions: "/api/voice/sessions".to_string(),
                interpret_ws: "/api/voice/interpret".to_string(),
            },
        }
    }

    /// Generate the full language list with native labels and flags.
    fn all_languages() -> Vec<LanguageOption> {
        vec![
            LanguageOption {
                code: "ko".into(),
                label: "한국어".into(),
                label_en: "Korean".into(),
                flag: "\u{1f1f0}\u{1f1f7}".into(),
            },
            LanguageOption {
                code: "en".into(),
                label: "English".into(),
                label_en: "English".into(),
                flag: "\u{1f1fa}\u{1f1f8}".into(),
            },
            LanguageOption {
                code: "ja".into(),
                label: "日本語".into(),
                label_en: "Japanese".into(),
                flag: "\u{1f1ef}\u{1f1f5}".into(),
            },
            LanguageOption {
                code: "zh".into(),
                label: "中文(简体)".into(),
                label_en: "Chinese (Simplified)".into(),
                flag: "\u{1f1e8}\u{1f1f3}".into(),
            },
            LanguageOption {
                code: "zh-TW".into(),
                label: "中文(繁體)".into(),
                label_en: "Chinese (Traditional)".into(),
                flag: "\u{1f1f9}\u{1f1fc}".into(),
            },
            LanguageOption {
                code: "es".into(),
                label: "Español".into(),
                label_en: "Spanish".into(),
                flag: "\u{1f1ea}\u{1f1f8}".into(),
            },
            LanguageOption {
                code: "fr".into(),
                label: "Français".into(),
                label_en: "French".into(),
                flag: "\u{1f1eb}\u{1f1f7}".into(),
            },
            LanguageOption {
                code: "de".into(),
                label: "Deutsch".into(),
                label_en: "German".into(),
                flag: "\u{1f1e9}\u{1f1ea}".into(),
            },
            LanguageOption {
                code: "it".into(),
                label: "Italiano".into(),
                label_en: "Italian".into(),
                flag: "\u{1f1ee}\u{1f1f9}".into(),
            },
            LanguageOption {
                code: "pt".into(),
                label: "Português".into(),
                label_en: "Portuguese".into(),
                flag: "\u{1f1e7}\u{1f1f7}".into(),
            },
            LanguageOption {
                code: "ru".into(),
                label: "Русский".into(),
                label_en: "Russian".into(),
                flag: "\u{1f1f7}\u{1f1fa}".into(),
            },
            LanguageOption {
                code: "ar".into(),
                label: "العربية".into(),
                label_en: "Arabic".into(),
                flag: "\u{1f1f8}\u{1f1e6}".into(),
            },
            LanguageOption {
                code: "hi".into(),
                label: "हिन्दी".into(),
                label_en: "Hindi".into(),
                flag: "\u{1f1ee}\u{1f1f3}".into(),
            },
            LanguageOption {
                code: "th".into(),
                label: "ไทย".into(),
                label_en: "Thai".into(),
                flag: "\u{1f1f9}\u{1f1ed}".into(),
            },
            LanguageOption {
                code: "vi".into(),
                label: "Tiếng Việt".into(),
                label_en: "Vietnamese".into(),
                flag: "\u{1f1fb}\u{1f1f3}".into(),
            },
            LanguageOption {
                code: "id".into(),
                label: "Bahasa Indonesia".into(),
                label_en: "Indonesian".into(),
                flag: "\u{1f1ee}\u{1f1e9}".into(),
            },
            LanguageOption {
                code: "ms".into(),
                label: "Bahasa Melayu".into(),
                label_en: "Malay".into(),
                flag: "\u{1f1f2}\u{1f1fe}".into(),
            },
            LanguageOption {
                code: "tl".into(),
                label: "Filipino".into(),
                label_en: "Filipino".into(),
                flag: "\u{1f1f5}\u{1f1ed}".into(),
            },
            LanguageOption {
                code: "nl".into(),
                label: "Nederlands".into(),
                label_en: "Dutch".into(),
                flag: "\u{1f1f3}\u{1f1f1}".into(),
            },
            LanguageOption {
                code: "pl".into(),
                label: "Polski".into(),
                label_en: "Polish".into(),
                flag: "\u{1f1f5}\u{1f1f1}".into(),
            },
            LanguageOption {
                code: "cs".into(),
                label: "Čeština".into(),
                label_en: "Czech".into(),
                flag: "\u{1f1e8}\u{1f1ff}".into(),
            },
            LanguageOption {
                code: "sv".into(),
                label: "Svenska".into(),
                label_en: "Swedish".into(),
                flag: "\u{1f1f8}\u{1f1ea}".into(),
            },
            LanguageOption {
                code: "da".into(),
                label: "Dansk".into(),
                label_en: "Danish".into(),
                flag: "\u{1f1e9}\u{1f1f0}".into(),
            },
            LanguageOption {
                code: "uk".into(),
                label: "Українська".into(),
                label_en: "Ukrainian".into(),
                flag: "\u{1f1fa}\u{1f1e6}".into(),
            },
            LanguageOption {
                code: "tr".into(),
                label: "Türkçe".into(),
                label_en: "Turkish".into(),
                flag: "\u{1f1f9}\u{1f1f7}".into(),
            },
        ]
    }
}

impl NavigationManifest {
    pub fn build() -> Self {
        let top_bar = TaskCategory::ALL
            .iter()
            .map(|cat| NavigationItem {
                id: cat.id().to_string(),
                label: cat.label().to_string(),
                kind: "task".to_string(),
                icon: match cat {
                    TaskCategory::WebGeneral => "language".to_string(),
                    TaskCategory::Document => "description".to_string(),
                    TaskCategory::Coding => "code".to_string(),
                    TaskCategory::Image => "image".to_string(),
                    TaskCategory::Music => "music_note".to_string(),
                    TaskCategory::Video => "videocam".to_string(),
                    TaskCategory::Translation => "translate".to_string(),
                },
                ui_mode: match cat {
                    TaskCategory::Translation => Some("voice_interpret".to_string()),
                    TaskCategory::Coding => Some("sandbox".to_string()),
                    _ => None,
                },
            })
            .collect();

        let sidebar = SidebarCategory::ALL
            .iter()
            .map(|cat| NavigationItem {
                id: cat.id().to_string(),
                label: cat.label().to_string(),
                kind: "nav".to_string(),
                icon: match cat {
                    SidebarCategory::Channels => "forum".to_string(),
                    SidebarCategory::Billing => "payment".to_string(),
                    SidebarCategory::MyPage => "person".to_string(),
                },
                ui_mode: None,
            })
            .collect();

        Self { top_bar, sidebar }
    }
}

// ── Tool preset routing ────────────────────────────────────────────

/// Returns the set of tool names that should be active for the given
/// task category. Tools not in this set are excluded from the LLM's
/// available tools for the session.
///
/// If `None` is returned, all tools are available (no filtering).
pub fn tool_preset(category: TaskCategory) -> Option<HashSet<&'static str>> {
    /// Common base tools shared across most categories.
    const BASE: &[&str] = &[
        "shell",
        "file_read",
        "file_write",
        "memory_store",
        "memory_recall",
        "memory_forget",
        "browser_open",
        "browser",
        "http_request",
        "composio",
    ];

    /// Vision tools (screenshot + image metadata).
    const VISION: &[&str] = &["screenshot", "image_info"];

    match category {
        // ── Coding: full toolset + sandbox capabilities ──
        // All tools enabled — sandbox loop handles the extra logic.
        TaskCategory::Coding => None,

        // ── Web/General, Image, Video: base + vision ──
        TaskCategory::WebGeneral | TaskCategory::Image | TaskCategory::Video => {
            Some(BASE.iter().chain(VISION.iter()).copied().collect())
        }

        // ── Document, Music: base (no vision) ──
        TaskCategory::Document | TaskCategory::Music => Some(BASE.iter().copied().collect()),

        // ── Translation: minimal toolset (mostly LLM-native) ──
        TaskCategory::Translation => Some(
            [
                "memory_store",
                "memory_recall",
                "memory_forget",
                "browser_open",
                "browser",
                "http_request",
                "file_read",
                "file_write",
            ]
            .into_iter()
            .collect(),
        ),
    }
}

/// Filter a list of tool specs, keeping only those in the category preset.
/// Returns all specs unchanged if the category has no preset (= all tools).
pub fn filter_tools_by_category<T: crate::tools::Tool + ?Sized>(
    tools: Vec<Box<T>>,
    category: TaskCategory,
) -> Vec<Box<T>> {
    match tool_preset(category) {
        Some(allowed) => tools
            .into_iter()
            .filter(|t| allowed.contains(t.name()))
            .collect(),
        None => tools,
    }
}

// ── System prompt supplement per category ───────────────────────────

/// Returns an additional system prompt section tailored to the active
/// task category, guiding the LLM's behaviour.
///
/// For Coding mode, the full structured methodology is returned via
/// `crate::sandbox::coding_system_prompt()` which includes all 6 phases,
/// meta-cognitive rules, and the run→observe→fix loop.
pub fn category_system_prompt(category: TaskCategory) -> String {
    match category {
        TaskCategory::Coding => crate::sandbox::coding_system_prompt(),
        TaskCategory::WebGeneral => {
            "## Active Mode: Web / General\n\n\
             You are a general-purpose assistant with web browsing capabilities.\n\
             - Use the browser and http_request tools to fetch live information.\n\
             - Summarize findings clearly with sources.\n\
             - Store important facts to memory for future reference."
                .to_string()
        }
        TaskCategory::Document => {
            "## Active Mode: Document\n\n\
             You are a document specialist.\n\
             - Help draft, edit, summarize, and analyze documents.\n\
             - Use file_read/file_write to work with local files.\n\
             - Use browser tools for research when needed.\n\
             - Maintain clear formatting and structure."
                .to_string()
        }
        TaskCategory::Image => {
            "## Active Mode: Image\n\n\
             You are an image specialist.\n\
             - Help generate, edit, analyze, and describe images.\n\
             - Use vision tools (screenshot, image_info) for analysis.\n\
             - Use shell for image processing commands (ImageMagick, ffmpeg, etc.).\n\
             - Use browser tools to find reference images when needed."
                .to_string()
        }
        TaskCategory::Music => {
            "## Active Mode: Music\n\n\
             You are a music specialist.\n\
             - Help with music generation, analysis, and processing.\n\
             - Use shell for audio processing (ffmpeg, sox, etc.).\n\
             - Use browser tools to find music resources and references.\n\
             - Assist with MIDI, notation, and audio format conversions."
                .to_string()
        }
        TaskCategory::Video => {
            "## Active Mode: Video\n\n\
             You are a video specialist.\n\
             - Help with video generation, editing, and analysis.\n\
             - Use shell for video processing (ffmpeg, etc.).\n\
             - Use vision tools to analyze frames and screenshots.\n\
             - Assist with format conversion, trimming, merging, and effects."
                .to_string()
        }
        TaskCategory::Translation => {
            "## Active Mode: Translation / Interpretation\n\n\
             You are a professional translator and real-time interpreter.\n\n\
             ### Text Translation\n\
             - Translate text accurately while preserving nuance and tone.\n\
             - Support 25 languages with particular strength in Korean, English, Japanese, Chinese.\n\
             - Use browser tools for terminology lookup when needed.\n\
             - For documents, use file_read/file_write to process files.\n\
             - Store recurring terminology to memory for consistency.\n\n\
             ### Voice Interpretation (Continuous Listening Mode)\n\
             When the user enables voice interpretation:\n\
             - The microphone stays continuously open — NO push-to-talk button needed.\n\
             - Gemini 2.5 Flash Live API handles automatic Voice Activity Detection (VAD).\n\
             - Speech is detected automatically, interpreted in real-time, and played back.\n\
             - Supports bidirectional interpretation (auto-detect speaker language).\n\
             - Domain specialization: general, business, medical, legal, technical.\n\
             - Formality levels: formal, neutral, casual.\n\n\
             ### Voice UX Flow\n\
             1. User selects language pair and toggles 'continuous interpretation ON'\n\
             2. Browser opens microphone, streams audio continuously to server\n\
             3. Server relays audio to Gemini Live with automatic VAD\n\
             4. Gemini detects speech segments, interprets, and streams back audio + text\n\
             5. Translated audio plays automatically; subtitles shown below"
                .to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_categories_have_labels_and_ids() {
        for cat in TaskCategory::ALL {
            assert!(!cat.label().is_empty());
            assert!(!cat.id().is_empty());
            assert_eq!(TaskCategory::from_id(cat.id()), Some(*cat));
        }
    }

    #[test]
    fn coding_uses_sandbox() {
        assert!(TaskCategory::Coding.uses_sandbox());
        assert!(!TaskCategory::WebGeneral.uses_sandbox());
        assert!(!TaskCategory::Document.uses_sandbox());
        assert!(!TaskCategory::Translation.uses_sandbox());
    }

    #[test]
    fn coding_has_no_tool_filter() {
        // Coding mode gets all tools (sandbox handles orchestration).
        assert!(tool_preset(TaskCategory::Coding).is_none());
    }

    #[test]
    fn translation_has_minimal_tools() {
        let preset = tool_preset(TaskCategory::Translation).unwrap();
        assert!(preset.contains("memory_store"));
        assert!(preset.contains("browser_open"));
        // Translation doesn't need shell or git.
        assert!(!preset.contains("git_operations"));
        assert!(!preset.contains("schedule"));
    }

    #[test]
    fn web_general_includes_browser() {
        let preset = tool_preset(TaskCategory::WebGeneral).unwrap();
        assert!(preset.contains("browser"));
        assert!(preset.contains("browser_open"));
        assert!(preset.contains("http_request"));
    }

    #[test]
    fn navigation_manifest_has_correct_counts() {
        let nav = NavigationManifest::build();
        assert_eq!(nav.top_bar.len(), TaskCategory::ALL.len());
        assert_eq!(nav.sidebar.len(), SidebarCategory::ALL.len());
    }

    #[test]
    fn navigation_items_have_correct_kinds() {
        let nav = NavigationManifest::build();
        for item in &nav.top_bar {
            assert_eq!(item.kind, "task");
        }
        for item in &nav.sidebar {
            assert_eq!(item.kind, "nav");
        }
    }

    #[test]
    fn sidebar_categories_have_ids() {
        for cat in SidebarCategory::ALL {
            assert!(!cat.label().is_empty());
            assert!(!cat.id().is_empty());
        }
    }

    #[test]
    fn from_id_returns_none_for_unknown() {
        assert!(TaskCategory::from_id("unknown").is_none());
        assert!(TaskCategory::from_id("").is_none());
    }

    #[test]
    fn category_system_prompts_are_non_empty() {
        for cat in TaskCategory::ALL {
            assert!(!category_system_prompt(*cat).is_empty());
        }
    }

    #[test]
    fn display_matches_id() {
        for cat in TaskCategory::ALL {
            assert_eq!(format!("{cat}"), cat.id());
        }
    }

    #[test]
    fn translation_nav_item_has_voice_interpret_ui_mode() {
        let nav = NavigationManifest::build();
        let translation_item = nav.top_bar.iter().find(|i| i.id == "translation").unwrap();
        assert_eq!(
            translation_item.ui_mode,
            Some("voice_interpret".to_string())
        );
    }

    #[test]
    fn coding_nav_item_has_sandbox_ui_mode() {
        let nav = NavigationManifest::build();
        let coding_item = nav.top_bar.iter().find(|i| i.id == "coding").unwrap();
        assert_eq!(coding_item.ui_mode, Some("sandbox".to_string()));
    }

    #[test]
    fn regular_nav_items_have_no_ui_mode() {
        let nav = NavigationManifest::build();
        let web_item = nav.top_bar.iter().find(|i| i.id == "web_general").unwrap();
        assert!(web_item.ui_mode.is_none());
        for item in &nav.sidebar {
            assert!(item.ui_mode.is_none());
        }
    }

    #[test]
    fn translation_ui_manifest_has_25_languages() {
        let manifest = TranslationUiManifest::build("ko", "en");
        assert_eq!(manifest.languages.len(), 25);
    }

    #[test]
    fn translation_ui_manifest_languages_have_flags() {
        let manifest = TranslationUiManifest::build("ko", "en");
        for lang in &manifest.languages {
            assert!(!lang.code.is_empty());
            assert!(!lang.label.is_empty());
            assert!(!lang.flag.is_empty());
        }
    }

    #[test]
    fn translation_ui_manifest_has_bidirectional_first() {
        let manifest = TranslationUiManifest::build("ko", "en");
        assert_eq!(manifest.directions.len(), 2);
        assert_eq!(manifest.directions[0].id, "bidirectional");
        assert_eq!(manifest.directions[1].id, "unidirectional");
    }

    #[test]
    fn translation_ui_manifest_defaults() {
        let manifest = TranslationUiManifest::build("ja", "ko");
        assert_eq!(manifest.defaults.language_a, "ja");
        assert_eq!(manifest.defaults.language_b, "ko");
        assert_eq!(manifest.defaults.direction, "bidirectional");
        assert_eq!(manifest.defaults.domain, "general");
        assert_eq!(manifest.defaults.formality, "neutral");
    }

    #[test]
    fn translation_ui_manifest_has_5_domains() {
        let manifest = TranslationUiManifest::build("ko", "en");
        assert_eq!(manifest.domains.len(), 5);
        let domain_ids: Vec<&str> = manifest.domains.iter().map(|d| d.id.as_str()).collect();
        assert!(domain_ids.contains(&"general"));
        assert!(domain_ids.contains(&"business"));
        assert!(domain_ids.contains(&"medical"));
        assert!(domain_ids.contains(&"legal"));
        assert!(domain_ids.contains(&"technical"));
    }

    #[test]
    fn translation_ui_manifest_has_3_formality_levels() {
        let manifest = TranslationUiManifest::build("ko", "en");
        assert_eq!(manifest.formality_levels.len(), 3);
    }

    #[test]
    fn translation_ui_manifest_endpoints() {
        let manifest = TranslationUiManifest::build("ko", "en");
        assert_eq!(manifest.endpoints.create_session, "/api/voice/sessions");
        assert_eq!(manifest.endpoints.interpret_ws, "/api/voice/interpret");
    }
}
