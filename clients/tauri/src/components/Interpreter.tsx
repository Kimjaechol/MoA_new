import { useState, useRef, useEffect, useCallback } from "react";
import { t, type Locale } from "../lib/i18n";
import {
  apiClient,
  type VoiceLanguage,
  type VoiceUiManifest,
} from "../lib/api";
import { AudioCapture, AudioPlayer, type AudioOutputMode } from "../lib/audio";

/**
 * Real-time voice interpreter component.
 *
 * Architecture: "끊김 없는 스트리밍 파이프라인"
 *
 * 1. Mic → 30ms PCM chunks → binary WebSocket → Gateway → Gemini Live
 * 2. Gemini Live → partial translated audio → binary WebSocket → immediate playback
 * 3. VAD handled server-side — client just streams continuously (hands-free)
 * 4. Single persistent WebSocket session for the entire conversation
 * 5. Transcript text displayed independently from audio (no delay coupling)
 */

// ── Types ────────────────────────────────────────────────────────

type SessionStatus =
  | "idle"
  | "creating"
  | "connecting"
  | "listening"
  | "error"
  | "closed";

interface TranscriptEntry {
  id: number;
  type: "input" | "output";
  text: string;
}

interface InterpreterProps {
  locale: Locale;
  onBack: () => void;
}

// ── Component ────────────────────────────────────────────────────

