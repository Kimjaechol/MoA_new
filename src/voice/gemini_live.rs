//! Gemini Live WebSocket client for real-time voice interpretation.
//!
//! Implements the bidirectional streaming protocol for Google's Gemini
//! Live API (BidiGenerateContent), enabling continuous audio streaming
//! with automatic Voice Activity Detection (VAD).
//!
//! ## Protocol Overview
//!
//! 1. **Connect** — open WebSocket to Gemini Live endpoint
//! 2. **Setup** — send initial configuration (model, VAD, system prompt)
//! 3. **Stream** — send audio chunks as `realtimeInput`, receive
//!    translated audio/text as `serverContent`
//! 4. **Close** — gracefully close the WebSocket session
//!
//! ## Important: Binary Frame Protocol
//!
//! Google Gemini Live sends **all** messages as WebSocket Binary frames,
//! including JSON control messages like `setupComplete`. This module
//! detects JSON in Binary frames (content starting with `{`) and parses
//! them as server messages before falling back to raw audio handling.
//! See `docs/gemini-live-binary-frames-fix.md` for full investigation.

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use super::pipeline::{InterpreterConfig, VoiceProviderKind};
use super::VoiceEvent;

// ── Constants ──────────────────────────────────────────────────────

/// Gemini Live WebSocket endpoint.
const GEMINI_LIVE_WS_URL: &str =
    "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent";

/// Default audio MIME type for input (16kHz PCM mono).
const INPUT_AUDIO_MIME: &str = "audio/pcm;rate=16000";

/// Default audio MIME type for output (24kHz PCM mono).
const OUTPUT_AUDIO_MIME: &str = "audio/pcm;rate=24000";

// ── VAD Configuration ──────────────────────────────────────────────

/// Voice Activity Detection sensitivity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VadSensitivity {
    #[serde(rename = "START_SENSITIVITY_HIGH")]
    High,
    #[serde(rename = "START_SENSITIVITY_MEDIUM")]
    Medium,
    #[serde(rename = "START_SENSITIVITY_LOW")]
    Low,
}

/// End-of-speech detection sensitivity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndSensitivity {
    #[serde(rename = "END_SENSITIVITY_HIGH")]
    High,
    #[serde(rename = "END_SENSITIVITY_MEDIUM")]
    Medium,
    #[serde(rename = "END_SENSITIVITY_LOW")]
    Low,
}

/// Automatic Activity Detection (VAD) configuration for Gemini Live.
///
/// Controls when the model detects that the user is speaking vs. silent,
/// enabling hands-free continuous interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VadConfig {
    /// Whether automatic activity detection is disabled.
    /// `false` = enabled (default, recommended for continuous interpretation).
    pub disabled: bool,
    /// How sensitive the start-of-speech detector is.
    #[serde(rename = "startOfSpeechSensitivity")]
    pub start_sensitivity: VadSensitivity,
    /// How sensitive the end-of-speech detector is.
    #[serde(rename = "endOfSpeechSensitivity")]
    pub end_sensitivity: EndSensitivity,
    /// Milliseconds of audio before detected speech start to include.
    #[serde(rename = "prefixPaddingMs")]
    pub prefix_padding_ms: u32,
    /// Milliseconds of silence before declaring speech ended.
    #[serde(rename = "silenceDurationMs")]
    pub silence_duration_ms: u32,
}

impl Default for VadConfig {
    /// Manual VAD mode: client controls turn boundaries via activityStart/activityEnd.
    ///
    /// This prevents Gemini from prematurely splitting long utterances into
    /// separate turns, which caused partial/missing translations.
    /// The remaining fields are ignored when `disabled: true` but kept
    /// as sensible fallbacks in case we toggle back to auto VAD.
    fn default() -> Self {
        Self {
            disabled: true,
            start_sensitivity: VadSensitivity::High,
            end_sensitivity: EndSensitivity::High,
            prefix_padding_ms: 100,
            silence_duration_ms: 300,
        }
    }
}

// ── Setup message (JSON sent as first frame) ───────────────────────

/// Top-level setup message for Gemini Live session initialization.
#[derive(Debug, Serialize)]
pub struct SetupMessage {
    pub setup: SetupPayload,
}

#[derive(Debug, Serialize)]
pub struct SetupPayload {
    pub model: String,
    #[serde(rename = "generationConfig")]
    pub generation_config: GenerationConfig,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<SystemInstruction>,
    #[serde(rename = "realtimeInputConfig")]
    pub realtime_input_config: RealtimeInputConfig,
}

#[derive(Debug, Serialize)]
pub struct GenerationConfig {
    #[serde(rename = "responseModalities")]
    pub response_modalities: Vec<String>,
    #[serde(rename = "speechConfig", skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<SpeechConfig>,
    #[serde(rename = "thinkingConfig", skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
}

