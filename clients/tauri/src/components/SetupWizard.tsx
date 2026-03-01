import { useState, useCallback } from "react";
import { type Locale, t } from "../lib/i18n";
import { apiClient } from "../lib/api";
import { isTauri, isGatewayRunning } from "../lib/tauri-bridge";

// ── i18n keys for setup wizard ──────────────────────────────────
// These are defined inline since they're only used here.
// They extend the main i18n system.
const wizardText: Record<Locale, Record<string, string>> = {
  en: {
    title: "Welcome to MoA",
    subtitle: "Let's set up your AI assistant",
    step_provider: "AI Provider",
    step_apikey: "API Key",
    step_complete: "Ready!",
    provider_label: "Choose your AI provider",
    provider_hint: "You can change this later in Settings",
    apikey_label: "Enter your API key",
    apikey_hint_with_key: "Your key stays on this device. Free to use with your own key.",
    apikey_hint_no_key: "No key? No problem! You can use operator credits instead.",
    apikey_placeholder: "sk-... or AIza...",
    skip_apikey: "Skip (use credits)",
    next: "Next",
    back: "Back",
    finish: "Start Chatting",
    checking_gateway: "Checking local AI engine...",
    gateway_ready: "ZeroClaw is running",
    gateway_not_ready: "ZeroClaw is starting...",
    setup_complete_title: "You're all set!",
    setup_complete_desc: "MoA is ready to assist you. You can always change these settings later.",
    provider_openrouter: "OpenRouter (Recommended)",
    provider_openrouter_desc: "Access 200+ models with one key",
    provider_anthropic: "Anthropic",
    provider_anthropic_desc: "Claude models (Opus, Sonnet, Haiku)",
    provider_openai: "OpenAI",
    provider_openai_desc: "GPT-4o, o1, and more",
    provider_google: "Google",
    provider_google_desc: "Gemini models",
    provider_ollama: "Ollama (Local)",
    provider_ollama_desc: "Free, runs on your machine",
  },
  ko: {
    title: "MoA에 오신 것을 환영합니다",
    subtitle: "AI 어시스턴트를 설정해 봅시다",
    step_provider: "AI 제공자",
    step_apikey: "API 키",
    step_complete: "완료!",
    provider_label: "AI 제공자를 선택하세요",
    provider_hint: "나중에 설정에서 변경할 수 있습니다",
    apikey_label: "API 키를 입력하세요",
    apikey_hint_with_key: "키는 이 기기에만 저장됩니다. 자신의 키로 무료 사용.",
    apikey_hint_no_key: "키가 없으신가요? 괜찮습니다! 크레딧으로 사용할 수 있습니다.",
    apikey_placeholder: "sk-... 또는 AIza...",
    skip_apikey: "건너뛰기 (크레딧 사용)",
    next: "다음",
    back: "이전",
    finish: "채팅 시작",
    checking_gateway: "로컬 AI 엔진 확인 중...",
    gateway_ready: "ZeroClaw 실행 중",
    gateway_not_ready: "ZeroClaw 시작 중...",
    setup_complete_title: "모든 준비가 완료되었습니다!",
    setup_complete_desc: "MoA가 도와드릴 준비가 되었습니다. 설정은 언제든 변경할 수 있습니다.",
    provider_openrouter: "OpenRouter (추천)",
    provider_openrouter_desc: "하나의 키로 200개 이상의 모델 사용",
    provider_anthropic: "Anthropic",
    provider_anthropic_desc: "Claude 모델 (Opus, Sonnet, Haiku)",
    provider_openai: "OpenAI",
    provider_openai_desc: "GPT-4o, o1 등",
    provider_google: "Google",
    provider_google_desc: "Gemini 모델",
    provider_ollama: "Ollama (로컬)",
    provider_ollama_desc: "무료, 내 컴퓨터에서 실행",
  },
};

function wt(key: string, locale: Locale): string {
  return wizardText[locale]?.[key] ?? wizardText.en[key] ?? key;
}

