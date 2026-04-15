// @Ref: SUMMARY §6D-6 — structure-mapped hub note engine (Phase 2 MVP).
//
// Hub compilation is bottom-up and driven by backlink accumulation.
// When the number of inbound wikilinks to an entity crosses a threshold
// (default 5) it becomes hub-worthy and is queued for compile. Compile
// classifies the entity type (statute / person / case / general concept),
// renders a skeleton template, maps backlinked docs into each section,
// warns on empty sections (Evidence Gap), and persists the markdown back
// to `hub_notes.content_md`. On-demand trigger only in this Phase; Phase 5
// adds idle-time automatic dispatch.

use super::store::VaultStore;
use crate::vault::wikilink::tokens::{detect_compound_tokens, CompoundTokenKind};
use anyhow::Result;
use rusqlite::params;
use std::collections::HashMap;

fn unix_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Default minimum backlink count that qualifies an entity for hub compile.
pub const HUB_THRESHOLD_DEFAULT: i64 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HubSubtype {
    /// §6D-6 법조문 뼈대: 조문 → 요건사실 → 법적효과 → 관련조문
    StatuteArticle,
    /// 인물 뼈대: 프로필 → 관련인물 → 관련사건 → 행위 시계열
    Person,
    /// 사건 뼈대: 6하원칙
    Case,
    /// 일반 개념 뼈대: 정의 → 하위분류 → 장단점 → 적용사례
    GeneralConcept,
}

impl HubSubtype {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StatuteArticle => "statute_article",
            Self::Person => "person",
            Self::Case => "case",
            Self::GeneralConcept => "general_concept",
        }
    }

    /// Heuristic classifier. Uses the wikilink compound-token detector
    /// on the entity name itself.
    pub fn classify(entity_name: &str) -> Self {
        let toks = detect_compound_tokens(entity_name);
        for t in &toks {
            match t.kind {
                CompoundTokenKind::StatuteArticle => return Self::StatuteArticle,
                CompoundTokenKind::CaseNumber | CompoundTokenKind::PrecedentCitation => {
                    return Self::Case
                }
                CompoundTokenKind::Organization => return Self::Person,
            }
        }
        // Lightweight person check: Korean surname-prefixed short name.
        static KR_SURNAMES: &[&str] = &[
            "김", "이", "박", "최", "정", "강", "조", "윤", "장", "임", "한", "오",
            "서", "신", "권", "황", "안", "송", "류", "전", "홍", "고", "문", "양",
            "손", "배", "백", "허", "남", "심",
        ];
        let t = entity_name.trim();
        if let Some(first) = t.chars().next() {
            if KR_SURNAMES.contains(&first.to_string().as_str())
                && t.chars().count() >= 2
                && t.chars().count() <= 4
            {
                return Self::Person;
            }
        }
        Self::GeneralConcept
    }
}

#[derive(Debug, Clone)]
pub struct HubCompileReport {
    pub entity_name: String,
    pub subtype: HubSubtype,
    pub backlink_count: i64,
    pub sections: usize,
    /// Number of skeleton sections with zero mapped documents.
    /// Surfaced to the UI as Evidence Gap warnings.
    pub evidence_gaps: usize,
    pub markdown: String,
}

