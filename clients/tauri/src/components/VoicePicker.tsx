import { useState, useEffect, useCallback, useMemo } from "react";
import { gatewayFetch } from "../lib/tauri-bridge";

// ── Types ──

interface Voice {
  voice_id: string;
  voice_name: string;
  gender: string;
  age: string;
  use_cases: string[];
  models: Array<{ version: string; emotions: string[] }>;
}

interface CategoryLabel {
  en: string;
  ko: string;
  ja?: string;
  zh?: string;
  native?: string;
}

interface VoicesResponse {
  voices: Voice[];
  categories: {
    gender: Record<string, CategoryLabel>;
    age: Record<string, CategoryLabel>;
    use_cases: Record<string, CategoryLabel>;
    mood: Record<string, CategoryLabel>;
    language: Record<string, CategoryLabel & { native: string }>;
  };
  mood_to_use_cases: Record<string, string[]>;
  smart_emotion: boolean;
  emotions: string[];
  languages_count: number;
}

interface VoicePickerProps {
  locale: string;
  onSelect: (voiceId: string, voiceName: string, gender: string) => void;
  selectedVoiceId?: string;
  onClose: () => void;
}

// ── Helpers ──

const label = (cat: CategoryLabel | undefined, locale: string): string => {
  if (!cat) return "";
  const key = locale.startsWith("ko") ? "ko" : locale.startsWith("ja") ? "ja" : locale.startsWith("zh") ? "zh" : "en";
  return (cat as any)[key] || cat.en || "";
};

const VOICE_STORAGE_KEY = "moa_typecast_voice";

export function saveSelectedVoice(voiceId: string, voiceName: string, gender: string) {
  localStorage.setItem(VOICE_STORAGE_KEY, JSON.stringify({ voiceId, voiceName, gender }));
}

export function loadSelectedVoice(): { voiceId: string; voiceName: string; gender: string } | null {
  try {
    const s = localStorage.getItem(VOICE_STORAGE_KEY);
    return s ? JSON.parse(s) : null;
  } catch { return null; }
}

// ── Component ──

