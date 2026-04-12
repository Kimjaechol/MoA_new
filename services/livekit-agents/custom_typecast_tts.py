"""
moa 음성 서비스 — Typecast TTS 커스텀 플러그인
================================================

Typecast TTS API (v1) 를 LiveKit Agents의 TTS 인터페이스에 맞춰 구현.

## API 스펙 (https://typecast.ai/docs/api-reference/text-to-speech)

- Endpoint: POST https://api.typecast.ai/v1/text-to-speech
- Auth: X-API-KEY 헤더
- Request: JSON { voice_id, text, model, prompt?, output? }
- Response: 바이너리 오디오 직접 반환 (동기, 폴링 불필요)
  - WAV: PCM 16-bit mono 44100Hz
  - MP3: 320kbps 44100Hz mono
- Streaming: POST /v1/text-to-speech/stream (청크 스트리밍)
- 감정: normal, happy, sad, angry, whisper, toneup, tonedown
- 모델: ssfm-v30 (최신, 37언어) / ssfm-v21 (안정, 27언어)

## LiveKit 통합 설계

LiveKit TTS 인터페이스는 `synthesize(text)` → `AsyncIterable[AudioFrame]` 패턴.
Typecast v1 API는 동기 응답이므로:
1. POST /v1/text-to-speech → 전체 WAV 바이너리 수신
2. WAV 헤더 스킵 (44 bytes)
3. PCM16 데이터를 100ms 프레임으로 분할
4. LiveKit AudioFrame으로 변환하여 yield

스트리밍 엔드포인트(/v1/text-to-speech/stream)가 있으면 청크 단위로
바로 yield하여 TTFA(Time To First Audio)를 줄일 수 있음 (향후 최적화).
"""

from __future__ import annotations

import io
import logging
import os
import struct
from dataclasses import dataclass, field
from typing import AsyncIterable

import aiohttp
from livekit.agents import tts

logger = logging.getLogger("moa.typecast_tts")

# ── Typecast API constants ─────────────────────────────────────────

TYPECAST_API_BASE = "https://api.typecast.ai"
TYPECAST_TTS_ENDPOINT = f"{TYPECAST_API_BASE}/v1/text-to-speech"
TYPECAST_TTS_STREAM_ENDPOINT = f"{TYPECAST_API_BASE}/v1/text-to-speech/stream"
TYPECAST_VOICES_ENDPOINT = f"{TYPECAST_API_BASE}/v2/voices"

# Default model — ssfm-v30 is latest (37 languages, 7 emotions)
DEFAULT_MODEL = "ssfm-v30"

# Output audio: WAV PCM 16-bit mono 44100Hz (Typecast default)
SAMPLE_RATE = 44100
NUM_CHANNELS = 1
BYTES_PER_SAMPLE = 2  # 16-bit
WAV_HEADER_SIZE = 44

# Frame size for LiveKit: 100ms of audio
FRAME_DURATION_MS = 100
SAMPLES_PER_FRAME = SAMPLE_RATE * FRAME_DURATION_MS // 1000
BYTES_PER_FRAME = SAMPLES_PER_FRAME * BYTES_PER_SAMPLE * NUM_CHANNELS


@dataclass
class TypecastVoiceOptions:
    """Voice configuration for Typecast TTS."""

    voice_id: str = ""
    """Typecast voice ID (format: tc_<objectid>). Use GET /v2/voices to list."""

    model: str = DEFAULT_MODEL
    """TTS model: 'ssfm-v30' (latest) or 'ssfm-v21' (stable)."""

    emotion: str = "normal"
    """Emotion preset: normal, happy, sad, angry, whisper, toneup, tonedown."""

    emotion_intensity: float = 1.0
    """Emotion intensity: 0.0–2.0 (default 1.0)."""

    speed: float = 1.0
    """Speech speed multiplier: 0.5–2.0 (default 1.0)."""

    pitch: int = 0
    """Pitch adjustment in semitones: -12 to +12 (default 0)."""

    volume: int = 100
    """Volume: 0–200 (default 100)."""


class TypecastTTS(tts.TTS):
    """LiveKit-compatible TTS using Typecast API (v1).

    Usage:
        tts = TypecastTTS(
            voice_options=TypecastVoiceOptions(
                voice_id="tc_672c5f5ce59fac2a48faeaee",
                emotion="happy",
            ),
            language="ko",
        )
    """

    def __init__(
        self,
        *,
        voice_options: TypecastVoiceOptions | None = None,
        language: str = "ko",
        api_key: str | None = None,
    ):
        super().__init__(
            capabilities=tts.TTSCapabilities(streaming=False),
            sample_rate=SAMPLE_RATE,
            num_channels=NUM_CHANNELS,
        )
        self._opts = voice_options or TypecastVoiceOptions()
        self._language = language
        self._api_key = api_key or os.environ.get("TYPECAST_API_KEY", "")

        if not self._api_key:
            raise ValueError(
                "Typecast API key is required. Set TYPECAST_API_KEY env var "
                "or pass api_key= to TypecastTTS()."
            )
        if not self._opts.voice_id:
            raise ValueError(
                "voice_id is required. Use GET https://api.typecast.ai/v2/voices "
                "to find available voice IDs (format: tc_<objectid>)."
            )

    def _build_request_body(self, text: str) -> dict:
        """Build the JSON request body for the Typecast TTS API."""
        body: dict = {
            "voice_id": self._opts.voice_id,
            "text": text[:2000],  # API limit: 2000 chars
            "model": self._opts.model,
        }

        # Language (auto-detected by default, but we specify for accuracy)
        if self._language:
            body["language"] = self._language

        # Emotion prompt (PresetPrompt format)
        if self._opts.emotion != "normal" or self._opts.emotion_intensity != 1.0:
            body["prompt"] = {
                "emotion_type": "preset",
                "emotion_preset": self._opts.emotion,
                "emotion_intensity": self._opts.emotion_intensity,
            }

        # Output settings
        output: dict = {"audio_format": "wav"}
        if self._opts.speed != 1.0:
            output["audio_tempo"] = self._opts.speed
        if self._opts.pitch != 0:
            output["audio_pitch"] = self._opts.pitch
        if self._opts.volume != 100:
            output["volume"] = self._opts.volume
        if output:
            body["output"] = output

        return body

    async def synthesize(self, text: str) -> tts.ChunkedStream:
        """Synthesize text to audio frames via Typecast TTS API.

        Typecast v1 returns the full WAV binary synchronously.
        We split it into 100ms PCM16 frames for LiveKit consumption.
        """
        return TypecastChunkedStream(
            text=text,
            api_key=self._api_key,
            request_body=self._build_request_body(text),
            tts_instance=self,
        )


