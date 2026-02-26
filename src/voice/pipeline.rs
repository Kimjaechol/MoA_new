//! Voice processing pipeline: real-time voice interpretation and conversation.
//!
//! Implements the voice provider trait, language detection, session management,
//! and billing integration for voice sessions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Language codes (25 supported languages) ──────────────────────

/// ISO 639-1 language codes supported by the voice pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageCode {
    // East Asia
    Ko,   // Korean
    Ja,   // Japanese
    Zh,   // Chinese (Simplified)
    ZhTw, // Chinese (Traditional)

    // Southeast Asia
    Th, // Thai
    Vi, // Vietnamese
    Id, // Indonesian
    Ms, // Malay
    Tl, // Filipino/Tagalog

    // South Asia
    Hi, // Hindi

    // Europe (Western)
    En, // English
    Es, // Spanish
    Fr, // French
    De, // German
    It, // Italian
    Pt, // Portuguese
    Nl, // Dutch
    Pl, // Polish
    Cs, // Czech
    Sv, // Swedish
    Da, // Danish

    // Eastern Europe
    Ru, // Russian
    Uk, // Ukrainian
    Tr, // Turkish

    // Middle East
    Ar, // Arabic
}

impl LanguageCode {
    /// Get the ISO 639-1 code string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ko => "ko",
            Self::Ja => "ja",
            Self::Zh => "zh",
            Self::ZhTw => "zh-TW",
            Self::Th => "th",
            Self::Vi => "vi",
            Self::Id => "id",
            Self::Ms => "ms",
            Self::Tl => "tl",
            Self::Hi => "hi",
            Self::En => "en",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::De => "de",
            Self::It => "it",
            Self::Pt => "pt",
            Self::Nl => "nl",
            Self::Pl => "pl",
            Self::Cs => "cs",
            Self::Sv => "sv",
            Self::Da => "da",
            Self::Ru => "ru",
            Self::Uk => "uk",
            Self::Tr => "tr",
            Self::Ar => "ar",
        }
    }

    /// Get the human-readable language name.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Ko => "Korean",
            Self::Ja => "Japanese",
            Self::Zh => "Chinese (Simplified)",
            Self::ZhTw => "Chinese (Traditional)",
            Self::Th => "Thai",
            Self::Vi => "Vietnamese",
            Self::Id => "Indonesian",
            Self::Ms => "Malay",
            Self::Tl => "Filipino",
            Self::Hi => "Hindi",
            Self::En => "English",
            Self::Es => "Spanish",
            Self::Fr => "French",
            Self::De => "German",
            Self::It => "Italian",
            Self::Pt => "Portuguese",
            Self::Nl => "Dutch",
            Self::Pl => "Polish",
            Self::Cs => "Czech",
            Self::Sv => "Swedish",
            Self::Da => "Danish",
            Self::Ru => "Russian",
            Self::Uk => "Ukrainian",
            Self::Tr => "Turkish",
            Self::Ar => "Arabic",
        }
    }

    /// Parse from string code (case-insensitive).
    pub fn from_str_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "ko" => Some(Self::Ko),
            "ja" => Some(Self::Ja),
            "zh" => Some(Self::Zh),
            "zh-tw" | "zh_tw" => Some(Self::ZhTw),
            "th" => Some(Self::Th),
            "vi" => Some(Self::Vi),
            "id" => Some(Self::Id),
            "ms" => Some(Self::Ms),
            "tl" => Some(Self::Tl),
            "hi" => Some(Self::Hi),
            "en" => Some(Self::En),
            "es" => Some(Self::Es),
            "fr" => Some(Self::Fr),
            "de" => Some(Self::De),
            "it" => Some(Self::It),
            "pt" => Some(Self::Pt),
            "nl" => Some(Self::Nl),
            "pl" => Some(Self::Pl),
            "cs" => Some(Self::Cs),
            "sv" => Some(Self::Sv),
            "da" => Some(Self::Da),
            "ru" => Some(Self::Ru),
            "uk" => Some(Self::Uk),
            "tr" => Some(Self::Tr),
            "ar" => Some(Self::Ar),
            _ => None,
        }
    }

    /// Return all 25 supported language codes.
    pub fn all() -> &'static [LanguageCode] {
        &[
            Self::Ko,
            Self::Ja,
            Self::Zh,
            Self::ZhTw,
            Self::Th,
            Self::Vi,
            Self::Id,
            Self::Ms,
            Self::Tl,
            Self::Hi,
            Self::En,
            Self::Es,
            Self::Fr,
            Self::De,
            Self::It,
            Self::Pt,
            Self::Nl,
            Self::Pl,
            Self::Cs,
            Self::Sv,
            Self::Da,
            Self::Ru,
            Self::Uk,
            Self::Tr,
            Self::Ar,
        ]
    }
}