export default function VoicePicker({ locale, onSelect, selectedVoiceId, onClose }: VoicePickerProps) {
  const [data, setData] = useState<VoicesResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Filters
  const [gender, setGender] = useState<string>("all");
  const [age, setAge] = useState<string>("all");
  const [mood, setMood] = useState<string>("all");
  const [searchText, setSearchText] = useState("");

  // Fetch voices on mount
  useEffect(() => {
    (async () => {
      try {
        const resp = await gatewayFetch("/api/voices/list", "GET", {});
        if (resp && resp.status === 200) {
          const json = JSON.parse(resp.body);
          setData(json);
        } else {
          setError(locale.startsWith("ko") ? "음성 목록을 불러올 수 없습니다." : "Failed to load voices.");
        }
      } catch (e: any) {
        setError(e.message || "Network error");
      } finally {
        setLoading(false);
      }
    })();
  }, [locale]);

  // Filter voices
  const filteredVoices = useMemo(() => {
    if (!data) return [];
    let voices = data.voices;

    if (gender !== "all") voices = voices.filter(v => v.gender === gender);
    if (age !== "all") voices = voices.filter(v => v.age === age);
    if (mood !== "all" && data.mood_to_use_cases[mood]) {
      const useCases = data.mood_to_use_cases[mood];
      voices = voices.filter(v => v.use_cases.some(uc => useCases.includes(uc)));
    }
    if (searchText.trim()) {
      const q = searchText.toLowerCase();
      voices = voices.filter(v => v.voice_name.toLowerCase().includes(q));
    }

    return voices;
  }, [data, gender, age, mood, searchText]);

  const handleSelect = useCallback((voice: Voice) => {
    saveSelectedVoice(voice.voice_id, voice.voice_name, voice.gender);
    onSelect(voice.voice_id, voice.voice_name, voice.gender);
  }, [onSelect]);

  if (loading) {
    return (
      <div className="voice-picker-overlay">
        <div className="voice-picker-modal">
          <div className="voice-picker-loading">
            {locale.startsWith("ko") ? "음성 목록 로딩 중..." : "Loading voices..."}
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="voice-picker-overlay">
        <div className="voice-picker-modal">
          <p className="voice-picker-error">{error}</p>
          <button onClick={onClose} className="voice-picker-close-btn">
            {locale.startsWith("ko") ? "닫기" : "Close"}
          </button>
        </div>
      </div>
    );
  }

  const cats = data!.categories;
  const isKo = locale.startsWith("ko");

  return (
    <div className="voice-picker-overlay" onClick={onClose}>
      <div className="voice-picker-modal" onClick={e => e.stopPropagation()}>
        {/* Header */}
        <div className="voice-picker-header">
          <h2>{isKo ? "🎙️ AI 음성 선택" : "🎙️ Choose AI Voice"}</h2>
          <span className="voice-picker-subtitle">
            {isKo
              ? `${filteredVoices.length}개 음성 · Smart Emotion · ${data!.languages_count}개 언어`
              : `${filteredVoices.length} voices · Smart Emotion · ${data!.languages_count} languages`}
          </span>
          <button onClick={onClose} className="voice-picker-x" aria-label="Close">✕</button>
        </div>

        {/* Filters */}
        <div className="voice-picker-filters">
          {/* Search */}
          <input
            type="text"
            className="voice-picker-search"
            placeholder={isKo ? "이름으로 검색..." : "Search by name..."}
            value={searchText}
            onChange={e => setSearchText(e.target.value)}
          />

          {/* Gender */}
          <div className="voice-picker-filter-row">
            <span className="voice-picker-filter-label">{isKo ? "성별" : "Gender"}</span>
            <div className="voice-picker-pills">
              <button className={gender === "all" ? "active" : ""} onClick={() => setGender("all")}>
                {isKo ? "전체" : "All"}
              </button>
              {Object.entries(cats.gender).map(([key, lbl]) => (
                <button key={key} className={gender === key ? "active" : ""} onClick={() => setGender(key)}>
                  {label(lbl, locale)}
                </button>
              ))}
            </div>
          </div>

          {/* Age */}
          <div className="voice-picker-filter-row">
            <span className="voice-picker-filter-label">{isKo ? "연령" : "Age"}</span>
            <div className="voice-picker-pills">
              <button className={age === "all" ? "active" : ""} onClick={() => setAge("all")}>
                {isKo ? "전체" : "All"}
              </button>
              {Object.entries(cats.age).map(([key, lbl]) => (
                <button key={key} className={age === key ? "active" : ""} onClick={() => setAge(key)}>
                  {label(lbl, locale)}
                </button>
              ))}
            </div>
          </div>

          {/* Mood */}
          <div className="voice-picker-filter-row">
            <span className="voice-picker-filter-label">{isKo ? "분위기" : "Mood"}</span>
            <div className="voice-picker-pills">
              <button className={mood === "all" ? "active" : ""} onClick={() => setMood("all")}>
                {isKo ? "전체" : "All"}
              </button>
              {Object.entries(cats.mood).map(([key, lbl]) => (
                <button key={key} className={mood === key ? "active" : ""} onClick={() => setMood(key)}>
                  {label(lbl, locale)}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Voice Grid */}
        <div className="voice-picker-grid">
          {filteredVoices.map(voice => {
            const isSelected = voice.voice_id === selectedVoiceId;
            const ageLabel = label(cats.age[voice.age], locale);
            const genderLabel = label(cats.gender[voice.gender], locale);
            const useCaseLabels = voice.use_cases
              .slice(0, 2)
              .map(uc => label(cats.use_cases[uc], locale))
              .filter(Boolean);

            return (
              <div
                key={voice.voice_id}
                className={`voice-card ${isSelected ? "selected" : ""}`}
                onClick={() => handleSelect(voice)}
              >
                <div className="voice-card-avatar">
                  {voice.gender === "female" ? "👩" : "👨"}
                </div>
                <div className="voice-card-info">
                  <div className="voice-card-name">{voice.voice_name}</div>
                  <div className="voice-card-meta">
                    {genderLabel} · {ageLabel}
                  </div>
                  <div className="voice-card-tags">
                    {useCaseLabels.map((tag, i) => (
                      <span key={i} className="voice-card-tag">{tag}</span>
                    ))}
                  </div>
                </div>
                {isSelected && <span className="voice-card-check">✓</span>}
              </div>
            );
          })}
          {filteredVoices.length === 0 && (
            <div className="voice-picker-empty">
              {isKo ? "조건에 맞는 음성이 없습니다." : "No voices match your filters."}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
