//! Coding mode split-screen layout definition.
//!
//! Defines the two-panel layout served to the web-chat frontend:
//!
//! - **Workflow Panel** (left / top) — shows the AI's 6-phase thinking
//!   process, problem analysis, code changes, terminal output.
//! - **Preview Panel** (right / bottom) — live preview of the app being
//!   built, updating in real-time as coding progresses.

use serde::{Deserialize, Serialize};

// ── Layout configuration ───────────────────────────────────────────

/// Split-screen layout orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutOrientation {
    /// Workflow left, Preview right (desktop default).
    Horizontal,
    /// Workflow top, Preview bottom (mobile / narrow viewport).
    Vertical,
}

/// Full coding layout descriptor sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingLayout {
    /// Panel orientation.
    pub orientation: LayoutOrientation,
    /// Workflow panel (left or top).
    pub workflow_panel: WorkflowPanelConfig,
    /// Preview panel (right or bottom).
    pub preview_panel: PreviewPanelConfig,
    /// Ratio: workflow panel width/height percentage (0–100).
    /// The preview panel gets the remainder.
    pub workflow_ratio: u8,
}

impl Default for CodingLayout {
    fn default() -> Self {
        Self {
            orientation: LayoutOrientation::Horizontal,
            workflow_panel: WorkflowPanelConfig::default(),
            preview_panel: PreviewPanelConfig::default(),
            workflow_ratio: 50,
        }
    }
}

impl CodingLayout {
    /// Layout optimized for narrow viewports (mobile).
    pub fn mobile() -> Self {
        Self {
            orientation: LayoutOrientation::Vertical,
            workflow_ratio: 55,
            ..Self::default()
        }
    }
}

// ── Workflow panel ─────────────────────────────────────────────────

/// Configuration for the workflow (thinking/coding) panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPanelConfig {
    /// Sub-sections displayed in the workflow panel.
    pub sections: Vec<WorkflowSection>,
}

impl Default for WorkflowPanelConfig {
    fn default() -> Self {
        Self {
            sections: vec![
                WorkflowSection::PhaseTracker,
                WorkflowSection::ThinkingProcess,
                WorkflowSection::CodeChanges,
                WorkflowSection::Terminal,
            ],
        }
    }
}

/// Subsections within the workflow panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowSection {
    /// Phase progress bar (Comprehend → Plan → ... → Deliver).
    PhaseTracker,
    /// AI's thinking process: problem analysis, reasoning, decisions.
    ThinkingProcess,
    /// File diffs and code changes being applied.
    CodeChanges,
    /// Terminal / shell output (build, test, server logs).
    Terminal,
    /// Error analysis and fix strategy.
    ErrorAnalysis,
}

impl WorkflowSection {
    pub fn label(self) -> &'static str {
        match self {
            WorkflowSection::PhaseTracker => "진행 상황",
            WorkflowSection::ThinkingProcess => "AI 사고 과정",
            WorkflowSection::CodeChanges => "코드 변경",
            WorkflowSection::Terminal => "터미널",
            WorkflowSection::ErrorAnalysis => "에러 분석",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            WorkflowSection::PhaseTracker => "timeline",
            WorkflowSection::ThinkingProcess => "psychology",
            WorkflowSection::CodeChanges => "difference",
            WorkflowSection::Terminal => "terminal",
            WorkflowSection::ErrorAnalysis => "bug_report",
        }
    }
}

// ── Preview panel ──────────────────────────────────────────────────

/// Configuration for the preview (webview) panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewPanelConfig {
    /// How to render the preview.
    pub preview_mode: PreviewMode,
    /// Whether the panel can be manually refreshed.
    pub allow_manual_refresh: bool,
    /// Whether to show a URL bar.
    pub show_url_bar: bool,
    /// Whether to show device frame selector (phone/tablet/desktop).
    pub show_device_frames: bool,
    /// Auto-refresh interval in milliseconds (0 = only on change).
    pub auto_refresh_ms: u64,
}

impl Default for PreviewPanelConfig {
    fn default() -> Self {
        Self {
            preview_mode: PreviewMode::Pending,
            allow_manual_refresh: true,
            show_url_bar: true,
            show_device_frames: true,
            auto_refresh_ms: 0,
        }
    }
}