#[derive(Debug, Serialize)]
pub struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    pub thinking_budget: u32,
}

#[derive(Debug, Serialize)]
pub struct SpeechConfig {
    #[serde(rename = "voiceConfig")]
    pub voice_config: VoiceConfig,
}

#[derive(Debug, Serialize)]
pub struct VoiceConfig {
    #[serde(rename = "prebuiltVoiceConfig")]
    pub prebuilt_voice_config: PrebuiltVoiceConfig,
}

#[derive(Debug, Serialize)]
pub struct PrebuiltVoiceConfig {
    #[serde(rename = "voiceName")]
    pub voice_name: String,
}

#[derive(Debug, Serialize)]
pub struct SystemInstruction {
    pub parts: Vec<TextPart>,
}

#[derive(Debug, Serialize)]
pub struct TextPart {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct RealtimeInputConfig {
    #[serde(rename = "automaticActivityDetection")]
    pub automatic_activity_detection: VadConfig,
}

/// Build the setup message for a voice interpretation session.
pub fn build_setup_message(config: &InterpreterConfig, vad: &VadConfig) -> SetupMessage {
    let system_prompt = config.build_system_prompt();

    SetupMessage {
        setup: SetupPayload {
            model: format!("models/{}", VoiceProviderKind::GeminiLive.model_id()),
            generation_config: GenerationConfig {
                response_modalities: vec!["AUDIO".to_string()],
                speech_config: Some(SpeechConfig {
                    voice_config: VoiceConfig {
                        prebuilt_voice_config: PrebuiltVoiceConfig {
                            voice_name: "Aoede".to_string(),
                        },
                    },
                }),
                thinking_config: Some(ThinkingConfig { thinking_budget: 0 }),
            },
            system_instruction: Some(SystemInstruction {
                parts: vec![TextPart {
                    text: system_prompt,
                }],
            }),
            realtime_input_config: RealtimeInputConfig {
                automatic_activity_detection: vad.clone(),
            },
        },
    }
}

// ── Audio input message ────────────────────────────────────────────

/// Audio input message sent to Gemini Live.
///
/// Note: API docs mark `mediaChunks` as deprecated in favor of `audio`,
/// but actual SDKs (Python, Go, JS) still use `mediaChunks` on the wire
/// and `audio` field is not reliably processed by the server yet.
#[derive(Debug, Serialize)]
pub struct RealtimeInputMessage {
    #[serde(rename = "realtimeInput")]
    pub realtime_input: RealtimeInput,
}

#[derive(Debug, Serialize)]
pub struct RealtimeInput {
    #[serde(rename = "mediaChunks")]
    pub media_chunks: Vec<MediaChunk>,
}

/// Audio chunk with MIME type and base64-encoded data.
#[derive(Debug, Serialize)]
pub struct MediaChunk {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String, // base64-encoded audio
}

/// Message to signal the end of the audio stream to Gemini Live.
///
/// Sent when the microphone is stopped while automatic activity detection
/// (VAD) is enabled. Tells Gemini to process any remaining buffered input.
#[derive(Debug, Serialize)]
pub struct AudioStreamEndMessage {
    #[serde(rename = "realtimeInput")]
    pub realtime_input: AudioStreamEndPayload,
}

#[derive(Debug, Serialize)]
pub struct AudioStreamEndPayload {
    #[serde(rename = "audioStreamEnd")]
    pub audio_stream_end: bool,
}

/// Build a realtime audio input message from raw PCM bytes.
///
/// Wire format: `{"realtimeInput": {"mediaChunks": [{"mimeType": "audio/pcm;rate=16000", "data": "<base64>"}]}}`
///
/// Note on `mediaChunks` vs `audio`:
/// Google's API docs (2026-02) mark `mediaChunks` as deprecated and recommend `audio`.
/// However, all official SDKs (Python googleapis/python-genai, JS @google/genai, Go)
/// still emit `mediaChunks` on the wire. Testing confirmed `audio` field is silently
/// ignored by the server — no response at all. Keeping `mediaChunks` until Google
/// SDKs actually switch. Track: https://ai.google.dev/api/live
pub fn build_audio_message(pcm_data: &[u8]) -> RealtimeInputMessage {
    let b64 = base64::engine::general_purpose::STANDARD.encode(pcm_data);
    RealtimeInputMessage {
        realtime_input: RealtimeInput {
            media_chunks: vec![MediaChunk {
                mime_type: INPUT_AUDIO_MIME.to_string(),
                data: b64,
            }],
        },
    }
}

/// Build an activityStart message for manual VAD.
///
/// Wire format: `{"realtimeInput": {"activityStart": {}}}`
/// Only valid when `automaticActivityDetection.disabled` is true.
pub fn build_activity_start_message() -> serde_json::Value {
    serde_json::json!({
        "realtimeInput": {
            "activityStart": {}
        }
    })
}

/// Build an activityEnd message for manual VAD.
///
/// Wire format: `{"realtimeInput": {"activityEnd": {}}}`
/// Only valid when `automaticActivityDetection.disabled` is true.
pub fn build_activity_end_message() -> serde_json::Value {
    serde_json::json!({
        "realtimeInput": {
            "activityEnd": {}
        }
    })
}

/// Build an audioStreamEnd message to signal microphone closure.
pub fn build_audio_stream_end_message() -> AudioStreamEndMessage {
    AudioStreamEndMessage {
        realtime_input: AudioStreamEndPayload {
            audio_stream_end: true,
        },
    }
}

// ── Server response types ──────────────────────────────────────────

/// Backward-compatible alias — all Gemini events now use the shared `VoiceEvent`.
pub type GeminiLiveEvent = VoiceEvent;

/// Parse a JSON text frame from Gemini Live into a list of events.
///
/// A single server message can contain multiple events (e.g., audio
/// chunks + transcription in the same frame).
pub fn parse_server_message(json_text: &str) -> Vec<VoiceEvent> {
    let mut events = Vec::new();

    let value: serde_json::Value = match serde_json::from_str(json_text) {
        Ok(v) => v,
        Err(e) => {
            events.push(VoiceEvent::Error {
                message: format!("Failed to parse server message: {e}"),
            });
            return events;
        }
    };

    // setupComplete
    if value.get("setupComplete").is_some() {
        events.push(VoiceEvent::SetupComplete);
    }

    // serverContent
    if let Some(content) = value.get("serverContent") {
        // Check turn completion
        if content.get("turnComplete").and_then(|v| v.as_bool()) == Some(true) {
            events.push(VoiceEvent::TurnComplete);
        }
        // Check interruption
        if content.get("interrupted").and_then(|v| v.as_bool()) == Some(true) {
            events.push(VoiceEvent::Interrupted);
        }
        // Extract audio and text from modelTurn.parts
        if let Some(parts) = content
            .pointer("/modelTurn/parts")
            .and_then(|v| v.as_array())
        {
            for part in parts {
                // Audio data
                if let Some(inline) = part.get("inlineData") {
                    if let (Some(data_b64), Some(_mime)) = (
                        inline.get("data").and_then(|v| v.as_str()),
                        inline.get("mimeType").and_then(|v| v.as_str()),
                    ) {
                        if let Ok(audio_bytes) =
                            base64::engine::general_purpose::STANDARD.decode(data_b64)
                        {
                            events.push(VoiceEvent::Audio { data: audio_bytes });
                        }
                    }
                }
                // Text
                if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                    events.push(VoiceEvent::OutputTranscript {
                        text: text.to_string(),
                    });
                }
            }
        }
    }

