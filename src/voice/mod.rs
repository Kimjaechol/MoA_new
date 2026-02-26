//! Voice processing pipeline for ZeroClaw.
//!
//! Provides real-time voice interpretation and voice-to-voice conversation
//! capabilities using multiple voice providers (Gemini Live, OpenAI Realtime).
//!
//! ## Design
//! - Trait-driven voice provider abstraction (`VoiceProvider`)
//! - 25-language support with Unicode-based language detection
//! - Bidirectional interpretation mode (A <-> B language auto-switch)
//! - Formality levels (formal / neutral / casual)
//! - Domain specialization (general / business / medical / legal / technical)
//! - Per-session billing integration (token-equivalent credit deduction)
//! - Gemini Live WebSocket client with automatic VAD for hands-free interpretation

pub mod gemini_live;
pub mod openai_realtime;
pub mod pipeline;

// ── Shared voice event type ──────────────────────────────────────

/// Provider-agnostic event produced by any voice session (Gemini Live, OpenAI Realtime, etc.).
///
/// Both `GeminiLiveSession` and `OpenAiRealtimeSession` emit these events
/// through their `event_rx` channels, enabling the gateway to relay them
/// to the browser with identical logic regardless of provider.
#[derive(Debug, Clone)]
pub enum VoiceEvent {
    /// Provider setup completed — ready to stream.
    SetupComplete,
    /// Translated/interpreted audio chunk (PCM16, 24kHz mono).
    Audio { data: Vec<u8> },
    /// Transcription of user's speech (input).
    InputTranscript { text: String },
    /// Transcription of model's speech (output / translated).
    OutputTranscript { text: String },
    /// Model finished a response turn.
    TurnComplete,
    /// The model was interrupted (user started speaking mid-response).
    Interrupted,
    /// Error from the provider.
    Error { message: String },
}

#[allow(unused_imports)]
pub use gemini_live::{ConnectionState, GeminiLiveSession, VadConfig, VadSensitivity};
#[allow(unused_imports)]
pub use openai_realtime::OpenAiRealtimeSession;
#[allow(unused_imports)]
pub use pipeline::{
    Domain, Formality, InterpreterConfig, InterpreterSession, InterpreterStats, InterpreterStatus,
    LanguageCode, VoiceProvider, VoiceProviderKind, VoiceSessionManager,
};
