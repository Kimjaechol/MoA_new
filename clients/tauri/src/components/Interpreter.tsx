import { useState, useRef, useCallback, useEffect } from "react";
import { t, type Locale } from "../lib/i18n";
import { apiClient } from "../lib/api";

interface InterpreterProps {
  locale: Locale;
  onBack: () => void;
  onToggleSidebar: () => void;
  sidebarOpen: boolean;
}

interface Transcript {
  id: string;
  type: "input" | "output";
  text: string;
  timestamp: number;
}

type ConnectionStatus = "idle" | "connecting" | "ready" | "listening" | "stopping" | "error";

const LANGUAGES = [
  { code: "ko", name: "한국어", flag: "🇰🇷" },
  { code: "en", name: "English", flag: "🇺🇸" },
  { code: "ja", name: "日本語", flag: "🇯🇵" },
  { code: "zh", name: "中文", flag: "🇨🇳" },
  { code: "es", name: "Español", flag: "🇪🇸" },
  { code: "fr", name: "Français", flag: "🇫🇷" },
  { code: "de", name: "Deutsch", flag: "🇩🇪" },
  { code: "th", name: "ไทย", flag: "🇹🇭" },
  { code: "vi", name: "Tiếng Việt", flag: "🇻🇳" },
  { code: "ru", name: "Русский", flag: "🇷🇺" },
  { code: "ar", name: "العربية", flag: "🇸🇦" },
  { code: "pt", name: "Português", flag: "🇧🇷" },
  { code: "it", name: "Italiano", flag: "🇮🇹" },
  { code: "hi", name: "हिन्दी", flag: "🇮🇳" },
  { code: "id", name: "Bahasa Indonesia", flag: "🇮🇩" },
  { code: "tr", name: "Türkçe", flag: "🇹🇷" },
];

// AudioWorklet processor code (inline, runs in audio thread)
const WORKLET_CODE = `
class PcmCaptureProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this._buffer = [];
    this._targetSamples = 1600; // 100ms at 16kHz
  }
  process(inputs) {
    const input = inputs[0];
    if (!input || !input[0]) return true;
    const samples = input[0];
    // Downsample from source rate to 16kHz happens via AudioContext
    for (let i = 0; i < samples.length; i++) {
      this._buffer.push(samples[i]);
    }
    while (this._buffer.length >= this._targetSamples) {
      const chunk = this._buffer.splice(0, this._targetSamples);
      const pcm16 = new Int16Array(chunk.length);
      for (let i = 0; i < chunk.length; i++) {
        const s = Math.max(-1, Math.min(1, chunk[i]));
        pcm16[i] = s < 0 ? s * 0x8000 : s * 0x7FFF;
      }
      this.port.postMessage(pcm16.buffer, [pcm16.buffer]);
    }
    return true;
  }
}
registerProcessor('pcm-capture-processor', PcmCaptureProcessor);
`;

