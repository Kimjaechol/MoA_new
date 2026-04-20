//! Cross-engine secretary voice matching.
//!
//! When a user drops from Tier A (Typecast, online premium) to Tier B
//! (CosyVoice 2, offline) or Tier C (Kokoro, offline), we don't just
//! swap the TTS engine — we want to keep the user's chosen "secretary
//! persona" consistent so the brand experience survives the transition.
//!
//! Scope
//! -----
//! * Consume a minimal user-facing voice profile (gender, age,
//!   language, use-case tags) — whatever the Typecast voice already
//!   exposes.
//! * Score every offline candidate from the bundled Kokoro + CosyVoice
//!   catalogs against that profile.
//! * Return the top candidate plus a human-readable rationale the UI
//!   can render under "Why we picked this voice".
//!
//! This module is engine-agnostic: catalogs are just `&[VoiceProfile]`
//! slices, so adding a new offline TTS later (IndexTTS-2, Fish Speech)
//! means adding its voice list to [`offline_catalog`] and nothing else.

use serde::{Deserialize, Serialize};

// ── Profile + engine ────────────────────────────────────────────────

/// Which offline engine a given voice runs on. Maps 1:1 to the §11.2
/// tier labels so the UI can render the tier badge without recomputing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfflineEngine {
    /// Tier B — CosyVoice 2 (1.5B params, Apache 2.0). Zero-shot
    /// cloning, ~150ms streaming latency. Requires T3 / T4 hardware.
    CosyVoice,
    /// Tier C — Kokoro (82M params, Apache 2.0). Bundled with every
    /// MoA install as the offline safety net.
    Kokoro,
}

impl OfflineEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            OfflineEngine::CosyVoice => "cosyvoice",
            OfflineEngine::Kokoro => "kokoro",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceGender {
    Male,
    Female,
}

impl VoiceGender {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "m" | "male" | "man" => Some(Self::Male),
            "f" | "female" | "woman" => Some(Self::Female),
            _ => None,
        }
    }
}

/// Broad age bucket. Kept coarse so offline catalogs don't have to
/// estimate exact decades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceAgeBand {
    Teen,
    YoungAdult,
    MiddleAge,
    Elder,
}

impl VoiceAgeBand {
    fn ordinal(self) -> i32 {
        match self {
            VoiceAgeBand::Teen => 0,
            VoiceAgeBand::YoungAdult => 1,
            VoiceAgeBand::MiddleAge => 2,
            VoiceAgeBand::Elder => 3,
        }
    }

    /// Parse Typecast's lower-snake_case age strings into our band.
    pub fn parse_typecast(raw: &str) -> Option<Self> {
        match raw {
            "teenager" => Some(Self::Teen),
            "young_adult" => Some(Self::YoungAdult),
            "middle_age" => Some(Self::MiddleAge),
            "elder" => Some(Self::Elder),
            _ => None,
        }
    }
}

/// Canonical voice descriptor used for cross-engine matching. Every
/// engine catalog normalises its internal representation into this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceProfile {
    /// Opaque engine-specific ID. Format stays free-form.
    pub id: String,
    /// Engine that hosts this voice (Typecast for the source side,
    /// Kokoro/CosyVoice on the offline side).
    pub engine: String,
    /// Display name the UI shows — "김 팀장", "Emily", etc.
    pub display_name: String,
    pub gender: VoiceGender,
    pub age_band: VoiceAgeBand,
    /// BCP-47-like language tag. `"ko"`, `"en"`, `"ja"`. Cross-language
    /// matches are penalised but not disqualified — some users want an
    /// English-speaking secretary even when their UI is Korean.
    pub language: String,
    /// Use-case tags the engine exposes. Used for soft penalties when
    /// a voice is flagged `Game`/`Anime` (unnatural for chat).
    pub use_cases: Vec<String>,
}

// ── Scoring ─────────────────────────────────────────────────────────

/// Hard-filter categories: a voice tagged only with any of these is
/// disqualified outright. Mirrors `typecast_interp::EXCLUDED_USE_CASES`
/// so the two matchers agree.
const EXCLUDED_USE_CASES: &[&str] = &["Game", "Anime", "TikTok/Reels/Shorts"];