// ── Language detection via Unicode ranges ─────────────────────────

/// Detect the most likely language from text using Unicode character ranges.
///
/// Falls back to the specified default language if detection is inconclusive.
pub fn detect_language(text: &str, default: LanguageCode) -> LanguageCode {
    if text.is_empty() {
        return default;
    }

    let mut hangul = 0u32;
    let mut kana = 0u32;
    let mut cjk = 0u32;
    let mut arabic = 0u32;
    let mut thai = 0u32;
    let mut devanagari = 0u32;
    let mut cyrillic = 0u32;
    let mut total = 0u32;

    for c in text.chars() {
        if c.is_whitespace() || c.is_ascii_punctuation() {
            continue;
        }
        total += 1;

        match c as u32 {
            // Hangul Syllables + Jamo
            0xAC00..=0xD7AF | 0x1100..=0x11FF | 0x3130..=0x318F => hangul += 1,
            // Hiragana + Katakana
            0x3040..=0x309F | 0x30A0..=0x30FF | 0x31F0..=0x31FF => kana += 1,
            // CJK Unified Ideographs
            0x4E00..=0x9FFF | 0x3400..=0x4DBF => cjk += 1,
            // Arabic
            0x0600..=0x06FF | 0x0750..=0x077F | 0x08A0..=0x08FF => arabic += 1,
            // Thai
            0x0E00..=0x0E7F => thai += 1,
            // Devanagari (Hindi)
            0x0900..=0x097F => devanagari += 1,
            // Cyrillic
            0x0400..=0x052F => cyrillic += 1,
            _ => {}
        }
    }

    if total == 0 {
        return default;
    }

    // Require at least 20% of non-space chars to match a script
    let threshold = total / 5;

    if hangul > threshold {
        return LanguageCode::Ko;
    }
    if kana > threshold {
        return LanguageCode::Ja;
    }
    // CJK without kana → Chinese (Simplified by default)
    if cjk > threshold && kana == 0 {
        return LanguageCode::Zh;
    }
    if arabic > threshold {
        return LanguageCode::Ar;
    }
    if thai > threshold {
        return LanguageCode::Th;
    }
    if devanagari > threshold {
        return LanguageCode::Hi;
    }
    if cyrillic > threshold {
        return LanguageCode::Ru;
    }

    default
}

// ── Formality and Domain ─────────────────────────────────────────

/// Formality level for interpretation output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Formality {
    Formal,
    #[default]
    Neutral,
    Casual,
}

/// Domain specialization for interpretation accuracy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Domain {
    #[default]
    General,
    Business,
    Medical,
    Legal,
    Technical,
}

// ── Voice provider abstraction ───────────────────────────────────

/// Kind of voice provider backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceProviderKind {
    /// Gemini 2.5 Flash Native Audio Dialog.
    GeminiLive,
    /// OpenAI GPT-4o Realtime.
    OpenAiRealtime,
}

impl VoiceProviderKind {
    /// Get the model identifier string for API calls.
    pub fn model_id(self) -> &'static str {
        match self {
            Self::GeminiLive => "gemini-2.5-flash-native-audio-preview-12-2025",
            Self::OpenAiRealtime => "gpt-4o-realtime-preview",
        }
    }
}

/// Voice provider trait for real-time audio streaming.
///
/// Implementations handle WebSocket/streaming connections to voice APIs.
#[async_trait]
pub trait VoiceProvider: Send + Sync {
    /// Connect to the voice API for a given user session.
    async fn connect(&self, session_id: &str, config: &InterpreterConfig) -> anyhow::Result<()>;

    /// Send an audio chunk (PCM/opus bytes) to the provider.
    async fn send_audio(&self, session_id: &str, chunk: &[u8]) -> anyhow::Result<()>;

    /// Send a text message to the provider (for text-based interpretation).
    async fn send_text(&self, session_id: &str, text: &str) -> anyhow::Result<String>;

    /// Disconnect a voice session.
    async fn disconnect(&self, session_id: &str) -> anyhow::Result<()>;

    /// Get the provider kind.
    fn kind(&self) -> VoiceProviderKind;
}

// ── Interpreter configuration ────────────────────────────────────