export function Interpreter({ locale, onBack }: InterpreterProps) {
  // UI manifest (languages, defaults)
  const [manifest, setManifest] = useState<VoiceUiManifest | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Language selection
  const [sourceLang, setSourceLang] = useState("ko");
  const [targetLang, setTargetLang] = useState("en");
  const [bidirectional, setBidirectional] = useState(false);

  // Output mode: speaker (loud) vs whisper (earpiece)
  const [outputMode, setOutputMode] = useState<AudioOutputMode>("speaker");

  // Session state
  const [status, setStatus] = useState<SessionStatus>("idle");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [transcripts, setTranscripts] = useState<TranscriptEntry[]>([]);
  const transcriptIdRef = useRef(0);
  const transcriptsEndRef = useRef<HTMLDivElement>(null);

  // Refs for audio/WS (avoid stale closure issues)
  const wsRef = useRef<WebSocket | null>(null);
  const captureRef = useRef<AudioCapture | null>(null);
  const playerRef = useRef<AudioPlayer | null>(null);
  const statusRef = useRef<SessionStatus>("idle");

  // Keep statusRef in sync
  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  // ── Load language manifest on mount ────────────────────────────

  useEffect(() => {
    apiClient
      .getVoiceUi()
      .then((m) => {
        setManifest(m);
        if (m.default_source) setSourceLang(m.default_source);
        if (m.default_target) setTargetLang(m.default_target);
      })
      .catch((e) => setLoadError(e.message));
  }, []);

  // Auto-scroll transcripts
  useEffect(() => {
    transcriptsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [transcripts.length]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      teardown();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ── Teardown helper ────────────────────────────────────────────

  const teardown = useCallback(() => {
    captureRef.current?.stop();
    captureRef.current = null;

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    playerRef.current?.stop();
    playerRef.current = null;
  }, []);

  // ── Swap languages ─────────────────────────────────────────────

  const swapLanguages = useCallback(() => {
    setSourceLang((prev) => {
      setTargetLang(prev);
      return targetLang;
    });
  }, [targetLang]);

  // ── Start interpretation session ───────────────────────────────

  const startSession = useCallback(async () => {
    try {
      setStatus("creating");
      setErrorMsg(null);
      setTranscripts([]);

      // 1. Create voice session on backend
      const session = await apiClient.createVoiceSession({
        source_language: sourceLang,
        target_language: targetLang,
        bidirectional,
      });

      // 2. Initialize audio player (from user gesture context)
      //    Minimal buffer: ~50ms for click-free playback
      const player = new AudioPlayer();
      player.init();
      player.setMode(outputMode);
      playerRef.current = player;

      // 3. Open persistent WebSocket to gateway
      setStatus("connecting");
      const wsUrl = apiClient.getVoiceWsUrl(session.session_id);
      const ws = new WebSocket(wsUrl);
      ws.binaryType = "arraybuffer"; // binary frames = translated PCM audio
      wsRef.current = ws;

      ws.onmessage = (event) => {
        if (event.data instanceof ArrayBuffer) {
          // Binary frame → partial translated audio.
          // Play IMMEDIATELY — no buffering, no waiting for sentence end.
          playerRef.current?.enqueue(event.data);
        } else {
          // Text frame → JSON control/transcript event
          handleServerEvent(event.data);
        }
      };

      ws.onerror = () => {
        setStatus("error");
        setErrorMsg(t("interp_ws_error", locale));
      };

      ws.onclose = () => {
        if (statusRef.current !== "idle") {
          setStatus("closed");
        }
      };

      // 4. Wait for WebSocket to open, then start mic
      await new Promise<void>((resolve, reject) => {
        ws.onopen = () => resolve();
        const origError = ws.onerror;
        ws.onerror = (e) => {
          origError?.call(ws, e);
          reject(new Error("WebSocket connection failed"));
        };
      });

      // 5. Start microphone capture — 30ms chunks, streamed continuously.
      //    VAD is handled server-side (Gemini Live). Client is hands-free.
      const capture = new AudioCapture();
      captureRef.current = capture;
      await capture.start((pcmChunk) => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(pcmChunk); // Binary frame — no Base64!
        }
      });

      setStatus("listening");
    } catch (err) {
      teardown();
      setStatus("error");
      setErrorMsg(
        err instanceof Error ? err.message : t("interp_start_failed", locale),
      );
    }
  }, [sourceLang, targetLang, bidirectional, outputMode, locale, teardown]);

  // ── Handle server events (JSON text frames) ────────────────────

  const handleServerEvent = useCallback((data: string) => {
    try {
      const msg = JSON.parse(data);

      if (msg.type === "input_transcript" && msg.text) {
        // User's speech transcribed (displayed independently from audio)
        const id = ++transcriptIdRef.current;
        setTranscripts((prev) => [...prev, { id, type: "input", text: msg.text }]);
      } else if (msg.type === "output_transcript" && msg.text) {
        // Translation text (for subtitle display, independent from audio playback)
        const id = ++transcriptIdRef.current;
        setTranscripts((prev) => [...prev, { id, type: "output", text: msg.text }]);
      } else if (msg.type === "interrupted") {
        // User started speaking mid-response → cut off current playback
        playerRef.current?.interrupt();
      } else if (msg.type === "error" && msg.message) {
        setErrorMsg(msg.message);
      }
    } catch {
      // Non-JSON or malformed — ignore silently
    }
  }, []);

  // ── Stop session ───────────────────────────────────────────────

  const stopSession = useCallback(() => {
    teardown();
    setStatus("idle");
  }, [teardown]);

  // ── Toggle output mode ─────────────────────────────────────────

  const toggleOutputMode = useCallback(() => {
    setOutputMode((prev) => {
      const next = prev === "speaker" ? "whisper" : "speaker";
      playerRef.current?.setMode(next);
      return next;
    });
  }, []);

  // ── Render ─────────────────────────────────────────────────────

  const isActive = status === "listening" || status === "connecting" || status === "creating";
  const languages = manifest?.languages ?? [];

  // Error state: show error with retry
  if (loadError) {
    return (
      <div className="interpreter-container">
        <div className="interpreter-header">
          <button className="interpreter-back-btn" onClick={onBack}>{"\u2190"}</button>
          <span className="interpreter-title">{t("interp_title", locale)}</span>
        </div>
        <div className="interpreter-error">{loadError}</div>
      </div>
    );
  }

  return (
    <div className="interpreter-container">
      {/* ── Header ── */}
      <div className="interpreter-header">
        <button className="interpreter-back-btn" onClick={onBack}>{"\u2190"}</button>
        <span className="interpreter-title">{t("interp_title", locale)}</span>
        <div className="interpreter-status">
          <div className={`status-dot ${isActive ? "connected" : ""}`} />
          <span>{t(`interp_status_${status}` as any, locale)}</span>
        </div>
      </div>

      {/* ── Language Selector ── */}
      <div className="interpreter-lang-row">
        <div className="interpreter-lang-select">
          <label>{t("interp_source", locale)}</label>
          <select
            value={sourceLang}
            onChange={(e) => setSourceLang(e.target.value)}
            disabled={isActive}
          >
            {languages.map((lang: VoiceLanguage) => (
              <option key={lang.code} value={lang.code}>
                {lang.flag} {lang.native_name}
              </option>
            ))}
          </select>
        </div>

        <button
          className="interpreter-swap-btn"
          onClick={swapLanguages}
          disabled={isActive}
          aria-label="Swap languages"
        >
          {"\u21C4"}
        </button>

        <div className="interpreter-lang-select">
          <label>{t("interp_target", locale)}</label>
          <select
            value={targetLang}
            onChange={(e) => setTargetLang(e.target.value)}
            disabled={isActive}
          >
            {languages.map((lang: VoiceLanguage) => (
              <option key={lang.code} value={lang.code}>
                {lang.flag} {lang.native_name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* ── Options Row ── */}
      <div className="interpreter-options-row">
        <label className="interpreter-checkbox">
          <input
            type="checkbox"
            checked={bidirectional}
            onChange={(e) => setBidirectional(e.target.checked)}
            disabled={isActive}
          />
          {t("interp_bidirectional", locale)}
        </label>

        <button
          className={`interpreter-output-mode ${outputMode}`}
          onClick={toggleOutputMode}
          title={outputMode === "speaker"
            ? t("interp_mode_speaker", locale)
            : t("interp_mode_whisper", locale)}
        >
          {outputMode === "speaker" ? (
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
              <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
              <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
            </svg>
          ) : (
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
              <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
            </svg>
          )}
          <span>{outputMode === "speaker"
            ? t("interp_mode_speaker", locale)
            : t("interp_mode_whisper", locale)}
          </span>
        </button>
      </div>

      {/* ── Transcript Area ── */}
      <div className="interpreter-transcripts">
        {transcripts.length === 0 && !isActive && (
          <div className="interpreter-placeholder">
            {t("interp_placeholder", locale)}
          </div>
        )}
        {transcripts.length === 0 && isActive && (
          <div className="interpreter-placeholder interpreter-pulse">
            {t("interp_listening_hint", locale)}
          </div>
        )}
        {transcripts.map((entry) => (
          <div key={entry.id} className={`interpreter-transcript ${entry.type}`}>
            <div className="transcript-label">
              {entry.type === "input"
                ? t("interp_you_said", locale)
                : t("interp_translation", locale)}
            </div>
            <div className="transcript-text">{entry.text}</div>
          </div>
        ))}
        <div ref={transcriptsEndRef} />
      </div>

      {/* ── Error Banner ── */}
      {errorMsg && (
        <div className="interpreter-error-banner">
          {errorMsg}
          <button onClick={() => setErrorMsg(null)}>{"\u2715"}</button>
        </div>
      )}

      {/* ── Mic Button (Session ON/OFF Toggle) ── */}
      <div className="interpreter-controls">
        {isActive ? (
          <button className="interpreter-mic-btn active" onClick={stopSession}>
            <div className="mic-pulse-ring" />
            <svg width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
              <rect x="6" y="6" width="12" height="12" rx="2" />
            </svg>
            <span>{t("interp_stop", locale)}</span>
          </button>
        ) : (
          <button
            className="interpreter-mic-btn"
            onClick={startSession}
            disabled={status === "creating"}
          >
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
              <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
              <line x1="12" y1="19" x2="12" y2="23" />
              <line x1="8" y1="23" x2="16" y2="23" />
            </svg>
            <span>{t("interp_start", locale)}</span>
          </button>
        )}
      </div>
    </div>
  );
}