/// Hard disqualifier when the candidate's ONLY use-cases are excluded.
fn is_hard_excluded(candidate: &VoiceProfile) -> bool {
    !candidate.use_cases.is_empty()
        && candidate
            .use_cases
            .iter()
            .all(|uc| EXCLUDED_USE_CASES.iter().any(|ex| uc.contains(ex)))
}

/// Score a single offline candidate against the source Typecast profile.
///
/// Higher = better. Returns 0 for disqualified candidates so a caller
/// can short-circuit with `max_by_key(score)` without extra filtering.
///
/// Weights (deliberately coarse so the algorithm stays auditable):
/// * Gender match (hard filter): 0 when mismatched, +10 otherwise
/// * Language match: +8 exact, +2 English fallback, 0 otherwise
/// * Age band distance: +6 exact, +3 adjacent, +1 two-away, 0 farther
/// * Use-case overlap: +1 per matching preferred tag (max +4)
pub fn score_candidate(source: &VoiceProfile, candidate: &VoiceProfile) -> u32 {
    if source.gender != candidate.gender {
        return 0;
    }
    if is_hard_excluded(candidate) {
        return 0;
    }

    let mut score = 10u32;

    // Language match.
    if source.language == candidate.language {
        score += 8;
    } else if candidate.language == "en" {
        // English is an acceptable fallback for most users — it covers
        // the "I learn English with my assistant" secondary persona.
        score += 2;
    }

    // Age proximity.
    let age_dist = (source.age_band.ordinal() - candidate.age_band.ordinal()).unsigned_abs();
    score += match age_dist {
        0 => 6,
        1 => 3,
        2 => 1,
        _ => 0,
    };

    // Use-case overlap (soft bonus).
    let overlap = candidate
        .use_cases
        .iter()
        .filter(|uc| source.use_cases.iter().any(|suc| suc == *uc))
        .take(4)
        .count() as u32;
    score += overlap;

    // Engine-quality tiebreaker: CosyVoice sounds audibly better than
    // Kokoro (1.5B vs 82M params). When two candidates are otherwise
    // equivalent on gender/language/age/use-cases, prefer CosyVoice.
    // +2 is small enough that Kokoro still wins on a real match-quality
    // advantage (e.g. exact language + exact age).
    if candidate.engine == "cosyvoice" {
        score += 2;
    }

    score
}

/// Human-readable rationale describing WHY a given candidate was picked.
/// Rendered by the UI under "Why we picked this voice".
pub fn rationale(source: &VoiceProfile, candidate: &VoiceProfile) -> String {
    let mut parts: Vec<String> = Vec::new();

    if source.gender == candidate.gender {
        parts.push(match candidate.gender {
            VoiceGender::Male => "동일 성별(남성)".to_string(),
            VoiceGender::Female => "동일 성별(여성)".to_string(),
        });
    }
    if source.language == candidate.language {
        parts.push(format!("동일 언어({})", candidate.language));
    } else if candidate.language == "en" {
        parts.push("영어 대체 비서".to_string());
    }
    let age_dist = (source.age_band.ordinal() - candidate.age_band.ordinal()).unsigned_abs();
    if age_dist == 0 {
        parts.push("연령대 일치".to_string());
    } else if age_dist == 1 {
        parts.push("인접 연령대".to_string());
    }
    if parts.is_empty() {
        "조건에 맞는 가장 가까운 오프라인 비서".to_string()
    } else {
        parts.join(", ")
    }
}

// ── Recommendation ──────────────────────────────────────────────────

/// Final recommendation returned from [`recommend`] and the
/// `/api/voices/secretary-suggest` endpoint. Shape is stable so the
/// React `SecretaryMigrator` component can swap its hardcoded table
/// for a live fetch without TS schema churn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recommendation {
    pub voice_id: String,
    pub engine: OfflineEngine,
    pub display_name: String,
    pub language: String,
    pub rationale: String,
    pub score: u32,
}

/// Pick the best offline candidate for a given Typecast source voice.
/// Returns `None` when no candidate scores above 0 (every candidate
/// failed the gender filter or was hard-excluded).
pub fn recommend(source: &VoiceProfile, offline_catalog: &[VoiceProfile]) -> Option<Recommendation> {
    let mut best: Option<(&VoiceProfile, u32)> = None;
    for candidate in offline_catalog {
        let s = score_candidate(source, candidate);
        if s == 0 {
            continue;
        }
        if best.map_or(true, |(_, best_score)| s > best_score) {
            best = Some((candidate, s));
        }
    }
    let (pick, score) = best?;
    let engine = engine_from_profile(pick)?;
    Some(Recommendation {
        voice_id: pick.id.clone(),
        engine,
        display_name: pick.display_name.clone(),
        language: pick.language.clone(),
        rationale: rationale(source, pick),
        score,
    })
}

