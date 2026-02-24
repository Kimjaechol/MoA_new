import { useState, useCallback, type KeyboardEvent } from "react";
import { t, type Locale } from "../lib/i18n";
import { apiClient } from "../lib/api";

interface LoginProps {
  locale: Locale;
  onLoginSuccess: (devices: Array<{ device_id: string; device_name: string; platform: string | null; last_seen: number; is_online: boolean; has_pairing_code: boolean }>) => void;
  onGoToSignUp: () => void;
  onGoToSettings: () => void;
}

export function Login({ locale, onLoginSuccess, onGoToSignUp, onGoToSettings }: LoginProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleLogin = useCallback(async () => {
    if (!username.trim() || !password) return;

    setIsLoading(true);
    setError(null);

    try {
      const result = await apiClient.login(username.trim(), password);
      onLoginSuccess(result.devices || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : t("login_failed", locale));
    } finally {
      setIsLoading(false);
    }
  }, [username, password, locale, onLoginSuccess]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Enter") handleLogin();
    },
    [handleLogin],
  );

  return (
    <div className="auth-container">
      <div className="auth-card">
        <div className="auth-logo">
          <div className="auth-logo-icon">MoA</div>
          <h1 className="auth-title">{t("login_title", locale)}</h1>
          <p className="auth-subtitle">{t("login_subtitle", locale)}</p>
        </div>

        {error && <div className="auth-error">{error}</div>}

        <div className="auth-field">
          <label className="auth-label">{t("username", locale)}</label>
          <input
            className="auth-input"
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("username", locale)}
            autoComplete="username"
            autoFocus
            disabled={isLoading}
          />
        </div>

        <div className="auth-field">
          <label className="auth-label">{t("password", locale)}</label>
          <input
            className="auth-input"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("password", locale)}
            autoComplete="current-password"
            disabled={isLoading}
          />
        </div>

        <button
          className="auth-btn auth-btn-primary"
          onClick={handleLogin}
          disabled={isLoading || !username.trim() || !password}
        >
          {isLoading ? t("logging_in", locale) : t("login_button", locale)}
        </button>

        <div className="auth-link">
          {t("no_account", locale)}{" "}
          <button className="auth-link-btn" onClick={onGoToSignUp} disabled={isLoading}>
            {t("signup", locale)}
          </button>
        </div>

        <div className="auth-settings-link">
          <button className="auth-link-btn" onClick={onGoToSettings} disabled={isLoading}>
            {t("advanced_settings", locale)}
          </button>
        </div>
      </div>
    </div>
  );
}
