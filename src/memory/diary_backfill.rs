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
use crate::providers::traits::Provider;

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

// ── Provider-driven backfill ──────────────────────────────────────

/// Per-day backfill outcome.
#[derive(Debug, Clone, Copy, Default)]
pub struct DayBackfillReport {
    /// How many `narrative_filled = 0` rows were considered for this day.
    pub considered: usize,
    /// How many of those rows had their 5W1H + narrative columns
    /// populated (i.e. the SLM emitted a parseable entry for them).
    pub filled: usize,
    /// How many rows were marked with one more narrative_failure because
    /// the SLM didn't return an entry for them in this run.
    pub failed: usize,
}

/// Read raw memories created during the local calendar day `date` (in
/// the user's timezone, expressed as `tz_offset_secs` east of UTC),
/// hand them to `provider`, parse the SLM's diary JSON, and persist the
/// 5W1H + narrative columns via [`SqliteMemory::set_5w1h`].
///
/// Skips rows where:
/// - `narrative_filled = 1` already (idempotent)
/// - `narrative_failures >= max_retries` (gives up + flags for review)
/// - `archived = 1` (decay sweep already retired the row)
/// - `content` is empty (already purged or deliberately blank)
///
/// On a successful SLM call but missing entry for some considered row,
/// the missing row's `narrative_failures` is incremented by one so the
/// retry policy eventually escalates it.
pub async fn run_diary_backfill_for_day(
    mem: &SqliteMemory,
    provider: &dyn Provider,
    date: NaiveDate,
    tz_offset_secs: i32,
    model: &str,
    max_retries: u32,
) -> Result<DayBackfillReport> {
    // Compute the UTC range covering the local-day boundaries:
    //   local_midnight - tz_offset → local_midnight + 86400 - tz_offset
    let local_midnight = NaiveDate::from_ymd_opt(date.year(), date.month(), date.day())
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| anyhow::anyhow!("invalid date {date}"))?;
    let start_utc = local_midnight.and_utc().timestamp() - i64::from(tz_offset_secs);
    let end_utc = start_utc + 86_400;
    let start_iso = DateTime::<Utc>::from_timestamp(start_utc, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default();
    let end_iso = DateTime::<Utc>::from_timestamp(end_utc, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default();

    // Pull candidate rows (key + raw content) in one query.
    let candidates: Vec<(String, String)> = {
        let conn = mem.connection();
        let mut stmt = conn.prepare(
            "SELECT key, content
               FROM memories
              WHERE narrative_filled = 0
                AND narrative_failures < ?1
                AND archived = 0
                AND content != ''
                AND created_at >= ?2
                AND created_at <  ?3",
        )?;
        let rows = stmt.query_map(
            rusqlite::params![max_retries as i64, start_iso, end_iso],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        out
    };

    if candidates.is_empty() {
        return Ok(DayBackfillReport::default());
    }

    // Build the JSON array the prompt template expects. We use the
    // memory's `key` as the prompt's `memory_id` so the SLM round-trip
    // gives us back the same identifier that `set_5w1h(key, ...)` needs.
    let payload: Vec<serde_json::Value> = candidates
        .iter()
        .map(|(key, content)| {
            serde_json::json!({
                "memory_id": key,
                "content": content,
            })
        })
        .collect();
    let payload_json = serde_json::to_string(&payload)?;

    let date_label = date.format("%Y-%m-%d").to_string();
    let (system, user) = build_diary_prompt(&date_label, &payload_json);

    // Fixed low temperature — we want deterministic structured extraction,
    // not creative writing. The SLM is just transcribing/structuring, not
    // generating new content.
    let response = provider
        .chat_with_system(Some(&system), &user, model, 0.2)
        .await?;
    let parsed = parse_diary_response(&response);

    // Apply each parsed entry; track which input keys were filled so the
    // remainder can have their failure counter bumped.
    let mut filled_keys = std::collections::HashSet::with_capacity(parsed.len());
    let mut filled = 0usize;
    for (memory_key, fields) in parsed {
        match mem.set_5w1h(&memory_key, &fields) {
            Ok(true) => {
                filled += 1;
                filled_keys.insert(memory_key);
            }
            Ok(false) => {
                // Key didn't match a row — likely an SLM hallucination
                // (made-up id). Don't count it; don't bump anyone.
                tracing::debug!(
                    memory_key,
                    "diary backfill: SLM returned unknown memory_id, ignoring"
                );
            }
            Err(e) => {
                tracing::warn!(memory_key, "diary backfill: set_5w1h failed: {e}");
            }
        }
    }

    let mut failed = 0usize;
    for (key, _) in &candidates {
        if !filled_keys.contains(key) {
            if let Err(e) = mem.bump_narrative_failure(key) {
                tracing::warn!(key, "diary backfill: bump_narrative_failure failed: {e}");
            } else {
                failed += 1;
            }
        }
    }

    Ok(DayBackfillReport {
        considered: candidates.len(),
        filled,
        failed,
    })
}

/// Catch-up sweep: process every pending day from `state.last_completed_date + 1`
/// through `local_yesterday(now_utc, tz_offset_secs)`, bounded by
/// `max_catchup_days`. Persists `BackfillState` after each successful
/// day so a mid-sweep crash resumes cleanly the next night.
///
/// Returns (days_processed, total_memories_filled). On the first SLM
/// failure for a day, the sweep aborts so a single API outage doesn't
/// cascade into N consecutive failure bumps for every memory of every
/// pending day; the next cycle picks up where we left off.
pub async fn run_diary_backfill_catchup(
    mem: &SqliteMemory,
    provider: &dyn Provider,
    user_id: &str,
    model: &str,
    max_catchup_days: u32,
    tz_offset_secs: i32,
    now_utc: DateTime<Utc>,
    default_start_date: NaiveDate,
    max_retries: u32,
) -> Result<(usize, usize)> {
    let mut state = BackfillState::load_or_init(mem, user_id, default_start_date)?;
    let yesterday = local_yesterday(now_utc, tz_offset_secs);
    let pending = state.pending_days(yesterday, max_catchup_days);

    if pending.is_empty() {
        return Ok((0, 0));
    }

    let mut days_done = 0usize;
    let mut total_filled = 0usize;
    let now_unix = now_utc.timestamp();

    for day in pending {
        match run_diary_backfill_for_day(mem, provider, day, tz_offset_secs, model, max_retries)
            .await
        {
            Ok(report) => {
                days_done += 1;
                total_filled += report.filled;
                state.last_completed_date = day;
                state.cumulative_days = state.cumulative_days.saturating_add(1);
                state.last_run_utc = Some(now_unix);
                state.consecutive_failures = 0;
                state.last_error = None;
                if let Err(e) = state.save(mem, now_unix) {
                    tracing::warn!("diary catchup: state.save failed: {e}");
                }
            }
            Err(e) => {
                state.consecutive_failures = state.consecutive_failures.saturating_add(1);
                state.last_error = Some(format!("{e}"));
                state.last_run_utc = Some(now_unix);
                let _ = state.save(mem, now_unix);
                tracing::warn!(
                    day = %day,
                    "diary catchup: aborting at day {day} after SLM failure: {e}"
                );
                break;
            }
        }
    }

    Ok((days_done, total_filled))
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
    use crate::providers::traits::Provider;
    use async_trait::async_trait;
    use chrono::TimeZone;
    use std::sync::{Arc, Mutex};

    fn temp_mem() -> (tempfile::TempDir, SqliteMemory) {
        let tmp = tempfile::TempDir::new().unwrap();
        let mem = SqliteMemory::new(tmp.path()).unwrap();
        (tmp, mem)
    }

    /// Scriptable mock Provider — returns a queued response per call,
    /// records the (system, user, model) tuple so tests can assert on
    /// what the diary backfill actually sent to the SLM.
    struct ScriptedProvider {
        responses: Mutex<Vec<String>>,
        calls: Arc<Mutex<Vec<(Option<String>, String, String)>>>,
    }

    impl ScriptedProvider {
        fn new(responses: Vec<&str>) -> (Self, Arc<Mutex<Vec<(Option<String>, String, String)>>>) {
            let calls = Arc::new(Mutex::new(Vec::new()));
            (
                Self {
                    responses: Mutex::new(responses.iter().map(|s| (*s).to_string()).collect()),
                    calls: calls.clone(),
                },
                calls,
            )
        }
    }

    #[async_trait]
    impl Provider for ScriptedProvider {
        async fn chat_with_system(
            &self,
            system_prompt: Option<&str>,
            message: &str,
            model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            self.calls.lock().unwrap().push((
                system_prompt.map(String::from),
                message.to_string(),
                model.to_string(),
            ));
            let mut q = self.responses.lock().unwrap();
            if q.is_empty() {
                anyhow::bail!("scripted provider: no more queued responses");
            }
            Ok(q.remove(0))
        }
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

    // ── Provider-driven backfill tests ────────────────────────────

    /// Insert a memory whose `created_at` is a specific UTC instant. The
    /// public `store()` API stamps `created_at = Local::now()`, which we
    /// can't control from a test, so we rewrite the column directly after
    /// the row exists.
    async fn insert_memory_at(
        mem: &SqliteMemory,
        key: &str,
        content: &str,
        created_at_utc: i64,
    ) {
        mem.store(key, content, MemoryCategory::Core, None)
            .await
            .unwrap();
        let iso = DateTime::<Utc>::from_timestamp(created_at_utc, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let conn = mem.connection();
        conn.execute(
            "UPDATE memories SET created_at = ?1 WHERE key = ?2",
            rusqlite::params![iso, key],
        )
        .unwrap();
    }

    fn narrative_filled_for(mem: &SqliteMemory, key: &str) -> bool {
        let conn = mem.connection();
        let v: i64 = conn
            .query_row(
                "SELECT narrative_filled FROM memories WHERE key = ?1",
                rusqlite::params![key],
                |r| r.get(0),
            )
            .unwrap_or(0);
        v == 1
    }

    fn narrative_failures_for(mem: &SqliteMemory, key: &str) -> i64 {
        let conn = mem.connection();
        conn.query_row(
            "SELECT narrative_failures FROM memories WHERE key = ?1",
            rusqlite::params![key],
            |r| r.get(0),
        )
        .unwrap_or(0)
    }

    #[tokio::test]
    async fn run_diary_backfill_for_day_fills_all_returned_entries() {
        let (_tmp, mem) = temp_mem();
        // Apr 12 2026 09:00 UTC == 18:00 local in Seoul (+9h). The local
        // calendar date is 2026-04-12.
        let utc_at = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap().timestamp();
        insert_memory_at(&mem, "memo_a", "오늘 골프 쳤음", utc_at).await;
        insert_memory_at(&mem, "memo_b", "회사 미팅", utc_at + 1).await;

        let scripted_response = serde_json::json!([
            {
                "memory_id": "memo_a",
                "who_actor": "user",
                "where_location": "제주 골프장",
                "what_subject": "골프 라운딩",
                "how_action": "친구들과 화기애애",
                "narrative": "[2026-04-12 @ 제주] 골프"
            },
            {
                "memory_id": "memo_b",
                "who_actor": "user",
                "what_subject": "분기 미팅",
                "narrative": "[2026-04-12 @ 회사] 분기 미팅"
            }
        ])
        .to_string();
        let (provider, calls) = ScriptedProvider::new(vec![&scripted_response]);

        let report = run_diary_backfill_for_day(
            &mem,
            &provider,
            NaiveDate::from_ymd_opt(2026, 4, 12).unwrap(),
            9 * 3600, // Seoul tz
            "gemma-4-it",
            DEFAULT_MAX_RETRIES,
        )
        .await
        .unwrap();

        assert_eq!(report.considered, 2);
        assert_eq!(report.filled, 2);
        assert_eq!(report.failed, 0);
        assert!(narrative_filled_for(&mem, "memo_a"));
        assert!(narrative_filled_for(&mem, "memo_b"));

        // Verify the SLM was called once with the 2026-04-12 date label.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        let (sys, user_msg, model) = &recorded[0];
        assert!(sys.as_deref().unwrap_or("").contains("how_action"));
        assert!(user_msg.contains("2026-04-12"));
        assert!(user_msg.contains("memo_a"));
        assert_eq!(model, "gemma-4-it");
    }

    #[tokio::test]
    async fn run_diary_backfill_for_day_bumps_failure_for_missing_entries() {
        let (_tmp, mem) = temp_mem();
        let utc_at = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap().timestamp();
        insert_memory_at(&mem, "memo_a", "골프", utc_at).await;
        insert_memory_at(&mem, "memo_b", "미팅", utc_at + 1).await;

        // SLM only returned an entry for memo_a — memo_b should get its
        // failure counter bumped.
        let response = r#"[{"memory_id":"memo_a","narrative":"a"}]"#;
        let (provider, _calls) = ScriptedProvider::new(vec![response]);

        let report = run_diary_backfill_for_day(
            &mem,
            &provider,
            NaiveDate::from_ymd_opt(2026, 4, 12).unwrap(),
            9 * 3600,
            "gemma-4-it",
            DEFAULT_MAX_RETRIES,
        )
        .await
        .unwrap();

        assert_eq!(report.considered, 2);
        assert_eq!(report.filled, 1);
        assert_eq!(report.failed, 1);
        assert!(narrative_filled_for(&mem, "memo_a"));
        assert!(!narrative_filled_for(&mem, "memo_b"));
        assert_eq!(narrative_failures_for(&mem, "memo_b"), 1);
        assert_eq!(narrative_failures_for(&mem, "memo_a"), 0);
    }

    #[tokio::test]
    async fn run_diary_backfill_for_day_skips_already_filled_and_capped_rows() {
        let (_tmp, mem) = temp_mem();
        let utc_at = Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap().timestamp();
        insert_memory_at(&mem, "memo_done", "이미 채워짐", utc_at).await;
        insert_memory_at(&mem, "memo_capped", "이미 3번 실패", utc_at + 1).await;
        insert_memory_at(&mem, "memo_fresh", "처리 대상", utc_at + 2).await;

        // Pre-fill memo_done; bump memo_capped to threshold.
        mem.set_5w1h(
            "memo_done",
            &Memory5W1H {
                narrative: Some("done".into()),
                ..Default::default()
            },
        )
        .unwrap();
        for _ in 0..DEFAULT_MAX_RETRIES {
            mem.bump_narrative_failure("memo_capped").unwrap();
        }

        let response = r#"[{"memory_id":"memo_fresh","narrative":"f"}]"#;
        let (provider, calls) = ScriptedProvider::new(vec![response]);

        let report = run_diary_backfill_for_day(
            &mem,
            &provider,
            NaiveDate::from_ymd_opt(2026, 4, 12).unwrap(),
            9 * 3600,
            "gemma-4-it",
            DEFAULT_MAX_RETRIES,
        )
        .await
        .unwrap();

        assert_eq!(report.considered, 1, "only memo_fresh should be in scope");
        assert_eq!(report.filled, 1);
        assert!(narrative_filled_for(&mem, "memo_fresh"));

        // Provider was called exactly once, carrying only memo_fresh.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        let user_msg = &recorded[0].1;
        assert!(user_msg.contains("memo_fresh"));
        assert!(!user_msg.contains("memo_done"));
        assert!(!user_msg.contains("memo_capped"));
    }

    #[tokio::test]
    async fn run_diary_backfill_for_day_returns_empty_when_no_candidates() {
        let (_tmp, mem) = temp_mem();
        // No rows inserted for the target date.
        let response = "[]";
        let (provider, calls) = ScriptedProvider::new(vec![response]);

        let report = run_diary_backfill_for_day(
            &mem,
            &provider,
            NaiveDate::from_ymd_opt(2026, 4, 12).unwrap(),
            9 * 3600,
            "gemma-4-it",
            DEFAULT_MAX_RETRIES,
        )
        .await
        .unwrap();

        assert_eq!(report.considered, 0);
        assert_eq!(report.filled, 0);
        // SLM should NOT have been called since there were zero candidates.
        assert!(calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn run_diary_backfill_catchup_walks_pending_days_and_persists_state() {
        let (_tmp, mem) = temp_mem();
        // Three days of memories: Apr 10, 11, 12 (each at 09:00 UTC).
        for (key, day) in [("d10", 10_i64), ("d11", 11), ("d12", 12)] {
            let utc_at = Utc
                .with_ymd_and_hms(2026, 4, day as u32, 9, 0, 0)
                .unwrap()
                .timestamp();
            insert_memory_at(&mem, key, &format!("memo {key}"), utc_at).await;
        }

        // Three days × one row each = three SLM calls.
        let r10 = r#"[{"memory_id":"d10","narrative":"x"}]"#;
        let r11 = r#"[{"memory_id":"d11","narrative":"x"}]"#;
        let r12 = r#"[{"memory_id":"d12","narrative":"x"}]"#;
        let (provider, calls) = ScriptedProvider::new(vec![r10, r11, r12]);

        // "now" = 2026-04-13 14:00 UTC, Seoul tz → local = 2026-04-13 23:00,
        // local_yesterday = 2026-04-12. Default start = 2026-04-10 (so the
        // first eligible day is 2026-04-10).
        let now_utc = Utc.with_ymd_and_hms(2026, 4, 13, 14, 0, 0).unwrap();
        let (days, filled) = run_diary_backfill_catchup(
            &mem,
            &provider,
            "user_1",
            "gemma-4-it",
            DEFAULT_MAX_CATCHUP_DAYS,
            9 * 3600,
            now_utc,
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
            DEFAULT_MAX_RETRIES,
        )
        .await
        .unwrap();

        assert_eq!(days, 3);
        assert_eq!(filled, 3);
        assert_eq!(calls.lock().unwrap().len(), 3);
        assert!(narrative_filled_for(&mem, "d10"));
        assert!(narrative_filled_for(&mem, "d11"));
        assert!(narrative_filled_for(&mem, "d12"));

        // BackfillState should now point at Apr 12 (== local_yesterday).
        let reloaded = BackfillState::load_or_init(
            &mem,
            "user_1",
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
        )
        .unwrap();
        assert_eq!(
            reloaded.last_completed_date,
            NaiveDate::from_ymd_opt(2026, 4, 12).unwrap()
        );
        assert_eq!(reloaded.cumulative_days, 3);
        assert_eq!(reloaded.consecutive_failures, 0);

        // Re-running catchup immediately is a no-op — state is current.
        let (provider2, _) = ScriptedProvider::new(vec![]);
        let (days2, filled2) = run_diary_backfill_catchup(
            &mem,
            &provider2,
            "user_1",
            "gemma-4-it",
            DEFAULT_MAX_CATCHUP_DAYS,
            9 * 3600,
            now_utc,
            NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
            DEFAULT_MAX_RETRIES,
        )
        .await
        .unwrap();
        assert_eq!(days2, 0);
        assert_eq!(filled2, 0);
    }

    #[tokio::test]
    async fn run_diary_backfill_catchup_aborts_on_provider_failure_and_records_error() {
        let (_tmp, mem) = temp_mem();
        let utc_at = Utc.with_ymd_and_hms(2026, 4, 11, 9, 0, 0).unwrap().timestamp();
        insert_memory_at(&mem, "d11", "memo", utc_at).await;
        insert_memory_at(
            &mem,
            "d12",
            "memo",
            Utc.with_ymd_and_hms(2026, 4, 12, 9, 0, 0).unwrap().timestamp(),
        )
        .await;

        // Empty queue → first chat_with_system call errors out.
        let (provider, _) = ScriptedProvider::new(vec![]);
        let now_utc = Utc.with_ymd_and_hms(2026, 4, 13, 14, 0, 0).unwrap();
        let (days, filled) = run_diary_backfill_catchup(
            &mem,
            &provider,
            "user_1",
            "gemma-4-it",
            DEFAULT_MAX_CATCHUP_DAYS,
            9 * 3600,
            now_utc,
            NaiveDate::from_ymd_opt(2026, 4, 11).unwrap(),
            DEFAULT_MAX_RETRIES,
        )
        .await
        .unwrap();

        assert_eq!(days, 0, "abort before completing any day");
        assert_eq!(filled, 0);

        // Failure was recorded in BackfillState.
        let reloaded = BackfillState::load_or_init(
            &mem,
            "user_1",
            NaiveDate::from_ymd_opt(2026, 4, 11).unwrap(),
        )
        .unwrap();
        assert!(reloaded.consecutive_failures >= 1);
        assert!(reloaded.last_error.is_some());
    }
}