/// Accumulate backlink counts from every row in `vault_links` into the
/// `hub_notes` table (upsert). Runs in O(links) — called after bulk
/// ingest or on demand. Hubs below `threshold` are still created but
/// marked with `importance_score=0`.
pub fn refresh_backlink_counts(vault: &VaultStore) -> Result<()> {
    let conn = vault.connection().lock();
    // Aggregate per target_raw.
    let mut stmt = conn.prepare(
        "SELECT target_raw, COUNT(*) FROM vault_links GROUP BY target_raw",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();
    drop(stmt);

    for (entity, count) in rows {
        let subtype = HubSubtype::classify(&entity);
        conn.execute(
            "INSERT INTO hub_notes
                (entity_name, hub_subtype, backlink_count, importance_score,
                 hub_threshold, pending_backlinks)
             VALUES (?1, ?2, ?3, ?4, ?5, ?3)
             ON CONFLICT(entity_name) DO UPDATE SET
                backlink_count = excluded.backlink_count,
                hub_subtype = excluded.hub_subtype,
                pending_backlinks = excluded.backlink_count,
                importance_score = excluded.importance_score",
            params![
                entity,
                subtype.as_str(),
                count,
                (count as f64).log2().max(0.0),
                HUB_THRESHOLD_DEFAULT,
            ],
        )?;
    }
    Ok(())
}

/// List entities that have crossed the threshold and are awaiting compile.
pub fn list_compile_candidates(vault: &VaultStore) -> Result<Vec<(String, i64)>> {
    let conn = vault.connection().lock();
    let mut stmt = conn.prepare(
        "SELECT entity_name, backlink_count FROM hub_notes
         WHERE backlink_count >= hub_threshold
         ORDER BY backlink_count DESC",
    )?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Compute the priority score for one hub candidate. Plan §7-5:
///   0.4 × backlinks (normalised)
/// + 0.3 × usage frequency (approximate via `pending_backlinks` staleness)
/// + 0.2 × time since last compile (recency penalty)
/// + 0.1 × pending backlinks ratio
///
/// Higher score = compile sooner.
pub fn priority_score(
    backlinks: i64,
    pending: i64,
    last_compiled_epoch: Option<u64>,
    now_epoch: u64,
) -> f32 {
    let backlink_component = (backlinks as f32).log2().max(0.0) / 10.0; // 1024 links → ~1.0
    let pending_ratio = if backlinks > 0 {
        pending as f32 / backlinks as f32
    } else {
        0.0
    };
    let time_component = match last_compiled_epoch {
        None => 1.0, // never compiled — highest urgency
        Some(t) => {
            let days_since = (now_epoch.saturating_sub(t) as f32) / 86_400.0;
            (days_since / 30.0).clamp(0.0, 1.0)
        }
    };
    // "usage frequency" proxy: hubs with a big pending_backlinks delta
    // are being referenced more than compiled; boost them.
    let usage_component = pending_ratio;

    0.4 * backlink_component
        + 0.3 * usage_component
        + 0.2 * time_component
        + 0.1 * pending_ratio
}

/// Pull the next hub from the compile queue by priority order. Skips hubs
/// below threshold. Returns `Ok(None)` when the queue is empty.
pub fn compile_queue_next(vault: &VaultStore) -> Result<Option<String>> {
    let conn = vault.connection().lock();
    let now = unix_epoch();
    let mut stmt = conn.prepare(
        "SELECT entity_name, backlink_count, pending_backlinks, last_compiled
         FROM hub_notes
         WHERE backlink_count >= hub_threshold",
    )?;
    let rows: Vec<(String, i64, i64, Option<i64>)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<i64>>(3)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    let best = rows
        .into_iter()
        .map(|(name, bl, pending, last)| {
            let score = priority_score(
                bl,
                pending,
                last.map(|t| t as u64),
                now,
            );
            (name, score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    Ok(best.map(|(name, _)| name))
}

/// Compile one hub note on demand. Idempotent — running twice overwrites
/// `content_md` with the latest render and updates `last_compiled`.
pub fn compile_hub(vault: &VaultStore, entity_name: &str) -> Result<HubCompileReport> {
    let subtype = HubSubtype::classify(entity_name);
    let backlinking_docs = fetch_backlinking_docs(vault, entity_name)?;
    let (markdown, sections, gaps) = render(subtype, entity_name, &backlinking_docs);

    let conn = vault.connection().lock();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    conn.execute(
        "INSERT INTO hub_notes
            (entity_name, hub_subtype, backlink_count, compile_type,
             last_compiled, structure_elements, mapped_documents,
             hub_threshold, content_md)
         VALUES (?1, ?2, ?3, 'full', ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(entity_name) DO UPDATE SET
            hub_subtype = excluded.hub_subtype,
            backlink_count = excluded.backlink_count,
            compile_type = 'full',
            last_compiled = excluded.last_compiled,
            structure_elements = excluded.structure_elements,
            mapped_documents = excluded.mapped_documents,
            pending_backlinks = 0,
            content_md = excluded.content_md",
        params![
            entity_name,
            subtype.as_str(),
            backlinking_docs.len() as i64,
            now as i64,
            sections as i64,
            backlinking_docs.len() as i64,
            HUB_THRESHOLD_DEFAULT,
            markdown,
        ],
    )?;

    Ok(HubCompileReport {
        entity_name: entity_name.to_string(),
        subtype,
        backlink_count: backlinking_docs.len() as i64,
        sections,
        evidence_gaps: gaps,
        markdown,
    })
}

/// Fetch source docs (title + id) that link to `entity_name`.
fn fetch_backlinking_docs(vault: &VaultStore, entity_name: &str) -> Result<Vec<(i64, String)>> {
    let conn = vault.connection().lock();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT d.id, COALESCE(d.title, CAST(d.id AS TEXT))
         FROM vault_links l JOIN vault_documents d ON d.id = l.source_doc_id
         WHERE l.target_raw = ?1
         ORDER BY d.id ASC",
    )?;
    let rows = stmt
        .query_map(params![entity_name], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Render skeleton markdown per subtype. Sections without mapped docs
/// emit ⚠️ Evidence Gap warnings.
fn render(
    subtype: HubSubtype,
    entity: &str,
    docs: &[(i64, String)],
) -> (String, usize, usize) {
    let skeleton: &[&str] = match subtype {
        HubSubtype::StatuteArticle => &[
            "조문 원문 / 정의",
            "요건사실",
            "법적 효과",
            "관련 조문 체계",
            "판례 / 해설 (백링크 종합)",
        ],
        HubSubtype::Person => &[
            "프로필",
            "관련 인물",
            "관련 사건",
            "주요 행위 시계열",
        ],
        HubSubtype::Case => &[
            "① 누가 (당사자 관계)",
            "② 언제 (시계열)",
            "③ 어디서 (장소 / 관할)",
            "④ 무엇을 (청구 취지)",
            "⑤ 어떻게 (행위 경위)",
            "⑥ 왜 (법적 근거)",
            "⑦ 쟁점 구조",
        ],
        HubSubtype::GeneralConcept => &[
            "정의",
            "하위 분류",
            "장점 / 단점",
            "적용 사례",
            "관련 개념 비교",
        ],
    };

    // Naive even-distribution mapping: each doc goes to ONE section
    // chosen by (doc_id mod sections). Phase 3 will replace this with
    // LLM-driven section assignment.
    let section_count = skeleton.len();
    let mut section_docs: HashMap<usize, Vec<&(i64, String)>> = HashMap::new();
    for d in docs {
        let idx = (d.0.unsigned_abs() as usize) % section_count;
        section_docs.entry(idx).or_default().push(d);
    }

    let mut md = String::with_capacity(512 + docs.len() * 32);
    md.push_str(&format!("# {entity}\n\n"));
    md.push_str(&format!(
        "> **Hub subtype**: `{}` · **Backlinks**: {}\n\n",
        subtype.as_str(),
        docs.len()
    ));

    let mut gaps = 0usize;
    for (idx, section_title) in skeleton.iter().enumerate() {
        let mapped = section_docs.get(&idx).cloned().unwrap_or_default();
        if mapped.is_empty() {
            md.push_str(&format!("## {section_title}\n\n"));
            md.push_str("⚠️ **Evidence Gap** — 매핑된 문서 0건.\n\n");
            gaps += 1;
        } else {
            md.push_str(&format!(
                "## {section_title}\n\n📎 {}건: {}\n\n",
                mapped.len(),
                mapped
                    .iter()
                    .map(|(id, t)| format!("[Doc-{id}] {t}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    (md, section_count, gaps)
}

// ── §7-6 Conflict resolution (3-tier) ─────────────────────────────────

/// Authority ranking of document types per Plan §7-6:
/// 판결문 > 준비서면 > 메모 (and everything else).
/// Higher = more authoritative.
pub fn doc_authority_rank(doc_type: &str) -> i32 {
    match doc_type.to_lowercase().as_str() {
        "판결문" | "judgment" | "ruling" => 100,
        "결정문" | "decision" => 90,
        "합의서" | "계약서" | "contract" => 80,
        "준비서면" | "brief" => 60,
        "소장" | "answer" | "complaint" => 55,
        "진술서" | "affidavit" | "statement" => 50,
        "메모" | "note" | "memo" => 20,
        _ => 30,
    }
}

/// Source-reliability ranking per Plan §7-6:
/// 법원발행 > 상대방제출 > 내부메모
pub fn source_reliability_rank(source: &str) -> i32 {
    match source.to_lowercase().as_str() {
        "법원" | "court" | "official" => 100,
        "공공기관" | "agency" | "government" => 80,
        "상대방" | "opposing" | "counterparty" => 50,
        "내부" | "internal" | "self" => 30,
        _ => 40,
    }
}

#[derive(Debug, Clone)]
pub struct ConflictingClaim {
    pub doc_id: i64,
    pub title: String,
    pub doc_type: String,
    pub source: String,
    /// ISO timestamp or epoch; later wins at equal authority.
    pub doc_date: Option<String>,
    pub content_snippet: String,
}

/// Resolve a conflict between competing claims about an entity.
/// Returns the `Vec<usize>` indices of `claims` ordered from most to
/// least authoritative. Tiebreaker order:
///   1. doc_authority_rank (desc)
///   2. doc_date (desc — newer wins)
///   3. source_reliability_rank (desc)
///   4. doc_id (asc — deterministic)
pub fn resolve_conflict(claims: &[ConflictingClaim]) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..claims.len()).collect();
    indices.sort_by(|&a, &b| {
        let ca = &claims[a];
        let cb = &claims[b];
        doc_authority_rank(&cb.doc_type)
            .cmp(&doc_authority_rank(&ca.doc_type))
            .then_with(|| cb.doc_date.cmp(&ca.doc_date))
            .then_with(|| {
                source_reliability_rank(&cb.source).cmp(&source_reliability_rank(&ca.source))
            })
            .then_with(|| ca.doc_id.cmp(&cb.doc_id))
    });
    indices
}

// ── §7-7 Impact-based incremental update (Light/Heavy/Full Rebuild) ──

/// The scope of an incremental hub update. Plan §7-7:
/// - Light: one existing section affected
/// - Heavy: multiple sections + aggregate re-compute
/// - Full Rebuild: skeleton itself changed (subtype migration)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel {
    Light,
    Heavy,
    FullRebuild,
}

/// Classify the impact of a new backlink arriving for `entity_name`.
///
/// Heuristic:
/// - If the current skeleton subtype no longer matches the canonical
///   classification of the entity (e.g. a previously-generic entity is
///   now revealed as a statute) → **FullRebuild**.
/// - If the new doc maps to ≥2 sections of the current skeleton (i.e.
///   it's a multi-section reference) → **Heavy**.
/// - Otherwise → **Light**.
pub fn classify_impact(vault: &VaultStore, entity_name: &str) -> Result<ImpactLevel> {
    let (current_subtype_str, last_section_count) = {
        let conn = vault.connection().lock();
        conn.query_row(
            "SELECT COALESCE(hub_subtype,''), COALESCE(structure_elements,0)
             FROM hub_notes WHERE entity_name = ?1",
            params![entity_name],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
        )
        .unwrap_or_else(|_| ("".into(), 0))
    };

    let expected_subtype = HubSubtype::classify(entity_name);
    if !current_subtype_str.is_empty()
        && current_subtype_str != expected_subtype.as_str()
    {
        return Ok(ImpactLevel::FullRebuild);
    }

    // Count backlinks that mention >1 section of the skeleton.
    let backlink_count = {
        let conn = vault.connection().lock();
        conn.query_row(
            "SELECT COUNT(*) FROM vault_links WHERE target_raw = ?1",
            params![entity_name],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
    };
    let expected_sections = skeleton_for(expected_subtype).len() as i64;

    if backlink_count >= expected_sections && last_section_count > 0 {
        Ok(ImpactLevel::Heavy)
    } else {
        Ok(ImpactLevel::Light)
    }
}

/// Incremental update entry point. Decides the impact level and runs
/// the appropriate sub-pipeline:
/// - **Light**: fast refresh — backlink count bumped, no re-render.
/// - **Heavy**: re-render the full hub markdown with the latest mapping.
/// - **FullRebuild**: drop existing skeleton + recompile from scratch
///   with the new subtype.
pub fn incremental_update(
    vault: &VaultStore,
    entity_name: &str,
) -> Result<(ImpactLevel, HubCompileReport)> {
    let impact = classify_impact(vault, entity_name)?;
    match impact {
        ImpactLevel::Light => {
            // Just bump the counter — no render needed. Caller can inspect
            // backlink_count without recompiling the whole skeleton.
            let backlinks = {
                let conn = vault.connection().lock();
                conn.query_row(
                    "SELECT COUNT(*) FROM vault_links WHERE target_raw = ?1",
                    params![entity_name],
                    |r| r.get::<_, i64>(0),
                )
                .unwrap_or(0)
            };
            let now = unix_epoch();
            let conn = vault.connection().lock();
            let subtype = HubSubtype::classify(entity_name);
            conn.execute(
                "INSERT INTO hub_notes
                    (entity_name, hub_subtype, backlink_count, compile_type, last_compiled, hub_threshold)
                 VALUES (?1, ?2, ?3, 'light', ?4, ?5)
                 ON CONFLICT(entity_name) DO UPDATE SET
                    backlink_count = excluded.backlink_count,
                    compile_type = 'light',
                    last_compiled = excluded.last_compiled",
                params![
                    entity_name,
                    subtype.as_str(),
                    backlinks,
                    now as i64,
                    HUB_THRESHOLD_DEFAULT,
                ],
            )?;
            drop(conn);
            // Return the current (unrendered) state — callers who want
            // the markdown should invoke compile_hub directly.
            let backlinking_docs = fetch_backlinking_docs(vault, entity_name)?;
            let report = HubCompileReport {
                entity_name: entity_name.into(),
                subtype,
                backlink_count: backlinks,
                sections: skeleton_for(subtype).len(),
                evidence_gaps: 0,
                markdown: format!(
                    "# {entity_name}\n\n_(light update — {} backlinks, skeleton cached)_\n",
                    backlinking_docs.len()
                ),
            };
            Ok((impact, report))
        }
        ImpactLevel::Heavy => {
            // Full re-render of the current skeleton.
            let report = compile_hub(vault, entity_name)?;
            // Mark compile_type as 'heavy' for observability.
            let conn = vault.connection().lock();
            conn.execute(
                "UPDATE hub_notes SET compile_type = 'heavy' WHERE entity_name = ?1",
                params![entity_name],
            )?;
            Ok((impact, report))
        }
        ImpactLevel::FullRebuild => {
            // Drop the existing hub row so compile_hub re-classifies from scratch.
            {
                let conn = vault.connection().lock();
                conn.execute(
                    "DELETE FROM hub_notes WHERE entity_name = ?1",
                    params![entity_name],
                )?;
            }
            let report = compile_hub(vault, entity_name)?;
            let conn = vault.connection().lock();
            conn.execute(
                "UPDATE hub_notes SET compile_type = 'full' WHERE entity_name = ?1",
                params![entity_name],
            )?;
            Ok((impact, report))
        }
    }
}

/// Expose skeleton length for callers needing to size sections.
fn skeleton_for(subtype: HubSubtype) -> &'static [&'static str] {
    match subtype {
        HubSubtype::StatuteArticle => &[
            "조문 원문 / 정의",
            "요건사실",
            "법적 효과",
            "관련 조문 체계",
            "판례 / 해설 (백링크 종합)",
        ],
        HubSubtype::Person => &[
            "프로필",
            "관련 인물",
            "관련 사건",
            "주요 행위 시계열",
        ],
        HubSubtype::Case => &[
            "① 누가 (당사자 관계)",
            "② 언제 (시계열)",
            "③ 어디서 (장소 / 관할)",
            "④ 무엇을 (청구 취지)",
            "⑤ 어떻게 (행위 경위)",
            "⑥ 왜 (법적 근거)",
            "⑦ 쟁점 구조",
        ],
        HubSubtype::GeneralConcept => &[
            "정의",
            "하위 분류",
            "장점 / 단점",
            "적용 사례",
            "관련 개념 비교",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::ingest::{IngestInput, SourceType};
    use parking_lot::Mutex;
    use rusqlite::Connection;
    use std::sync::Arc;

    async fn mem_store() -> VaultStore {
        let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
        VaultStore::with_shared_connection(conn).unwrap()
    }

    #[test]
    fn subtype_classifies_statute() {
        assert_eq!(HubSubtype::classify("민법 제750조"), HubSubtype::StatuteArticle);
    }

    #[test]
    fn subtype_classifies_case() {
        assert_eq!(HubSubtype::classify("2024가합12345"), HubSubtype::Case);
    }

    #[test]
    fn subtype_classifies_person() {
        assert_eq!(HubSubtype::classify("홍길동"), HubSubtype::Person);
    }

    #[test]
    fn subtype_falls_back_to_general_concept() {
        assert_eq!(HubSubtype::classify("투자사기"), HubSubtype::GeneralConcept);
    }

    #[tokio::test]
    async fn refresh_counts_and_compile() {
        let vault = mem_store().await;
        // Ingest several docs all linking to "민법 제750조".
        for i in 0..6 {
            let md = format!(
                "# 사건{i}\n\n본 사건은 민법 제750조에 근거한 손해배상 청구다. \
민법 제750조의 요건은 고의/과실과 위법성이다. {body}",
                body = "추가 본문 ".repeat(80)
            );
            vault
                .ingest_markdown(IngestInput {
                    source_type: SourceType::LocalFile,
                    source_device_id: "dev",
                    original_path: None,
                    title: Some(&format!("사건 {i}")),
                    markdown: &md,
                    html_content: None,
                    doc_type: None,
                    domain: "legal",
                })
                .await
                .unwrap();
        }

        refresh_backlink_counts(&vault).unwrap();
        let cands = list_compile_candidates(&vault).unwrap();
        assert!(
            cands.iter().any(|(e, n)| e == "민법 제750조" && *n >= 6),
            "expected 민법 제750조 to be a compile candidate; got {cands:?}"
        );

        let report = compile_hub(&vault, "민법 제750조").unwrap();
        assert_eq!(report.subtype, HubSubtype::StatuteArticle);
        assert!(report.sections >= 4);
        assert!(report.markdown.contains("# 민법 제750조"));
        // At least one section should have a 📎 mapping with backlink docs.
        assert!(report.markdown.contains("📎"));
    }

    #[tokio::test]
    async fn evidence_gap_flagged_when_no_backlinks() {
        let vault = mem_store().await;
        // Compile with no backlinks — all sections empty.
        let report = compile_hub(&vault, "고립된엔티티").unwrap();
        assert!(report.evidence_gaps >= 1);
        assert!(report.markdown.contains("Evidence Gap"));
    }

    // ── F2 Production tests ──────────────────────────────────────

    #[test]
    fn priority_score_prefers_never_compiled() {
        let fresh = priority_score(20, 20, None, 1_700_000_000);
        let stale =
            priority_score(20, 5, Some(1_700_000_000 - 3600), 1_700_000_000);
        assert!(fresh > stale);
    }

    #[test]
    fn priority_score_prefers_more_backlinks() {
        let big = priority_score(200, 10, None, 1_700_000_000);
        let small = priority_score(2, 1, None, 1_700_000_000);
        assert!(big > small);
    }

    #[test]
    fn doc_authority_ranking_matches_plan() {
        assert!(doc_authority_rank("판결문") > doc_authority_rank("준비서면"));
        assert!(doc_authority_rank("준비서면") > doc_authority_rank("메모"));
    }

    #[test]
    fn source_reliability_ranking_matches_plan() {
        assert!(source_reliability_rank("법원") > source_reliability_rank("상대방"));
        assert!(source_reliability_rank("상대방") > source_reliability_rank("내부"));
    }

    #[test]
    fn conflict_resolver_prefers_authority_then_recency() {
        let claims = vec![
            ConflictingClaim {
                doc_id: 1,
                title: "오래된 판결문".into(),
                doc_type: "판결문".into(),
                source: "법원".into(),
                doc_date: Some("2020-01-01".into()),
                content_snippet: "".into(),
            },
            ConflictingClaim {
                doc_id: 2,
                title: "최근 준비서면".into(),
                doc_type: "준비서면".into(),
                source: "상대방".into(),
                doc_date: Some("2026-04-01".into()),
                content_snippet: "".into(),
            },
            ConflictingClaim {
                doc_id: 3,
                title: "최신 판결문".into(),
                doc_type: "판결문".into(),
                source: "법원".into(),
                doc_date: Some("2026-04-15".into()),
                content_snippet: "".into(),
            },
        ];
        let order = resolve_conflict(&claims);
        // Expect: claim 2 (newest 판결문) → claim 0 (older 판결문) → claim 1 (준비서면)
        assert_eq!(order[0], 2);
        assert_eq!(order[1], 0);
        assert_eq!(order[2], 1);
    }

    #[tokio::test]
    async fn compile_queue_next_picks_highest_priority() {
        let vault = mem_store().await;
        // Seed two hub rows manually with different scores.
        {
            let conn = vault.connection().lock();
            conn.execute(
                "INSERT INTO hub_notes (entity_name, hub_subtype, backlink_count,
                    hub_threshold, pending_backlinks, last_compiled)
                 VALUES
                    ('small', 'general_concept', 5, 5, 0, NULL),
                    ('huge', 'statute_article', 200, 5, 200, NULL)",
                [],
            )
            .unwrap();
        }
        let next = compile_queue_next(&vault).unwrap();
        assert_eq!(next.as_deref(), Some("huge"));
    }

    #[tokio::test]
    async fn classify_impact_detects_full_rebuild_on_subtype_shift() {
        let vault = mem_store().await;
        // Start with generic concept classification persisted.
        {
            let conn = vault.connection().lock();
            conn.execute(
                "INSERT INTO hub_notes (entity_name, hub_subtype, backlink_count,
                    hub_threshold, structure_elements)
                 VALUES ('민법 제750조', 'general_concept', 5, 5, 5)",
                [],
            )
            .unwrap();
        }
        // Current classifier will say StatuteArticle → FullRebuild.
        let impact = classify_impact(&vault, "민법 제750조").unwrap();
        assert_eq!(impact, ImpactLevel::FullRebuild);
    }

    #[tokio::test]
    async fn incremental_update_full_rebuild_resets_subtype() {
        let vault = mem_store().await;
        // Seed mis-classified hub.
        {
            let conn = vault.connection().lock();
            conn.execute(
                "INSERT INTO hub_notes (entity_name, hub_subtype, backlink_count,
                    hub_threshold, structure_elements, content_md)
                 VALUES ('민법 제750조', 'general_concept', 5, 5, 5, 'old content')",
                [],
            )
            .unwrap();
        }
        let (impact, _report) = incremental_update(&vault, "민법 제750조").unwrap();
        assert_eq!(impact, ImpactLevel::FullRebuild);
        let subtype: String = {
            let c = vault.connection().lock();
            c.query_row(
                "SELECT hub_subtype FROM hub_notes WHERE entity_name = '민법 제750조'",
                [],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(subtype, "statute_article");
    }

    #[tokio::test]
    async fn incremental_update_light_path_is_cheap() {
        let vault = mem_store().await;
        // Seed a correctly-classified hub.
        {
            let conn = vault.connection().lock();
            conn.execute(
                "INSERT INTO hub_notes (entity_name, hub_subtype, backlink_count,
                    hub_threshold, structure_elements)
                 VALUES ('sole', 'general_concept', 1, 5, 0)",
                [],
            )
            .unwrap();
        }
        let (impact, _) = incremental_update(&vault, "sole").unwrap();
        assert_eq!(impact, ImpactLevel::Light);
        let compile_type: String = {
            let c = vault.connection().lock();
            c.query_row(
                "SELECT compile_type FROM hub_notes WHERE entity_name = 'sole'",
                [],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(compile_type, "light");
    }
}
