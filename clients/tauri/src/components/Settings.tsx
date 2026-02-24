import { useState, useCallback, useEffect } from "react";
import { t, type Locale } from "../lib/i18n";
import { apiClient, type SyncStatus, type PlatformInfo, type DeviceInfo } from "../lib/api";
import { isTauri } from "../lib/tauri-bridge";

interface SettingsProps {
  locale: Locale;
  onLocaleChange: (locale: Locale) => void;
  onBack: () => void;
  onLogout: () => void;
}

export function Settings({ locale, onLocaleChange, onBack, onLogout }: SettingsProps) {
  const [serverUrl, setServerUrl] = useState(apiClient.getServerUrl());
  const [isHealthChecking, setIsHealthChecking] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [isSyncing, setIsSyncing] = useState(false);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [editingPairingDevice, setEditingPairingDevice] = useState<string | null>(null);
  const [newPairingCode, setNewPairingCode] = useState("");
  const inTauri = isTauri();
  const user = apiClient.getUser();
  const isLoggedIn = apiClient.isLoggedIn();
  const currentDeviceId = apiClient.getDeviceId();

  useEffect(() => {
    if (inTauri) {
      apiClient.getSyncStatus().then(setSyncStatus).catch(() => {});
      apiClient.getPlatformInfo().then(setPlatformInfo).catch(() => {});
    }
    if (isLoggedIn) {
      apiClient.getDevices().then(setDevices).catch(() => {});
    }
  }, [inTauri, isLoggedIn]);

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

  const handleTriggerSync = useCallback(async () => {
    setIsSyncing(true);
    setMessage(null);
    try {
      const result = await apiClient.triggerFullSync();
      if (result) {
        setMessage({ type: "success", text: t("sync_triggered", locale) });
      }
      const status = await apiClient.getSyncStatus();
      setSyncStatus(status);
    } catch (err) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : t("sync_failed", locale),
      });
    } finally {
      setIsSyncing(false);
      clearMessage();
    }
  }, [locale, clearMessage]);

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

  const handleLogout = useCallback(async () => {
    await onLogout();
  }, [onLogout]);

  const handleSetPairingCode = useCallback(async (deviceId: string) => {
    if (!newPairingCode.trim()) return;
    try {
      await apiClient.setDevicePairingCode(deviceId, newPairingCode.trim());
      setMessage({ type: "success", text: t("pairing_code_set", locale) });
      setEditingPairingDevice(null);
      setNewPairingCode("");
      // Refresh devices
      const updated = await apiClient.getDevices();
      setDevices(updated);
    } catch (err) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : "Failed",
      });
    }
    clearMessage();
  }, [newPairingCode, locale, clearMessage]);

  const handleRemovePairingCode = useCallback(async (deviceId: string) => {
    try {
      await apiClient.setDevicePairingCode(deviceId, null);
      setMessage({ type: "success", text: t("pairing_code_removed", locale) });
      const updated = await apiClient.getDevices();
      setDevices(updated);
    } catch (err) {
      setMessage({
        type: "error",
        text: err instanceof Error ? err.message : "Failed",
      });
    }
    clearMessage();
  }, [locale, clearMessage]);

  const formatLastSeen = (timestamp: number) => {
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    if (diff < 120) return locale === "ko" ? "\uBC29\uAE08 \uC804" : "Just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}${locale === "ko" ? "\uBD84 \uC804" : "m ago"}`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}${locale === "ko" ? "\uC2DC\uAC04 \uC804" : "h ago"}`;
    return `${Math.floor(diff / 86400)}${locale === "ko" ? "\uC77C \uC804" : "d ago"}`;
  };

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

          {/* Account section */}
          {isLoggedIn && user && (
            <div className="settings-section">
              <div className="settings-section-title">{t("account_info", locale)}</div>
              <div className="settings-card">
                <div className="settings-field">
                  <label className="settings-label">{t("username", locale)}</label>
                  <div className="settings-token-display">{user.username}</div>
                </div>
                <div className="settings-actions" style={{ marginTop: 12 }}>
                  <button className="settings-btn settings-btn-danger" onClick={handleLogout}>
                    {t("logout", locale)}
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Devices section */}
          {isLoggedIn && devices.length > 0 && (
            <div className="settings-section">
              <div className="settings-section-title">{t("my_devices", locale)}</div>
              <div className="settings-card">
                {devices.map((device) => {
                  const isLocal = device.device_id === currentDeviceId;
                  return (
                    <div key={device.device_id} className="settings-device-item">
                      <div className="settings-device-header">
                        <div className="settings-device-name">
                          {device.device_name}
                          {isLocal && (
                            <span className="device-badge device-badge-local">
                              {t("device_this", locale)}
                            </span>
                          )}
                        </div>
                        <div className={`device-status-mini ${device.is_online ? "online" : "offline"}`}>
                          <div className={`status-dot ${device.is_online ? "connected" : ""}`} />
                          <span>{device.is_online ? t("device_online", locale) : t("device_offline", locale)}</span>
                        </div>
                      </div>
                      <div className="settings-device-meta">
                        {device.platform && <span>{device.platform}</span>}
                        <span>{formatLastSeen(device.last_seen)}</span>
                      </div>

                      {/* Pairing code management */}
                      <div className="settings-device-pairing">
                        {editingPairingDevice === device.device_id ? (
                          <div className="settings-device-pairing-edit">
                            <input
                              className="settings-input"
                              type="password"
                              value={newPairingCode}
                              onChange={(e) => setNewPairingCode(e.target.value)}
                              onKeyDown={(e) => { if (e.key === "Enter") handleSetPairingCode(device.device_id); }}
                              placeholder={t("new_pairing_code", locale)}
                              autoFocus
                            />
                            <div className="settings-device-pairing-btns">
                              <button
                                className="settings-btn settings-btn-primary settings-btn-sm"
                                onClick={() => handleSetPairingCode(device.device_id)}
                                disabled={!newPairingCode.trim()}
                              >
                                {t("save_pairing_code", locale)}
                              </button>
                              <button
                                className="settings-btn settings-btn-secondary settings-btn-sm"
                                onClick={() => { setEditingPairingDevice(null); setNewPairingCode(""); }}
                              >
                                {locale === "ko" ? "\uCDE8\uC18C" : "Cancel"}
                              </button>
                            </div>
                          </div>
                        ) : (
                          <div className="settings-device-pairing-btns">
                            <button
                              className="settings-btn settings-btn-secondary settings-btn-sm"
                              onClick={() => { setEditingPairingDevice(device.device_id); setNewPairingCode(""); }}
                            >
                              {device.has_pairing_code ? t("change_pairing_code", locale) : t("set_pairing_code", locale)}
                            </button>
                            {device.has_pairing_code && (
                              <button
                                className="settings-btn settings-btn-danger settings-btn-sm"
                                onClick={() => handleRemovePairingCode(device.device_id)}
                              >
                                {t("remove_pairing_code", locale)}
                              </button>
                            )}
                          </div>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {message && (
            <div className={`settings-message ${message.type}`}>{message.text}</div>
          )}

          {/* Advanced settings (server URL) */}
          <div className="settings-section">
            <details>
              <summary className="settings-section-title" style={{ cursor: "pointer" }}>
                {t("advanced_settings", locale)}
              </summary>
              <div className="settings-card" style={{ marginTop: 8 }}>
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
              </div>
            </details>
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

          {/* Sync section (Tauri only) */}
          {inTauri && (
            <div className="settings-section">
              <div className="settings-section-title">{t("sync_status", locale)}</div>
              <div className="settings-card">
                {syncStatus ? (
                  <>
                    <div className={`settings-status ${syncStatus.connected ? "connected" : "disconnected"}`}>
                      <div className={`status-dot ${syncStatus.connected ? "connected" : ""}`} />
                      {syncStatus.connected ? t("sync_connected", locale) : t("sync_disconnected", locale)}
                    </div>
                    <div className="settings-field" style={{ marginTop: 12 }}>
                      <label className="settings-label">{t("sync_device_id", locale)}</label>
                      <div className="settings-token-display" style={{ fontSize: 11 }}>
                        {syncStatus.device_id}
                      </div>
                    </div>
                    {isLoggedIn && (
                      <div className="settings-actions" style={{ marginTop: 12 }}>
                        <button
                          className="settings-btn settings-btn-secondary"
                          onClick={handleTriggerSync}
                          disabled={isSyncing}
                        >
                          {isSyncing ? t("sync_triggering", locale) : t("sync_trigger", locale)}
                        </button>
                      </div>
                    )}
                  </>
                ) : (
                  <div className="settings-status disconnected">
                    <div className="status-dot" />
                    {t("sync_disconnected", locale)}
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Platform info (Tauri only) */}
          {inTauri && platformInfo && (
            <div className="settings-section">
              <div className="settings-section-title">{t("platform", locale)}</div>
              <div className="settings-card">
                <p style={{ fontSize: 13, color: "var(--color-text-secondary)" }}>
                  {platformInfo.os} / {platformInfo.arch}
                  {platformInfo.is_mobile ? " (Mobile)" : " (Desktop)"}
                </p>
              </div>
            </div>
          )}

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
