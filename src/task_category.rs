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
    pub fn label(&self) -> &'static str {
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
    pub fn id(&self) -> &'static str {
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
    pub fn uses_sandbox(&self) -> bool {
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

    pub fn label(&self) -> &'static str {
        match self {
            SidebarCategory::Channels => "채널",
            SidebarCategory::Billing => "결제",
            SidebarCategory::MyPage => "마이페이지",
        }
    }

    pub fn id(&self) -> &'static str {
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
pub fn tool_preset(category: &TaskCategory) -> Option<HashSet<&'static str>> {
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
        TaskCategory::Document | TaskCategory::Music => {
            Some(BASE.iter().copied().collect())
        }

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
    category: &TaskCategory,
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
pub fn category_system_prompt(category: &TaskCategory) -> String {
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
             You are a professional translator and interpreter.\n\
             - Translate text accurately while preserving nuance and tone.\n\
             - Support all major languages with particular strength in Korean, English, Japanese, Chinese.\n\
             - Use browser tools for terminology lookup when needed.\n\
             - For documents, use file_read/file_write to process files.\n\
             - Store recurring terminology to memory for consistency."
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
        assert!(tool_preset(&TaskCategory::Coding).is_none());
    }

    #[test]
    fn translation_has_minimal_tools() {
        let preset = tool_preset(&TaskCategory::Translation).unwrap();
        assert!(preset.contains("memory_store"));
        assert!(preset.contains("browser_open"));
        // Translation doesn't need shell or git.
        assert!(!preset.contains("git_operations"));
        assert!(!preset.contains("schedule"));
    }

    #[test]
    fn web_general_includes_browser() {
        let preset = tool_preset(&TaskCategory::WebGeneral).unwrap();
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
            assert!(!category_system_prompt(cat).is_empty());
        }
    }

    #[test]
    fn display_matches_id() {
        for cat in TaskCategory::ALL {
            assert_eq!(format!("{cat}"), cat.id());
        }
    }
}