class TypecastChunkedStream(tts.ChunkedStream):
    """Async iterator that fetches audio from Typecast and yields frames."""

    def __init__(
        self,
        *,
        text: str,
        api_key: str,
        request_body: dict,
        tts_instance: TypecastTTS,
    ):
        super().__init__(tts=tts_instance, input_text=text)
        self._api_key = api_key
        self._request_body = request_body

    async def _run(self) -> None:
        """Fetch audio from Typecast API and push frames to the queue."""
        headers = {
            "X-API-KEY": self._api_key,
            "Content-Type": "application/json",
            "Accept": "audio/wav",
        }

        async with aiohttp.ClientSession() as session:
            async with session.post(
                TYPECAST_TTS_ENDPOINT,
                json=self._request_body,
                headers=headers,
                timeout=aiohttp.ClientTimeout(total=30),
            ) as resp:
                if resp.status != 200:
                    error_text = await resp.text()
                    logger.error(
                        "Typecast TTS API error (HTTP %s): %s",
                        resp.status,
                        error_text[:500],
                    )
                    raise RuntimeError(
                        f"Typecast TTS failed (HTTP {resp.status}): {error_text[:200]}"
                    )

                # Read the full WAV response
                audio_bytes = await resp.read()

                if len(audio_bytes) <= WAV_HEADER_SIZE:
                    raise RuntimeError(
                        f"Typecast returned too-short audio ({len(audio_bytes)} bytes)"
                    )

                # Skip WAV header (44 bytes) to get raw PCM16 data
                pcm_data = audio_bytes[WAV_HEADER_SIZE:]

                # Split into 100ms frames and push to LiveKit
                offset = 0
                while offset + BYTES_PER_FRAME <= len(pcm_data):
                    frame_bytes = pcm_data[offset : offset + BYTES_PER_FRAME]
                    frame = tts.SynthesizedAudio(
                        frame=tts.AudioFrame(
                            data=frame_bytes,
                            sample_rate=SAMPLE_RATE,
                            num_channels=NUM_CHANNELS,
                            samples_per_channel=SAMPLES_PER_FRAME,
                        ),
                        request_id="",
                    )
                    self._event_ch.send_nowait(frame)
                    offset += BYTES_PER_FRAME

                # Handle remaining bytes (last partial frame)
                remaining = pcm_data[offset:]
                if remaining:
                    # Pad to frame size with silence
                    padded = remaining + b"\x00" * (BYTES_PER_FRAME - len(remaining))
                    samples = len(remaining) // (BYTES_PER_SAMPLE * NUM_CHANNELS)
                    frame = tts.SynthesizedAudio(
                        frame=tts.AudioFrame(
                            data=padded,
                            sample_rate=SAMPLE_RATE,
                            num_channels=NUM_CHANNELS,
                            samples_per_channel=samples if samples > 0 else SAMPLES_PER_FRAME,
                        ),
                        request_id="",
                    )
                    self._event_ch.send_nowait(frame)

        logger.info(
            "Typecast TTS: synthesized %d bytes of audio for %d chars of text",
            len(audio_bytes),
            len(self._input_text),
        )


# ── Utility: list available Korean voices ──────────────────────────

async def list_korean_voices(api_key: str | None = None) -> list[dict]:
    """Fetch all Typecast voices and filter for Korean-capable ones.

    Usage:
        voices = await list_korean_voices()
        for v in voices:
            print(f"{v['voice_id']} — {v['voice_name']} ({v['gender']})")
    """
    key = api_key or os.environ.get("TYPECAST_API_KEY", "")
    if not key:
        raise ValueError("TYPECAST_API_KEY required")

    async with aiohttp.ClientSession() as session:
        async with session.get(
            TYPECAST_VOICES_ENDPOINT,
            headers={"X-API-KEY": key},
            params={"model": "ssfm-v30"},
        ) as resp:
            if resp.status != 200:
                raise RuntimeError(f"Typecast voices API error: HTTP {resp.status}")
            voices = await resp.json()

    # ssfm-v30 supports 37 languages including Korean;
    # all voices in the listing support the model's full language set.
    # Filter by use_cases or gender as needed.
    return voices if isinstance(voices, list) else []
