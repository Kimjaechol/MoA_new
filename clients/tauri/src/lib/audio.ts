/**
 * Ultra-low-latency audio capture and playback for real-time interpretation.
 *
 * Strategy: "끊김 없는 스트리밍 파이프라인"
 * - Capture: getUserMedia → AudioWorklet (30ms chunks) → 16kHz mono PCM Int16 LE
 * - Transport: Binary WebSocket frames (no Base64 overhead)
 * - Playback: AudioContext with minimal buffering (~50ms), immediate partial output
 *
 * The goal is phrase-level / word-level partial translation with near-zero
 * perceived delay: audio chunks arrive from the server mid-sentence and are
 * played back immediately.
 */

// ── Constants ──────────────────────────────────────────────────────

/** Input sample rate for Gemini Live (16kHz mono). */
const INPUT_SAMPLE_RATE = 16000;

/** Output sample rate from Gemini Live (24kHz mono). */
const OUTPUT_SAMPLE_RATE = 24000;

/**
 * Capture chunk size in frames.
 * 480 frames at 16kHz = 30ms — small enough for near-instant streaming,
 * large enough to avoid excessive WebSocket frame overhead.
 */
const CAPTURE_CHUNK_FRAMES = 480;

// ── Audio output mode ──────────────────────────────────────────────

export type AudioOutputMode = "speaker" | "whisper";

/** Gain values: speaker = full volume, whisper = earpiece-level. */
const GAIN_VALUES: Record<AudioOutputMode, number> = {
  speaker: 1.0,
  whisper: 0.15,
};

// ── AudioWorklet processor code (inline) ───────────────────────────

/**
 * AudioWorklet processor that accumulates samples and emits fixed-size
 * chunks (CAPTURE_CHUNK_FRAMES) as Int16 LE via MessagePort.
 *
 * Inlined as a Blob URL to avoid same-origin restrictions.
 */
const WORKLET_CODE = `
class PcmCaptureProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this._buffer = new Float32Array(${CAPTURE_CHUNK_FRAMES});
    this._pos = 0;
  }

  process(inputs) {
    const input = inputs[0];
    if (!input || !input[0]) return true;
    const channelData = input[0];

    let i = 0;
    while (i < channelData.length) {
      const remaining = ${CAPTURE_CHUNK_FRAMES} - this._pos;
      const toCopy = Math.min(remaining, channelData.length - i);
      this._buffer.set(channelData.subarray(i, i + toCopy), this._pos);
      this._pos += toCopy;
      i += toCopy;

      if (this._pos >= ${CAPTURE_CHUNK_FRAMES}) {
        // Convert Float32 → Int16 LE
        const int16 = new Int16Array(${CAPTURE_CHUNK_FRAMES});
        for (let j = 0; j < ${CAPTURE_CHUNK_FRAMES}; j++) {
          const s = Math.max(-1, Math.min(1, this._buffer[j]));
          int16[j] = s < 0 ? s * 0x8000 : s * 0x7FFF;
        }
        this.port.postMessage(int16.buffer, [int16.buffer]);
        this._buffer = new Float32Array(${CAPTURE_CHUNK_FRAMES});
        this._pos = 0;
      }
    }
    return true;
  }
}

registerProcessor('pcm-capture-processor', PcmCaptureProcessor);
`;

// ── Audio Capture ──────────────────────────────────────────────────

/**
 * Captures microphone audio as 16kHz mono PCM Int16 LE.
 *
 * Prefers AudioWorklet (30ms chunks) for low latency, falls back to
 * ScriptProcessorNode (16ms chunks) if AudioWorklet is unavailable.
 */
export class AudioCapture {
  private stream: MediaStream | null = null;
  private audioCtx: AudioContext | null = null;
  private source: MediaStreamAudioSourceNode | null = null;
  private workletNode: AudioWorkletNode | null = null;
  private processorNode: ScriptProcessorNode | null = null;
  private onData: ((pcm: ArrayBuffer) => void) | null = null;