fn engine_from_profile(p: &VoiceProfile) -> Option<OfflineEngine> {
    match p.engine.as_str() {
        "cosyvoice" => Some(OfflineEngine::CosyVoice),
        "kokoro" => Some(OfflineEngine::Kokoro),
        _ => None,
    }
}

// ── Bundled offline catalog ─────────────────────────────────────────
//
// Starter set. Lives here (not in a config file) because the bundle is
// what gets installed with MoA — and changing what's bundled is a
// deliberate code-review decision, not a runtime knob.
//
// Kokoro: ships with every install (300 MB). The 6 entries below cover
//   the common "office secretary" archetypes across Korean, English,
//   and Japanese.
// CosyVoice: optional T3/T4 download (4-5 GB). Its zero-shot cloning
//   means we don't ship a fixed catalog, but these 4 prebaked voices
//   give the UI something to recommend before the user records their
//   own reference audio.

fn kokoro_bundle() -> Vec<VoiceProfile> {
    vec![
        VoiceProfile {
            id: "kokoro-ko-female-default".into(),
            engine: "kokoro".into(),
            display_name: "민지 (오프라인 기본)".into(),
            gender: VoiceGender::Female,
            age_band: VoiceAgeBand::YoungAdult,
            language: "ko".into(),
            use_cases: vec!["Conversational".into(), "Voicemail/Voice Assistant".into()],
        },
        VoiceProfile {
            id: "kokoro-ko-female-mid".into(),
            engine: "kokoro".into(),
            display_name: "서연 팀장 (오프라인 기본)".into(),
            gender: VoiceGender::Female,
            age_band: VoiceAgeBand::MiddleAge,
            language: "ko".into(),
            use_cases: vec!["Announcer".into(), "News Reporter".into()],
        },
        VoiceProfile {
            id: "kokoro-ko-male-mid".into(),
            engine: "kokoro".into(),
            display_name: "지훈 실장 (오프라인 기본)".into(),
            gender: VoiceGender::Male,
            age_band: VoiceAgeBand::MiddleAge,
            language: "ko".into(),
            use_cases: vec!["Announcer".into()],
        },
        VoiceProfile {
            id: "kokoro-en-female-us-02".into(),
            engine: "kokoro".into(),
            display_name: "Lydia (offline basic)".into(),
            gender: VoiceGender::Female,
            age_band: VoiceAgeBand::YoungAdult,
            language: "en".into(),
            use_cases: vec!["Voicemail/Voice Assistant".into(), "Conversational".into()],
        },
        VoiceProfile {
            id: "kokoro-en-male-uk-01".into(),
            engine: "kokoro".into(),
            display_name: "Owen (offline basic)".into(),
            gender: VoiceGender::Male,
            age_band: VoiceAgeBand::YoungAdult,
            language: "en".into(),
            use_cases: vec!["Documentary".into(), "Radio/Podcast".into()],
        },
        VoiceProfile {
            id: "kokoro-ja-male-01".into(),
            engine: "kokoro".into(),
            display_name: "健二 (オフライン)".into(),
            gender: VoiceGender::Male,
            age_band: VoiceAgeBand::MiddleAge,
            language: "ja".into(),
            use_cases: vec!["Conversational".into()],
        },
    ]
}

