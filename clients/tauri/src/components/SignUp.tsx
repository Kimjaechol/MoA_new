import { useState, useCallback, type KeyboardEvent } from "react";
import { t, type Locale } from "../lib/i18n";
import { apiClient } from "../lib/api";

interface SignUpProps {
  locale: Locale;
  onSignUpSuccess: () => void;
  onGoToLogin: () => void;
}

export function SignUp({ locale, onSignUpSuccess, onGoToLogin }: SignUpProps) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [passwordConfirm, setPasswordConfirm] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSignUp = useCallback(async () => {
    if (!username.trim() || !password || !passwordConfirm) return;

    if (password !== passwordConfirm) {
      setError(locale === "ko" ? "\uBE44\uBC00\uBC88\uD638\uAC00 \uC77C\uCE58\uD558\uC9C0 \uC54A\uC2B5\uB2C8\uB2E4." : "Passwords do not match.");
      return;
    }

    if (password.length < 8) {
      setError(locale === "ko" ? "\uBE44\uBC00\uBC88\uD638\uB294 8\uC790 \uC774\uC0C1\uC774\uC5B4\uC57C \uD569\uB2C8\uB2E4." : "Password must be at least 8 characters.");
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      await apiClient.register(username.trim(), password);
      onSignUpSuccess();
    } catch (err) {
      setError(err instanceof Error ? err.message : t("signup_failed", locale));
    } finally {
      setIsLoading(false);
    }
  }, [username, password, passwordConfirm, locale, onSignUpSuccess]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Enter") handleSignUp();
    },
    [handleSignUp],
  );

  return (
    <div className="auth-container">
      <div className="auth-card">
        <div className="auth-logo">
          <div className="auth-logo-icon">MoA</div>
          <h1 className="auth-title">{t("signup_title", locale)}</h1>
          <p className="auth-subtitle">{t("signup_subtitle", locale)}</p>
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
            placeholder={locale === "ko" ? "8\uC790 \uC774\uC0C1" : "Min 8 characters"}
            autoComplete="new-password"
            disabled={isLoading}
          />
        </div>

        <div className="auth-field">
          <label className="auth-label">{t("password_confirm", locale)}</label>
          <input
            className="auth-input"
            type="password"
            value={passwordConfirm}
            onChange={(e) => setPasswordConfirm(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={t("password_confirm", locale)}
            autoComplete="new-password"
            disabled={isLoading}
          />
        </div>

        <button
          className="auth-btn auth-btn-primary"
          onClick={handleSignUp}
          disabled={isLoading || !username.trim() || !password || !passwordConfirm}
        >
          {isLoading ? t("signing_up", locale) : t("signup_button", locale)}
        </button>

        <div className="auth-link">
          {t("have_account", locale)}{" "}
          <button className="auth-link-btn" onClick={onGoToLogin} disabled={isLoading}>
            {t("login", locale)}
          </button>
        </div>
      </div>
    </div>
  );
}
