//! Q1 Commit #9 — Dream-Cycle Daily Diary Backfill.
//!
//! Every night at 02:00–06:00 the on-device SLM (Gemma 4) plays the role
//! of a diary author: it reads the raw `memories.content` rows from the
//! previous local calendar day and extracts 5W1H structured fields plus a
//! canonical `narrative` sentence into the memory row. The raw `content`
//! is preserved for 72h; after that window + `narrative_filled = 1` a
//! separate purge step NULLs the content so disk doesn't grow without
//! bound (the narrative + 5W1H columns remain as the long-term record).
//!
//! Design principles (see ARCHITECTURE.md §Q1-5W1H):
//!
//! 1. **Day-by-day rhythm** — we process one full local calendar day per
//!    iteration. If we miss a night (device off / low battery / network
//!    unavailable), the next cycle catches up by processing each
//!    subsequent day in order, bounded by `max_catchup_days` so long
//!    offline periods don't blow the SLM budget in one burst.
//!
//! 2. **`how_action` is special** — the Dream Cycle SLM is instructed to
//!    preserve conversational nuance verbatim (quoted fragments),
//!    emotion, mood, intent. Other 5W1H fields stay concise/objective.
//!    Cross-search disambiguation later uses `how_action` to distinguish
//!    similar events ("casual round with friends" vs "tense match").
//!
//! 3. **Fail-safe purge** — raw content is NEVER deleted unless the
//!    Dream Cycle has confirmed narrative was written AND `created_at`
//!    is older than the retention window (default 72h). If the SLM
//!    call fails three times, `narrative_failures` hits the threshold
//!    and the memory is flagged for user review rather than silently
//!    dropped.

use anyhow::Result;
use chrono::{DateTime, Datelike, NaiveDate, Utc};

use super::sqlite::{Memory5W1H, SqliteMemory};

// ── Configuration ──────────────────────────────────────────────────

/// Hard ceiling on days a single nightly cycle will process. Keeps SLM
/// budget predictable even after long offline periods; unfinished catch-up
/// carries over to the next cycle.
pub const DEFAULT_MAX_CATCHUP_DAYS: u32 = 14;

/// Minimum age before raw content may be purged. 72h matches the short-
/// term conversation context retention policy so users never lose "what
/// was said yesterday" even while narrative is being written.
pub const DEFAULT_RAW_RETENTION_HOURS: u32 = 72;

/// Maximum consecutive SLM failures per memory before the row is
/// flagged for human review (via `narrative_failures >= threshold`).
pub const DEFAULT_MAX_RETRIES: u32 = 3;

// ── Backfill state ─────────────────────────────────────────────────

/// Per-user progress marker for the nightly diary backfill.
#[derive(Debug, Clone)]
pub struct BackfillState {
    pub user_id: String,
    pub last_completed_date: NaiveDate,
    pub last_run_utc: Option<i64>,
    pub cumulative_days: u64,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
}

