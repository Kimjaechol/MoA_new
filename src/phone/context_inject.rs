// Context Injection — build system prompt from brain for phone calls (v3.0 Section B)
//
// When a known caller is matched via ontology:
// 1. Load the caller's compiled_truth from linked memories
// 2. Load recent timeline entries (last N)
// 3. Build a system prompt that gives the AI phone assistant context
//
// The prompt is injected into Gemini Live's system instruction before
// the call is connected.

use anyhow::Result;

use crate::memory::sqlite::SqliteMemory;
use crate::ontology::types::OntologyObject;

/// Maximum tokens for the injected context (keeps latency low).
const MAX_CONTEXT_TOKENS: usize = 2000;
/// Approximate chars per token for truncation.
const CHARS_PER_TOKEN: usize = 4;
/// Number of most-recent timeline entries to include.
const RECENT_TIMELINE_LIMIT: usize = 5;

/// Built context for a phone call's system prompt.
#[derive(Debug, Clone)]
pub struct CallContext {
    /// The caller's display name (from ontology title).
    pub caller_name: String,
    /// Compiled truth summary (if available).
    pub compiled_truth: Option<String>,
    /// Recent timeline entries formatted as text.
    pub recent_events: Vec<String>,
    /// The assembled system prompt fragment.
    pub system_prompt_fragment: String,
}

/// Build call context from a matched ontology object.
///
/// Loads compiled_truth and recent timeline from the brain layer,
/// then assembles a concise system prompt fragment.
pub fn build_call_context(
    memory: &SqliteMemory,
    caller_object: &OntologyObject,
    linked_memory_key: Option<&str>,
) -> Result<CallContext> {
    let caller_name = caller_object
        .title
        .clone()
        .unwrap_or_else(|| "알 수 없는 발신자".to_string());

    // Load compiled truth if we have a linked memory key.
    let compiled_truth = if let Some(key) = linked_memory_key {
        memory
            .get_compiled_truth(key)?
            .map(|(truth, _version)| truth)
    } else {
        None
    };

    // Load recent timeline entries for the linked memory.
    //
    // The caller supplies a memory `key` (e.g. "client_a"); we resolve it to
    // the internal `memory_id` via `memory_id_for_key`, then query the
    // append-only `memory_timeline` in descending `event_at` order.
    let recent_events = if let Some(key) = linked_memory_key {
        match memory.memory_id_for_key(key)? {
            Some(memory_id) => memory
                .get_timeline(&memory_id, RECENT_TIMELINE_LIMIT)?
                .into_iter()
                .map(format_timeline_entry)
                .collect(),
            None => Vec::new(),
        }
    } else {
        Vec::new()
    };

    // Assemble system prompt fragment.
    let max_chars = MAX_CONTEXT_TOKENS * CHARS_PER_TOKEN;
    let system_prompt_fragment = build_prompt_fragment(
        &caller_name,
        compiled_truth.as_deref(),
        &recent_events,
        max_chars,
    );

    Ok(CallContext {
        caller_name,
        compiled_truth,
        recent_events,
        system_prompt_fragment,
    })
}