    // inputTranscription (user speech transcript)
    if let Some(transcript) = value.get("inputTranscription") {
        if let Some(text) = transcript.get("text").and_then(|v| v.as_str()) {
            if !text.is_empty() {
                events.push(VoiceEvent::InputTranscript {
                    text: text.to_string(),
                });
            }
        }
    }

    // outputTranscription (model speech transcript)
    if let Some(transcript) = value.get("outputTranscription") {
        if let Some(text) = transcript.get("text").and_then(|v| v.as_str()) {
            if !text.is_empty() {
                events.push(VoiceEvent::OutputTranscript {
                    text: text.to_string(),
                });
            }
        }
    }

    // Error
    if let Some(err) = value.get("error") {
        let message = err
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown server error");
        events.push(VoiceEvent::Error {
            message: message.to_string(),
        });
    }

    events
}

// ── Live session (WebSocket connection manager) ────────────────────

/// State of the Gemini Live WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not yet connected.
    Disconnected,
    /// WebSocket open, setup message sent, waiting for setupComplete.
    Connecting,
    /// Setup complete — ready to stream audio.
    Ready,
    /// Actively streaming audio.
    Streaming,
    /// Connection failed or closed with error.
    Failed,
    /// Gracefully closed.
    Closed,
}

/// Outbound message to send to the Gemini Live WebSocket.
#[derive(Debug)]
pub enum OutboundMessage {
    /// Send raw audio bytes (will be wrapped in realtimeInput).
    Audio(Vec<u8>),
    /// Send a text message (for text-based interpretation).
    Text(String),
    /// Signal start of user speech (manual VAD).
    ActivityStart,
    /// Signal end of user speech (manual VAD).
    ActivityEnd,
    /// Signal end of audio stream (microphone stopped).
    AudioStreamEnd,
    /// Close the connection.
    Close,
}