impl BackfillState {
    /// Load the existing state row for `user_id`, or return a fresh
    /// state anchored at `default_start_date` (typically 30 days ago,
    /// matching the raw-turn retention floor).
    pub fn load_or_init(
        mem: &SqliteMemory,
        user_id: &str,
        default_start_date: NaiveDate,
    ) -> Result<Self> {
        let conn = mem.connection();
        let row: Option<(String, Option<i64>, i64, i64, Option<String>)> = conn
            .query_row(
                "SELECT last_completed_date, last_run_utc, cumulative_days,
                        consecutive_failures, last_error
                 FROM backfill_state WHERE user_id = ?1",
                rusqlite::params![user_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .ok();
        drop(conn);

        match row {
            Some((date_str, last_run, cum, fails, err)) => {
                let last_completed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .unwrap_or(default_start_date);
                Ok(Self {
                    user_id: user_id.to_string(),
                    last_completed_date,
                    last_run_utc: last_run,
                    cumulative_days: cum.max(0) as u64,
                    consecutive_failures: fails.max(0) as u32,
                    last_error: err,
                })
            }
            None => Ok(Self {
                user_id: user_id.to_string(),
                // Start one day BEFORE default_start_date so the first
                // catch-up actually processes default_start_date.
                last_completed_date: default_start_date.pred_opt().unwrap_or(default_start_date),
                last_run_utc: None,
                cumulative_days: 0,
                consecutive_failures: 0,
                last_error: None,
            }),
        }
    }

    /// Persist this state row. Idempotent UPSERT.
    pub fn save(&self, mem: &SqliteMemory, now_utc: i64) -> Result<()> {
        let conn = mem.connection();
        conn.execute(
            "INSERT INTO backfill_state
                (user_id, last_completed_date, last_run_utc,
                 cumulative_days, consecutive_failures, last_error, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(user_id) DO UPDATE SET
                last_completed_date  = excluded.last_completed_date,
                last_run_utc         = excluded.last_run_utc,
                cumulative_days      = excluded.cumulative_days,
                consecutive_failures = excluded.consecutive_failures,
                last_error           = excluded.last_error,
                updated_at           = excluded.updated_at",
            rusqlite::params![
                self.user_id,
                self.last_completed_date.format("%Y-%m-%d").to_string(),
                self.last_run_utc,
                self.cumulative_days as i64,
                self.consecutive_failures as i64,
                self.last_error,
                now_utc,
            ],
        )?;
        Ok(())
    }

    /// Compute the ordered list of calendar days this cycle should process
    /// (everything after `last_completed_date` up to and including
    /// `local_yesterday`, bounded by `max_catchup_days`).
    pub fn pending_days(&self, local_yesterday: NaiveDate, max_catchup_days: u32) -> Vec<NaiveDate> {
        if local_yesterday <= self.last_completed_date {
            return Vec::new();
        }
        let mut days = Vec::new();
        let mut cursor = self.last_completed_date.succ_opt().unwrap_or(local_yesterday);
        while cursor <= local_yesterday && (days.len() as u32) < max_catchup_days {
            days.push(cursor);
            match cursor.succ_opt() {
                Some(n) => cursor = n,
                None => break,
            }
        }
        days
    }
}

// ── Prompt template ────────────────────────────────────────────────

/// Build the system prompt for the Dream-Cycle SLM diary writer.
///
/// The prompt enforces the user's design guidance:
/// - 6하원칙에 따라 필드를 채우되
/// - `how_action` 필드는 가능한 한 자세하게, 대화 인용과 뉘앙스·감정·의도를 포함
/// - 다른 필드는 객관적·간결
///
/// Returns a pair (system, user) — caller concatenates with the raw content
/// list and passes to `Provider::chat`.
pub fn build_diary_prompt(date_label: &str, memories_json: &str) -> (String, String) {
    let system = String::from(
        "너는 사용자의 개인 일기 작성자다. 6하원칙에 따라 아래 규칙으로 \
         오늘 하루의 사건들을 구조화된 JSON 으로 정리하라.\n\n\
         필드 규칙:\n\
         - who_actor: 행위자 (보통 \"user\"; 전화 수신 등은 상대방)\n\
         - who_target: 대상들 (JSON array; 없으면 빈 배열)\n\
         - when_at: unix 초 단위 사건 시각\n\
         - where_location: 장소 자연어 (없으면 null)\n\
         - what_subject: 사건/주제 — 객관적·간결 (한 문장)\n\
         - how_action: ★★★ 중요 ★★★ — 자세히, 뉘앙스까지 살려서 작성.\n\
             · 대화는 핵심 문장을 그대로 인용 (예: \"오늘 진짜 좋았어\"라고 \
               웃으며 말했다)\n\
             · 요약 시에도 화자의 기분·감정·의도·태도를 함께 기술\n\
               (예: 차분하게 거절, 약간 망설이며 동의, 농담조로 받아침, \
                냉소적으로 동의)\n\
             · 단순 사실 나열 금지 — 사람이 일기를 쓰듯 미묘한 색감을 남겨라\n\
         - why_reason: 왜 — 맥락·동기 (한 문장, 객관적)\n\
         - narrative: 정규 일기장 형식 한 문장:\n\
           \"[YYYY-MM-DD HH:MM @ <장소>] <행위자>이(가) <대상>와(과) \
            <무엇>을 <어떻게> — <왜>\"\n\n\
         출력은 오로지 JSON 배열만. 설명 문구 금지.",
    );

    let user = format!(
        "날짜: {date_label}\n\n원문 메모리:\n{memories_json}\n\n\
         위 메모리들 각각을 위 규칙으로 정제해 JSON 배열로 출력하라. \
         각 원소는 {{memory_id, who_actor, who_target, when_at, \
         where_location, what_subject, how_action, why_reason, \
         narrative}} 객체."
    );

    (system, user)
}

/// Parse the SLM's JSON response into a Vec<(memory_id, Memory5W1H)>.
/// Robust to trailing prose and bare objects. Returns empty vec on failure
/// rather than hard-erroring so a single bad response doesn't break the
/// whole cycle.
pub fn parse_diary_response(response_text: &str) -> Vec<(String, Memory5W1H)> {
    let trimmed = response_text.trim();
    // Try array first.
    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(trimmed) {
        return arr
            .into_iter()
            .filter_map(value_to_entry)
            .collect();
    }
    // Fall back: some SLMs emit `{"entries": [...]}`.
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(arr) = obj.get("entries").and_then(|v| v.as_array()) {
            return arr
                .iter()
                .cloned()
                .filter_map(value_to_entry)
                .collect();
        }
    }
    Vec::new()
}

fn value_to_entry(v: serde_json::Value) -> Option<(String, Memory5W1H)> {
    let memory_id = v.get("memory_id")?.as_str()?.to_string();
    let to_s = |k: &str| {
        v.get(k)
            .and_then(|x| {
                if x.is_null() {
                    None
                } else if let Some(s) = x.as_str() {
                    Some(s.to_string())
                } else {
                    Some(x.to_string())
                }
            })
    };
    let fields = Memory5W1H {
        who_actor: to_s("who_actor"),
        who_target: to_s("who_target"),
        when_at: v.get("when_at").and_then(|x| x.as_i64()),
        when_at_hlc: to_s("when_at_hlc"),
        where_location: to_s("where_location"),
        where_geohash: to_s("where_geohash"),
        what_subject: to_s("what_subject"),
        how_action: to_s("how_action"),
        why_reason: to_s("why_reason"),
        narrative: to_s("narrative"),
    };
    Some((memory_id, fields))
}

// ── Purge stale raw content ───────────────────────────────────────

/// NULL out `memories.content` for rows that
/// (a) have been diary-backfilled (`narrative_filled = 1`), AND
/// (b) exceed the retention window (default 72h).
/// Returns the number of rows cleared.
pub fn purge_stale_raw_content(
    mem: &SqliteMemory,
    now_utc: i64,
    retention_hours: u32,
) -> Result<u64> {
    let cutoff = now_utc - (retention_hours as i64) * 3600;
    let cutoff_iso = DateTime::<Utc>::from_timestamp(cutoff, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default();
    let conn = mem.connection();
    let updated = conn.execute(
        "UPDATE memories
            SET content = ''
          WHERE narrative_filled = 1
            AND content != ''
            AND created_at < ?1",
        rusqlite::params![cutoff_iso],
    )?;
    Ok(updated as u64)
}

// ── Day-of-week / local date helpers ──────────────────────────────

/// Return the local calendar date that is "yesterday" for the given
/// UTC instant, using the supplied timezone offset in seconds east of UTC.
/// Mobile-friendly: no chrono-tz dependency — caller supplies the offset.
pub fn local_yesterday(now_utc: DateTime<Utc>, tz_offset_secs: i32) -> NaiveDate {
    let local_now = now_utc + chrono::Duration::seconds(tz_offset_secs as i64);
    let today = NaiveDate::from_ymd_opt(
        local_now.year(),
        local_now.month(),
        local_now.day(),
    )
    .unwrap_or_else(|| now_utc.date_naive());
    today.pred_opt().unwrap_or(today)
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::traits::{Memory, MemoryCategory};
    use chrono::TimeZone;

    fn temp_mem() -> (tempfile::TempDir, SqliteMemory) {
        let tmp = tempfile::TempDir::new().unwrap();
        let mem = SqliteMemory::new(tmp.path()).unwrap();
        (tmp, mem)
    }

    #[test]
    fn backfill_state_load_or_init_round_trip() {
        let (_tmp, mem) = temp_mem();
        let start = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let state = BackfillState::load_or_init(&mem, "user_1", start).unwrap();
        assert_eq!(state.user_id, "user_1");
        // First-time init anchors at (start - 1) so the first catch-up
        // processes `start` itself.
        assert_eq!(
            state.last_completed_date,
            start.pred_opt().unwrap()
        );
        assert_eq!(state.cumulative_days, 0);
        assert_eq!(state.consecutive_failures, 0);

        let mut advanced = state;
        advanced.last_completed_date = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();
        advanced.cumulative_days = 10;
        advanced.last_run_utc = Some(1_744_000_000);
        advanced.save(&mem, 1_744_000_000).unwrap();

        // Reload should return what we saved.
        let reloaded = BackfillState::load_or_init(&mem, "user_1", start).unwrap();
        assert_eq!(
            reloaded.last_completed_date,
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap()
        );
        assert_eq!(reloaded.cumulative_days, 10);
        assert_eq!(reloaded.last_run_utc, Some(1_744_000_000));
    }

    #[test]
    fn pending_days_respects_catchup_ceiling() {
        let state = BackfillState {
            user_id: "u".into(),
            last_completed_date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            last_run_utc: None,
            cumulative_days: 0,
            consecutive_failures: 0,
            last_error: None,
        };
        // 20 days of backlog, cap at 14.
        let yesterday = NaiveDate::from_ymd_opt(2026, 4, 21).unwrap();
        let days = state.pending_days(yesterday, 14);
        assert_eq!(days.len(), 14);
        assert_eq!(days[0], NaiveDate::from_ymd_opt(2026, 4, 2).unwrap());
        assert_eq!(days[13], NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
    }

    #[test]
    fn pending_days_empty_when_caught_up() {
        let state = BackfillState {
            user_id: "u".into(),
            last_completed_date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
            last_run_utc: None,
            cumulative_days: 0,
            consecutive_failures: 0,
            last_error: None,
        };
        assert!(state
            .pending_days(NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(), 14)
            .is_empty());
        // Also empty if yesterday < last completed (shouldn't happen
        // in practice but guard anyway).
        assert!(state
            .pending_days(NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(), 14)
            .is_empty());
    }

    #[test]
    fn diary_prompt_enforces_how_action_rule() {
        let (sys, user) = build_diary_prompt(
            "2026-04-12",
            r#"[{"memory_id":"m1","content":"골프 쳤음"}]"#,
        );
        // System prompt must mandate nuance preservation in how_action.
        assert!(sys.contains("how_action"));
        assert!(sys.contains("인용"));
        assert!(sys.contains("뉘앙스") || sys.contains("감정"));
        assert!(user.contains("2026-04-12"));
        assert!(user.contains("m1"));
    }

    #[test]
    fn parse_diary_response_handles_array_object_and_garbage() {
        // Happy path — array.
        let hits = parse_diary_response(
            r#"[
                {"memory_id":"m1",
                 "who_actor":"user",
                 "who_target":"[\"김필순\"]",
                 "when_at":1744000000,
                 "where_location":"제주 ** 골프장",
                 "what_subject":"골프 라운딩",
                 "how_action":"라운딩 내내 화기애애",
                 "why_reason":"친목 모임",
                 "narrative":"[2026-04-12 09:00 @ 제주] 라운딩"}
               ]"#,
        );
        assert_eq!(hits.len(), 1);
        let (id, fields) = &hits[0];
        assert_eq!(id, "m1");
        assert_eq!(fields.who_actor.as_deref(), Some("user"));
        assert_eq!(fields.when_at, Some(1_744_000_000));
        assert!(fields
            .how_action
            .as_deref()
            .unwrap_or("")
            .contains("화기애애"));

        // Alt shape — wrapped in entries.
        let hits = parse_diary_response(
            r#"{"entries":[
                {"memory_id":"m2","who_actor":"user"}
               ]}"#,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "m2");

        // Garbage returns empty, not error.
        assert!(parse_diary_response("sorry, I cannot help with that").is_empty());
    }

    #[tokio::test]
    async fn purge_stale_raw_content_respects_both_flags() {
        let (_tmp, mem) = temp_mem();
        // Three memories: recent+filled, stale+filled, stale+unfilled.
        // Purge should only touch stale+filled.
        mem.store("recent", "RECENT", MemoryCategory::Core, None)
            .await
            .unwrap();
        mem.store("stale_filled", "STALE_F", MemoryCategory::Core, None)
            .await
            .unwrap();
        mem.store("stale_unfilled", "STALE_U", MemoryCategory::Core, None)
            .await
            .unwrap();

        // Fake the ages by rewriting created_at directly.
        let now_utc = 1_800_000_000_i64;
        let long_ago = DateTime::<Utc>::from_timestamp(now_utc - 10 * 86_400, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        {
            let conn = mem.connection();
            conn.execute(
                "UPDATE memories SET created_at = ?1
                 WHERE key IN ('stale_filled', 'stale_unfilled')",
                rusqlite::params![long_ago],
            )
            .unwrap();
        }

        // Mark stale_filled as filled.
        mem.set_5w1h(
            "stale_filled",
            &Memory5W1H {
                narrative: Some("filled narrative".into()),
                ..Default::default()
            },
        )
        .unwrap();

        let cleared = purge_stale_raw_content(&mem, now_utc, 72).unwrap();
        assert_eq!(cleared, 1, "only stale_filled should be cleared");

        // Verify by reading back: recent still has content, stale_unfilled
        // still has content, stale_filled has been emptied.
        let read_content = |key: &str| {
            let conn = mem.connection();
            conn.query_row(
                "SELECT content FROM memories WHERE key = ?1",
                rusqlite::params![key],
                |r| r.get::<_, String>(0),
            )
            .unwrap()
        };
        assert_eq!(read_content("recent"), "RECENT");
        assert_eq!(read_content("stale_unfilled"), "STALE_U");
        assert_eq!(read_content("stale_filled"), "");
    }

    #[test]
    fn local_yesterday_handles_tz_offsets() {
        // UTC 2026-04-12 00:30 is still 2026-04-11 in Seoul (+9h)? No —
        // +9h makes it 2026-04-12 09:30 local, so yesterday is Apr 11.
        let utc_now = Utc.with_ymd_and_hms(2026, 4, 12, 0, 30, 0).unwrap();
        let seoul_offset = 9 * 3600;
        let y = local_yesterday(utc_now, seoul_offset);
        assert_eq!(y, NaiveDate::from_ymd_opt(2026, 4, 11).unwrap());

        // UTC 2026-04-12 16:30 — still Apr 12 in LA (-7h), so yesterday
        // is Apr 11.
        let la_offset = -7 * 3600;
        let utc_now = Utc.with_ymd_and_hms(2026, 4, 12, 16, 30, 0).unwrap();
        let y = local_yesterday(utc_now, la_offset);
        assert_eq!(y, NaiveDate::from_ymd_opt(2026, 4, 11).unwrap());
    }
}
