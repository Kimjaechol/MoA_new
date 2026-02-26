//! OpenAI Realtime WebSocket client for real-time voice interpretation.
//!
//! Implements the OpenAI Realtime API protocol with semantic VAD support.
//! The key advantage over Gemini Live is semantic turn detection — the model
//! decides when the user has finished speaking based on meaning, not just silence.
//!
//! ## Protocol Overview
//!
//! 1. **Connect** — WebSocket to `wss://api.openai.com/v1/realtime?model=...`
//! 2. **Setup** — send `session.update` with instructions, audio format, VAD config
//! 3. **Stream** — send `input_audio_buffer.append` (Base64 PCM16 24kHz),
//!    receive `response.audio.delta` (Base64 PCM16 24kHz)
//! 4. **Close** — gracefully close the WebSocket session

use base64::Engine;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as WsMessage;

use super::pipeline::{InterpreterConfig, VoiceProviderKind};
use super::VoiceEvent;

// ── Constants ──────────────────────────────────────────────────────

/// OpenAI Realtime WebSocket endpoint.
const OPENAI_REALTIME_WS_URL: &str = "wss://api.openai.com/v1/realtime";

/// Input/output audio: PCM16, 24kHz, mono.
/// OpenAI Realtime uses 24kHz for both input and output (unlike Gemini's 16kHz input).
pub const INPUT_SAMPLE_RATE: u32 = 24000;

// ── Outbound message ───────────────────────────────────────────────

/// Outbound message to send to the OpenAI Realtime WebSocket.
#[derive(Debug)]
enum OutboundMessage {
    /// Send raw audio bytes (PCM16 24kHz → Base64 → input_audio_buffer.append).
    Audio(Vec<u8>),
    /// Send a text message (conversation item).
    Text(String),
    /// Close the connection.
    Close,
}

// ── Session ────────────────────────────────────────────────────────

/// A handle for interacting with an OpenAI Realtime session.
///
/// Created by [`OpenAiRealtimeSession::connect`]. Audio is sent via
/// `send_audio`, events are received via `event_rx`.
pub struct OpenAiRealtimeSession {
    /// Channel to send audio/text to OpenAI Realtime.
    outbound_tx: mpsc::Sender<OutboundMessage>,
    /// Channel to receive events from OpenAI Realtime.
    pub event_rx: Arc<Mutex<mpsc::Receiver<VoiceEvent>>>,
    /// Session ID for logging.
    session_id: String,
}

impl OpenAiRealtimeSession {
    /// Connect to the OpenAI Realtime API and establish a streaming session.
    pub async fn connect(
        session_id: String,
        api_key: &str,
        config: &InterpreterConfig,
    ) -> anyhow::Result<Self> {
        let model = VoiceProviderKind::OpenAiRealtime.model_id();
        let url = format!("{OPENAI_REALTIME_WS_URL}?model={model}");

        tracing::info!(
            session_id = %session_id,
            model = model,
            source = config.source_language.as_str(),
            target = config.target_language.as_str(),
            "Connecting to OpenAI Realtime"
        );

        // Build WebSocket request with auth headers
        let mut request = url
            .into_client_request()
            .map_err(|e| anyhow::anyhow!("Failed to build WebSocket request: {e}"))?;
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {api_key}")
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid auth header: {e}"))?,
        );
        request.headers_mut().insert(
            "OpenAI-Beta",
            "realtime=v1"
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid header: {e}"))?,
        );

        let (ws_stream, _response) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to OpenAI Realtime: {e}"))?;

        let (ws_sender, ws_receiver) = ws_stream.split();
        let ws_sender = Arc::new(Mutex::new(ws_sender));

        // Channels for bidirectional communication
        let (outbound_tx, outbound_rx) = mpsc::channel::<OutboundMessage>(256);
        let (event_tx, event_rx) = mpsc::channel::<VoiceEvent>(256);

        // Send session.update to configure the session
        let session_update = build_session_update(config);
        let update_json = serde_json::to_string(&session_update)?;
        tracing::debug!(session_id = %session_id, "Sending OpenAI session.update");
        {
            let mut sender = ws_sender.lock().await;
            sender
                .send(WsMessage::Text(update_json))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to send session.update: {e}"))?;
        }

