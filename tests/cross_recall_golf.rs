//! Q1 Commit #10 — 5-dimensional `cross_recall` integration test.
//!
//! Walks through the canonical "지난주 제주도 골프 누구와" recall
//! scenario described in `ARCHITECTURE.md §Q1-5W1H`:
//!
//! 1. Three raw memory rows are inserted, of which one is the golf round.
//! 2. The golf row is annotated with 5W1H + narrative as if the nightly
//!    Dream Cycle SLM had already done its diary backfill.
//! 3. A First-Brain wiki page about the playing partner is inserted and
//!    linked back to the golf memory.
//! 4. `SqliteMemory::cross_recall("지난주 제주도 골프", limit=5,
//!    RecallProfile::Mobile)` is invoked — Mobile drops the vector dim,
//!    proving the four text dims alone surface the right answer.
//! 5. We assert the golf row leads the result list and that multiple
//!    dimensions contributed (raw FTS5, narrative FTS5, structured 5W1H,
//!    and wiki BFS via the partner page).

use zeroclaw::memory::cross_recall::RecallProfile;
use zeroclaw::memory::sqlite::{Memory5W1H, SqliteMemory};
use zeroclaw::memory::traits::{Memory, MemoryCategory};

#[tokio::test]
async fn cross_recall_finds_last_weeks_golf_round_with_partner_via_wiki() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mem = SqliteMemory::new(tmp.path()).unwrap();

    // ── Step 1: three raw memories — only one is the golf round.
    mem.store(
        "memo_golf",
        "오늘 친구 김필순과 제주 라운딩 18홀 다녀왔다.",
        MemoryCategory::Core,
        None,
    )
    .await
    .unwrap();
    mem.store(
        "memo_meeting",
        "오후에 회사에서 분기 전략 미팅을 진행했다.",
        MemoryCategory::Core,
        None,
    )
    .await
    .unwrap();
    mem.store(
        "memo_call",
        "어머니께 전화드려서 안부 인사를 나눴다.",
        MemoryCategory::Core,
        None,
    )
    .await
    .unwrap();

    // ── Step 2: simulate the Dream Cycle 5W1H backfill on the golf row.
    let filled = mem
        .set_5w1h(
            "memo_golf",
            &Memory5W1H {
                who_actor: Some("user".into()),
                who_target: Some("[\"김필순\"]".into()),
                when_at: Some(1_744_000_000),
                where_location: Some("제주 골프장".into()),
                what_subject: Some("골프 라운딩".into()),
                how_action: Some(
                    "라운딩 내내 화기애애했고 \"오랜만에 진짜 시원하다\"고 \
                     웃으며 말했다. 컨디션 좋아 보였다."
                        .into(),
                ),
                why_reason: Some("오랜 친구와의 친목 모임".into()),
                narrative: Some(
                    "[2026-04-12 09:00 @ 제주 골프장] user 가 김필순과 \
                     골프 라운딩을 화기애애하게 진행 — 친목 모임"
                        .into(),
                ),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(filled, "set_5w1h should report at least one row updated");

    // Find the memory row id so we can cross-reference it from a wiki page.
    let golf_id: String = {
        let conn = mem.connection();
        conn.query_row(
            "SELECT id FROM memories WHERE key = ?1",
            rusqlite::params!["memo_golf"],
            |r| r.get(0),
        )
        .unwrap()
    };

    // ── Step 3: First-Brain wiki page about the playing partner, linked
    // back to the golf memory. This exercises Dim 3 (wiki BFS) on its own
    // even when the partner's name doesn't appear in the raw query terms
    // (the page surfaces because its title/markdown contains the place
    // and subject).
    {
        let conn = mem.connection();
        conn.execute(
            "INSERT INTO first_brain_pages
                (page_kind, slug, title, markdown,
                 memory_id, updated_at, updated_by)
             VALUES ('person', 'kim-pilsoon', '김필순',
                     '## 김필순\n\n제주 골프장에서 함께 라운딩 — 2026-04-12.',
                     ?1, 1744000100, 'dream_cycle')",
            rusqlite::params![&golf_id],
        )
        .unwrap();
    }

    // ── Step 4: cross-recall under the Mobile profile (no embedder).
    let results = mem
        .cross_recall("제주 골프 라운딩", 5, RecallProfile::Mobile)
        .await
        .expect("cross_recall must succeed");

    assert!(
        !results.is_empty(),
        "cross_recall returned zero rows; expected the golf memory at the top"
    );

    // ── Step 5: verify the golf memory leads, and that multiple text dims
    // contributed. We don't pin to an exact dim count because BM25 weight
    // is data-dependent — but at least 2 of the 4 mobile-profile dims
    // (raw FTS, narrative FTS, structured 5W1H, wiki BFS) must fire.
    let top = &results[0];
    assert_eq!(
        top.entry.key, "memo_golf",
        "golf memory should lead the cross_recall ranking; got key={}",
        top.entry.key
    );
    assert_eq!(
        top.dim_scores.vector, None,
        "Mobile profile must skip the vector dimension"
    );

    let matched = top.dim_scores.matched_count();
    assert!(
        matched >= 2,
        "expected the golf row to match >=2 dims (raw/narrative/structured/wiki); \
         got {matched} (dims = {:?})",
        top.dim_scores
    );

    // Sanity: meeting/call rows must rank below the golf row OR be absent.
    for r in &results[1..] {
        assert_ne!(
            r.entry.key, "memo_golf",
            "duplicate golf row in result list — fuser bug"
        );
        assert!(
            r.final_score <= top.final_score,
            "result list must be sorted by final_score desc"
        );
    }
}

#[tokio::test]
async fn cross_recall_returns_empty_for_blank_query() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mem = SqliteMemory::new(tmp.path()).unwrap();
    mem.store("k", "some content", MemoryCategory::Core, None)
        .await
        .unwrap();

    let results = mem
        .cross_recall("", 5, RecallProfile::Mobile)
        .await
        .unwrap();
    assert!(results.is_empty(), "blank query must short-circuit");

    let results = mem
        .cross_recall("   ", 5, RecallProfile::Mobile)
        .await
        .unwrap();
    assert!(results.is_empty(), "whitespace-only query must short-circuit");
}

#[tokio::test]
async fn cross_recall_zero_limit_is_a_noop() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mem = SqliteMemory::new(tmp.path()).unwrap();
    mem.store("k", "content", MemoryCategory::Core, None)
        .await
        .unwrap();
    let results = mem
        .cross_recall("content", 0, RecallProfile::Mobile)
        .await
        .unwrap();
    assert!(results.is_empty(), "limit=0 must return empty");
}
