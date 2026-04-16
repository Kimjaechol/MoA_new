import { useState, useEffect, useCallback } from "react";
import {
  listArchivedMemories,
  restoreArchivedMemory,
  type ArchivedMemory,
} from "../lib/tauri-bridge";
import type { Locale } from "../lib/i18n";

interface ArchiveListProps {
  locale: Locale;
  onBack: () => void;
}

export function ArchiveList({ locale, onBack }: ArchiveListProps) {
  const [items, setItems] = useState<ArchivedMemory[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [restoring, setRestoring] = useState<Set<string>>(new Set());

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listArchivedMemories();
      setItems(result?.archived ?? []);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const handleRestore = useCallback(
    async (id: string) => {
      setRestoring((prev) => new Set(prev).add(id));
      try {
        const ok = await restoreArchivedMemory(id);
        if (ok) {
          setItems((prev) => prev.filter((m) => m.id !== id));
        }
      } catch (e) {
        setError(String(e));
      } finally {
        setRestoring((prev) => {
          const next = new Set(prev);
          next.delete(id);
          return next;
        });
      }
    },
    [],
  );

  const isKo = locale === "ko";

  return (
    <div style={{ padding: 24, maxWidth: 800, margin: "0 auto" }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
        <button
          onClick={onBack}
          style={{
            background: "none",
            border: "1px solid var(--border-color, #444)",
            borderRadius: 6,
            padding: "4px 12px",
            cursor: "pointer",
            color: "inherit",
          }}
        >
          {isKo ? "뒤로" : "Back"}
        </button>
        <h2 style={{ margin: 0 }}>{isKo ? "아카이브된 기억" : "Archived Memories"}</h2>
        <span style={{ color: "var(--text-secondary, #888)", fontSize: 14 }}>
          ({items.length})
        </span>
      </div>

      {error && (
        <div style={{ color: "#f44", marginBottom: 12 }}>{error}</div>
      )}

      {loading ? (
        <div style={{ color: "var(--text-secondary, #888)" }}>
          {isKo ? "불러오는 중..." : "Loading..."}
        </div>
      ) : items.length === 0 ? (
        <div style={{ color: "var(--text-secondary, #888)" }}>
          {isKo ? "아카이브된 기억이 없습니다." : "No archived memories."}
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {items.map((m) => (
            <div
              key={m.id}
              style={{
                border: "1px solid var(--border-color, #333)",
                borderRadius: 8,
                padding: 12,
                background: "var(--bg-secondary, #1a1a1a)",
              }}
            >
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontWeight: 600, marginBottom: 4 }}>{m.key}</div>
                  <div
                    style={{
                      fontSize: 13,
                      color: "var(--text-secondary, #888)",
                      whiteSpace: "pre-wrap",
                      maxHeight: 80,
                      overflow: "hidden",
                    }}
                  >
                    {m.content}
                  </div>
                  <div style={{ fontSize: 11, color: "var(--text-tertiary, #666)", marginTop: 4 }}>
                    {m.category} &middot; {m.updated_at}
                  </div>
                  {m.consolidated_summary && (
                    <div
                      style={{
                        fontSize: 12,
                        marginTop: 6,
                        padding: "4px 8px",
                        background: "var(--bg-tertiary, #252525)",
                        borderRadius: 4,
                        color: "var(--text-secondary, #aaa)",
                      }}
                    >
                      {isKo ? "통합됨" : "Consolidated"}: {m.consolidated_summary}
                      {m.consolidated_fact_type && (
                        <span style={{ marginLeft: 8, opacity: 0.7 }}>
                          ({m.consolidated_fact_type})
                        </span>
                      )}
                    </div>
                  )}
                </div>
                <button
                  onClick={() => handleRestore(m.id)}
                  disabled={restoring.has(m.id)}
                  style={{
                    marginLeft: 12,
                    padding: "6px 14px",
                    border: "1px solid var(--border-color, #444)",
                    borderRadius: 6,
                    background: "none",
                    cursor: restoring.has(m.id) ? "wait" : "pointer",
                    color: "inherit",
                    fontSize: 13,
                    flexShrink: 0,
                  }}
                >
                  {restoring.has(m.id)
                    ? isKo
                      ? "복구 중..."
                      : "Restoring..."
                    : isKo
                      ? "복구"
                      : "Restore"}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