/// How the preview content is rendered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PreviewMode {
    /// No preview yet — waiting for the dev server to start.
    Pending,
    /// Live iframe pointing to a dev server URL (web apps).
    Iframe { url: String },
    /// Periodic screenshot capture from an emulator (native mobile).
    EmulatorCapture {
        /// Base64-encoded latest screenshot, or URL to screenshot endpoint.
        image_source: String,
        /// Capture interval in milliseconds.
        capture_interval_ms: u64,
        /// Device label (e.g. "iPhone 15", "Pixel 8").
        device_label: String,
    },
    /// ANSI terminal output rendered as HTML (CLI apps).
    TerminalRender {
        /// The terminal output (ANSI escape codes preserved).
        content: String,
    },
    /// HTTP request/response viewer (API servers).
    ApiViewer {
        /// Base URL of the API server.
        base_url: String,
    },
    /// Static file preview (images, PDFs, etc.).
    StaticFile {
        /// File path or URL.
        source: String,
        /// MIME type.
        mime_type: String,
    },
    /// Preview is unavailable for this project type.
    Unavailable { reason: String },
}

// ── Project type detection → preview strategy ──────────────────────

/// Detected project type used to choose the preview strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    /// React, Vue, Angular, Svelte, Next.js, Nuxt, etc.
    WebFrontend,
    /// Express, Fastify, Django, Flask, Axum, Actix, etc.
    ApiServer,
    /// Full-stack web app (frontend + backend).
    FullStack,
    /// React Native / Expo.
    ReactNative,
    /// Flutter.
    Flutter,
    /// iOS (Swift/ObjC) or Android (Kotlin/Java) native.
    MobileNative,
    /// Electron, Tauri.
    DesktopApp,
    /// Command-line tool.
    CliTool,
    /// Library / package (no runnable output).
    Library,
    /// Unknown — couldn't detect.
    Unknown,
}

impl ProjectType {
    /// Determine the best preview mode for this project type.
    pub fn default_preview_mode(self) -> PreviewMode {
        match self {
            ProjectType::WebFrontend | ProjectType::FullStack | ProjectType::DesktopApp => {
                // Will be replaced with actual URL once dev server starts.
                PreviewMode::Pending
            }
            ProjectType::ApiServer => PreviewMode::ApiViewer {
                base_url: "http://localhost:3000".to_string(),
            },
            ProjectType::ReactNative => PreviewMode::Iframe {
                // Expo web mode.
                url: "http://localhost:19006".to_string(),
            },
            ProjectType::Flutter => PreviewMode::Iframe {
                url: "http://localhost:8080".to_string(),
            },
            ProjectType::MobileNative => PreviewMode::EmulatorCapture {
                image_source: String::new(),
                capture_interval_ms: 2000,
                device_label: "Emulator".to_string(),
            },
            ProjectType::CliTool => PreviewMode::TerminalRender {
                content: String::new(),
            },
            ProjectType::Library => PreviewMode::Unavailable {
                reason: "Library projects have no visual output. \
                         Test results are shown in the terminal panel."
                    .to_string(),
            },
            ProjectType::Unknown => PreviewMode::Pending,
        }
    }

    /// Dev server start command hint for this project type.
    pub fn dev_server_hint(self) -> Option<&'static str> {
        match self {
            ProjectType::WebFrontend | ProjectType::FullStack => {
                Some("npm run dev / yarn dev / pnpm dev")
            }
            ProjectType::ApiServer => Some("npm start / python manage.py runserver / cargo run"),
            ProjectType::ReactNative => Some("npx expo start --web"),
            ProjectType::Flutter => Some("flutter run -d web-server --web-port=8080"),
            ProjectType::DesktopApp => Some("npm run dev (Electron) / cargo tauri dev"),
            _ => None,
        }
    }
}

