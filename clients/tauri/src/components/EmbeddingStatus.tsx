import { useState, useEffect, useRef } from "react";
import {
  checkEmbeddingModel,
  monitorEmbeddingDownload,
  type EmbeddingModelStatus,
} from "../lib/tauri-bridge";
import type { Locale } from "../lib/i18n";

interface EmbeddingStatusProps {
  locale: Locale;
  /** Poll interval while a download is in flight. Defaults to 2000ms. */
  pollIntervalMs?: number;
}

function formatBytes(n: number): string {
  if (n <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(units.length - 1, Math.floor(Math.log(n) / Math.log(1024)));
  return `${(n / Math.pow(1024, i)).toFixed(i >= 2 ? 1 : 0)} ${units[i]}`;
}

function formatEta(remainingBytes: number, bytesPerSec: number, isKo: boolean): string {
  if (bytesPerSec <= 0 || remainingBytes <= 0) return isKo ? "계산 중..." : "calculating...";
  const secs = remainingBytes / bytesPerSec;
  if (secs < 60) return isKo ? `${Math.round(secs)}초 남음` : `${Math.round(secs)}s left`;
  const mins = secs / 60;
  if (mins < 60) return isKo ? `약 ${Math.round(mins)}분 남음` : `~${Math.round(mins)}m left`;
  const hrs = mins / 60;
  return isKo ? `약 ${hrs.toFixed(1)}시간 남음` : `~${hrs.toFixed(1)}h left`;
}

/**
 * PR #1 — BGE-M3 embedding model status card.
 *
 * Renders three states:
 * - "not downloaded" + guidance for proactive trigger
 * - "downloading" + progress bar + ETA (dir-size polling based)
 * - "ready" + total size
 *
 * The actual download is driven by fastembed when the agent/eval binary
 * first calls `TextEmbedding::try_new()`. This component is observability
 * only — it doesn't drive the download, but makes the 1.1 GB pull visible
 * to the user so they don't think the app hung.
 */
export function EmbeddingStatus({ locale, pollIntervalMs = 2000 }: EmbeddingStatusProps) {
  const [status, setStatus] = useState<EmbeddingModelStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  // Rolling window of (timestamp_ms, bytes) samples for ETA calculation.
  const samplesRef = useRef<Array<{ ts: number; bytes: number }>>([]);

  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const tick = async () => {
      try {
        const s = await (samplesRef.current.length === 0
          ? checkEmbeddingModel()
          : monitorEmbeddingDownload());
        if (cancelled) return;
        if (s) {
          setStatus(s);
          setError(null);
          // Keep up to 30s of samples for a rolling speed estimate.
          const now = Date.now();
          samplesRef.current.push({ ts: now, bytes: s.size_bytes });
          const cutoff = now - 30_000;
          samplesRef.current = samplesRef.current.filter((x) => x.ts >= cutoff);
        }
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
      // Continue polling while download is in flight.
      if (!cancelled) {
        const installed = status?.installed ?? false;
        const interval = installed ? pollIntervalMs * 5 : pollIntervalMs;
        timer = setTimeout(tick, interval);
      }
    };
    tick();
    return () => {
      cancelled = true;
      if (timer) clearTimeout(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pollIntervalMs]);

  const isKo = locale === "ko";

  if (error) {
    return (
      <div style={{ padding: 16, border: "1px solid #a44", borderRadius: 8, color: "#f88" }}>
        {isKo ? "임베딩 모델 상태 확인 실패" : "Failed to check embedding model"}: {error}
      </div>
    );
  }
  if (!status) {
    return (
      <div style={{ padding: 16, color: "var(--text-secondary, #888)" }}>
        {isKo ? "불러오는 중..." : "Loading..."}
      </div>
    );
  }

  // Derive speed and ETA from the rolling window.
  const samples = samplesRef.current;
  let speedBps = 0;
  if (samples.length >= 2) {
    const first = samples[0];
    const last = samples[samples.length - 1];
    const dt = (last.ts - first.ts) / 1000;
    if (dt > 0) speedBps = Math.max(0, (last.bytes - first.bytes) / dt);
  }
  const remaining = Math.max(0, status.target_bytes - status.size_bytes);
  const downloading = status.model_present && !status.installed;

  return (
    <div
      style={{
        padding: 16,
        border: "1px solid var(--border-color, #333)",
        borderRadius: 8,
        background: "var(--bg-secondary, #1a1a1a)",
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h3 style={{ margin: 0, fontSize: 15 }}>
          {isKo ? "온디바이스 임베딩 모델 (BGE-M3)" : "On-device Embedding Model (BGE-M3)"}
        </h3>
        <span
          style={{
            fontSize: 12,
            padding: "2px 8px",
            borderRadius: 4,
            background: status.installed
              ? "var(--success-bg, #1f3a1f)"
              : downloading
                ? "var(--warning-bg, #3a2f1f)"
                : "var(--bg-tertiary, #252525)",
            color: status.installed
              ? "#8f8"
              : downloading
                ? "#fc8"
                : "var(--text-secondary, #888)",
          }}
        >
          {status.installed
            ? isKo
              ? "준비 완료"
              : "Ready"
            : downloading
              ? isKo
                ? "다운로드 중"
                : "Downloading"
              : isKo
                ? "미설치"
                : "Not installed"}
        </span>
      </div>

      <div style={{ fontSize: 12, color: "var(--text-tertiary, #666)", marginTop: 4 }}>
        {status.cache_dir || (isKo ? "캐시 경로 없음" : "(no cache dir)")}
      </div>

      {(downloading || status.installed) && (
        <div style={{ marginTop: 12 }}>
          <div
            style={{
              height: 8,
              background: "var(--bg-tertiary, #252525)",
              borderRadius: 4,
              overflow: "hidden",
            }}
          >
            <div
              style={{
                width: `${Math.min(100, status.progress * 100).toFixed(1)}%`,
                height: "100%",
                background: status.installed ? "#4a4" : "#ca4",
                transition: "width 400ms ease",
              }}
            />
          </div>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              fontSize: 12,
              marginTop: 6,
              color: "var(--text-secondary, #999)",
            }}
          >
            <span>
              {formatBytes(status.size_bytes)} / {formatBytes(status.target_bytes)}
              {" ("}
              {(status.progress * 100).toFixed(1)}
              {"%)"}
            </span>
            {downloading && (
              <span>
                {speedBps > 0
                  ? `${formatBytes(speedBps)}/s · ${formatEta(remaining, speedBps, isKo)}`
                  : isKo
                    ? "속도 측정 중..."
                    : "measuring..."}
              </span>
            )}
          </div>
        </div>
      )}

      {!status.model_present && (
        <p style={{ fontSize: 13, color: "var(--text-secondary, #888)", marginTop: 12, marginBottom: 0 }}>
          {isKo
            ? "에이전트를 처음 실행하거나 저장된 메모리를 검색하면 fastembed가 ~1.1GB의 BGE-M3 가중치를 자동으로 다운로드합니다. 이 카드는 그 과정을 실시간으로 보여줍니다."
            : "fastembed will download ~1.1 GB of BGE-M3 weights automatically the first time the agent runs or a memory is recalled. This card shows that progress in real time."}
        </p>
      )}
    </div>
  );
}