fn cosyvoice_bundle() -> Vec<VoiceProfile> {
    vec![
        VoiceProfile {
            id: "cv2-ko-adult-male-01".into(),
            engine: "cosyvoice".into(),
            display_name: "박 대리 (오프라인 프로)".into(),
            gender: VoiceGender::Male,
            age_band: VoiceAgeBand::YoungAdult,
            language: "ko".into(),
            use_cases: vec!["Conversational".into(), "E-learning/Explainer".into()],
        },
        VoiceProfile {
            id: "cv2-ko-adult-female-01".into(),
            engine: "cosyvoice".into(),
            display_name: "윤 주임 (오프라인 프로)".into(),
            gender: VoiceGender::Female,
            age_band: VoiceAgeBand::YoungAdult,
            language: "ko".into(),
            use_cases: vec!["Conversational".into(), "Voicemail/Voice Assistant".into()],
        },
        VoiceProfile {
            id: "cv2-ko-mid-male-formal".into(),
            engine: "cosyvoice".into(),
            display_name: "정 실장 (오프라인 프로)".into(),
            gender: VoiceGender::Male,
            age_band: VoiceAgeBand::MiddleAge,
            language: "ko".into(),
            use_cases: vec!["Announcer".into(), "News Reporter".into()],
        },
        VoiceProfile {
            id: "cv2-ko-mid-female-formal".into(),
            engine: "cosyvoice".into(),
            display_name: "한 부장 (오프라인 프로)".into(),
            gender: VoiceGender::Female,
            age_band: VoiceAgeBand::MiddleAge,
            language: "ko".into(),
            use_cases: vec!["Announcer".into()],
        },
        VoiceProfile {
            id: "cv2-en-young-female".into(),
            engine: "cosyvoice".into(),
            display_name: "Ivy (offline pro)".into(),
            gender: VoiceGender::Female,
            age_band: VoiceAgeBand::YoungAdult,
            language: "en".into(),
            use_cases: vec!["Conversational".into(), "Voicemail/Voice Assistant".into()],
        },
    ]
}