interface Provider {
  id: string;
  nameKey: string;
  descKey: string;
}

const PROVIDERS: Provider[] = [
  { id: "openrouter", nameKey: "provider_openrouter", descKey: "provider_openrouter_desc" },
  { id: "anthropic", nameKey: "provider_anthropic", descKey: "provider_anthropic_desc" },
  { id: "openai", nameKey: "provider_openai", descKey: "provider_openai_desc" },
  { id: "google", nameKey: "provider_google", descKey: "provider_google_desc" },
  { id: "ollama", nameKey: "provider_ollama", descKey: "provider_ollama_desc" },
];

interface SetupWizardProps {
  locale: Locale;
  onComplete: () => void;
}

type Step = "provider" | "apikey" | "complete";
const STEPS: Step[] = ["provider", "apikey", "complete"];

export function SetupWizard({ locale, onComplete }: SetupWizardProps) {
  const [step, setStep] = useState<Step>("provider");
  const [selectedProvider, setSelectedProvider] = useState<string>("openrouter");
  const [apiKey, setApiKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [gatewayStatus, setGatewayStatus] = useState<"checking" | "ready" | "starting">("checking");

  const stepIndex = STEPS.indexOf(step);

  // Check gateway status when reaching complete step
  const checkGateway = useCallback(async () => {
    setGatewayStatus("checking");
    if (isTauri()) {
      const running = await isGatewayRunning();
      setGatewayStatus(running ? "ready" : "starting");
    } else {
      // In browser mode, check via health endpoint
      try {
        await apiClient.healthCheck();
        setGatewayStatus("ready");
      } catch {
        setGatewayStatus("starting");
      }
    }
  }, []);

  const handleNext = useCallback(async () => {
    if (step === "provider") {
      if (selectedProvider === "ollama") {
        // Ollama doesn't need an API key, skip to complete
        setStep("complete");
        checkGateway();
        return;
      }
      setStep("apikey");
    } else if (step === "apikey") {
      setSaving(true);
      try {
        // Save provider preference
        localStorage.setItem("moa_setup_provider", selectedProvider);

        // Save API key if provided
        if (apiKey.trim()) {
          const storageKey = `moa_api_key_${selectedProvider}`;
          localStorage.setItem(storageKey, apiKey.trim());

          // Also sync to local ZeroClaw agent
          await apiClient.saveApiKeyToAgent(selectedProvider, apiKey.trim());
        }

        localStorage.setItem("moa_setup_complete", "true");
      } catch {
        // Non-critical, continue anyway
      }
      setSaving(false);
      setStep("complete");
      checkGateway();
    }
  }, [step, selectedProvider, apiKey, checkGateway]);

  const handleSkipApiKey = useCallback(async () => {
    localStorage.setItem("moa_setup_provider", selectedProvider);
    localStorage.setItem("moa_setup_complete", "true");
    setStep("complete");
    checkGateway();
  }, [selectedProvider, checkGateway]);

  const handleBack = useCallback(() => {
    if (step === "apikey") setStep("provider");
    else if (step === "complete") setStep("apikey");
  }, [step]);

  const handleFinish = useCallback(() => {
    localStorage.setItem("moa_setup_complete", "true");
    onComplete();
  }, [onComplete]);

  return (
    <div className="setup-wizard">
      <div className="setup-wizard-container">
        {/* Header */}
        <div className="setup-wizard-header">
          <div className="setup-wizard-logo">
            <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
              <rect width="48" height="48" rx="12" fill="url(#logo-gradient)" />
              <text x="24" y="32" textAnchor="middle" fill="white" fontSize="22" fontWeight="700">M</text>
              <defs>
                <linearGradient id="logo-gradient" x1="0" y1="0" x2="48" y2="48">
                  <stop stopColor="#6366f1" />
                  <stop offset="1" stopColor="#4f46e5" />
                </linearGradient>
              </defs>
            </svg>
          </div>
          <h1>{wt("title", locale)}</h1>
          <p>{wt("subtitle", locale)}</p>
        </div>

        {/* Step indicator */}
        <div className="setup-wizard-steps">
          {STEPS.map((s, i) => (
            <div
              key={s}
              className={`setup-step-dot ${i <= stepIndex ? "active" : ""} ${i === stepIndex ? "current" : ""}`}
            >
              <span className="setup-step-number">{i + 1}</span>
              <span className="setup-step-label">
                {wt(`step_${s}`, locale)}
              </span>
            </div>
          ))}
        </div>

        {/* Content */}
        <div className="setup-wizard-content">
          {step === "provider" && (
            <div className="setup-section">
              <h2>{wt("provider_label", locale)}</h2>
              <p className="setup-hint">{wt("provider_hint", locale)}</p>
              <div className="setup-provider-list">
                {PROVIDERS.map((p) => (
                  <button
                    key={p.id}
                    className={`setup-provider-option ${selectedProvider === p.id ? "selected" : ""}`}
                    onClick={() => setSelectedProvider(p.id)}
                  >
                    <div className="setup-provider-radio">
                      <div className={`radio-dot ${selectedProvider === p.id ? "checked" : ""}`} />
                    </div>
                    <div className="setup-provider-info">
                      <span className="setup-provider-name">{wt(p.nameKey, locale)}</span>
                      <span className="setup-provider-desc">{wt(p.descKey, locale)}</span>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          )}

          {step === "apikey" && (
            <div className="setup-section">
              <h2>{wt("apikey_label", locale)}</h2>
              <p className="setup-hint">
                {apiKey.trim() ? wt("apikey_hint_with_key", locale) : wt("apikey_hint_no_key", locale)}
              </p>
              <div className="setup-apikey-input">
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder={wt("apikey_placeholder", locale)}
                  className="setup-input"
                  autoFocus
                />
              </div>
              <button
                className="setup-skip-btn"
                onClick={handleSkipApiKey}
              >
                {wt("skip_apikey", locale)}
              </button>
            </div>
          )}

          {step === "complete" && (
            <div className="setup-section setup-complete">
              <div className="setup-complete-icon">
                <svg width="64" height="64" viewBox="0 0 64 64" fill="none">
                  <circle cx="32" cy="32" r="32" fill="rgba(34, 197, 94, 0.15)" />
                  <path d="M20 32L28 40L44 24" stroke="#22c55e" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
                </svg>
              </div>
              <h2>{wt("setup_complete_title", locale)}</h2>
              <p className="setup-hint">{wt("setup_complete_desc", locale)}</p>
              <div className="setup-gateway-status">
                <div className={`gateway-indicator ${gatewayStatus}`} />
                <span>
                  {gatewayStatus === "checking" && wt("checking_gateway", locale)}
                  {gatewayStatus === "ready" && wt("gateway_ready", locale)}
                  {gatewayStatus === "starting" && wt("gateway_not_ready", locale)}
                </span>
              </div>
            </div>
          )}
        </div>

        {/* Navigation */}
        <div className="setup-wizard-nav">
          {stepIndex > 0 && step !== "complete" && (
            <button className="setup-nav-btn setup-nav-back" onClick={handleBack}>
              {wt("back", locale)}
            </button>
          )}
          <div className="setup-nav-spacer" />
          {step === "provider" && (
            <button className="setup-nav-btn setup-nav-next" onClick={handleNext}>
              {wt("next", locale)}
            </button>
          )}
          {step === "apikey" && (
            <button
              className="setup-nav-btn setup-nav-next"
              onClick={handleNext}
              disabled={saving}
            >
              {saving ? "..." : wt("next", locale)}
            </button>
          )}
          {step === "complete" && (
            <button className="setup-nav-btn setup-nav-finish" onClick={handleFinish}>
              {wt("finish", locale)}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
