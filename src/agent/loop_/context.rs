use crate::memory::{self, Memory};
use crate::ontology::OntologyRepo;
use crate::providers::ChatMessage;
use std::collections::HashSet;
use std::fmt::Write;

/// Maximum number of long-term memory entries to recall per message.
const MAX_RECALL_ENTRIES: usize = 100;

/// Maximum number of ontology objects to search per message.
const MAX_ONTOLOGY_ENTRIES: usize = 100;

/// Maximum cross-search enrichment entries (prevents runaway queries).
const MAX_CROSS_SEARCH_ENTRIES: usize = 20;

/// Build context preamble by searching both long-term memory and ontology
/// for relevant entries, with **bidirectional cross-referencing**.
///
/// Cross-search protocol:
/// 1. Search memory (vector+keyword) → extract time/place/person keywords
/// 2. Search ontology (FTS5) → extract time/place/person keywords
/// 3. Use ontology keywords to enrich memory search → find related conversations
/// 4. Use memory keywords to enrich ontology search → find related relationships
/// 5. Combine all results with deduplication
pub(super) async fn build_context(
    mem: &dyn Memory,
    user_msg: &str,
    min_relevance_score: f64,
    session_id: Option<&str>,
    ontology: Option<&OntologyRepo>,
) -> String {
    let mut context = String::with_capacity(8192);
    let mut seen_memory_keys = HashSet::new();
    let mut cross_search_keywords = Vec::new();

    // ── Phase 0: Essential profile recall (ALWAYS loaded) ──────
    // These keys are loaded regardless of the user's message content.
    // Without this, greeting messages like "안녕" would not retrieve
    // the user's name, occupation, or preferred form of address.
    const ESSENTIAL_PROFILE_KEYS: &[&str] = &[
        "user_profile_identity",
        "user_profile_family",
        "user_profile_work",
        "user_profile_lifestyle",
        "user_profile_communication",
        "user_profile_routine",
        "user_moa_preferences",
    ];

    let mut essential_loaded = false;
    for key in ESSENTIAL_PROFILE_KEYS {
        if let Ok(Some(entry)) = mem.get(key).await {
            if !essential_loaded {
                context.push_str("[User profile — always loaded]\n");
                essential_loaded = true;
            }
            seen_memory_keys.insert(entry.key.clone());
            let ts_hint = if entry.timestamp.is_empty() {
                String::new()
            } else {
                let short_ts = if entry.timestamp.len() > 19 {
                    &entry.timestamp[..19]
                } else {
                    &entry.timestamp
                };
                format!(" [{}]", short_ts)
            };
            let line = format!("- {}:{} {}\n", entry.key, ts_hint, entry.content);
            context.push_str(&line);
            extract_cross_search_keywords(&entry.content, &mut cross_search_keywords);
        }
    }
    if essential_loaded {
        context.push('\n');
    }

    // ── Phase 1: Primary memory recall ──────────────────────────
    if let Ok(entries) = mem.recall(user_msg, MAX_RECALL_ENTRIES, session_id).await {
        let relevant: Vec<_> = entries
            .iter()
            .filter(|e| match e.score {
                Some(score) => score >= min_relevance_score,
                None => true,
            })
            .collect();

        if !relevant.is_empty() {
            context.push_str("[Memory context]\n");
            for entry in &relevant {
                if memory::is_assistant_autosave_key(&entry.key) {
                    continue;
                }
                seen_memory_keys.insert(entry.key.clone());
                // Include timestamp so the LLM knows WHEN this memory was recorded
                let ts_hint = if entry.timestamp.is_empty() {
                    String::new()
                } else {
                    // Truncate to date+time (no timezone suffix) for readability
                    let short_ts = if entry.timestamp.len() > 19 {
                        &entry.timestamp[..19]
                    } else {
                        &entry.timestamp
                    };
                    format!(" [{}]", short_ts)
                };
                let line = format!("- {}:{} {}\n", entry.key, ts_hint, entry.content);
                context.push_str(&line);

                // Extract time/place/person keywords from memory content
                // for cross-searching into ontology.
                extract_cross_search_keywords(&entry.content, &mut cross_search_keywords);
            }
            if context == "[Memory context]\n" {
                context.clear();
            } else {
                context.push('\n');
            }
        }
    }

    // ── Phase 2: Primary ontology search ────────────────────────
    let mut ontology_cross_keywords = Vec::new();
    if let Some(repo) = ontology {
        let owner = session_id.unwrap_or("cli_interactive");
        if let Ok(objects) =
            repo.search_objects(owner, None, user_msg, MAX_ONTOLOGY_ENTRIES)
        {
            if !objects.is_empty() {
                context.push_str("[Ontology context]\n");
                for obj in &objects {
                    let title = obj.title.as_deref().unwrap_or("(untitled)");
                    let props = if obj.properties.is_null() || obj.properties.as_object().is_some_and(|m| m.is_empty()) {
                        String::new()
                    } else {
                        obj.properties.to_string()
                    };
                    if props.is_empty() {
                        let _ = writeln!(context, "- {title}");
                    } else {
                        let _ = writeln!(context, "- {title}: {props}");
                    }

                    // Extract keywords from ontology objects for cross-searching memory
                    if let Some(t) = obj.title.as_deref() {
                        ontology_cross_keywords.push(t.to_string());
                    }
                    extract_cross_search_keywords_from_json(&obj.properties, &mut ontology_cross_keywords);
                }
                context.push('\n');
            }
        }

        // ── Phase 3: Cross-search — ontology → memory enrichment ──
        // Use keywords from ontology results to find related conversations in memory
        if !ontology_cross_keywords.is_empty() {
            let cross_query = ontology_cross_keywords
                .iter()
                .take(5) // Limit to top 5 keywords
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");

            if let Ok(enriched) = mem.recall(&cross_query, MAX_CROSS_SEARCH_ENTRIES, session_id).await {
                let new_entries: Vec<_> = enriched
                    .iter()
                    .filter(|e| {
                        e.score.unwrap_or(1.0) >= min_relevance_score
                            && !memory::is_assistant_autosave_key(&e.key)
                            && !seen_memory_keys.contains(&e.key)
                    })
                    .collect();

                if !new_entries.is_empty() {
                    context.push_str("[Cross-referenced memories (from ontology context)]\n");
                    for entry in &new_entries {
                        seen_memory_keys.insert(entry.key.clone());
                        let _ = writeln!(context, "- {}: {}", entry.key, entry.content);
                    }
                    context.push('\n');
                }
            }
        }

        // ── Phase 4: Cross-search — memory → ontology enrichment ──
        // Use keywords from memory results to find related relationships in ontology
        if !cross_search_keywords.is_empty() {
            let cross_query = cross_search_keywords
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");

            if let Ok(enriched_objects) =
                repo.search_objects(owner, None, &cross_query, MAX_CROSS_SEARCH_ENTRIES)
            {
                // Filter out objects already shown in primary ontology results
                let new_objects: Vec<_> = enriched_objects
                    .iter()
                    .filter(|o| {
                        let title = o.title.as_deref().unwrap_or("");
                        !ontology_cross_keywords.contains(&title.to_string())
                    })
                    .collect();

                if !new_objects.is_empty() {
                    context.push_str("[Cross-referenced relationships (from memory context)]\n");
                    for obj in &new_objects {
                        let title = obj.title.as_deref().unwrap_or("(untitled)");
                        let props = if obj.properties.is_null() || obj.properties.as_object().is_some_and(|m| m.is_empty()) {
                            String::new()
                        } else {
                            obj.properties.to_string()
                        };
                        if props.is_empty() {
                            let _ = writeln!(context, "- {title}");
                        } else {
                            let _ = writeln!(context, "- {title}: {props}");
                        }
                    }
                    context.push('\n');
                }
            }
        }
    }

    context
}