        // Spawn outbound task
        let ws_sender_out = Arc::clone(&ws_sender);
        let sid_out = session_id.clone();
        tokio::spawn(async move {
            Self::outbound_loop(outbound_rx, ws_sender_out, sid_out).await;
        });

        // Spawn inbound task
        let sid_in = session_id.clone();
        tokio::spawn(async move {
            Self::inbound_loop(ws_receiver, event_tx, sid_in).await;
        });

        Ok(Self {
            outbound_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            session_id,
        })
    }

    /// Send a raw PCM audio chunk (24kHz PCM16) to OpenAI Realtime.
    pub async fn send_audio(&self, pcm_data: &[u8]) -> anyhow::Result<()> {
        if pcm_data.is_empty() {
            return Ok(());
        }
        self.outbound_tx
            .send(OutboundMessage::Audio(pcm_data.to_vec()))
            .await
            .map_err(|_| anyhow::anyhow!("Audio channel closed"))
    }

    /// Send a text message for text-based interpretation.
    pub async fn send_text(&self, text: &str) -> anyhow::Result<()> {
        self.outbound_tx
            .send(OutboundMessage::Text(text.to_string()))
            .await
            .map_err(|_| anyhow::anyhow!("Audio channel closed"))
    }

    /// Close the session gracefully.
    pub async fn close(&self) {
        let _ = self.outbound_tx.send(OutboundMessage::Close).await;
    }

    /// Get session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    // ── Internal loops ────────────────────────────────────────────

    /// Outbound loop: encode audio and send to OpenAI Realtime WebSocket.
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
        session_id: String,
    ) {
        let mut audio_chunk_count: u64 = 0;

        while let Some(msg) = rx.recv().await {
            match msg {
                OutboundMessage::Audio(pcm) => {
                    audio_chunk_count += 1;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&pcm);
                    let msg = serde_json::json!({
                        "type": "input_audio_buffer.append",
                        "audio": b64,
                    });
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if audio_chunk_count == 1 || audio_chunk_count.is_multiple_of(50) {
                            tracing::info!(
                                session_id = %session_id,
                                chunk = audio_chunk_count,
                                pcm_bytes = pcm.len(),
                                "Sending audio chunk to OpenAI Realtime"
                            );
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
                }
                OutboundMessage::Text(text) => {
                    let msg = serde_json::json!({
                        "type": "conversation.item.create",
                        "item": {
                            "type": "message",
                            "role": "user",
                            "content": [{
                                "type": "input_text",
                                "text": text,
                            }]
                        }
                    });
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let mut sender = ws_sender.lock().await;
                        if sender.send(WsMessage::Text(json)).await.is_err() {
                            break;
                        }
                        // Trigger response after text input
                        let response_create = serde_json::json!({"type": "response.create"});
                        if let Ok(json) = serde_json::to_string(&response_create) {
                            if sender.send(WsMessage::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                OutboundMessage::Close => {
                    let mut sender = ws_sender.lock().await;
                    let _ = sender.send(WsMessage::Close(None)).await;
                    break;
                }
            }
        }

        tracing::debug!(session_id = %session_id, "OpenAI Realtime outbound loop terminated");
    }

    /// Inbound loop: receive events from OpenAI Realtime and dispatch as VoiceEvents.
    async fn inbound_loop(
        mut ws_receiver: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        event_tx: mpsc::Sender<VoiceEvent>,
        session_id: String,
    ) {
        let start_time = std::time::Instant::now();
        let mut audio_response_count: u64 = 0;

        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(WsMessage::Text(text)) => {
                    let events = parse_server_event(
                        &text,
                        &session_id,
                        start_time,
                        &mut audio_response_count,
                    );
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
                Ok(WsMessage::Close(frame)) => {
                    tracing::info!(
                        session_id = %session_id,
                        close_frame = ?frame,
                        "OpenAI Realtime connection closed"
                    );
                    break;
                }
                Ok(
                    WsMessage::Ping(_)
                    | WsMessage::Pong(_)
                    | WsMessage::Frame(_)
                    | WsMessage::Binary(_),
                ) => {
                    // Binary frames not expected from OpenAI Realtime; ping/pong handled by tungstenite
                }
                Err(e) => {
                    tracing::error!(
                        session_id = %session_id,
                        error = %e,
                        "OpenAI Realtime WebSocket error"
                    );
                    let _ = event_tx
                        .send(VoiceEvent::Error {
                            message: format!("WebSocket error: {e}"),
                        })
                        .await;
                    break;
                }
            }
        }

        tracing::debug!(session_id = %session_id, "OpenAI Realtime inbound loop terminated");
    }
}

// ── Session update message ─────────────────────────────────────────

/// Build the `session.update` message for OpenAI Realtime.
fn build_session_update(config: &InterpreterConfig) -> serde_json::Value {
    let system_prompt = config.build_system_prompt();

    serde_json::json!({
        "type": "session.update",
        "session": {
            "instructions": system_prompt,
            "input_audio_format": "pcm16",
            "output_audio_format": "pcm16",
            "input_audio_transcription": {
                "model": "gpt-4o-mini-transcribe"
            },
            "turn_detection": {
                "type": "semantic_vad",
                "eagerness": "high"
            }
        }
    })
}

// ── Server event parsing ───────────────────────────────────────────

/// Parse an OpenAI Realtime server event into VoiceEvents.
fn parse_server_event(
    json_text: &str,
    session_id: &str,
    start_time: std::time::Instant,
    audio_response_count: &mut u64,
) -> Vec<VoiceEvent> {
    let mut events = Vec::new();
    let elapsed = start_time.elapsed().as_secs_f32();

    let value: serde_json::Value = match serde_json::from_str(json_text) {
        Ok(v) => v,
        Err(e) => {
            events.push(VoiceEvent::Error {
                message: format!("Failed to parse OpenAI event: {e}"),
            });
            return events;
        }
    };

    let event_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match event_type {
        // Session lifecycle
        "session.created" | "session.updated" => {
            tracing::info!(
                session_id = %session_id,
                event_type = event_type,
                "OpenAI Realtime session ready"
            );
            events.push(VoiceEvent::SetupComplete);
        }

        // Audio output
        "response.audio.delta" => {
            if let Some(delta_b64) = value.get("delta").and_then(|v| v.as_str()) {
                if let Ok(audio_bytes) = base64::engine::general_purpose::STANDARD.decode(delta_b64)
                {
                    *audio_response_count += 1;
                    if *audio_response_count == 1 || (*audio_response_count).is_multiple_of(50) {
                        tracing::info!(
                            session_id = %session_id,
                            t = format!("{elapsed:.1}s"),
                            audio_n = *audio_response_count,
                            bytes = audio_bytes.len(),
                            "⬇ OpenAI audio response"
                        );
                    }
                    events.push(VoiceEvent::Audio { data: audio_bytes });
                }
            }
        }

        // Output transcript (model speech)
        "response.audio_transcript.delta" => {
            if let Some(text) = value.get("delta").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    tracing::info!(
                        session_id = %session_id,
                        t = format!("{elapsed:.1}s"),
                        text = %text,
                        "⬇ Output transcript delta"
                    );
                    events.push(VoiceEvent::OutputTranscript {
                        text: text.to_string(),
                    });
                }
            }
        }

        // Input transcript (user speech)
        "conversation.item.input_audio_transcription.completed" => {
            if let Some(text) = value.get("transcript").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    tracing::info!(
                        session_id = %session_id,
                        t = format!("{elapsed:.1}s"),
                        text = %text,
                        "⬇ Input transcript"
                    );
                    events.push(VoiceEvent::InputTranscript {
                        text: text.to_string(),
                    });
                }
            }
        }

        // Response done
        "response.done" => {
            tracing::info!(
                session_id = %session_id,
                t = format!("{elapsed:.1}s"),
                "⬇ Response done (turn complete)"
            );
            events.push(VoiceEvent::TurnComplete);
        }

        // VAD events (informational logging only)
        "input_audio_buffer.speech_started" => {
            tracing::info!(
                session_id = %session_id,
                t = format!("{elapsed:.1}s"),
                "⬇ Semantic VAD: speech started"
            );
        }
        "input_audio_buffer.speech_stopped" => {
            tracing::info!(
                session_id = %session_id,
                t = format!("{elapsed:.1}s"),
                "⬇ Semantic VAD: speech stopped"
            );
        }

        // Interruption (response cancelled due to new input)
        "response.cancelled" => {
            tracing::info!(
                session_id = %session_id,
                t = format!("{elapsed:.1}s"),
                "⬇ Response cancelled (interrupted)"
            );
            events.push(VoiceEvent::Interrupted);
        }

        // Error
        "error" => {
            let message = value
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown OpenAI error");
            tracing::error!(
                session_id = %session_id,
                error = %message,
                "OpenAI Realtime error"
            );
            events.push(VoiceEvent::Error {
                message: message.to_string(),
            });
        }

        // Other events — log at debug level
        _ => {
            tracing::debug!(
                session_id = %session_id,
                event_type = event_type,
                "OpenAI Realtime event (unhandled)"
            );
        }
    }

    events
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_session_update_contains_instructions() {
        let config = InterpreterConfig::default();
        let msg = build_session_update(&config);

        assert_eq!(msg["type"], "session.update");
        assert!(msg["session"]["instructions"]
            .as_str()
            .unwrap()
            .contains("interpreter"));
        assert_eq!(msg["session"]["input_audio_format"], "pcm16");
        assert_eq!(msg["session"]["output_audio_format"], "pcm16");
        assert_eq!(msg["session"]["turn_detection"]["type"], "semantic_vad");
    }

    #[test]
    fn parse_session_created() {
        let json = r#"{"type": "session.created", "session": {}}"#;
        let mut count = 0;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], VoiceEvent::SetupComplete));
    }

    #[test]
    fn parse_audio_delta() {
        let audio_b64 = base64::engine::general_purpose::STANDARD.encode([10u8, 20, 30]);
        let json = format!(r#"{{"type": "response.audio.delta", "delta": "{audio_b64}"}}"#);
        let mut count = 0;
        let events = parse_server_event(&json, "test", std::time::Instant::now(), &mut count);

        assert_eq!(events.len(), 1);
        if let VoiceEvent::Audio { data } = &events[0] {
            assert_eq!(data, &[10u8, 20, 30]);
        } else {
            panic!("Expected Audio event");
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn parse_output_transcript_delta() {
        let json = r#"{"type": "response.audio_transcript.delta", "delta": "こんにちは"}"#;
        let mut count = 0;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);

        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            VoiceEvent::OutputTranscript { text } if text == "こんにちは"
        ));
    }

    #[test]
    fn parse_input_transcription_completed() {
        let json = r#"{"type": "conversation.item.input_audio_transcription.completed", "transcript": "안녕하세요"}"#;
        let mut count = 0;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);

        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            VoiceEvent::InputTranscript { text } if text == "안녕하세요"
        ));
    }

    #[test]
    fn parse_response_done() {
        let json = r#"{"type": "response.done"}"#;
        let mut count = 0;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], VoiceEvent::TurnComplete));
    }

    #[test]
    fn parse_error_event() {
        let json = r#"{"type": "error", "error": {"message": "Rate limit exceeded"}}"#;
        let mut count = 0;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);

        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            VoiceEvent::Error { message } if message.contains("Rate limit")
        ));
    }

    #[test]
    fn parse_vad_events_produce_no_voice_events() {
        let json = r#"{"type": "input_audio_buffer.speech_started"}"#;
        let mut count = 0;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);
        assert!(events.is_empty());

        let json = r#"{"type": "input_audio_buffer.speech_stopped"}"#;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);
        assert!(events.is_empty());
    }

    #[test]
    fn parse_response_cancelled() {
        let json = r#"{"type": "response.cancelled"}"#;
        let mut count = 0;
        let events = parse_server_event(json, "test", std::time::Instant::now(), &mut count);

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], VoiceEvent::Interrupted));
    }

    #[test]
    fn input_sample_rate_is_24khz() {
        assert_eq!(INPUT_SAMPLE_RATE, 24000);
    }
}