/// Render a single timeline entry as one line for prompt injection.
///
/// Format: `YYYY-MM-DD [event_type] truncated_content`.
fn format_timeline_entry(entry: crate::memory::sqlite::TimelineEntry) -> String {
    // `event_at` is milliseconds-since-epoch per `append_timeline`.
    let secs = entry.event_at / 1000;
    let date = chrono::DateTime::from_timestamp(secs, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Keep each line compact; full content lives in the timeline, not the prompt.
    const PER_ENTRY_CHAR_CAP: usize = 160;
    let content = entry.content.replace('\n', " ");
    let trimmed = if content.chars().count() > PER_ENTRY_CHAR_CAP {
        let truncated: String = content.chars().take(PER_ENTRY_CHAR_CAP).collect();
        format!("{truncated}...")
    } else {
        content
    };

    format!("{date} [{}] {trimmed}", entry.event_type)
}

/// Build context for an unknown (anonymous) caller.
pub fn build_anonymous_context() -> CallContext {
    let system_prompt_fragment = "\
        [발신자 정보 없음] 새로운 발신자입니다. \
        정중하게 응대하고, 이름과 용건을 먼저 확인하세요. \
        의뢰인 접수가 필요할 수 있습니다."
        .to_string();

    CallContext {
        caller_name: "알 수 없는 발신자".to_string(),
        compiled_truth: None,
        recent_events: Vec::new(),
        system_prompt_fragment,
    }
}

/// Assemble the system prompt fragment from context parts.
fn build_prompt_fragment(
    caller_name: &str,
    compiled_truth: Option<&str>,
    recent_events: &[String],
    max_chars: usize,
) -> String {
    let mut parts = Vec::new();

    parts.push(format!("[발신자: {caller_name}]"));

    if let Some(truth) = compiled_truth {
        let truncated = if truth.len() > max_chars / 2 {
            format!("{}...", truncate_at_char_boundary(truth, max_chars / 2))
        } else {
            truth.to_string()
        };
        parts.push(format!("[요약]\n{truncated}"));
    }

    if !recent_events.is_empty() {
        let events_text: String = recent_events
            .iter()
            .take(5)
            .map(|e| format!("- {e}"))
            .collect::<Vec<_>>()
            .join("\n");
        parts.push(format!("[최근 이력]\n{events_text}"));
    }

    let mut result = parts.join("\n\n");

    // Enforce total length limit (char-boundary safe for UTF-8)
    if result.len() > max_chars {
        result = truncate_at_char_boundary(&result, max_chars);
        result.push_str("...");
    }

    result
}

/// Truncate a string at the last char boundary ≤ `max_bytes`.
/// Safe for multi-byte UTF-8 content (Korean, emoji, etc.).
fn truncate_at_char_boundary(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    // Find the largest valid char boundary ≤ max_bytes
    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    s[..boundary].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anonymous_context_has_prompt() {
        let ctx = build_anonymous_context();
        assert_eq!(ctx.caller_name, "알 수 없는 발신자");
        assert!(!ctx.system_prompt_fragment.is_empty());
        assert!(ctx.system_prompt_fragment.contains("새로운 발신자"));
    }

    #[test]
    fn prompt_fragment_with_truth_only() {
        let fragment = build_prompt_fragment(
            "김철수",
            Some("이혼소송 진행중. 재산분할 쟁점."),
            &[],
            8000,
        );
        assert!(fragment.contains("김철수"));
        assert!(fragment.contains("이혼소송"));
    }

    #[test]
    fn prompt_fragment_with_events() {
        let events = vec![
            "2024-03-01: 첫 상담 전화".to_string(),
            "2024-03-05: 서류 요청".to_string(),
        ];
        let fragment = build_prompt_fragment("박변호사", None, &events, 8000);
        assert!(fragment.contains("박변호사"));
        assert!(fragment.contains("첫 상담 전화"));
        assert!(fragment.contains("서류 요청"));
    }

    #[test]
    fn prompt_fragment_truncates() {
        let long_truth = "가".repeat(5000);
        let fragment = build_prompt_fragment("테스트", Some(&long_truth), &[], 2000);
        assert!(fragment.len() <= 2010); // small slack for "..."
    }

    #[test]
    fn prompt_fragment_full_context() {
        let events = vec!["이벤트1".to_string()];
        let fragment = build_prompt_fragment(
            "홍길동",
            Some("주요 의뢰인. 형사사건 담당."),
            &events,
            8000,
        );
        assert!(fragment.contains("홍길동"));
        assert!(fragment.contains("주요 의뢰인"));
        assert!(fragment.contains("이벤트1"));
    }

    #[test]
    fn format_timeline_entry_uses_date_and_type() {
        let entry = crate::memory::sqlite::TimelineEntry {
            uuid: "u1".into(),
            event_type: "call".into(),
            // 2024-03-01T00:00:00Z in ms → 2024-03-01 (UTC)
            event_at: 1_709_251_200_000,
            source_ref: "call-001".into(),
            content: "첫 상담 전화 내용".into(),
            metadata_json: None,
            device_id: "dev1".into(),
            created_at: 0,
        };
        let line = format_timeline_entry(entry);
        assert!(line.starts_with("2024-03-01"), "got: {line}");
        assert!(line.contains("[call]"));
        assert!(line.contains("첫 상담 전화 내용"));
    }

    #[test]
    fn format_timeline_entry_truncates_long_content() {
        let long = "가".repeat(500);
        let entry = crate::memory::sqlite::TimelineEntry {
            uuid: "u2".into(),
            event_type: "doc".into(),
            event_at: 1_709_251_200_000,
            source_ref: "s".into(),
            content: long,
            metadata_json: None,
            device_id: "d".into(),
            created_at: 0,
        };
        let line = format_timeline_entry(entry);
        assert!(line.ends_with("..."));
        // Should be roughly ~170 bytes (160 chars + header + ellipsis).
        // Korean chars are 3 bytes each so 160 chars ≈ 480 bytes + date/header.
        assert!(line.contains("[doc]"));
    }

    #[test]
    fn format_timeline_entry_collapses_newlines() {
        let entry = crate::memory::sqlite::TimelineEntry {
            uuid: "u3".into(),
            event_type: "chat".into(),
            event_at: 1_709_251_200_000,
            source_ref: "s".into(),
            content: "line1\nline2\nline3".into(),
            metadata_json: None,
            device_id: "d".into(),
            created_at: 0,
        };
        let line = format_timeline_entry(entry);
        assert!(!line.contains('\n'), "newlines should be collapsed: {line}");
        assert!(line.contains("line1 line2 line3"));
    }

    #[test]
    fn build_call_context_loads_timeline_by_key() {
        use crate::memory::sqlite::SqliteMemory;
        use crate::memory::traits::{Memory, MemoryCategory};

        let tmp = tempfile::TempDir::new().unwrap();
        let mem = SqliteMemory::new(tmp.path()).unwrap();

        // Seed a memory entry and append two timeline events.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            mem.store("client_a", "Client A notes", MemoryCategory::Core, None)
                .await
                .unwrap();
        });
        let id = mem
            .memory_id_for_key("client_a")
            .unwrap()
            .expect("id lookup");

        mem.append_timeline(
            &id,
            "call",
            1_709_251_200_000,
            "call-001",
            "첫 상담",
            None,
            "dev1",
            None,
        )
        .unwrap();
        mem.append_timeline(
            &id,
            "doc",
            1_709_337_600_000,
            "doc-1",
            "서류 접수",
            None,
            "dev1",
            None,
        )
        .unwrap();

        let obj = OntologyObject {
            id: 1,
            type_id: 1,
            title: Some("김철수".to_string()),
            properties: serde_json::json!({}),
            owner_user_id: "user1".to_string(),
            themes: vec![],
            created_at: 0,
            updated_at: 0,
        };

        let ctx = build_call_context(&mem, &obj, Some("client_a")).unwrap();
        assert_eq!(ctx.caller_name, "김철수");
        assert_eq!(ctx.recent_events.len(), 2);
        // Descending order by event_at: doc (newer) comes first.
        assert!(ctx.recent_events[0].contains("[doc]"));
        assert!(ctx.recent_events[0].contains("서류 접수"));
        assert!(ctx.recent_events[1].contains("[call]"));
        assert!(ctx.system_prompt_fragment.contains("김철수"));
        assert!(ctx.system_prompt_fragment.contains("서류 접수"));
    }

    #[test]
    fn build_call_context_empty_when_key_missing() {
        use crate::memory::sqlite::SqliteMemory;

        let tmp = tempfile::TempDir::new().unwrap();
        let mem = SqliteMemory::new(tmp.path()).unwrap();

        let obj = OntologyObject {
            id: 1,
            type_id: 1,
            title: Some("Ghost".to_string()),
            properties: serde_json::json!({}),
            owner_user_id: "user1".to_string(),
            themes: vec![],
            created_at: 0,
            updated_at: 0,
        };

        // Key does not exist → no compiled truth, no timeline, but still a valid ctx.
        let ctx = build_call_context(&mem, &obj, Some("nonexistent_key")).unwrap();
        assert_eq!(ctx.caller_name, "Ghost");
        assert!(ctx.compiled_truth.is_none());
        assert!(ctx.recent_events.is_empty());
        assert!(ctx.system_prompt_fragment.contains("Ghost"));
    }
}