/// Process-wide bundled catalog. When `include_cosyvoice` is false,
/// only Kokoro voices are returned — use this on hosts that haven't
/// downloaded the optional CosyVoice package.
pub fn offline_catalog(include_cosyvoice: bool) -> Vec<VoiceProfile> {
    let mut out = kokoro_bundle();
    if include_cosyvoice {
        out.extend(cosyvoice_bundle());
    }
    out
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn src(
        gender: VoiceGender,
        age: VoiceAgeBand,
        language: &str,
        use_cases: &[&str],
    ) -> VoiceProfile {
        VoiceProfile {
            id: "typecast-source".into(),
            engine: "typecast".into(),
            display_name: "source".into(),
            gender,
            age_band: age,
            language: language.into(),
            use_cases: use_cases.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn gender_mismatch_scores_zero() {
        let s = src(VoiceGender::Male, VoiceAgeBand::MiddleAge, "ko", &[]);
        let c = VoiceProfile {
            gender: VoiceGender::Female,
            ..s.clone()
        };
        assert_eq!(score_candidate(&s, &c), 0);
    }

    #[test]
    fn hard_excluded_scores_zero_even_on_match() {
        let s = src(VoiceGender::Female, VoiceAgeBand::YoungAdult, "ko", &[]);
        let c = VoiceProfile {
            gender: VoiceGender::Female,
            age_band: VoiceAgeBand::YoungAdult,
            language: "ko".into(),
            use_cases: vec!["Game".into()],
            ..s.clone()
        };
        assert_eq!(score_candidate(&s, &c), 0);
    }

    #[test]
    fn same_gender_language_age_scores_high() {
        let s = src(VoiceGender::Male, VoiceAgeBand::MiddleAge, "ko", &[]);
        let c = VoiceProfile {
            engine: "kokoro".into(),
            ..s.clone()
        };
        let score = score_candidate(&s, &c);
        // 10 (gender) + 8 (lang) + 6 (exact age) = 24
        assert_eq!(score, 24);
    }

    #[test]
    fn adjacent_age_scores_less_than_exact() {
        let s = src(VoiceGender::Male, VoiceAgeBand::MiddleAge, "ko", &[]);
        let exact = VoiceProfile {
            age_band: VoiceAgeBand::MiddleAge,
            engine: "kokoro".into(),
            ..s.clone()
        };
        let adjacent = VoiceProfile {
            age_band: VoiceAgeBand::YoungAdult,
            engine: "kokoro".into(),
            ..s.clone()
        };
        assert!(score_candidate(&s, &exact) > score_candidate(&s, &adjacent));
    }

    #[test]
    fn cross_language_still_scores_when_english_fallback() {
        let s = src(VoiceGender::Female, VoiceAgeBand::YoungAdult, "ko", &[]);
        let english = VoiceProfile {
            engine: "kokoro".into(),
            language: "en".into(),
            ..s.clone()
        };
        let spanish = VoiceProfile {
            engine: "kokoro".into(),
            language: "es".into(),
            ..s.clone()
        };
        let english_score = score_candidate(&s, &english);
        let spanish_score = score_candidate(&s, &spanish);
        assert!(english_score > spanish_score);
        assert!(english_score > 0);
    }

    #[test]
    fn use_case_overlap_adds_points() {
        let s = src(
            VoiceGender::Female,
            VoiceAgeBand::YoungAdult,
            "ko",
            &["Conversational", "Voicemail/Voice Assistant"],
        );
        let with_overlap = VoiceProfile {
            engine: "kokoro".into(),
            use_cases: vec![
                "Conversational".into(),
                "Voicemail/Voice Assistant".into(),
            ],
            ..s.clone()
        };
        let without_overlap = VoiceProfile {
            engine: "kokoro".into(),
            use_cases: vec![],
            ..s.clone()
        };
        assert_eq!(
            score_candidate(&s, &with_overlap) - score_candidate(&s, &without_overlap),
            2
        );
    }

    #[test]
    fn recommend_returns_none_when_all_candidates_fail_hard_filter() {
        let s = src(VoiceGender::Male, VoiceAgeBand::MiddleAge, "ko", &[]);
        let all_female = vec![VoiceProfile {
            engine: "kokoro".into(),
            gender: VoiceGender::Female,
            ..s.clone()
        }];
        assert!(recommend(&s, &all_female).is_none());
    }

    #[test]
    fn recommend_picks_highest_score() {
        let s = src(
            VoiceGender::Male,
            VoiceAgeBand::MiddleAge,
            "ko",
            &["Announcer"],
        );
        let catalog = offline_catalog(true);
        let rec = recommend(&s, &catalog).expect("should find a match");
        // Expected winner: 정 실장 (Korean male, MiddleAge, Announcer overlap).
        assert_eq!(rec.voice_id, "cv2-ko-mid-male-formal");
        assert_eq!(rec.engine, OfflineEngine::CosyVoice);
        assert!(rec.score >= 25);
        assert!(rec.rationale.contains("동일 성별"));
    }

    #[test]
    fn recommend_prefers_cosyvoice_when_available_else_kokoro() {
        let s = src(VoiceGender::Female, VoiceAgeBand::YoungAdult, "ko", &[]);
        let with_cosy = offline_catalog(true);
        let kokoro_only = offline_catalog(false);
        let rec_with = recommend(&s, &with_cosy).unwrap();
        let rec_without = recommend(&s, &kokoro_only).unwrap();
        assert_ne!(rec_with.engine, rec_without.engine);
        assert_eq!(rec_without.engine, OfflineEngine::Kokoro);
    }

    #[test]
    fn recommend_japanese_source_picks_japanese_kokoro() {
        let s = src(VoiceGender::Male, VoiceAgeBand::MiddleAge, "ja", &[]);
        let rec = recommend(&s, &offline_catalog(true)).unwrap();
        assert_eq!(rec.voice_id, "kokoro-ja-male-01");
        assert_eq!(rec.engine, OfflineEngine::Kokoro);
    }

    #[test]
    fn voice_age_band_parse_typecast_strings() {
        assert_eq!(
            VoiceAgeBand::parse_typecast("middle_age"),
            Some(VoiceAgeBand::MiddleAge)
        );
        assert_eq!(
            VoiceAgeBand::parse_typecast("young_adult"),
            Some(VoiceAgeBand::YoungAdult)
        );
        assert_eq!(VoiceAgeBand::parse_typecast("elder"), Some(VoiceAgeBand::Elder));
        assert_eq!(VoiceAgeBand::parse_typecast("teenager"), Some(VoiceAgeBand::Teen));
        assert_eq!(VoiceAgeBand::parse_typecast("garbage"), None);
    }

    #[test]
    fn voice_gender_parse_accepts_common_spellings() {
        assert_eq!(VoiceGender::parse("male"), Some(VoiceGender::Male));
        assert_eq!(VoiceGender::parse("MALE"), Some(VoiceGender::Male));
        assert_eq!(VoiceGender::parse("f"), Some(VoiceGender::Female));
        assert_eq!(VoiceGender::parse("woman"), Some(VoiceGender::Female));
        assert_eq!(VoiceGender::parse("robot"), None);
    }
}
