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
pub mod pipeline;

#[allow(unused_imports)]
pub use gemini_live::{
    ConnectionState, GeminiLiveEvent, GeminiLiveSession, VadConfig, VadSensitivity,
};
#[allow(unused_imports)]
pub use pipeline::{
    Domain, Formality, InterpreterConfig, InterpreterSession, InterpreterStats, InterpreterStatus,
    LanguageCode, VoiceProvider, VoiceProviderKind, VoiceSessionManager,
};