/// Heuristic detection of project type from file presence.
///
/// The caller passes in a list of filenames present in the project root.
/// This avoids filesystem access inside the function.
pub fn detect_project_type(root_files: &[&str]) -> ProjectType {
    let has = |name: &str| root_files.contains(&name);

    // ── Mobile frameworks ──
    if has("app.json") && has("babel.config.js") {
        return ProjectType::ReactNative; // Expo / RN
    }
    if has("pubspec.yaml") {
        return ProjectType::Flutter;
    }
    if has("Podfile") && !has("package.json") {
        return ProjectType::MobileNative; // iOS
    }
    if has("build.gradle") && !has("package.json") {
        return ProjectType::MobileNative; // Android
    }

    // ── Desktop ──
    if has("electron-builder.yml") || has("electron-builder.json") {
        return ProjectType::DesktopApp;
    }
    if has("tauri.conf.json") {
        return ProjectType::DesktopApp;
    }

    // ── Web frontend ──
    if has("next.config.js") || has("next.config.mjs") || has("next.config.ts") {
        return ProjectType::FullStack;
    }
    if has("nuxt.config.ts") || has("nuxt.config.js") {
        return ProjectType::FullStack;
    }
    if has("vite.config.ts")
        || has("vite.config.js")
        || has("webpack.config.js")
        || has("svelte.config.js")
        || has("angular.json")
    {
        return ProjectType::WebFrontend;
    }

    // ── Rust ──
    if has("Cargo.toml") {
        // Limitation: distinguishing a Rust API server (axum/actix/rocket)
        // from a CLI tool requires reading Cargo.toml dependencies, which
        // this heuristic does not do.  Callers can override the detected
        // type when more context is available.
        if has("src/main.rs") {
            return ProjectType::CliTool;
        }
        return ProjectType::Library;
    }

    // ── Python ──
    if has("manage.py") || has("app.py") || has("main.py") {
        if has("templates") || has("static") {
            return ProjectType::FullStack;
        }
        return ProjectType::ApiServer;
    }

    // ── Node.js ──
    if has("package.json") {
        if has("src/index.html") || has("public/index.html") || has("index.html") {
            return ProjectType::WebFrontend;
        }
        return ProjectType::ApiServer;
    }

    // ── Go ──
    if has("go.mod") {
        if has("main.go") || has("cmd") {
            return ProjectType::CliTool;
        }
        return ProjectType::Library;
    }

    ProjectType::Unknown
}

// ── Workflow panel live state ───────────────────────────────────────

/// Real-time state of the workflow panel, updated as coding progresses.
/// Sent to the frontend via SSE or polling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPanelState {
    /// Current coding phase.
    pub current_phase: super::CodingPhase,
    /// Phase completion status for the progress bar.
    pub phase_status: Vec<PhaseDisplayStatus>,
    /// Current thinking / reasoning text (streamed).
    pub thinking: String,
    /// Recent code changes (file path + diff summary).
    pub recent_changes: Vec<CodeChange>,
    /// Recent terminal output lines.
    pub terminal_lines: Vec<TerminalLine>,
    /// Current error being analysed (if any).
    pub active_error: Option<ActiveError>,
}

/// Display status of a single phase in the progress bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseDisplayStatus {
    pub phase: super::CodingPhase,
    pub label: String,
    pub status: PhaseProgressStatus,
}

/// Visual status of a phase in the progress tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseProgressStatus {
    /// Phase completed successfully.
    Completed,
    /// Phase currently in progress.
    Active,
    /// Phase was skipped.
    Skipped,
    /// Phase not yet reached.
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub change_type: String,  // "created", "modified", "deleted"
    pub summary: String,      // e.g. "+42 -3 lines"
    pub diff_preview: String, // first few lines of diff
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalLine {
    pub timestamp_ms: u64,
    pub stream: String, // "stdout", "stderr", "system"
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveError {
    pub error_class: String,
    pub severity: u8,
    pub message: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub fix_strategy: String,
    pub attempt_number: usize,
}

// ── Preview panel live state ───────────────────────────────────────

/// Real-time state of the preview panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewPanelState {
    /// Current preview mode.
    pub mode: PreviewMode,
    /// Whether the dev server is running.
    pub server_running: bool,
    /// Dev server URL (if running).
    pub server_url: Option<String>,
    /// Dev server port.
    pub server_port: Option<u16>,
    /// Last server health check result.
    pub last_health_check: Option<PreviewHealthCheck>,
    /// Console log lines captured from the preview (browser console).
    pub console_logs: Vec<ConsoleLogEntry>,
    /// Selected device frame for responsive preview.
    pub device_frame: DeviceFrame,
}