/// Configuration for a voice interpretation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterConfig {
    /// Source language (input).
    pub source_language: LanguageCode,
    /// Target language (output).
    pub target_language: LanguageCode,
    /// Enable bidirectional auto-switch between source and target.
    pub bidirectional: bool,
    /// Formality level of interpretation output.
    pub formality: Formality,
    /// Domain specialization.
    pub domain: Domain,
    /// Preserve tone and emotion in interpretation.
    pub preserve_tone: bool,
    /// API key override (uses provider default if None).
    pub api_key: Option<String>,
    /// Voice provider to use.
    pub provider: VoiceProviderKind,
}

impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            source_language: LanguageCode::Ko,
            target_language: LanguageCode::En,
            bidirectional: false,
            formality: Formality::default(),
            domain: Domain::default(),
            preserve_tone: true,
            api_key: None,
            provider: VoiceProviderKind::GeminiLive,
        }
    }
}

impl InterpreterConfig {
    /// Build a system prompt for the interpretation session.
    pub fn build_system_prompt(&self) -> String {
        let formality_instruction = match self.formality {
            Formality::Formal => {
                "Use formal, polite language appropriate for professional or official settings."
            }
            Formality::Neutral => "Use standard, everyday language.",
            Formality::Casual => {
                "Use casual, friendly language appropriate for informal conversations."
            }
        };

        let domain_instruction = match self.domain {
            Domain::General => "",
            Domain::Business => " Specialize in business and corporate terminology.",
            Domain::Medical => " Specialize in medical and healthcare terminology.",
            Domain::Legal => " Specialize in legal and judicial terminology.",
            Domain::Technical => " Specialize in technical and engineering terminology.",
        };

        let tone_instruction = if self.preserve_tone {
            " Preserve the speaker's emotional tone, emphasis, and intent."
        } else {
            ""
        };

        let direction = if self.bidirectional {
            format!(
                "Bidirectional: {} ↔ {}. \
                 Detect input language. Output the OTHER language immediately. \
                 Never output the same language as input.",
                self.source_language.display_name(),
                self.target_language.display_name(),
            )
        } else {
            format!(
                "{} → {} only.",
                self.source_language.display_name(),
                self.target_language.display_name(),
            )
        };

        format!(
            "You are a live interpreter. {direction} {formality_instruction}{domain_instruction}{tone_instruction} \
             CRITICAL: Speak the translation IMMEDIATELY. Never explain, never describe what you are doing, \
             never output text like \"Translating...\" — just speak the translated words directly."
        )
    }
}

// ── Session status ───────────────────────────────────────────────

/// Status of a voice interpretation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterpreterStatus {
    /// Session created but not connected.
    Idle,
    /// Connecting to voice provider.
    Connecting,
    /// Connected and ready to receive audio.
    Ready,
    /// Actively listening to audio input.
    Listening,
    /// Processing/interpreting audio.
    Interpreting,
    /// Outputting interpreted audio/text.
    Speaking,
    /// Error state.
    Error,
    /// Session closed.
    Closed,
}

// ── Session statistics ───────────────────────────────────────────

/// Statistics for a voice interpretation session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterpreterStats {
    /// Number of utterances processed.
    pub utterance_count: u64,
    /// Total session duration in milliseconds.
    pub total_duration_ms: u64,
    /// Average interpretation latency in milliseconds.
    pub avg_latency_ms: f64,
    /// Total source language word count.
    pub source_words: u64,
    /// Total target language word count.
    pub target_words: u64,
    /// Estimated tokens consumed (for billing).
    pub estimated_tokens: u64,
}

impl InterpreterStats {
    /// Record a completed utterance interpretation.
    pub fn record_utterance(
        &mut self,
        latency_ms: u64,
        source_word_count: u64,
        target_word_count: u64,
    ) {
        self.utterance_count += 1;
        self.source_words += source_word_count;
        self.target_words += target_word_count;

        // Running average for latency
        let prev_total = self.avg_latency_ms * (self.utterance_count - 1) as f64;
        self.avg_latency_ms = (prev_total + latency_ms as f64) / self.utterance_count as f64;

        // Rough token estimate: ~3/4 tokens per word for source + target
        self.estimated_tokens += (source_word_count + target_word_count) * 3 / 4;
    }

    /// Estimate cost in credits based on token consumption.
    /// Voice sessions use a higher rate: 1 credit per 500 estimated tokens.
    pub fn estimated_credits(&self) -> u64 {
        self.estimated_tokens.div_ceil(500)
    }
}