/// Extract time, place, and person keywords from memory content
/// for cross-referencing with ontology.
fn extract_cross_search_keywords(content: &str, keywords: &mut Vec<String>) {
    // Look for structured metadata patterns in promoted memories:
    // [category] 시간: ... | 장소: ... | 상대방: ... | 행위: ...
    for line in content.lines() {
        let line = line.trim();

        // Extract Korean metadata fields
        for prefix in &["시간:", "장소:", "상대방:", "행위:"] {
            if let Some(pos) = line.find(prefix) {
                let after = &line[pos + prefix.len()..];
                let value = after.split('|').next().unwrap_or(after).trim();
                if !value.is_empty() && value != "unknown" && value != "user" {
                    keywords.push(value.to_string());
                }
            }
        }

        // Extract English metadata fields (for non-Korean content)
        for prefix in &["time:", "location:", "counterpart:", "action:"] {
            if let Some(pos) = line.find(prefix) {
                let after = &line[pos + prefix.len()..];
                let value = after.split('|').next().unwrap_or(after).trim();
                if !value.is_empty() && value != "unknown" && value != "user" {
                    keywords.push(value.to_string());
                }
            }
        }
    }
}

/// Extract keywords from ontology object JSON properties.
fn extract_cross_search_keywords_from_json(props: &serde_json::Value, keywords: &mut Vec<String>) {
    if let Some(obj) = props.as_object() {
        for (key, value) in obj {
            // Focus on identity/temporal/spatial fields
            if matches!(key.as_str(),
                "name" | "location" | "time" | "date" | "counterpart"
                | "channel" | "category" | "topic" | "subject"
            ) {
                if let Some(s) = value.as_str() {
                    if !s.is_empty() && s.len() < 100 {
                        keywords.push(s.to_string());
                    }
                }
            }
        }
    }
}