impl Default for PreviewPanelState {
    fn default() -> Self {
        Self {
            mode: PreviewMode::Pending,
            server_running: false,
            server_url: None,
            server_port: None,
            last_health_check: None,
            console_logs: Vec::new(),
            device_frame: DeviceFrame::Desktop,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewHealthCheck {
    pub status_code: u16,
    pub response_time_ms: u64,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleLogEntry {
    pub level: String, // "log", "warn", "error", "info"
    pub message: String,
    pub timestamp_ms: u64,
}

/// Device frame presets for responsive preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceFrame {
    /// No frame, full width.
    Desktop,
    /// 768×1024 (iPad).
    Tablet,
    /// 390×844 (iPhone 15).
    Phone,
    /// Custom dimensions.
    Custom,
}

impl DeviceFrame {
    pub fn dimensions(self) -> (u32, u32) {
        match self {
            DeviceFrame::Desktop => (1280, 800),
            DeviceFrame::Tablet => (768, 1024),
            DeviceFrame::Phone => (390, 844),
            DeviceFrame::Custom => (0, 0), // frontend supplies dimensions
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DeviceFrame::Desktop => "Desktop",
            DeviceFrame::Tablet => "Tablet",
            DeviceFrame::Phone => "Phone",
            DeviceFrame::Custom => "Custom",
        }
    }
}

// ── Combined coding session state ──────────────────────────────────

/// Complete state of a coding session, combining both panels.
/// This is the payload returned by `GET /api/coding/state`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodingSessionState {
    pub layout: CodingLayout,
    pub project_type: ProjectType,
    pub workflow: WorkflowPanelState,
    pub preview: PreviewPanelState,
    /// Session ID for multiplexing.
    pub session_id: String,
    /// Sandbox iteration count (from the run→observe→fix loop).
    pub sandbox_iteration: usize,
    /// Total elapsed time in seconds.
    pub elapsed_secs: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_is_horizontal_50_50() {
        let layout = CodingLayout::default();
        assert_eq!(layout.orientation, LayoutOrientation::Horizontal);
        assert_eq!(layout.workflow_ratio, 50);
    }

    #[test]
    fn mobile_layout_is_vertical() {
        let layout = CodingLayout::mobile();
        assert_eq!(layout.orientation, LayoutOrientation::Vertical);
        assert_eq!(layout.workflow_ratio, 55);
    }

    #[test]
    fn workflow_panel_default_sections() {
        let config = WorkflowPanelConfig::default();
        assert_eq!(config.sections.len(), 4);
        assert_eq!(config.sections[0], WorkflowSection::PhaseTracker);
        assert_eq!(config.sections[3], WorkflowSection::Terminal);
    }

    #[test]
    fn workflow_sections_have_labels_and_icons() {
        let sections = [
            WorkflowSection::PhaseTracker,
            WorkflowSection::ThinkingProcess,
            WorkflowSection::CodeChanges,
            WorkflowSection::Terminal,
            WorkflowSection::ErrorAnalysis,
        ];
        for s in &sections {
            assert!(!s.label().is_empty());
            assert!(!s.icon().is_empty());
        }
    }

    #[test]
    fn detect_react_project() {
        let files = ["package.json", "vite.config.ts", "src", "tsconfig.json"];
        assert_eq!(detect_project_type(&files), ProjectType::WebFrontend);
    }

    #[test]
    fn detect_nextjs_project() {
        let files = ["package.json", "next.config.js", "pages"];
        assert_eq!(detect_project_type(&files), ProjectType::FullStack);
    }

    #[test]
    fn detect_expo_project() {
        let files = ["package.json", "app.json", "babel.config.js"];
        assert_eq!(detect_project_type(&files), ProjectType::ReactNative);
    }

    #[test]
    fn detect_flutter_project() {
        let files = ["pubspec.yaml", "lib", "android", "ios"];
        assert_eq!(detect_project_type(&files), ProjectType::Flutter);
    }

    #[test]
    fn detect_rust_cli() {
        let files = ["Cargo.toml", "src/main.rs"];
        assert_eq!(detect_project_type(&files), ProjectType::CliTool);
    }

    #[test]
    fn detect_rust_library() {
        let files = ["Cargo.toml", "src/lib.rs"];
        assert_eq!(detect_project_type(&files), ProjectType::Library);
    }

    #[test]
    fn detect_python_api() {
        let files = ["manage.py", "requirements.txt"];
        assert_eq!(detect_project_type(&files), ProjectType::ApiServer);
    }

    #[test]
    fn detect_python_fullstack() {
        let files = ["manage.py", "templates", "static"];
        assert_eq!(detect_project_type(&files), ProjectType::FullStack);
    }

    #[test]
    fn detect_electron_app() {
        let files = ["package.json", "electron-builder.json", "src"];
        assert_eq!(detect_project_type(&files), ProjectType::DesktopApp);
    }

    #[test]
    fn detect_tauri_app() {
        let files = ["package.json", "tauri.conf.json"];
        assert_eq!(detect_project_type(&files), ProjectType::DesktopApp);
    }

    #[test]
    fn detect_ios_native() {
        let files = ["Podfile", "AppDelegate.swift"];
        assert_eq!(detect_project_type(&files), ProjectType::MobileNative);
    }

    #[test]
    fn detect_android_native() {
        let files = ["build.gradle", "settings.gradle", "app"];
        assert_eq!(detect_project_type(&files), ProjectType::MobileNative);
    }

    #[test]
    fn detect_node_api() {
        let files = ["package.json", "server.js"];
        assert_eq!(detect_project_type(&files), ProjectType::ApiServer);
    }

    #[test]
    fn detect_unknown() {
        let files = ["README.md", "LICENSE"];
        assert_eq!(detect_project_type(&files), ProjectType::Unknown);
    }

    #[test]
    fn project_type_preview_modes() {
        // Web → Pending (then Iframe once server starts).
        assert!(matches!(
            ProjectType::WebFrontend.default_preview_mode(),
            PreviewMode::Pending
        ));
        // API → ApiViewer.
        assert!(matches!(
            ProjectType::ApiServer.default_preview_mode(),
            PreviewMode::ApiViewer { .. }
        ));
        // React Native → Iframe (Expo web).
        assert!(matches!(
            ProjectType::ReactNative.default_preview_mode(),
            PreviewMode::Iframe { .. }
        ));
        // Mobile native → EmulatorCapture.
        assert!(matches!(
            ProjectType::MobileNative.default_preview_mode(),
            PreviewMode::EmulatorCapture { .. }
        ));
        // CLI → TerminalRender.
        assert!(matches!(
            ProjectType::CliTool.default_preview_mode(),
            PreviewMode::TerminalRender { .. }
        ));
        // Library → Unavailable.
        assert!(matches!(
            ProjectType::Library.default_preview_mode(),
            PreviewMode::Unavailable { .. }
        ));
    }

    #[test]
    fn device_frame_dimensions() {
        assert_eq!(DeviceFrame::Phone.dimensions(), (390, 844));
        assert_eq!(DeviceFrame::Tablet.dimensions(), (768, 1024));
        assert_eq!(DeviceFrame::Desktop.dimensions(), (1280, 800));
    }

    #[test]
    fn project_type_dev_server_hints() {
        assert!(ProjectType::WebFrontend.dev_server_hint().is_some());
        assert!(ProjectType::ReactNative.dev_server_hint().is_some());
        assert!(ProjectType::Flutter.dev_server_hint().is_some());
        assert!(ProjectType::Library.dev_server_hint().is_none());
        assert!(ProjectType::Unknown.dev_server_hint().is_none());
    }

    #[test]
    fn preview_panel_state_default() {
        let state = PreviewPanelState::default();
        assert!(!state.server_running);
        assert!(state.server_url.is_none());
        assert!(state.console_logs.is_empty());
        assert_eq!(state.device_frame, DeviceFrame::Desktop);
    }

    #[test]
    fn coding_layout_serialization_roundtrip() {
        let layout = CodingLayout::default();
        let json = serde_json::to_string(&layout).unwrap();
        let parsed: CodingLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.orientation, LayoutOrientation::Horizontal);
        assert_eq!(parsed.workflow_ratio, 50);
    }
}