export function Interpreter({
  locale,
  onBack,
  onToggleSidebar,
  sidebarOpen,
}: InterpreterProps) {
  void onBack; // available for future navigation
  const [status, setStatus] = useState<ConnectionStatus>("idle");
  const [sourceLang, setSourceLang] = useState("ko");
  const [targetLang, setTargetLang] = useState("ja");
  const [bidirectional, setBidirectional] = useState(true);
  const [transcripts, setTranscripts] = useState<Transcript[]>([]);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const workletNodeRef = useRef<AudioWorkletNode | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const playbackCtxRef = useRef<AudioContext | null>(null);
  const transcriptEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll transcripts
  useEffect(() => {
    transcriptEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [transcripts]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      endSession();
    };
  }, []);

  const addTranscript = useCallback((type: "input" | "output", text: string) => {
    setTranscripts((prev) => [
      ...prev,
      { id: crypto.randomUUID(), type, text, timestamp: Date.now() },
    ]);
  }, []);

  const playAudioChunk = useCallback(async (pcmData: ArrayBuffer) => {
    if (!playbackCtxRef.current) {
      playbackCtxRef.current = new AudioContext({ sampleRate: 24000 });
    }
    const ctx = playbackCtxRef.current;
    const int16 = new Int16Array(pcmData);
    const float32 = new Float32Array(int16.length);
    for (let i = 0; i < int16.length; i++) {
      float32[i] = int16[i] / 32768;
    }
    const buffer = ctx.createBuffer(1, float32.length, 24000);
    buffer.getChannelData(0).set(float32);
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(ctx.destination);
    source.start();
  }, []);

  const startSession = useCallback(async () => {
    setError(null);
    setStatus("connecting");
    setTranscripts([]);

    try {
      // 1. Create voice session via REST API
      const serverUrl = apiClient.getServerUrl();
      const token = apiClient.getToken();

      const res = await fetch(`${serverUrl}/api/voice/sessions`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          source_language: sourceLang,
          target_language: targetLang,
          bidirectional,
        }),
      });

      if (!res.ok) {
        const data = await res.json().catch(() => ({ error: "Failed to create session" }));
        throw new Error(data.error || `Session creation failed (${res.status})`);
      }

      const session = await res.json();
      const sessionId = session.session_id;

      // 2. Connect WebSocket (pass token as query param since WS can't use headers)
      const wsUrl = serverUrl.replace(/^http/, "ws") + `/api/voice/interpret?session_id=${sessionId}&token=${token}`;
      const ws = new WebSocket(wsUrl);
      wsRef.current = ws;

      ws.binaryType = "arraybuffer";

      ws.onopen = () => {
        setStatus("ready");
      };

      ws.onmessage = (event) => {
        if (event.data instanceof ArrayBuffer) {
          // Binary: translated audio PCM
          playAudioChunk(event.data);
        } else {
          // Text: JSON event
          try {
            const msg = JSON.parse(event.data);
            switch (msg.type) {
              case "ready":
                setStatus("listening");
                startMicrophone();
                break;
              case "input_transcript":
                if (msg.text) addTranscript("input", msg.text);
                break;
              case "output_transcript":
                if (msg.text) addTranscript("output", msg.text);
                break;
              case "turn_complete":
                setStatus((prev) => {
                  if (prev === "stopping") {
                    // Drain complete — schedule full cleanup
                    setTimeout(() => endSession(), 0);
                    return prev; // endSession will set idle
                  }
                  return prev;
                });
                break;
              case "error":
                setError(msg.message);
                break;
            }
          } catch {
            // ignore parse errors
          }
        }
      };

      ws.onerror = () => {
        // Ignore errors during intentional close (stop button or draining)
        if (wsRef.current === null) return;
        setStatus((prev) => {
          if (prev === "stopping") return prev; // ignore during drain
          setError("WebSocket connection error");
          return "error";
        });
      };

      ws.onclose = () => {
        if (status !== "idle") {
          setStatus("idle");
        }
      };
    } catch (e) {
      setError(e instanceof Error ? e.message : "Connection failed");
      setStatus("error");
    }
  }, [sourceLang, targetLang, bidirectional, addTranscript, playAudioChunk, status]);

  const startMicrophone = useCallback(async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          sampleRate: 16000,
          channelCount: 1,
          echoCancellation: true,
          noiseSuppression: true,
        },
      });
      streamRef.current = stream;

      const ctx = new AudioContext({ sampleRate: 16000 });
      audioContextRef.current = ctx;

      // Register worklet
      const blob = new Blob([WORKLET_CODE], { type: "application/javascript" });
      const url = URL.createObjectURL(blob);
      await ctx.audioWorklet.addModule(url);
      URL.revokeObjectURL(url);

      const source = ctx.createMediaStreamSource(stream);
      const workletNode = new AudioWorkletNode(ctx, "pcm-capture-processor");
      workletNodeRef.current = workletNode;

      workletNode.port.onmessage = (e) => {
        const ws = wsRef.current;
        if (ws && ws.readyState === WebSocket.OPEN) {
          ws.send(e.data);
        }
      };

      source.connect(workletNode);
      workletNode.connect(ctx.destination); // needed to keep processor alive

      setStatus("listening");
    } catch (e) {
      setError("Microphone access denied");
      setStatus("error");
    }
  }, []);

  // Stop microphone only — keep WS and playback alive for remaining translation
  const stopMicrophone = useCallback(() => {
    if (workletNodeRef.current) {
      workletNodeRef.current.disconnect();
      workletNodeRef.current = null;
    }
    if (audioContextRef.current) {
      audioContextRef.current.close();
      audioContextRef.current = null;
    }
    if (streamRef.current) {
      streamRef.current.getTracks().forEach((t) => t.stop());
      streamRef.current = null;
    }
  }, []);

  // Full cleanup — close WS, playback, reset state
  const endSession = useCallback(() => {
    stopMicrophone();

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    if (playbackCtxRef.current) {
      playbackCtxRef.current.close();
      playbackCtxRef.current = null;
    }

    setStatus("idle");
  }, [stopMicrophone]);

  // Graceful stop: mic off → notify server → wait for turn_complete → end
  const handleStop = useCallback(() => {
    stopMicrophone();

    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      try {
        wsRef.current.send(JSON.stringify({ type: "stop" }));
      } catch {
        // ignore send errors
      }
    }

    setStatus("stopping");

    // Safety timeout: if turn_complete never arrives, force end after 10s
    setTimeout(() => {
      setStatus((prev) => {
        if (prev === "stopping") {
          endSession();
        }
        return prev === "stopping" ? "idle" : prev;
      });
    }, 10000);
  }, [stopMicrophone, endSession]);

  const swapLanguages = () => {
    setSourceLang(targetLang);
    setTargetLang(sourceLang);
  };

  const isActive = status === "listening" || status === "ready" || status === "connecting" || status === "stopping";

  return (
    <div className="interpreter-page">
      {/* Header */}
      <div className="chat-header">
        <button className="header-toggle-btn" onClick={onToggleSidebar}>
          {sidebarOpen ? "\u2715" : "\u2630"}
        </button>
        <div className="header-title">{t("interpreter", locale)}</div>
        <div className={`connection-badge ${status === "listening" ? "connected" : ""}`}>
          {status === "idle"
            ? t("interpreter_idle", locale)
            : status === "connecting"
            ? t("interpreter_connecting", locale)
            : status === "listening"
            ? t("interpreter_listening", locale)
            : status === "stopping"
            ? t("interpreter_stopping", locale)
            : status === "ready"
            ? t("interpreter_ready", locale)
            : t("interpreter_error", locale)}
        </div>
      </div>

      {/* Language selector */}
      <div className="interpreter-controls">
        <div className="lang-selector">
          <select
            value={sourceLang}
            onChange={(e) => setSourceLang(e.target.value)}
            disabled={isActive}
            className="lang-select"
          >
            {LANGUAGES.map((l) => (
              <option key={l.code} value={l.code}>
                {l.flag} {l.name}
              </option>
            ))}
          </select>

          <button
            className="lang-swap-btn"
            onClick={swapLanguages}
            disabled={isActive}
            title="Swap languages"
          >
            ⇄
          </button>

          <select
            value={targetLang}
            onChange={(e) => setTargetLang(e.target.value)}
            disabled={isActive}
            className="lang-select"
          >
            {LANGUAGES.map((l) => (
              <option key={l.code} value={l.code}>
                {l.flag} {l.name}
              </option>
            ))}
          </select>
        </div>

        <label className="bidi-toggle">
          <input
            type="checkbox"
            checked={bidirectional}
            onChange={(e) => setBidirectional(e.target.checked)}
            disabled={isActive}
          />
          <span>{t("interpreter_bidirectional", locale)}</span>
        </label>
      </div>

      {/* Transcript area */}
      <div className="interpreter-transcripts">
        {transcripts.length === 0 && status === "idle" && (
          <div className="interpreter-empty">
            <div className="interpreter-empty-icon">🎙️</div>
            <p>{t("interpreter_hint", locale)}</p>
          </div>
        )}
        {transcripts.map((tr) => (
          <div key={tr.id} className={`transcript-item transcript-${tr.type}`}>
            <span className="transcript-badge">
              {tr.type === "input" ? "🗣️" : "🔊"}
            </span>
            <span className="transcript-text">{tr.text}</span>
          </div>
        ))}
        {status === "listening" && (
          <div className="transcript-item transcript-listening">
            <span className="listening-pulse" />
            <span className="transcript-text">{t("interpreter_listening_hint", locale)}</span>
          </div>
        )}
        <div ref={transcriptEndRef} />
      </div>

      {/* Error display */}
      {error && (
        <div className="interpreter-error">
          {error}
        </div>
      )}

      {/* Action button */}
      <div className="interpreter-actions">
        {!isActive ? (
          <button className="interpreter-start-btn" onClick={startSession}>
            🎙️ {t("interpreter_start", locale)}
          </button>
        ) : (
          <button
            className="interpreter-stop-btn"
            onClick={handleStop}
            disabled={status === "stopping"}
          >
            {status === "stopping"
              ? `⏳ ${t("interpreter_stopping", locale)}`
              : `⏹ ${t("interpreter_stop", locale)}`}
          </button>
        )}
      </div>
    </div>
  );
}