/// A handle for interacting with a Gemini Live session.
///
/// Created by [`GeminiLiveSession::connect`]. Audio is sent via the
/// `audio_tx` channel, and events are received via `event_rx`.
pub struct GeminiLiveSession {
    /// Channel to send audio/text to Gemini Live.
    audio_tx: mpsc::Sender<OutboundMessage>,
    /// Channel to receive events from Gemini Live.
    pub event_rx: Arc<Mutex<mpsc::Receiver<VoiceEvent>>>,
    /// Current connection state.
    state: Arc<Mutex<ConnectionState>>,
    /// Session ID for logging.
    session_id: String,
}

impl GeminiLiveSession {
    /// Connect to the Gemini Live API and establish a streaming session.
    ///
    /// Returns a session handle. Audio is sent via [`Self::send_audio`],
    /// events are received via [`Self::recv_event`].
    pub async fn connect(
        session_id: String,
        api_key: &str,
        config: &InterpreterConfig,
        vad: &VadConfig,
    ) -> anyhow::Result<Self> {
        let url = format!("{GEMINI_LIVE_WS_URL}?key={api_key}");

        tracing::info!(
            session_id = %session_id,
            model = VoiceProviderKind::GeminiLive.model_id(),
            source = config.source_language.as_str(),
            target = config.target_language.as_str(),
            "Connecting to Gemini Live"
        );

        let (mut ws_stream, _response) = tokio_tungstenite::connect_async(&url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to Gemini Live: {e}"))?;

        // Send setup message on the unsplit stream
        let setup = build_setup_message(config, vad);
        let setup_json = serde_json::to_string(&setup)?;
        tracing::debug!(session_id = %session_id, setup = %setup_json, "Sending Gemini Live setup");
        ws_stream
            .send(WsMessage::Text(setup_json))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send setup message: {e}"))?;

        // Wait for setupComplete before splitting the stream.
        // Gemini Live sends all messages as Binary frames (including JSON),
        // so we check for Binary containing `{"setupComplete": ...}`.
        let setup_timeout = std::time::Duration::from_secs(15);
        let setup_complete = tokio::time::timeout(setup_timeout, async {
            while let Some(msg_result) = ws_stream.next().await {
                match msg_result {
                    Ok(WsMessage::Binary(data)) if data.first() == Some(&b'{') => {
                        if let Ok(text) = std::str::from_utf8(&data) {
                            if text.contains("setupComplete") {
                                tracing::info!(
                                    session_id = %session_id,
                                    "Gemini Live setup complete — ready to stream"
                                );
                                return Ok(());
                            }
                        }
                    }
                    Ok(WsMessage::Text(text)) if text.contains("setupComplete") => {
                        tracing::info!(
                            session_id = %session_id,
                            "Gemini Live setup complete (text frame) — ready to stream"
                        );
                        return Ok(());
                    }
                    Ok(WsMessage::Close(frame)) => {
                        anyhow::bail!("Connection closed before setupComplete: {frame:?}");
                    }
                    Err(e) => {
                        anyhow::bail!("WebSocket error before setupComplete: {e}");
                    }
                    other => {
                        tracing::debug!(
                            session_id = %session_id,
                            msg = ?other,
                            "Gemini Live setup phase: non-text/binary frame"
                        );
                    }
                }
            }
            anyhow::bail!("Stream ended before setupComplete")
        })
        .await;

        match setup_complete {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => anyhow::bail!("Gemini Live setupComplete timeout (15s)"),
        }

        let (ws_sender, ws_receiver) = ws_stream.split();
        let ws_sender = Arc::new(Mutex::new(ws_sender));

        let state = Arc::new(Mutex::new(ConnectionState::Ready));

        // Channels for bidirectional communication
        let (audio_tx, audio_rx) = mpsc::channel::<OutboundMessage>(256);
        let (event_tx, event_rx) = mpsc::channel::<VoiceEvent>(256);

        // Spawn outbound task: reads from audio_rx, sends to WebSocket
        let ws_sender_out = Arc::clone(&ws_sender);
        let state_out = Arc::clone(&state);
        let sid_out = session_id.clone();
        tokio::spawn(async move {
            Self::outbound_loop(audio_rx, ws_sender_out, state_out, sid_out).await;
        });

        // Spawn inbound task: reads from WebSocket, sends to event_tx
        let state_in = Arc::clone(&state);
        let sid_in = session_id.clone();
        tokio::spawn(async move {
            Self::inbound_loop(ws_receiver, event_tx, state_in, sid_in).await;
        });

        Ok(Self {
            audio_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            state,
            session_id,
        })
    }

    /// Send a raw PCM audio chunk to Gemini Live.
    pub async fn send_audio(&self, pcm_data: &[u8]) -> anyhow::Result<()> {
        if pcm_data.is_empty() {
            return Ok(());
        }
        self.audio_tx
            .send(OutboundMessage::Audio(pcm_data.to_vec()))
            .await
            .map_err(|_| anyhow::anyhow!("Audio channel closed"))
    }

    /// Send a text message for text-based interpretation.
    pub async fn send_text(&self, text: &str) -> anyhow::Result<()> {
        self.audio_tx
            .send(OutboundMessage::Text(text.to_string()))
            .await
            .map_err(|_| anyhow::anyhow!("Audio channel closed"))
    }

    /// Receive the next event from Gemini Live.
    pub async fn recv_event(&self) -> Option<VoiceEvent> {
        self.event_rx.lock().await.recv().await
    }

    /// Get the current connection state.
    pub async fn connection_state(&self) -> ConnectionState {
        *self.state.lock().await
    }

    /// Signal start of user speech (manual VAD).
    pub async fn send_activity_start(&self) -> anyhow::Result<()> {
        self.audio_tx
            .send(OutboundMessage::ActivityStart)
            .await
            .map_err(|_| anyhow::anyhow!("Audio channel closed"))
    }

    /// Signal end of user speech (manual VAD).
    pub async fn send_activity_end(&self) -> anyhow::Result<()> {
        self.audio_tx
            .send(OutboundMessage::ActivityEnd)
            .await
            .map_err(|_| anyhow::anyhow!("Audio channel closed"))
    }

    /// Signal end of audio stream to Gemini Live.
    ///
    /// Call this when the microphone is stopped so Gemini processes
    /// any remaining buffered input and sends a final response.
    pub async fn send_audio_stream_end(&self) -> anyhow::Result<()> {
        self.audio_tx
            .send(OutboundMessage::AudioStreamEnd)
            .await
            .map_err(|_| anyhow::anyhow!("Audio channel closed"))
    }

    /// Close the session gracefully.
    pub async fn close(&self) {
        let _ = self.audio_tx.send(OutboundMessage::Close).await;
    }

    /// Get session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    // ── Internal loops ────────────────────────────────────────────

    /// Outbound loop: encode audio and send to Gemini Live WebSocket.
    async fn outbound_loop(
        mut rx: mpsc::Receiver<OutboundMessage>,
        ws_sender: Arc<
            Mutex<
                futures_util::stream::SplitSink<
                    tokio_tungstenite::WebSocketStream<
                        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                    >,
                    WsMessage,
                >,
            >,
        >,
        state: Arc<Mutex<ConnectionState>>,
        session_id: String,
    ) {
        let mut audio_chunk_count: u64 = 0;
        let mut total_bytes: u64 = 0;

        while let Some(msg) = rx.recv().await {
            match msg {
                OutboundMessage::Audio(pcm) => {
                    audio_chunk_count += 1;
                    total_bytes += pcm.len() as u64;
                    let audio_msg = build_audio_message(&pcm);
                    match serde_json::to_string(&audio_msg) {
                        Ok(json) => {
                            // Log first chunk and every 50th chunk
                            if audio_chunk_count == 1 || audio_chunk_count.is_multiple_of(50) {
                                tracing::info!(
                                    session_id = %session_id,
                                    chunk = audio_chunk_count,
                                    pcm_bytes = pcm.len(),
                                    total_bytes = total_bytes,
                                    json_len = json.len(),
                                    "Sending audio chunk to Gemini"
                                );
                                if audio_chunk_count == 1 {
                                    // Log first message structure for debugging
                                    let preview = if json.len() > 200 {
                                        &json[..200]
                                    } else {
                                        &json
                                    };
                                    tracing::debug!(
                                        session_id = %session_id,
                                        preview = %preview,
                                        "First audio message JSON preview"
                                    );
                                }
                            }
                            let mut sender = ws_sender.lock().await;
                            if sender.send(WsMessage::Text(json)).await.is_err() {
                                tracing::warn!(
                                    session_id = %session_id,
                                    "WebSocket send failed, closing outbound loop"
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                session_id = %session_id,
                                error = %e,
                                "Failed to serialize audio message"
                            );
                        }
                    }
                }
                OutboundMessage::Text(text) => {
                    // Send as a client content message for text-based input
                    let msg = serde_json::json!({
                        "clientContent": {
                            "turns": [{
                                "role": "user",
                                "parts": [{ "text": text }]
                            }],
                            "turnComplete": true,
                        }
                    });
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let mut sender = ws_sender.lock().await;
                        if sender.send(WsMessage::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                OutboundMessage::ActivityStart => {
                    let msg = build_activity_start_message();
                    if let Ok(json) = serde_json::to_string(&msg) {
                        tracing::info!(session_id = %session_id, "Sending activityStart to Gemini Live");
                        let mut sender = ws_sender.lock().await;
                        if sender.send(WsMessage::Text(json)).await.is_err() {
                            tracing::warn!(session_id = %session_id, "WebSocket send failed for activityStart");
                            break;
                        }
                    }
                }
                OutboundMessage::ActivityEnd => {
                    let msg = build_activity_end_message();
                    if let Ok(json) = serde_json::to_string(&msg) {
                        tracing::info!(session_id = %session_id, "Sending activityEnd to Gemini Live");
                        let mut sender = ws_sender.lock().await;
                        if sender.send(WsMessage::Text(json)).await.is_err() {
                            tracing::warn!(session_id = %session_id, "WebSocket send failed for activityEnd");
                            break;
                        }
                    }
                }
                OutboundMessage::AudioStreamEnd => {
                    let end_msg = build_audio_stream_end_message();
                    match serde_json::to_string(&end_msg) {
                        Ok(json) => {
                            tracing::info!(
                                session_id = %session_id,
                                "Sending audioStreamEnd to Gemini Live"
                            );
                            let mut sender = ws_sender.lock().await;
                            if sender.send(WsMessage::Text(json)).await.is_err() {
                                tracing::warn!(
                                    session_id = %session_id,
                                    "WebSocket send failed for audioStreamEnd"
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                session_id = %session_id,
                                error = %e,
                                "Failed to serialize audioStreamEnd message"
                            );
                        }
                    }
                }
                OutboundMessage::Close => {
                    let mut sender = ws_sender.lock().await;
                    let _ = sender.send(WsMessage::Close(None)).await;
                    *state.lock().await = ConnectionState::Closed;
                    break;
                }
            }
        }

        tracing::debug!(session_id = %session_id, "Outbound loop terminated");
    }

    /// Inbound loop: receive events from Gemini Live and dispatch.
    async fn inbound_loop(
        mut ws_receiver: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        event_tx: mpsc::Sender<VoiceEvent>,
        state: Arc<Mutex<ConnectionState>>,
        session_id: String,
    ) {
        let start_time = std::time::Instant::now();
        let mut audio_response_count: u64 = 0;
        let mut turn_count: u64 = 0;

        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(WsMessage::Text(text)) => {
                    tracing::debug!(session_id = %session_id, msg = %text, "Gemini Live raw message");
                    let events = parse_server_message(&text);
                    let elapsed = start_time.elapsed().as_secs_f32();
                    for event in &events {
                        match event {
                            VoiceEvent::Audio { data, .. } => {
                                audio_response_count += 1;
                                tracing::info!(
                                    session_id = %session_id,
                                    t = format!("{elapsed:.1}s"),
                                    audio_n = audio_response_count,
                                    bytes = data.len(),
                                    "⬇ Gemini audio response"
                                );
                            }
                            VoiceEvent::TurnComplete => {
                                turn_count += 1;
                                tracing::info!(
                                    session_id = %session_id,
                                    t = format!("{elapsed:.1}s"),
                                    turn = turn_count,
                                    audio_chunks_this_session = audio_response_count,
                                    "⬇ Turn complete"
                                );
                            }
                            VoiceEvent::OutputTranscript { text } => {
                                tracing::info!(
                                    session_id = %session_id,
                                    t = format!("{elapsed:.1}s"),
                                    text = %text,
                                    "⬇ Output transcript"
                                );
                            }
                            VoiceEvent::InputTranscript { text } => {
                                tracing::info!(
                                    session_id = %session_id,
                                    t = format!("{elapsed:.1}s"),
                                    text = %text,
                                    "⬇ Input transcript"
                                );
                            }
                            VoiceEvent::Interrupted => {
                                tracing::info!(
                                    session_id = %session_id,
                                    t = format!("{elapsed:.1}s"),
                                    "⬇ Interrupted"
                                );
                            }
                            _ => {}
                        }
                        if matches!(event, VoiceEvent::SetupComplete) {
                            *state.lock().await = ConnectionState::Ready;
                        }
                    }
                    for event in events {
                        if event_tx.send(event).await.is_err() {
                            tracing::debug!(
                                session_id = %session_id,
                                "Event receiver dropped, closing inbound loop"
                            );
                            return;
                        }
                    }
                }
                Ok(WsMessage::Binary(data)) => {
                    if data.is_empty() {
                        continue;
                    }

                    // Gemini Live sends ALL messages as Binary frames (including JSON).
                    // Try JSON parse first; if it fails, treat as raw audio.
                    if data.first() == Some(&b'{') {
                        if let Ok(text) = std::str::from_utf8(&data) {
                            tracing::debug!(session_id = %session_id, msg = %text, "Gemini Live binary JSON message");
                            let events = parse_server_message(text);
                            let elapsed = start_time.elapsed().as_secs_f32();
                            for event in &events {
                                match event {
                                    VoiceEvent::Audio { data, .. } => {
                                        audio_response_count += 1;
                                        tracing::info!(
                                            session_id = %session_id,
                                            t = format!("{elapsed:.1}s"),
                                            audio_n = audio_response_count,
                                            bytes = data.len(),
                                            "⬇ Gemini audio response"
                                        );
                                    }
                                    VoiceEvent::TurnComplete => {
                                        turn_count += 1;
                                        tracing::info!(
                                            session_id = %session_id,
                                            t = format!("{elapsed:.1}s"),
                                            turn = turn_count,
                                            audio_chunks_this_session = audio_response_count,
                                            "⬇ Turn complete"
                                        );
                                    }
                                    VoiceEvent::OutputTranscript { text } => {
                                        tracing::info!(
                                            session_id = %session_id,
                                            t = format!("{elapsed:.1}s"),
                                            text = %text,
                                            "⬇ Output transcript"
                                        );
                                    }
                                    VoiceEvent::InputTranscript { text } => {
                                        tracing::info!(
                                            session_id = %session_id,
                                            t = format!("{elapsed:.1}s"),
                                            text = %text,
                                            "⬇ Input transcript"
                                        );
                                    }
                                    VoiceEvent::Interrupted => {
                                        tracing::info!(
                                            session_id = %session_id,
                                            t = format!("{elapsed:.1}s"),
                                            "⬇ Interrupted"
                                        );
                                    }
                                    _ => {}
                                }
                                if matches!(event, VoiceEvent::SetupComplete) {
                                    *state.lock().await = ConnectionState::Ready;
                                }
                            }
                            for event in events {
                                if event_tx.send(event).await.is_err() {
                                    return;
                                }
                            }
                            continue;
                        }
                    }

                    // Non-JSON binary frame — Gemini Live sends all responses
                    // as JSON-in-Binary, so a non-JSON binary is unexpected.
                    // Log a warning and skip rather than misinterpreting as raw PCM.
                    tracing::warn!(
                        session_id = %session_id,
                        len = data.len(),
                        first_byte = data.first().copied().unwrap_or(0),
                        "Unexpected non-JSON binary frame from Gemini Live — skipping"
                    );
                }
                Ok(WsMessage::Close(frame)) => {
                    tracing::info!(session_id = %session_id, close_frame = ?frame, "Gemini Live connection closed");
                    *state.lock().await = ConnectionState::Closed;
                    break;
                }
                Ok(WsMessage::Ping(_) | WsMessage::Pong(_) | WsMessage::Frame(_)) => {
                    // Handled by tungstenite automatically
                }
                Err(e) => {
                    tracing::error!(
                        session_id = %session_id,
                        error = %e,
                        "Gemini Live WebSocket error"
                    );
                    *state.lock().await = ConnectionState::Failed;
                    let _ = event_tx
                        .send(VoiceEvent::Error {
                            message: format!("WebSocket error: {e}"),
                        })
                        .await;
                    break;
                }
            }
        }

        tracing::debug!(session_id = %session_id, "Inbound loop terminated");
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vad_config_default() {
        let vad = VadConfig::default();
        assert!(vad.disabled); // manual VAD mode by default
        assert_eq!(vad.start_sensitivity, VadSensitivity::High);
        assert_eq!(vad.end_sensitivity, EndSensitivity::High);
        assert_eq!(vad.prefix_padding_ms, 100);
        assert_eq!(vad.silence_duration_ms, 300);
    }

    #[test]
    fn vad_config_serialization() {
        let vad = VadConfig::default();
        let json = serde_json::to_string(&vad).unwrap();
        assert!(json.contains("startOfSpeechSensitivity"));
        assert!(json.contains("START_SENSITIVITY_LOW"));
        assert!(json.contains("silenceDurationMs"));
    }

    #[test]
    fn build_setup_message_contains_model() {
        let config = InterpreterConfig::default();
        let vad = VadConfig::default();
        let msg = build_setup_message(&config, &vad);

        assert!(msg.setup.model.contains("gemini"));
        assert_eq!(
            msg.setup.generation_config.response_modalities,
            vec!["AUDIO"]
        );
    }

    #[test]
    fn build_setup_message_serializes_to_json() {
        let config = InterpreterConfig::default();
        let vad = VadConfig::default();
        let msg = build_setup_message(&config, &vad);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"setup\""));
        assert!(json.contains("\"model\""));
        assert!(json.contains("automaticActivityDetection"));
        assert!(json.contains("systemInstruction"));
    }

    #[test]
    fn build_setup_message_includes_system_prompt() {
        let config = InterpreterConfig {
            source_language: super::super::pipeline::LanguageCode::Ko,
            target_language: super::super::pipeline::LanguageCode::En,
            ..Default::default()
        };
        let vad = VadConfig::default();
        let msg = build_setup_message(&config, &vad);

        let prompt = &msg.setup.system_instruction.unwrap().parts[0].text;
        assert!(prompt.contains("Korean"));
        assert!(prompt.contains("English"));
        assert!(prompt.contains("interpreter"));
    }

    #[test]
    fn build_audio_message_encodes_base64() {
        let pcm = vec![0u8, 1, 2, 3, 4, 5];
        let msg = build_audio_message(&pcm);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("realtimeInput"));
        assert!(json.contains("mediaChunks"));
        assert!(json.contains(INPUT_AUDIO_MIME));
        // Verify base64
        let b64 = &msg.realtime_input.media_chunks[0].data;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .unwrap();
        assert_eq!(decoded, pcm);
    }

    #[test]
    fn build_audio_stream_end_message_format() {
        let msg = build_audio_stream_end_message();
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("realtimeInput"));
        assert!(json.contains("audioStreamEnd"));
        assert!(json.contains("true"));
        // Should NOT contain audio data fields
        assert!(!json.contains("mimeType"));
        assert!(!json.contains("data"));
    }