// ── Interpreter session ──────────────────────────────────────────

/// A voice interpretation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterSession {
    /// Unique session identifier.
    pub id: String,
    /// User who owns this session.
    pub user_id: String,
    /// Session configuration.
    pub config: InterpreterConfig,
    /// Current session status.
    pub status: InterpreterStatus,
    /// Session statistics.
    pub stats: InterpreterStats,
    /// Session creation timestamp (epoch ms).
    pub created_at: u64,
}

impl InterpreterSession {
    /// Create a new session.
    pub fn new(session_id: String, user_id: String, config: InterpreterConfig) -> Self {
        let now_ms = now_epoch_ms();

        Self {
            id: session_id,
            user_id,
            config,
            status: InterpreterStatus::Idle,
            stats: InterpreterStats::default(),
            created_at: now_ms,
        }
    }
}

// ── Session manager ──────────────────────────────────────────────

/// Manages active voice interpretation sessions.
pub struct VoiceSessionManager {
    /// Active sessions indexed by session ID.
    sessions: Arc<Mutex<HashMap<String, InterpreterSession>>>,
    /// Maximum concurrent sessions per user.
    max_sessions_per_user: usize,
    /// Whether voice features are enabled.
    enabled: bool,
    /// Default source language code (from config).
    default_source_language: String,
    /// Default target language code (from config).
    default_target_language: String,
    /// Default voice provider ("gemini" or "openai").
    default_provider: Option<String>,
}

impl VoiceSessionManager {
    /// Create a new session manager.
    pub fn new(enabled: bool, max_sessions_per_user: usize) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions_per_user,
            enabled,
            default_source_language: "ko".to_string(),
            default_target_language: "en".to_string(),
            default_provider: None,
        }
    }

    /// Create a new session manager with explicit default languages.
    pub fn with_defaults(
        enabled: bool,
        max_sessions_per_user: usize,
        default_source_language: String,
        default_target_language: String,
        default_provider: Option<String>,
    ) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_sessions_per_user,
            enabled,
            default_source_language,
            default_target_language,
            default_provider,
        }
    }

    /// Get the default source language code.
    pub fn default_source_language(&self) -> &str {
        &self.default_source_language
    }

    /// Get the default target language code.
    pub fn default_target_language(&self) -> &str {
        &self.default_target_language
    }

    /// Get the default voice provider name.
    pub fn default_provider(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    /// Create a new interpretation session.
    pub async fn create_session(
        &self,
        user_id: &str,
        config: InterpreterConfig,
    ) -> anyhow::Result<InterpreterSession> {
        if !self.enabled {
            anyhow::bail!("Voice features are disabled");
        }

        let mut sessions = self.sessions.lock().await;

        // Check per-user session limit
        let user_session_count = sessions
            .values()
            .filter(|s| s.user_id == user_id && s.status != InterpreterStatus::Closed)
            .count();

        if user_session_count >= self.max_sessions_per_user {
            anyhow::bail!(
                "Maximum concurrent voice sessions ({}) reached for user",
                self.max_sessions_per_user
            );
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = InterpreterSession::new(session_id.clone(), user_id.to_string(), config);
        sessions.insert(session_id, session.clone());

        Ok(session)
    }

    /// Get a session by ID.
    pub async fn get_session(&self, session_id: &str) -> Option<InterpreterSession> {
        let sessions = self.sessions.lock().await;
        sessions.get(session_id).cloned()
    }

    /// Update session status.
    pub async fn update_status(
        &self,
        session_id: &str,
        status: InterpreterStatus,
    ) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {session_id}"))?;
        session.status = status;
        Ok(())
    }

    /// Record a completed utterance for a session.
    pub async fn record_utterance(
        &self,
        session_id: &str,
        latency_ms: u64,
        source_words: u64,
        target_words: u64,
    ) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {session_id}"))?;
        session
            .stats
            .record_utterance(latency_ms, source_words, target_words);
        Ok(())
    }

    /// Close a session and return final stats.
    pub async fn close_session(&self, session_id: &str) -> anyhow::Result<InterpreterStats> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {session_id}"))?;

        let now_ms = now_epoch_ms();

        session.status = InterpreterStatus::Closed;
        session.stats.total_duration_ms = now_ms.saturating_sub(session.created_at);

        Ok(session.stats.clone())
    }

    /// List all active sessions for a user.
    pub async fn list_user_sessions(&self, user_id: &str) -> Vec<InterpreterSession> {
        let sessions = self.sessions.lock().await;
        sessions
            .values()
            .filter(|s| s.user_id == user_id && s.status != InterpreterStatus::Closed)
            .cloned()
            .collect()
    }

    /// Clean up closed sessions older than the given age in milliseconds.
    pub async fn cleanup_closed(&self, max_age_ms: u64) {
        let now_ms = now_epoch_ms();

        let mut sessions = self.sessions.lock().await;
        sessions.retain(|_, s| {
            if s.status == InterpreterStatus::Closed {
                now_ms.saturating_sub(s.created_at) < max_age_ms
            } else {
                true
            }
        });
    }

    /// Get total active session count.
    pub async fn active_session_count(&self) -> usize {
        let sessions = self.sessions.lock().await;
        sessions
            .values()
            .filter(|s| s.status != InterpreterStatus::Closed)
            .count()
    }
}