/// Build hardware datasheet context from RAG when peripherals are enabled.
/// Includes pin-alias lookup (e.g. "red_led" → 13) when query matches, plus retrieved chunks.
pub(super) fn build_hardware_context(
    rag: &crate::rag::HardwareRag,
    user_msg: &str,
    boards: &[String],
    chunk_limit: usize,
) -> String {
    if rag.is_empty() || boards.is_empty() {
        return String::new();
    }

    let mut context = String::new();

    // Pin aliases: when user says "red led", inject "red_led: 13" for matching boards
    let pin_ctx = rag.pin_alias_context(user_msg, boards);
    if !pin_ctx.is_empty() {
        context.push_str(&pin_ctx);
    }

    let chunks = rag.retrieve(user_msg, boards, chunk_limit);
    if chunks.is_empty() && pin_ctx.is_empty() {
        return String::new();
    }

    if !chunks.is_empty() {
        context.push_str("[Hardware documentation]\n");
    }
    for chunk in chunks {
        let board_tag = chunk.board.as_deref().unwrap_or("generic");
        let _ = writeln!(
            context,
            "--- {} ({}) ---\n{}\n",
            chunk.source, board_tag, chunk.content
        );
    }
    context.push('\n');
    context
}

/// Truncate a string to at most `max_chars` characters, appending "…" if truncated.
/// This is UTF-8 safe — it counts Unicode scalar values, not bytes.
fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