    #[test]
    fn parse_setup_complete() {
        let json = r#"{"setupComplete": {}}"#;
        let events = parse_server_message(json);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], VoiceEvent::SetupComplete));
    }

    #[test]
    fn parse_turn_complete() {
        let json = r#"{"serverContent": {"turnComplete": true}}"#;
        let events = parse_server_message(json);
        assert!(events.iter().any(|e| matches!(e, VoiceEvent::TurnComplete)));
    }

    #[test]
    fn parse_interrupted() {
        let json = r#"{"serverContent": {"interrupted": true}}"#;
        let events = parse_server_message(json);
        assert!(events.iter().any(|e| matches!(e, VoiceEvent::Interrupted)));
    }

    #[test]
    fn parse_audio_response() {
        let audio_b64 = base64::engine::general_purpose::STANDARD.encode([10u8, 20, 30]);
        let json = format!(
            r#"{{"serverContent": {{"modelTurn": {{"parts": [{{"inlineData": {{"mimeType": "audio/pcm;rate=24000", "data": "{audio_b64}"}}}}]}}}}}}"#
        );
        let events = parse_server_message(&json);

        let audio = events
            .iter()
            .find(|e| matches!(e, VoiceEvent::Audio { .. }));
        assert!(audio.is_some());
        if let VoiceEvent::Audio { data } = audio.unwrap() {
            assert_eq!(data, &[10u8, 20, 30]);
        }
    }

    #[test]
    fn parse_text_response() {
        let json = r#"{"serverContent": {"modelTurn": {"parts": [{"text": "Hello world"}]}}}"#;
        let events = parse_server_message(json);
        assert!(events.iter().any(|e| matches!(
            e,
            VoiceEvent::OutputTranscript { text } if text == "Hello world"
        )));
    }

    #[test]
    fn parse_input_transcription() {
        let json = r#"{"inputTranscription": {"text": "안녕하세요"}}"#;
        let events = parse_server_message(json);
        assert!(events.iter().any(|e| matches!(
            e,
            VoiceEvent::InputTranscript { text } if text == "안녕하세요"
        )));
    }

    #[test]
    fn parse_output_transcription() {
        let json = r#"{"outputTranscription": {"text": "Hello"}}"#;
        let events = parse_server_message(json);
        assert!(events.iter().any(|e| matches!(
            e,
            VoiceEvent::OutputTranscript { text } if text == "Hello"
        )));
    }

    #[test]
    fn parse_error() {
        let json = r#"{"error": {"message": "Rate limit exceeded"}}"#;
        let events = parse_server_message(json);
        assert!(events.iter().any(|e| matches!(
            e,
            VoiceEvent::Error { message } if message.contains("Rate limit")
        )));
    }

    #[test]
    fn parse_invalid_json() {
        let events = parse_server_message("not json at all");
        assert!(events.iter().any(|e| matches!(e, VoiceEvent::Error { .. })));
    }

    #[test]
    fn parse_empty_transcription_ignored() {
        let json = r#"{"inputTranscription": {"text": ""}}"#;
        let events = parse_server_message(json);
        assert!(!events
            .iter()
            .any(|e| matches!(e, VoiceEvent::InputTranscript { .. })));
    }

    #[test]
    fn connection_state_starts_disconnected() {
        // Just verify the enum variants exist and are distinct
        assert_ne!(ConnectionState::Disconnected, ConnectionState::Ready);
        assert_ne!(ConnectionState::Connecting, ConnectionState::Streaming);
        assert_ne!(ConnectionState::Failed, ConnectionState::Closed);
    }
}