// ── Gemini Live voice provider (stub) ────────────────────────────

/// Gemini 2.5 Flash Native Audio voice provider.
///
/// Connects to Google's Gemini Live API for real-time voice interpretation.
/// Uses WebSocket streaming for bidirectional audio.
pub struct GeminiLiveProvider {
    /// API key for Gemini API.
    api_key: Option<String>,
    /// VAD threshold (default 0.4).
    vad_threshold: f32,
    /// Silence detection timeout in milliseconds (default 300ms).
    silence_timeout_ms: u64,
}

impl GeminiLiveProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key,
            vad_threshold: 0.4,
            silence_timeout_ms: 300,
        }
    }

    /// Get VAD threshold.
    pub fn vad_threshold(&self) -> f32 {
        self.vad_threshold
    }

    /// Get silence timeout.
    pub fn silence_timeout_ms(&self) -> u64 {
        self.silence_timeout_ms
    }
}

#[async_trait]
impl VoiceProvider for GeminiLiveProvider {
    async fn connect(&self, session_id: &str, config: &InterpreterConfig) -> anyhow::Result<()> {
        if self.api_key.is_none() {
            anyhow::bail!("Gemini API key is required for voice sessions");
        }

        tracing::info!(
            session_id = session_id,
            model = VoiceProviderKind::GeminiLive.model_id(),
            source = config.source_language.as_str(),
            target = config.target_language.as_str(),
            "Connecting to Gemini Live voice API"
        );

        // WebSocket connection would be established here in production.
        // The actual WebSocket streaming implementation depends on the
        // runtime environment and is handled by the gateway layer.
        Ok(())
    }

    async fn send_audio(&self, session_id: &str, chunk: &[u8]) -> anyhow::Result<()> {
        if chunk.is_empty() {
            return Ok(());
        }

        tracing::trace!(
            session_id = session_id,
            chunk_size = chunk.len(),
            "Sending audio chunk to Gemini Live"
        );

        Ok(())
    }

    async fn send_text(&self, session_id: &str, text: &str) -> anyhow::Result<String> {
        tracing::debug!(
            session_id = session_id,
            text_len = text.len(),
            "Sending text to Gemini Live for interpretation"
        );

        // In production, this would send the text to the Gemini API
        // and return the interpreted text.
        Ok(format!("[Gemini interpretation of: {}]", text))
    }

    async fn disconnect(&self, session_id: &str) -> anyhow::Result<()> {
        tracing::info!(session_id = session_id, "Disconnecting Gemini Live session");
        Ok(())
    }

    fn kind(&self) -> VoiceProviderKind {
        VoiceProviderKind::GeminiLive
    }
}

// ── OpenAI Realtime voice provider (stub) ────────────────────────

/// OpenAI GPT-4o Realtime voice provider.
///
/// Fallback voice provider using OpenAI's Realtime API.
pub struct OpenAiRealtimeProvider {
    /// API key for OpenAI API.
    api_key: Option<String>,
}

impl OpenAiRealtimeProvider {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl VoiceProvider for OpenAiRealtimeProvider {
    async fn connect(&self, session_id: &str, config: &InterpreterConfig) -> anyhow::Result<()> {
        if self.api_key.is_none() {
            anyhow::bail!("OpenAI API key is required for voice sessions");
        }

        tracing::info!(
            session_id = session_id,
            model = VoiceProviderKind::OpenAiRealtime.model_id(),
            source = config.source_language.as_str(),
            target = config.target_language.as_str(),
            "Connecting to OpenAI Realtime voice API"
        );

        Ok(())
    }