  /**
   * Start capturing microphone audio.
   * @param onData Called with raw PCM Int16 LE ArrayBuffer per chunk (~30ms).
   */
  async start(onData: (pcm: ArrayBuffer) => void): Promise<void> {
    this.onData = onData;

    this.stream = await navigator.mediaDevices.getUserMedia({
      audio: {
        sampleRate: { ideal: INPUT_SAMPLE_RATE },
        channelCount: 1,
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
    });

    this.audioCtx = new AudioContext({ sampleRate: INPUT_SAMPLE_RATE });

    // Resume context if suspended (browser autoplay policy)
    if (this.audioCtx.state === "suspended") {
      await this.audioCtx.resume();
    }

    this.source = this.audioCtx.createMediaStreamSource(this.stream);

    // Try AudioWorklet first for ultra-low latency
    try {
      await this.startWithWorklet();
    } catch {
      // Fallback to ScriptProcessorNode
      this.startWithScriptProcessor();
    }
  }

  private async startWithWorklet(): Promise<void> {
    const ctx = this.audioCtx!;
    const blob = new Blob([WORKLET_CODE], { type: "application/javascript" });
    const url = URL.createObjectURL(blob);

    try {
      await ctx.audioWorklet.addModule(url);
    } finally {
      URL.revokeObjectURL(url);
    }

    this.workletNode = new AudioWorkletNode(ctx, "pcm-capture-processor");
    this.workletNode.port.onmessage = (event: MessageEvent) => {
      if (this.onData && event.data instanceof ArrayBuffer) {
        this.onData(event.data);
      }
    };

    this.source!.connect(this.workletNode);
    // AudioWorklet doesn't need to connect to destination
    this.workletNode.connect(ctx.destination);
  }

  private startWithScriptProcessor(): void {
    const ctx = this.audioCtx!;
    // 256 frames at 16kHz = 16ms per chunk
    this.processorNode = ctx.createScriptProcessor(256, 1, 1);

    this.processorNode.onaudioprocess = (event) => {
      if (!this.onData) return;
      const float32 = event.inputBuffer.getChannelData(0);
      const int16 = float32ToInt16LE(float32);
      this.onData(int16.buffer);
    };

    this.source!.connect(this.processorNode);
    this.processorNode.connect(ctx.destination);
  }

  /** Stop capturing and release microphone. */
  stop(): void {
    this.onData = null;

    if (this.workletNode) {
      this.workletNode.disconnect();
      this.workletNode = null;
    }
    if (this.processorNode) {
      this.processorNode.disconnect();
      this.processorNode = null;
    }
    if (this.source) {
      this.source.disconnect();
      this.source = null;
    }
    if (this.audioCtx) {
      this.audioCtx.close().catch(() => {});
      this.audioCtx = null;
    }
    if (this.stream) {
      this.stream.getTracks().forEach((t) => t.stop());
      this.stream = null;
    }
  }
}

// ── Audio Playback (Ultra-Low-Latency) ─────────────────────────────

/**
 * Queued PCM audio player optimized for minimal latency.
 *
 * Design:
 * - Receives 24kHz PCM Int16 LE chunks from server (partial translations).
 * - Schedules playback immediately — no waiting for complete sentences.
 * - ~50ms effective buffer: just enough to prevent click/pop artifacts.
 * - GainNode for speaker (full volume) vs whisper (earpiece-level) mode.
 */
export class AudioPlayer {
  private audioCtx: AudioContext | null = null;
  private gainNode: GainNode | null = null;
  /** Next scheduled playback time (AudioContext.currentTime). */
  private nextPlayTime = 0;
  /** Tiny overlap to prevent gaps between chunks. */
  private readonly OVERLAP_SEC = 0.002; // 2ms crossfade buffer
  private mode: AudioOutputMode = "speaker";

  /** Initialize the audio context. Call from a user gesture handler. */
  init(): void {
    if (this.audioCtx) return;
    this.audioCtx = new AudioContext({ sampleRate: OUTPUT_SAMPLE_RATE });
    this.gainNode = this.audioCtx.createGain();
    this.gainNode.gain.value = GAIN_VALUES[this.mode];
    this.gainNode.connect(this.audioCtx.destination);
  }

  /** Set output mode (speaker or whisper). Takes effect immediately. */
  setMode(mode: AudioOutputMode): void {
    this.mode = mode;
    if (this.gainNode && this.audioCtx) {
      // Smooth ramp to avoid clicks
      this.gainNode.gain.linearRampToValueAtTime(
        GAIN_VALUES[mode],
        this.audioCtx.currentTime + 0.05,
      );
    }
  }

  getMode(): AudioOutputMode {
    return this.mode;
  }

  /**
   * Enqueue raw PCM Int16 LE bytes for immediate playback.
   *
   * Even very short chunks (20-40ms) are played immediately.
   * The AudioContext schedules them back-to-back with minimal gap.
   */
  enqueue(pcmData: ArrayBuffer): void {
    if (!this.audioCtx || !this.gainNode) {
      this.init();
    }
    const ctx = this.audioCtx!;

    // Resume context if suspended (mobile browsers)
    if (ctx.state === "suspended") {
      ctx.resume();
    }

    // Decode Int16 LE → Float32
    const int16 = new Int16Array(pcmData);
    if (int16.length === 0) return;

    const float32 = new Float32Array(int16.length);
    for (let i = 0; i < int16.length; i++) {
      float32[i] = int16[i] / 32768;
    }

    const buffer = ctx.createBuffer(1, float32.length, OUTPUT_SAMPLE_RATE);
    buffer.getChannelData(0).set(float32);

    // Schedule immediately — keep nextPlayTime just barely ahead of now.
    const now = ctx.currentTime;
    if (this.nextPlayTime < now) {
      // Caught up or first chunk — start with minimal delay (~50ms buffer)
      this.nextPlayTime = now + 0.05;
    }

    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(this.gainNode!);
    source.start(this.nextPlayTime);
    this.nextPlayTime += buffer.duration - this.OVERLAP_SEC;
  }

  /** Stop all playback immediately. */
  stop(): void {
    if (this.audioCtx) {
      this.audioCtx.close().catch(() => {});
      this.audioCtx = null;
      this.gainNode = null;
    }
    this.nextPlayTime = 0;
  }

  /**
   * Interrupt current playback (e.g., VAD detected user speaking again).
   * Recreates the AudioContext to instantly cut off all scheduled audio.
   */
  interrupt(): void {
    const mode = this.mode;
    this.stop();
    this.init();
    this.setMode(mode);
  }
}

// ── Helpers ────────────────────────────────────────────────────────

/** Convert Float32 samples [-1, 1] to Int16 LE. */
function float32ToInt16LE(float32: Float32Array): Int16Array {
  const int16 = new Int16Array(float32.length);
  for (let i = 0; i < float32.length; i++) {
    const s = Math.max(-1, Math.min(1, float32[i]));
    int16[i] = s < 0 ? s * 0x8000 : s * 0x7fff;
  }
  return int16;
}
