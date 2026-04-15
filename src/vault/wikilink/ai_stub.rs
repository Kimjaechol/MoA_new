// @Ref: SUMMARY §3 Steps 2a + 4 — AIEngine trait + provider-free heuristic impl.
//
// Production path: `LlmAIEngine` wraps a provider (Haiku/Opus). Tests and
// offline environments use `HeuristicAIEngine` which applies rule-based
// extraction only — no network, no flakes, deterministic output.

use super::tokens::{CompoundToken, CompoundTokenKind};
use async_trait::async_trait;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyConcept {
    pub term: String,
    /// 1–10, where 10 = this document's raison d'être.
    pub importance: u8,
}

#[derive(Debug, Clone, Default)]
pub struct GatekeepVerdict {
    pub kept: Vec<String>,
    /// Pairs of (representative, alias) that the gatekeeper identified as
    /// synonyms within this document. Fed to Step 5 for `[[rep|alias]]`
    /// and to Step 6 for long-term vocabulary_relations learning.
    pub synonym_pairs: Vec<(String, String)>,
}

/// Structured briefing narrative produced by `AIEngine::narrate_briefing`.
/// Each field is a markdown-ready block. Empty strings are permitted for
/// sections the AI had no evidence for.
#[derive(Debug, Clone, Default)]
pub struct BriefingNarrative {
    /// 사건 경과 (timeline)
    pub timeline: String,
    /// 양측 주장 대비
    pub contentions: String,
    /// 핵심 쟁점
    pub issues: String,
    /// 증거 현황 및 미비
    pub evidence: String,
    /// 관련 판례 요약
    pub precedents: String,
    /// 다음 기일 준비 체크리스트
    pub checklist: String,
    /// 전략 제안 (강점 / 약점)
    pub strategy: String,
}

/// Pluggable AI driver — production uses LLM, tests use heuristic.
#[async_trait]
pub trait AIEngine: Send + Sync {
    async fn extract_key_concepts(
        &self,
        markdown: &str,
        compounds: &[CompoundToken],
    ) -> anyhow::Result<Vec<KeyConcept>>;

    async fn gatekeep(
        &self,
        candidates: &[String],
        doc_preview: &str,
    ) -> anyhow::Result<GatekeepVerdict>;

    /// Synthesize a 7-section case briefing from the supplied context.
    /// Default implementation returns an empty narrative so existing
    /// engines remain valid without overriding.
    async fn narrate_briefing(
        &self,
        _case_number: &str,
        _primary_docs: &[(i64, String, String)], // (doc_id, title, content_preview)
        _related_docs: &[(i64, String)],
    ) -> anyhow::Result<BriefingNarrative> {
        Ok(BriefingNarrative::default())
    }

    /// Assign each backlinked document to one or more skeleton sections
    /// of the hub note. Returns a vector aligned with `docs` where each
    /// entry is the list of section indices (0-based) the doc belongs to.
    /// Empty vectors are permitted (doc not pinned to any section).
    ///
    /// Default: hash-mod distribution (`doc_id mod section_count`) so
    /// every doc lands in exactly one section — matches the historical
    /// behaviour before this trait method existed.
    async fn assign_hub_sections(
        &self,
        _subtype: &str,
        sections: &[&str],
        docs: &[(i64, String, String)], // (doc_id, title, content_preview)
    ) -> anyhow::Result<Vec<Vec<usize>>> {
        let n = sections.len().max(1);
        Ok(docs
            .iter()
            .map(|(id, _, _)| vec![id.unsigned_abs() as usize % n])
            .collect())
    }

    /// Detect contradictions among a set of claims about the same entity.
    /// Default: empty — only LlmAIEngine produces real detections.
    async fn detect_contradictions(
        &self,
        _entity: &str,
        _claims: &[ContentClaim],
    ) -> anyhow::Result<Vec<Contradiction>> {
        Ok(Vec::new())
    }
}

/// A single factual statement extracted from a vault document, used as
/// input to `AIEngine::detect_contradictions`.
#[derive(Debug, Clone)]
pub struct ContentClaim {
    pub doc_id: i64,
    pub title: String,
    /// ≤500 char snippet centred on the entity mention.
    pub statement: String,
}

/// A detected contradiction between two documents about the same entity.
#[derive(Debug, Clone)]
pub struct Contradiction {
    pub left_doc_id: i64,
    pub right_doc_id: i64,
    /// Short human-readable summary (e.g. "A says 2024-01-01, B says 2026-04-01").
    pub description: String,
    /// 1 (minor) – 10 (fundamental / case-altering)
    pub severity: u8,
}

/// Provider-free default. Strategy:
/// - Every compound token becomes a key concept at importance 9.
/// - The first H1 title (if any) is extracted as importance 10.
/// - Gatekeeper passes all candidates through; synonym pairs derived
///   from regex-detectable statute short-forms (e.g. "750조" ↔ "민법 제750조").
pub struct HeuristicAIEngine;