    async fn send_audio(&self, session_id: &str, chunk: &[u8]) -> anyhow::Result<()> {
        if chunk.is_empty() {
            return Ok(());
        }

        tracing::trace!(
            session_id = session_id,
            chunk_size = chunk.len(),
            "Sending audio chunk to OpenAI Realtime"
        );

        Ok(())
    }

    async fn send_text(&self, session_id: &str, text: &str) -> anyhow::Result<String> {
        tracing::debug!(
            session_id = session_id,
            text_len = text.len(),
            "Sending text to OpenAI Realtime for interpretation"
        );

        Ok(format!("[OpenAI interpretation of: {}]", text))
    }

    async fn disconnect(&self, session_id: &str) -> anyhow::Result<()> {
        tracing::info!(
            session_id = session_id,
            "Disconnecting OpenAI Realtime session"
        );
        Ok(())
    }

    fn kind(&self) -> VoiceProviderKind {
        VoiceProviderKind::OpenAiRealtime
    }
}

// ── Factory ──────────────────────────────────────────────────────

/// Create a voice provider by kind.
pub fn create_voice_provider(
    kind: VoiceProviderKind,
    api_key: Option<String>,
) -> Box<dyn VoiceProvider> {
    match kind {
        VoiceProviderKind::GeminiLive => Box::new(GeminiLiveProvider::new(api_key)),
        VoiceProviderKind::OpenAiRealtime => Box::new(OpenAiRealtimeProvider::new(api_key)),
    }
}

/// Get current time in epoch milliseconds.
fn now_epoch_ms() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    )
    .unwrap_or(u64::MAX)
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_code_roundtrip() {
        for lang in LanguageCode::all() {
            let code = lang.as_str();
            let parsed = LanguageCode::from_str_code(code);
            assert_eq!(parsed, Some(*lang), "Roundtrip failed for {code}");
        }
    }

    #[test]
    fn language_code_count() {
        assert_eq!(LanguageCode::all().len(), 25);
    }

    #[test]
    fn language_code_display_names() {
        assert_eq!(LanguageCode::Ko.display_name(), "Korean");
        assert_eq!(LanguageCode::En.display_name(), "English");
        assert_eq!(LanguageCode::Ja.display_name(), "Japanese");
        assert_eq!(LanguageCode::Ar.display_name(), "Arabic");
    }

    #[test]
    fn language_code_case_insensitive_parse() {
        assert_eq!(LanguageCode::from_str_code("KO"), Some(LanguageCode::Ko));
        assert_eq!(LanguageCode::from_str_code("En"), Some(LanguageCode::En));
        assert_eq!(
            LanguageCode::from_str_code("ZH-TW"),
            Some(LanguageCode::ZhTw)
        );
        assert_eq!(
            LanguageCode::from_str_code("zh_tw"),
            Some(LanguageCode::ZhTw)
        );
    }

    #[test]
    fn language_code_unknown_returns_none() {
        assert_eq!(LanguageCode::from_str_code("xx"), None);
        assert_eq!(LanguageCode::from_str_code(""), None);
    }

    #[test]
    fn detect_korean() {
        let text = "안녕하세요 세계";
        assert_eq!(detect_language(text, LanguageCode::En), LanguageCode::Ko);
    }

    #[test]
    fn detect_japanese() {
        let text = "こんにちは世界";
        assert_eq!(detect_language(text, LanguageCode::En), LanguageCode::Ja);
    }

    #[test]
    fn detect_chinese() {
        let text = "你好世界";
        assert_eq!(detect_language(text, LanguageCode::En), LanguageCode::Zh);
    }

    #[test]
    fn detect_arabic() {
        let text = "مرحبا بالعالم";
        assert_eq!(detect_language(text, LanguageCode::En), LanguageCode::Ar);
    }

    #[test]
    fn detect_thai() {
        let text = "สวัสดีชาวโลก";
        assert_eq!(detect_language(text, LanguageCode::En), LanguageCode::Th);
    }

    #[test]
    fn detect_hindi() {
        let text = "नमस्ते दुनिया";
        assert_eq!(detect_language(text, LanguageCode::En), LanguageCode::Hi);
    }

    #[test]
    fn detect_cyrillic() {
        let text = "Привет мир";
        assert_eq!(detect_language(text, LanguageCode::En), LanguageCode::Ru);
    }

    #[test]
    fn detect_empty_returns_default() {
        assert_eq!(detect_language("", LanguageCode::Ko), LanguageCode::Ko);
    }

    #[test]
    fn detect_ascii_returns_default() {
        // Pure ASCII text can't be distinguished between Latin-script languages
        assert_eq!(
            detect_language("hello world", LanguageCode::En),
            LanguageCode::En
        );
    }

    #[test]
    fn formality_default() {
        assert_eq!(Formality::default(), Formality::Neutral);
    }

    #[test]
    fn domain_default() {
        assert_eq!(Domain::default(), Domain::General);
    }

    #[test]
    fn interpreter_config_default() {
        let config = InterpreterConfig::default();
        assert_eq!(config.source_language, LanguageCode::Ko);
        assert_eq!(config.target_language, LanguageCode::En);
        assert!(!config.bidirectional);
        assert!(config.preserve_tone);
        assert_eq!(config.provider, VoiceProviderKind::GeminiLive);
    }

    #[test]
    fn interpreter_config_system_prompt_unidirectional() {
        let config = InterpreterConfig {
            source_language: LanguageCode::Ko,
            target_language: LanguageCode::En,
            bidirectional: false,
            formality: Formality::Formal,
            domain: Domain::Business,
            preserve_tone: true,
            ..Default::default()
        };

        let prompt = config.build_system_prompt();
        assert!(prompt.contains("Korean"));
        assert!(prompt.contains("English"));
        assert!(prompt.contains("formal"));
        assert!(prompt.contains("business"));
        assert!(prompt.contains("tone"));
        assert!(!prompt.contains("bidirectional mode"));
    }

    #[test]
    fn interpreter_config_system_prompt_bidirectional() {
        let config = InterpreterConfig {
            source_language: LanguageCode::Ja,
            target_language: LanguageCode::Ko,
            bidirectional: true,
            ..Default::default()
        };

        let prompt = config.build_system_prompt();
        assert!(prompt.contains("bidirectional mode"));
        // Should mention both directions explicitly
        assert!(prompt.contains("speaks Japanese, immediately interpret into Korean"));
        assert!(prompt.contains("speaks Korean, immediately interpret into Japanese"));
    }

    #[test]
    fn voice_provider_kind_model_ids() {
        assert_eq!(
            VoiceProviderKind::GeminiLive.model_id(),
            "gemini-2.5-flash-native-audio-preview-12-2025"
        );
        assert_eq!(
            VoiceProviderKind::OpenAiRealtime.model_id(),
            "gpt-4o-realtime-preview"
        );
    }

    #[test]
    fn interpreter_stats_record_utterance() {
        let mut stats = InterpreterStats::default();

        stats.record_utterance(100, 10, 12);
        assert_eq!(stats.utterance_count, 1);
        assert_eq!(stats.source_words, 10);
        assert_eq!(stats.target_words, 12);
        assert!((stats.avg_latency_ms - 100.0).abs() < 0.01);

        stats.record_utterance(200, 20, 25);
        assert_eq!(stats.utterance_count, 2);
        assert_eq!(stats.source_words, 30);
        assert_eq!(stats.target_words, 37);
        assert!((stats.avg_latency_ms - 150.0).abs() < 0.01);
    }

    #[test]
    fn interpreter_stats_estimated_credits() {
        let mut stats = InterpreterStats::default();
        assert_eq!(stats.estimated_credits(), 0);

        stats.estimated_tokens = 1000;
        assert_eq!(stats.estimated_credits(), 2);

        stats.estimated_tokens = 500;
        assert_eq!(stats.estimated_credits(), 1);

        stats.estimated_tokens = 501;
        assert_eq!(stats.estimated_credits(), 2);
    }

    #[test]
    fn interpreter_session_creation() {
        let config = InterpreterConfig::default();
        let session = InterpreterSession::new(
            "session-001".to_string(),
            "zeroclaw_user".to_string(),
            config,
        );

        assert_eq!(session.id, "session-001");
        assert_eq!(session.user_id, "zeroclaw_user");
        assert_eq!(session.status, InterpreterStatus::Idle);
        assert_eq!(session.stats.utterance_count, 0);
        assert!(session.created_at > 0);
    }

    #[tokio::test]
    async fn session_manager_create_and_get() {
        let manager = VoiceSessionManager::new(true, 3);

        let session = manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await
            .unwrap();

        let retrieved = manager.get_session(&session.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, "zeroclaw_user");
    }

    #[tokio::test]
    async fn session_manager_enforces_limit() {
        let manager = VoiceSessionManager::new(true, 2);

        manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await
            .unwrap();
        manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await
            .unwrap();

        let result = manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Maximum concurrent voice sessions"));
    }

    #[tokio::test]
    async fn session_manager_disabled() {
        let manager = VoiceSessionManager::new(false, 3);
        let result = manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disabled"));
    }

    #[tokio::test]
    async fn session_manager_close_returns_stats() {
        let manager = VoiceSessionManager::new(true, 3);

        let session = manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await
            .unwrap();

        manager
            .record_utterance(&session.id, 150, 10, 12)
            .await
            .unwrap();

        let stats = manager.close_session(&session.id).await.unwrap();
        assert_eq!(stats.utterance_count, 1);
        assert_eq!(stats.source_words, 10);
        // Duration may be 0 if test completes within the same millisecond
        let _ = stats.total_duration_ms;

        // Verify session is closed
        let session = manager.get_session(&session.id).await.unwrap();
        assert_eq!(session.status, InterpreterStatus::Closed);
    }

    #[tokio::test]
    async fn session_manager_list_user_sessions() {
        let manager = VoiceSessionManager::new(true, 5);

        manager
            .create_session("user_a", InterpreterConfig::default())
            .await
            .unwrap();
        manager
            .create_session("user_a", InterpreterConfig::default())
            .await
            .unwrap();
        manager
            .create_session("user_b", InterpreterConfig::default())
            .await
            .unwrap();

        let user_a_sessions = manager.list_user_sessions("user_a").await;
        assert_eq!(user_a_sessions.len(), 2);

        let user_b_sessions = manager.list_user_sessions("user_b").await;
        assert_eq!(user_b_sessions.len(), 1);
    }

    #[tokio::test]
    async fn session_manager_active_count() {
        let manager = VoiceSessionManager::new(true, 5);

        let s1 = manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await
            .unwrap();
        manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await
            .unwrap();

        assert_eq!(manager.active_session_count().await, 2);

        manager.close_session(&s1.id).await.unwrap();
        assert_eq!(manager.active_session_count().await, 1);
    }

    #[tokio::test]
    async fn session_manager_update_status() {
        let manager = VoiceSessionManager::new(true, 3);

        let session = manager
            .create_session("zeroclaw_user", InterpreterConfig::default())
            .await
            .unwrap();

        manager
            .update_status(&session.id, InterpreterStatus::Listening)
            .await
            .unwrap();

        let updated = manager.get_session(&session.id).await.unwrap();
        assert_eq!(updated.status, InterpreterStatus::Listening);
    }

    #[test]
    fn gemini_provider_defaults() {
        let provider = GeminiLiveProvider::new(Some("test-key".to_string()));
        assert_eq!(provider.vad_threshold(), 0.4);
        assert_eq!(provider.silence_timeout_ms(), 300);
        assert_eq!(provider.kind(), VoiceProviderKind::GeminiLive);
    }

    #[test]
    fn openai_provider_kind() {
        let provider = OpenAiRealtimeProvider::new(Some("test-key".to_string()));
        assert_eq!(provider.kind(), VoiceProviderKind::OpenAiRealtime);
    }

    #[tokio::test]
    async fn gemini_provider_connect_requires_key() {
        let provider = GeminiLiveProvider::new(None);
        let config = InterpreterConfig::default();
        let result = provider.connect("session-1", &config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key"));
    }

    #[tokio::test]
    async fn openai_provider_connect_requires_key() {
        let provider = OpenAiRealtimeProvider::new(None);
        let config = InterpreterConfig::default();
        let result = provider.connect("session-1", &config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key"));
    }

    #[tokio::test]
    async fn gemini_provider_send_text() {
        let provider = GeminiLiveProvider::new(Some("test-key".to_string()));
        let result = provider.send_text("session-1", "hello").await.unwrap();
        assert!(result.contains("hello"));
    }

    #[tokio::test]
    async fn openai_provider_send_text() {
        let provider = OpenAiRealtimeProvider::new(Some("test-key".to_string()));
        let result = provider.send_text("session-1", "hello").await.unwrap();
        assert!(result.contains("hello"));
    }

    #[test]
    fn create_voice_provider_gemini() {
        let provider = create_voice_provider(VoiceProviderKind::GeminiLive, None);
        assert_eq!(provider.kind(), VoiceProviderKind::GeminiLive);
    }

    #[test]
    fn create_voice_provider_openai() {
        let provider = create_voice_provider(VoiceProviderKind::OpenAiRealtime, None);
        assert_eq!(provider.kind(), VoiceProviderKind::OpenAiRealtime);
    }
}
