//! Q1 Commit #10 — 5-dimensional cross-recall over the First Brain.
//!
//! `SqliteMemory::cross_recall` (in `sqlite.rs`) fans a query out across
//! five complementary search dimensions defined in this module, then
//! fuses the per-dimension rankings with Reciprocal Rank Fusion (RRF):
//!
//! | Dim | Source                                           | Strength             |
//! |-----|--------------------------------------------------|----------------------|
//! |  1  | `memories_fts` (raw content)                     | exact keyword recall |
//! |  2  | `memories_narrative_fts.narrative`               | Dream-Cycle distilled summary |
//! |  3  | `first_brain_pages_fts` + `first_brain_links` BFS | LLM-wiki graph context |
//! |  4  | `memories_narrative_fts.{who/what/how/why}`      | structured 5W1H cue   |
//! |  5  | `memories.embedding` (cosine)                    | semantic similarity   |
//!
//! Profile auto-selection: when [`SqliteMemory::embedder_is_active`]
//! returns false (e.g. mobile build with `NoopEmbedding`), the vector
//! dimension is silently dropped. The remaining four text dimensions are
//! sufficient on their own — Karpathy-style — because the Dream Cycle
//! has already distilled raw turns into narratives + 5W1H + wiki pages.
//!
//! See `ARCHITECTURE.md §Q1-5W1H / §First-Brain Wiki` for the design
//! rationale and the "지난주 골프장 누구와" walkthrough.
//!
//! [`SqliteMemory::embedder_is_active`]: super::sqlite::SqliteMemory::embedder_is_active

use std::collections::HashMap;

use anyhow::Result;
use rusqlite::{params, Connection};

use super::traits::MemoryEntry;

// ── Public types ───────────────────────────────────────────────────

/// Which fan-out profile to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecallProfile {
    /// Pick `Mobile` when no embedder is loaded, otherwise `Desktop`.
    Auto,
    /// Skip the vector dimension; keep the four text dimensions.
    Mobile,
    /// All five dimensions including vector cosine similarity.
    Desktop,
}

impl Default for RecallProfile {
    fn default() -> Self {
        Self::Auto
    }
}

/// Per-dimension contribution (for debugging / introspection).
/// Each entry is the raw rank-1 score from that dimension when this row
/// matched it, or `None` when the dimension didn't return this row.
#[derive(Debug, Clone, Default)]
pub struct DimScores {
    pub raw_fts: Option<f32>,
    pub narrative_fts: Option<f32>,
    pub wiki_bfs: Option<f32>,
    pub structured: Option<f32>,
    pub vector: Option<f32>,
}

impl DimScores {
    /// Number of dimensions that returned this row (0..=5).
    pub fn matched_count(&self) -> u8 {
        u8::from(self.raw_fts.is_some())
            + u8::from(self.narrative_fts.is_some())
            + u8::from(self.wiki_bfs.is_some())
            + u8::from(self.structured.is_some())
            + u8::from(self.vector.is_some())
    }
}

#[derive(Debug, Clone)]
pub struct CrossRecallResult {
    pub entry: MemoryEntry,
    pub final_score: f32,
    pub dim_scores: DimScores,
}

/// RRF constant. 60 is the de-facto default in the IR literature; tuned
/// to flatten extreme rank-1 outliers without diluting low-rank hits.
pub(crate) const DEFAULT_RRF_K: f32 = 60.0;

/// How deep to fan a single dimension before fusing. Two options trade
/// off recall vs latency — we default to `limit * 4` so the fuser has
/// enough candidates to produce a stable top-N after dedup.
pub(crate) fn per_dim_depth(limit: usize) -> usize {
    (limit * 4).max(20)
}

// ── Profile resolution ─────────────────────────────────────────────

/// Resolve `Auto` to a concrete profile using the embedder availability
/// signal. Concrete profiles pass through unchanged so callers can force
/// Mobile-mode on a desktop build (useful for offline / battery testing).
pub fn resolve_profile(profile: RecallProfile, embedder_active: bool) -> RecallProfile {
    match profile {
        RecallProfile::Auto => {
            if embedder_active {
                RecallProfile::Desktop
            } else {
                RecallProfile::Mobile
            }
        }
        other => other,
    }
}