#[async_trait]
impl AIEngine for HeuristicAIEngine {
    async fn extract_key_concepts(
        &self,
        markdown: &str,
        compounds: &[CompoundToken],
    ) -> anyhow::Result<Vec<KeyConcept>> {
        let mut concepts = Vec::new();

        // Compound tokens → high importance.
        for c in compounds {
            let imp = match c.kind {
                CompoundTokenKind::StatuteArticle
                | CompoundTokenKind::PrecedentCitation
                | CompoundTokenKind::CaseNumber => 9,
                CompoundTokenKind::Organization => 8,
            };
            concepts.push(KeyConcept {
                term: c.canonical.clone(),
                importance: imp,
            });
        }

        // H1 title → importance 10 (first one only).
        for line in markdown.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("# ") {
                let title = rest.trim();
                if !title.is_empty() {
                    concepts.push(KeyConcept {
                        term: title.to_string(),
                        importance: 10,
                    });
                    break;
                }
            }
        }

        Ok(concepts)
    }

    async fn gatekeep(
        &self,
        candidates: &[String],
        _doc_preview: &str,
    ) -> anyhow::Result<GatekeepVerdict> {
        let mut kept: Vec<String> = candidates.to_vec();
        kept.sort();
        kept.dedup();

        // Detect synonym pairs: "제NNN조" ↔ "민법 제NNN조" etc. (structural).
        let mut synonym_pairs: Vec<(String, String)> = Vec::new();
        for cand in &kept {
            if let Some((rep, alias)) = detect_statute_short_form(cand, &kept) {
                synonym_pairs.push((rep, alias));
            }
        }

        Ok(GatekeepVerdict {
            kept,
            synonym_pairs,
        })
    }

    async fn narrate_briefing(
        &self,
        case_number: &str,
        primary_docs: &[(i64, String, String)],
        related_docs: &[(i64, String)],
    ) -> anyhow::Result<BriefingNarrative> {
        // Structured template: deterministic, no LLM. Lists docs
        // per-section and leaves a note requesting LLM fill-in.
        let mut timeline = String::from("이 사건 **관련 문서 시계열**:\n\n");
        for (id, title, _) in primary_docs {
            timeline.push_str(&format!("- [Doc-{id}] {title}\n"));
        }

        let mut evidence = String::from("이 사건과 직접 매핑된 문서:\n\n");
        for (id, title, _) in primary_docs {
            evidence.push_str(&format!("- [Doc-{id}] {title}\n"));
        }
        if primary_docs.is_empty() {
            evidence.push_str("(사건 프론트매터 매칭 없음 — 문서에 `case_number` 필드 기재 필요)\n");
        }

        let precedents = if related_docs.is_empty() {
            "관련 판례·자료 매핑 없음.".to_string()
        } else {
            let mut s = String::from("1-depth 그래프 확장으로 식별된 관련 자료:\n\n");
            for (id, title) in related_docs {
                s.push_str(&format!("- [Doc-{id}] {title}\n"));
            }
            s
        };

        Ok(BriefingNarrative {
            timeline,
            contentions: format!(
                "사건번호 {case_number}에 대한 양측 주장 대비는 LLM 서사 합성에서 제공됩니다 (Heuristic 엔진은 구조만 채움)."
            ),
            issues: "핵심 쟁점은 LLM 서사 합성에서 제공됩니다.".to_string(),
            evidence,
            precedents,
            checklist: "- [ ] 쟁점 정리\n- [ ] 증거 현황\n- [ ] 다음 기일 준비사항\n".to_string(),
            strategy: "전략 제안은 LLM 서사 합성에서 제공됩니다.".to_string(),
        })
    }
}

/// If `candidate` is "민법 제750조" and "제750조" is also in the set,
/// return (rep="민법 제750조", alias="제750조"). Heuristic helper.
fn detect_statute_short_form(candidate: &str, all: &[String]) -> Option<(String, String)> {
    if let Some((_law, rest)) = candidate.split_once(' ') {
        if rest.starts_with("제") && rest.contains("조") && all.iter().any(|c| c == rest) {
            return Some((candidate.to_string(), rest.to_string()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::wikilink::tokens::detect_compound_tokens;

    #[tokio::test]
    async fn heuristic_promotes_compound_tokens() {
        let md = "본문에 민법 제750조가 등장한다.";
        let compounds = detect_compound_tokens(md);
        let concepts = HeuristicAIEngine
            .extract_key_concepts(md, &compounds)
            .await
            .unwrap();
        assert!(concepts.iter().any(|k| k.term == "민법 제750조" && k.importance >= 9));
    }

    #[tokio::test]
    async fn heuristic_extracts_h1_as_top_concept() {
        let md = "# 불법행위 손해배상 분석\n\n본문";
        let concepts = HeuristicAIEngine
            .extract_key_concepts(md, &[])
            .await
            .unwrap();
        assert!(concepts
            .iter()
            .any(|k| k.term == "불법행위 손해배상 분석" && k.importance == 10));
    }

    #[tokio::test]
    async fn gatekeeper_detects_statute_short_form() {
        let candidates = vec!["민법 제750조".to_string(), "제750조".to_string()];
        let verdict = HeuristicAIEngine
            .gatekeep(&candidates, "ignored preview")
            .await
            .unwrap();
        assert_eq!(verdict.synonym_pairs.len(), 1);
        assert_eq!(verdict.synonym_pairs[0].0, "민법 제750조");
        assert_eq!(verdict.synonym_pairs[0].1, "제750조");
    }

    #[tokio::test]
    async fn gatekeeper_deduplicates() {
        let candidates = vec!["A".into(), "A".into(), "B".into()];
        let verdict = HeuristicAIEngine
            .gatekeep(&candidates, "")
            .await
            .unwrap();
        assert_eq!(verdict.kept.len(), 2);
    }
}
