import { useEffect, useRef, useState } from "react";
import { t, type Locale } from "../lib/i18n";
import { apiClient } from "../lib/api";

/**
 * First-launch onboarding screen for the Gemma 4 base gun.
 *
 * Polls `GET /api/local-llm/bootstrap-status` on the MoA gateway every 750 ms
 * and renders the current install / download stage. Calls `onReady()` when
 * the backend reports the local model is usable (stages `done`, `skipped`)
 * OR when the user explicitly chooses to skip (error + BYOK fallback path).
 *
 * Design notes:
 * - Poll interval is intentionally slow (750 ms) — the user's attention is on
 *   a multi-minute download and a tighter loop would waste battery without
 *   helping UX.
 * - We NEVER block the user from advancing to chat: the "나중에 설정" button
 *   fires `onReady()` with `ready=false` so the outer router can decide
 *   whether to show a cloud-fallback path.
 * - Stage labels come from i18n so the Korean spec text lands verbatim.
 */
interface BootstrapStatus {
  stage:
    | "not_started"
    | "skipped"
    | "probing"
    | "checking_disk"
    | "installing_ollama"
    | "waiting_for_daemon"
    | "pulling_model"
    | "persisting"
    | "done"
    | "error";
  attempt?: number;
  fraction?: number;
  pull_status?: string;
  model?: string;
  message?: string;
}

interface Props {
  locale: Locale;
  onReady: (ready: boolean) => void;
}

const POLL_INTERVAL_MS = 750;

export function LocalLlmBootstrap({ locale, onReady }: Props) {
  const [status, setStatus] = useState<BootstrapStatus>({ stage: "not_started" });
  const [failedPolls, setFailedPolls] = useState(0);
  const timer = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    let cancelled = false;

    const poll = async () => {
      try {
        const serverUrl = apiClient.getServerUrl();
        const res = await fetch(`${serverUrl}/api/local-llm/bootstrap-status`, {
          method: "GET",
          headers: { "Content-Type": "application/json" },
        });
        if (!res.ok) {
          // Treat HTTP errors as transient — the gateway may still be
          // spinning up. Only give up after N consecutive failures.
          setFailedPolls((n) => n + 1);
          return;
        }
        const data = (await res.json()) as BootstrapStatus;
        if (cancelled) return;
        setFailedPolls(0);
        setStatus(data);
        if (data.stage === "done" || data.stage === "skipped") {
          onReady(true);
        }
      } catch {
        setFailedPolls((n) => n + 1);
      }
    };

    void poll();
    timer.current = setInterval(poll, POLL_INTERVAL_MS);
    return () => {
      cancelled = true;
      if (timer.current) clearInterval(timer.current);
    };
  }, [onReady]);

  const fractionPct =
    status.stage === "pulling_model" && typeof status.fraction === "number"
      ? Math.round(Math.max(0, Math.min(1, status.fraction)) * 100)
      : null;

  const stageLabel = t(
    ("local_llm_bootstrap_stage_" + status.stage) as any,
    locale,
  );

  const isTerminal = status.stage === "done" || status.stage === "skipped";
  const isError = status.stage === "error";

  return (
    <div className="auth-container">
      <div className="auth-card" style={{ textAlign: "center" }}>
        <div className="auth-logo">
          <div className="auth-logo-icon">M</div>
          <h1 className="auth-title">
            {t("local_llm_bootstrap_title", locale)}
          </h1>
          <p className="auth-subtitle">
            {t("local_llm_bootstrap_subtitle", locale)}
          </p>
        </div>

        <div
          style={{
            marginTop: 24,
            padding: 16,
            borderRadius: 12,
            background: "rgba(120, 99, 255, 0.08)",
            border: "1px solid rgba(120, 99, 255, 0.24)",
          }}
        >
          <div style={{ fontSize: 14, marginBottom: 12, opacity: 0.85 }}>
            {stageLabel}
          </div>
          {fractionPct !== null && (
            <>
              <div
                style={{
                  position: "relative",
                  height: 10,
                  borderRadius: 6,
                  background: "rgba(255,255,255,0.08)",
                  overflow: "hidden",
                }}
              >
                <div
                  style={{
                    position: "absolute",
                    inset: 0,
                    width: `${fractionPct}%`,
                    background:
                      "linear-gradient(90deg, #7863ff 0%, #a68bff 100%)",
                    transition: "width 400ms ease",
                  }}
                />
              </div>
              <div style={{ fontSize: 12, marginTop: 6, opacity: 0.7 }}>
                {fractionPct}%{status.attempt && status.attempt > 1
                  ? ` · ${t("local_llm_bootstrap_retry", locale).replace(
                      "{n}",
                      String(status.attempt),
                    )}`
                  : ""}
              </div>
            </>
          )}
          {isError && (
            <div style={{ fontSize: 12, marginTop: 8, color: "#ff8a8a" }}>
              {status.message || t("local_llm_bootstrap_stage_error", locale)}
            </div>
          )}
          {failedPolls > 5 && !isTerminal && (
            <div style={{ fontSize: 12, marginTop: 8, opacity: 0.6 }}>
              {t("local_llm_bootstrap_poll_retry", locale)}
            </div>
          )}
        </div>

        <div style={{ marginTop: 24 }}>
          <button
            className="auth-link-btn"
            onClick={() => onReady(false)}
            disabled={isTerminal}
          >
            {t("local_llm_bootstrap_skip", locale)}
          </button>
        </div>
      </div>
    </div>
  );
}
