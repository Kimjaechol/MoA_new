import { useState, useCallback, useEffect } from "react";
import { t, type Locale } from "../lib/i18n";
import { apiClient } from "../lib/api";

interface SettingsProps {
  locale: Locale;
  onLocaleChange: (locale: Locale) => void;
  onConnectionChange: (connected: boolean) => void;
  onBack: () => void;
}

export function Settings({ locale, onLocaleChange, onConnectionChange, onBack }: SettingsProps) {
  const [serverUrl, setServerUrl] = useState(apiClient.getServerUrl());
  const [pairingCode, setPairingCode] = useState("");
  const [pairUsername, setPairUsername] = useState("");
  const [pairPassword, setPairPassword] = useState("");
  const [isPairing, setIsPairing] = useState(false);
  const [isHealthChecking, setIsHealthChecking] = useState(false);
  const [isConnected, setIsConnected] = useState(apiClient.isConnected());
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    setIsConnected(apiClient.isConnected());
  }, []);

  const clearMessage = useCallback(() => {
    setTimeout(() => setMessage(null), 5000);
  }, []);

  const handleServerUrlChange = useCallback(
    (url: string) => {
      setServerUrl(url);
      apiClient.setServerUrl(url);
    },
    [],
  );

  const handlePair = useCallback(async () => {
    if (!pairingCode.trim()) return;

    setIsPairing(true);
    setMessage(null);

    try {
      const result = await apiClient.pair(
        pairingCode.trim(),
        pairUsername.trim() || undefined,
        pairPassword || undefined,
      );
      if (result.paired) {
        setIsConnected(true);
        onConnectionChange(true);
        setMessage({ type: "success", text: t("pair_success", locale) });
        setPairingCode("");
        setPairUsername("");
        setPairPassword("");
      } else {
        setMessage({ type: "error", text: t("pair_failed", locale) });
      }
    } catch (err) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : t("pair_failed", locale),
      });
    } finally {
      setIsPairing(false);
      clearMessage();
    }
  }, [pairingCode, pairUsername, pairPassword, locale, onConnectionChange, clearMessage]);

  const handleDisconnect = useCallback(() => {
    apiClient.disconnect();
    setIsConnected(false);
    onConnectionChange(false);
    setMessage(null);
  }, [onConnectionChange]);

  const handleHealthCheck = useCallback(async () => {
    setIsHealthChecking(true);
    setMessage(null);

    try {
      const result = await apiClient.healthCheck();
      if (result.status === "ok") {
        setMessage({ type: "success", text: t("server_healthy", locale) });
      } else {
        setMessage({ type: "error", text: t("server_unreachable", locale) });
      }
    } catch (err) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : t("server_unreachable", locale),
      });
    } finally {
      setIsHealthChecking(false);
      clearMessage();
    }
  }, [locale, clearMessage]);

  return (
    <div className="settings-container">
      {/* Header */}
      <div className="settings-header">
        <button className="settings-back-btn" onClick={onBack} aria-label={t("back_to_chat", locale)}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="15 18 9 12 15 6" />
          </svg>
        </button>
        <span className="settings-header-title">{t("settings", locale)}</span>
      </div>

      {/* Body */}
      <div className="settings-body">
        <div className="settings-inner">

          {/* Connection section */}
          <div className="settings-section">
            <div className="settings-section-title">{t("connection_status", locale)}</div>
            <div className="settings-card">
              <div className={`settings-status ${isConnected ? "connected" : "disconnected"}`}>
                <div className={`status-dot ${isConnected ? "connected" : ""}`} />
                {isConnected ? t("connected", locale) : t("disconnected", locale)}
              </div>

              {isConnected && (
                <>
                  <div className="settings-field" style={{ marginTop: 16 }}>
                    <label className="settings-label">{t("token", locale)}</label>
                    <div className="settings-token-display">{apiClient.getMaskedToken()}</div>
                  </div>
                  <div className="settings-actions">
                    <button className="settings-btn settings-btn-danger" onClick={handleDisconnect}>
                      {t("disconnect", locale)}
                    </button>
                  </div>
                </>
              )}
            </div>
          </div>

          {/* Server section */}
          <div className="settings-section">
            <div className="settings-section-title">{t("server_url", locale)}</div>
            <div className="settings-card">
              <div className="settings-field">
                <label className="settings-label">{t("server_url", locale)}</label>
                <div className="settings-input-row">
                  <input
                    className="settings-input"
                    type="url"
                    value={serverUrl}
                    onChange={(e) => handleServerUrlChange(e.target.value)}
                    placeholder="https://moanew-production.up.railway.app"
                  />
                  <button
                    className="settings-btn settings-btn-secondary"
                    onClick={handleHealthCheck}
                    disabled={isHealthChecking}
                  >
                    {isHealthChecking ? t("checking", locale) : t("health_check", locale)}
                  </button>
                </div>
              </div>

              {!isConnected && (
                <>
                  <div className="settings-field">
                    <label className="settings-label">{t("username", locale)}</label>
                    <input
                      className="settings-input"
                      type="text"
                      value={pairUsername}
                      onChange={(e) => setPairUsername(e.target.value)}
                      placeholder="Enter username"
                      autoComplete="username"
                    />
                  </div>
                  <div className="settings-field">
                    <label className="settings-label">{t("password", locale)}</label>
                    <input
                      className="settings-input"
                      type="password"
                      value={pairPassword}
                      onChange={(e) => setPairPassword(e.target.value)}
                      placeholder="Enter password"
                      autoComplete="current-password"
                    />
                  </div>
                  <div className="settings-field">
                    <label className="settings-label">{t("pairing_code", locale)}</label>
                    <div className="settings-input-row">
                      <input
                        className="settings-input"
                        type="text"
                        value={pairingCode}
                        onChange={(e) => setPairingCode(e.target.value)}
                        placeholder="Enter pairing code"
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handlePair();
                        }}
                      />
                      <button
                        className="settings-btn settings-btn-primary"
                        onClick={handlePair}
                        disabled={isPairing || !pairingCode.trim()}
                      >
                        {isPairing ? t("pairing", locale) : t("pair", locale)}
                      </button>
                    </div>
                  </div>
                </>
              )}

              {message && (
                <div className={`settings-message ${message.type}`}>{message.text}</div>
              )}
            </div>
          </div>

          {/* Language section */}
          <div className="settings-section">
            <div className="settings-section-title">{t("language", locale)}</div>
            <div className="settings-card">
              <div className="settings-lang-selector">
                <button
                  className={`settings-lang-btn ${locale === "en" ? "active" : ""}`}
                  onClick={() => onLocaleChange("en")}
                >
                  English
                </button>
                <button
                  className={`settings-lang-btn ${locale === "ko" ? "active" : ""}`}
                  onClick={() => onLocaleChange("ko")}
                >
                  {"\uD55C\uAD6D\uC5B4"}
                </button>
              </div>
            </div>
          </div>

          {/* About */}
          <div className="settings-section">
            <div className="settings-section-title">About</div>
            <div className="settings-card">
              <p style={{ fontSize: 13, color: "var(--color-text-secondary)", marginBottom: 4 }}>
                <strong>MoA - Master of AI</strong>
              </p>
              <p style={{ fontSize: 12, color: "var(--color-text-muted)" }}>
                Powered by ZeroClaw Agent Runtime
              </p>
              <p style={{ fontSize: 12, color: "var(--color-text-muted)", marginTop: 4 }}>
                Version 0.1.0
              </p>
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