/// Build cross-session recent conversation context from stored turns.
///
/// Formats the most recent turns as `[Recent conversation history]` for injection
/// into the LLM context, providing conversational continuity across sessions.
///
/// # Parameters
/// - `turns`: recent conversation turns (oldest-first, chronological order)
/// - `skip_current`: number of trailing turns to skip (e.g. 1 to skip the
///   current user message that was just appended)
/// - `max_bytes`: maximum total bytes for the context block
/// - `turn_max_chars`: maximum characters per individual turn content
pub(super) fn build_cross_session_context(
    turns: &[ChatMessage],
    skip_current: usize,
    max_bytes: usize,
    turn_max_chars: usize,
) -> String {
    if turns.is_empty() {
        return String::new();
    }

    let take_count = turns.len().saturating_sub(skip_current);
    if take_count == 0 {
        return String::new();
    }

    const HEADER: &str = "[Recent conversation history — verbatim, continue this conversation naturally]\n";

    // Pre-allocate: estimate ~80 bytes per turn to reduce reallocations for
    // large turn counts (up to 600).
    let estimated = HEADER.len() + take_count.min(600) * 80;
    let mut ctx = String::with_capacity(estimated.min(max_bytes + 256));
    ctx.push_str(HEADER);
    let mut total = HEADER.len();

    for turn in turns.iter().take(take_count) {
        let label = if turn.role == "user" { "User" } else { "Assistant" };
        let content = &turn.content;

        // Calculate line length without allocating a temporary String when
        // the turn content fits within the character limit.
        let char_count = content.chars().count();
        if char_count <= turn_max_chars {
            // Fast path: content fits — write directly into ctx.
            let line_len = label.len() + 2 + content.len() + 1; // "Label: content\n"
            if total + line_len > max_bytes {
                break;
            }
            total += line_len;
            ctx.push_str(label);
            ctx.push_str(": ");
            ctx.push_str(content);
            ctx.push('\n');
        } else {
            // Slow path: content needs truncation.
            let truncated = truncate_chars(content, turn_max_chars);
            let line_len = label.len() + 2 + truncated.len() + 1;
            if total + line_len > max_bytes {
                break;
            }
            total += line_len;
            ctx.push_str(label);
            ctx.push_str(": ");
            ctx.push_str(&truncated);
            ctx.push('\n');
        }
    }

    if ctx.len() == HEADER.len() {
        String::new()
    } else {
        ctx.push('\n');
        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_ascii() {
        assert_eq!(truncate_chars("hello world", 5), "hello…");
        assert_eq!(truncate_chars("hello", 5), "hello");
        assert_eq!(truncate_chars("hi", 5), "hi");
    }

    #[test]
    fn truncate_chars_multibyte() {
        // Korean text: each character is 3 bytes in UTF-8
        let korean = "안녕하세요 반갑습니다";
        let result = truncate_chars(korean, 5);
        assert_eq!(result, "안녕하세요…");
        // Ensure no panic on multi-byte boundaries
        assert_eq!(truncate_chars(korean, 1), "안…");
    }

    #[test]
    fn build_cross_session_empty() {
        assert_eq!(build_cross_session_context(&[], 0, 16000, 600), "");
    }

    #[test]
    fn build_cross_session_skips_current() {
        let turns = vec![
            ChatMessage { role: "user".into(), content: "hello".into() },
            ChatMessage { role: "assistant".into(), content: "hi there".into() },
            ChatMessage { role: "user".into(), content: "current msg".into() },
        ];
        let ctx = build_cross_session_context(&turns, 1, 16000, 600);
        assert!(ctx.contains("User: hello"));
        assert!(ctx.contains("Assistant: hi there"));
        assert!(!ctx.contains("current msg"));
    }

    #[test]
    fn build_cross_session_respects_byte_limit() {
        let turns: Vec<ChatMessage> = (0..100)
            .map(|i| ChatMessage {
                role: "user".into(),
                content: format!("message number {i} with some content padding"),
            })
            .collect();
        let ctx = build_cross_session_context(&turns, 0, 500, 600);
        assert!(ctx.len() <= 500 + 100); // small overshoot from last line is ok
    }

    #[test]
    fn build_cross_session_truncates_long_turns() {
        let turns = vec![ChatMessage {
            role: "user".into(),
            content: "a".repeat(1000),
        }];
        let ctx = build_cross_session_context(&turns, 0, 16000, 10);
        assert!(ctx.contains(&format!("User: {}…", "a".repeat(10))));
    }
}