// ── Query term escaping ───────────────────────────────────────────

pub(crate) fn fts5_query_terms(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|w| !w.is_empty())
        .map(|w| {
            // FTS5 phrase syntax — escape embedded double quotes by
            // doubling them per FTS5 grammar.
            let escaped = w.replace('"', "\"\"");
            format!("\"{escaped}\"")
        })
        .collect::<Vec<_>>()
        .join(" OR ")
}

// ── Dim 1: raw content FTS5 ────────────────────────────────────────

pub(crate) fn dim_raw_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, f32)>> {
    let fts_query = fts5_query_terms(query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare_cached(
        "SELECT m.id, bm25(memories_fts) AS score
           FROM memories_fts f
           JOIN memories     m ON m.rowid = f.rowid
          WHERE memories_fts MATCH ?1
            AND m.archived = 0
          ORDER BY score
          LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![fts_query, limit as i64], |r| {
        let id: String = r.get(0)?;
        let raw: f64 = r.get(1)?;
        // BM25 is negative in SQLite's FTS5 (lower = better); negate so
        // higher = better, matching the convention of every other dim.
        #[allow(clippy::cast_possible_truncation)]
        Ok((id, (-raw) as f32))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// ── Dim 2: narrative FTS5 (Dream-Cycle distilled diary sentence) ───

pub(crate) fn dim_narrative_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, f32)>> {
    let inner = fts5_query_terms(query);
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    // Column-restrict the FTS5 query to the `narrative` column only.
    // FTS5 syntax: `{narrative}: <terms>`.
    let scoped = format!("{{narrative}}: {inner}");
    let mut stmt = conn.prepare_cached(
        "SELECT m.id, bm25(memories_narrative_fts) AS score
           FROM memories_narrative_fts f
           JOIN memories                m ON m.rowid = f.rowid
          WHERE memories_narrative_fts MATCH ?1
            AND m.archived = 0
            AND m.narrative_filled = 1
          ORDER BY score
          LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![scoped, limit as i64], |r| {
        let id: String = r.get(0)?;
        let raw: f64 = r.get(1)?;
        #[allow(clippy::cast_possible_truncation)]
        Ok((id, (-raw) as f32))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// ── Dim 3: First-Brain Wiki BFS ────────────────────────────────────

/// Search the wiki page index, then take one BFS hop along
/// `first_brain_links` to surface neighbours of every hit. Each surfaced
/// page contributes its `memory_id` (when set) into the result list.
/// The score is the wiki page BM25 score, attenuated by hop distance.
pub(crate) fn dim_wiki_bfs(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, f32)>> {
    let fts_query = fts5_query_terms(query);
    if fts_query.is_empty() {
        return Ok(Vec::new());
    }

    // Step 1 — direct page matches.
    let mut stmt = conn.prepare_cached(
        "SELECT p.id, p.memory_id, bm25(first_brain_pages_fts) AS score
           FROM first_brain_pages_fts f
           JOIN first_brain_pages     p ON p.id = f.rowid
          WHERE first_brain_pages_fts MATCH ?1
          ORDER BY score
          LIMIT ?2",
    )?;
    let direct_rows = stmt.query_map(params![fts_query, limit as i64], |r| {
        let pid: i64 = r.get(0)?;
        let mid: Option<String> = r.get(1)?;
        let raw: f64 = r.get(2)?;
        #[allow(clippy::cast_possible_truncation)]
        Ok((pid, mid, (-raw) as f32))
    })?;

    let mut by_memory: HashMap<String, f32> = HashMap::new();
    let mut hop_seeds: Vec<(i64, f32)> = Vec::new();
    for row in direct_rows {
        let (pid, mid, score) = row?;
        if let Some(memory_id) = mid {
            // Take the max — a memory referenced by multiple pages keeps
            // the strongest contribution.
            let entry = by_memory.entry(memory_id).or_insert(0.0);
            if score > *entry {
                *entry = score;
            }
        }
        hop_seeds.push((pid, score));
    }

    // Step 2 — one BFS hop along outbound links. Pull connected pages,
    // attenuate the score by 0.5 to model graph distance.
    if !hop_seeds.is_empty() {
        let placeholders: String = (1..=hop_seeds.len())
            .map(|i| format!("?{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!(
            "SELECT l.source_page_id, p.memory_id
               FROM first_brain_links l
               JOIN first_brain_pages p ON p.id = l.target_page_id
              WHERE l.source_page_id IN ({placeholders})
                AND l.target_page_id IS NOT NULL
                AND p.memory_id IS NOT NULL"
        );
        let mut hop_stmt = conn.prepare(&sql)?;
        let id_params: Vec<Box<dyn rusqlite::types::ToSql>> = hop_seeds
            .iter()
            .map(|(pid, _)| Box::new(*pid) as Box<dyn rusqlite::types::ToSql>)
            .collect();
        let p_refs: Vec<&dyn rusqlite::types::ToSql> =
            id_params.iter().map(AsRef::as_ref).collect();
        let seed_score: HashMap<i64, f32> = hop_seeds.iter().copied().collect();
        let hop_rows = hop_stmt.query_map(p_refs.as_slice(), |r| {
            let src: i64 = r.get(0)?;
            let mid: Option<String> = r.get(1)?;
            Ok((src, mid))
        })?;
        for row in hop_rows {
            let (src, mid) = row?;
            if let Some(memory_id) = mid {
                let attenuated = seed_score.get(&src).copied().unwrap_or(0.0) * 0.5;
                let entry = by_memory.entry(memory_id).or_insert(0.0);
                if attenuated > *entry {
                    *entry = attenuated;
                }
            }
        }
    }

    let mut scored: Vec<(String, f32)> = by_memory.into_iter().collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored)
}

// ── Dim 4: structured 5W1H FTS5 (who/what/how/why columns) ────────

pub(crate) fn dim_structured_fts(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<(String, f32)>> {
    let inner = fts5_query_terms(query);
    if inner.is_empty() {
        return Ok(Vec::new());
    }
    // Column-restrict to the four structured columns, EXCLUDING the
    // `narrative` column (Dim 2 owns that). FTS5 column-filter syntax
    // accepts a brace-grouped list before the colon.
    let scoped = format!("{{who_target what_subject how_action why_reason}}: {inner}");
    let mut stmt = conn.prepare_cached(
        "SELECT m.id, bm25(memories_narrative_fts) AS score
           FROM memories_narrative_fts f
           JOIN memories                m ON m.rowid = f.rowid
          WHERE memories_narrative_fts MATCH ?1
            AND m.archived = 0
            AND m.narrative_filled = 1
          ORDER BY score
          LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![scoped, limit as i64], |r| {
        let id: String = r.get(0)?;
        let raw: f64 = r.get(1)?;
        #[allow(clippy::cast_possible_truncation)]
        Ok((id, (-raw) as f32))
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

// ── Dim 5: vector cosine similarity ────────────────────────────────

pub(crate) fn dim_vector(
    conn: &Connection,
    embedding: &[f32],
    limit: usize,
) -> Result<Vec<(String, f32)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, embedding
           FROM memories
          WHERE embedding IS NOT NULL AND archived = 0",
    )?;
    let rows = stmt.query_map([], |r| {
        let id: String = r.get(0)?;
        let blob: Vec<u8> = r.get(1)?;
        Ok((id, blob))
    })?;
    let mut scored: Vec<(String, f32)> = Vec::new();
    for r in rows {
        let (id, blob) = r?;
        let emb = super::vector::bytes_to_vec(&blob);
        let sim = super::vector::cosine_similarity(embedding, &emb);
        if sim > 0.0 {
            scored.push((id, sim));
        }
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored)
}

// ── 5-way RRF fusion ───────────────────────────────────────────────

/// Per-id state during fusion: accumulating final score plus the per-dim
/// rank-1 scores for inspection.
#[derive(Debug, Clone)]
pub(crate) struct FusedRow {
    pub(crate) id: String,
    pub(crate) final_score: f32,
    pub(crate) dims: DimScores,
}

pub(crate) fn fuse_five_way(
    raw: &[(String, f32)],
    narrative: &[(String, f32)],
    wiki: &[(String, f32)],
    structured: &[(String, f32)],
    vector: &[(String, f32)],
    limit: usize,
) -> Vec<FusedRow> {
    let mut map: HashMap<String, FusedRow> = HashMap::new();

    let mut accumulate = |list: &[(String, f32)], assign: fn(&mut DimScores, f32)| {
        for (rank_zero, (id, score)) in list.iter().enumerate() {
            let rank = rank_zero + 1;
            #[allow(clippy::cast_precision_loss)]
            let rrf = 1.0 / (DEFAULT_RRF_K + rank as f32);
            let entry = map.entry(id.clone()).or_insert_with(|| FusedRow {
                id: id.clone(),
                final_score: 0.0,
                dims: DimScores::default(),
            });
            entry.final_score += rrf;
            assign(&mut entry.dims, *score);
        }
    };

    accumulate(raw, |d, s| d.raw_fts = Some(s));
    accumulate(narrative, |d, s| d.narrative_fts = Some(s));
    accumulate(wiki, |d, s| d.wiki_bfs = Some(s));
    accumulate(structured, |d, s| d.structured = Some(s));
    accumulate(vector, |d, s| d.vector = Some(s));

    let mut out: Vec<FusedRow> = map.into_values().collect();
    out.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out.truncate(limit);
    out
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_profile_auto_picks_mobile_when_embedder_inactive() {
        assert_eq!(resolve_profile(RecallProfile::Auto, false), RecallProfile::Mobile);
        assert_eq!(resolve_profile(RecallProfile::Auto, true), RecallProfile::Desktop);
    }

    #[test]
    fn resolve_profile_passes_concrete_profiles_through() {
        assert_eq!(resolve_profile(RecallProfile::Mobile, true), RecallProfile::Mobile);
        assert_eq!(resolve_profile(RecallProfile::Desktop, false), RecallProfile::Desktop);
    }

    #[test]
    fn fts5_query_terms_uses_or_join() {
        let q = fts5_query_terms("alpha beta gamma");
        assert!(q.contains("\"alpha\""));
        assert!(q.contains("\"beta\""));
        assert!(q.contains("\"gamma\""));
        assert!(q.contains(" OR "));
    }

    #[test]
    fn fts5_query_terms_empty_input_returns_empty() {
        assert!(fts5_query_terms("").is_empty());
        assert!(fts5_query_terms("   ").is_empty());
    }

    #[test]
    fn dim_scores_matched_count_sums_present_dims() {
        let mut s = DimScores::default();
        assert_eq!(s.matched_count(), 0);
        s.raw_fts = Some(1.0);
        s.narrative_fts = Some(1.0);
        s.vector = Some(1.0);
        assert_eq!(s.matched_count(), 3);
    }

    #[test]
    fn fuse_five_way_rrf_prefers_multi_dim_hits() {
        // Document A appears in all five dims at rank 1; B only in raw at rank 1.
        // A's RRF score must dominate B's.
        let make = |id: &str| (id.to_string(), 1.0_f32);
        let raw = vec![make("A"), make("B")];
        let narrative = vec![make("A")];
        let wiki = vec![make("A")];
        let structured = vec![make("A")];
        let vector = vec![make("A")];
        let fused = fuse_five_way(&raw, &narrative, &wiki, &structured, &vector, 10);
        assert_eq!(fused.len(), 2);
        assert_eq!(fused[0].id, "A");
        assert_eq!(fused[1].id, "B");
        assert!(fused[0].final_score > fused[1].final_score);
        assert_eq!(fused[0].dims.matched_count(), 5);
        assert_eq!(fused[1].dims.matched_count(), 1);
    }

    #[test]
    fn fuse_five_way_handles_all_empty_inputs() {
        let fused = fuse_five_way(&[], &[], &[], &[], &[], 10);
        assert!(fused.is_empty());
    }

    #[test]
    fn fuse_five_way_truncates_to_limit() {
        let xs: Vec<(String, f32)> = (0..20).map(|i| (format!("doc-{i}"), 1.0)).collect();
        let fused = fuse_five_way(&xs, &[], &[], &[], &[], 5);
        assert_eq!(fused.len(), 5);
    }

    #[test]
    fn per_dim_depth_floors_at_twenty() {
        assert_eq!(per_dim_depth(0), 20);
        assert_eq!(per_dim_depth(3), 20);
        assert_eq!(per_dim_depth(10), 40);
    }
}
