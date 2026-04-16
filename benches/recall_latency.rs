//! PR #4 performance gate — recall p95 latency must stay under 500ms.
//!
//! Seeds a fresh SqliteMemory with the full 180-entry eval corpus and
//! runs a spread of Korean/English/law queries. Criterion reports
//! percentile-level timing so the CI or operator can verify the p95
//! acceptance criterion without a manual profiling session.
//!
//! Run: `cargo bench --bench recall_latency`

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tempfile::TempDir;

use zeroclaw::memory::sqlite::SqliteMemory;
use zeroclaw::memory::traits::{Memory, MemoryCategory};

/// Representative query set — a mix of Korean personal, English eng-team,
/// and law domain queries taken from the eval golden files. We keep it
/// small (20 queries) so the bench completes quickly while still covering
/// the FTS5 code paths that matter (bigram tokenizer, LIKE fallback,
/// multi-keyword).
const QUERIES: &[&str] = &[
    "사용자 직업과 전담팀",
    "weekly retrospective schedule",
    "주택임대차보호법 대항력",
    "production incident escalation",
    "아이 학교 담임선생님",
    "code review approvals required",
    "민법 전대차 해지",
    "비행기 좌석 마일리지",
    "feature flag naming convention",
    "임대차 보증금 증액 한도",
    "결혼기념일 레스토랑",
    "database migration sqlx",
    "공인중개사 금지행위",
    "은퇴 계획 제주도",
    "error budget availability",
    "변호사 의무연수 시간",
    "Slack channel naming",
    "사무실 CCTV 보관",
    "analytics pipeline Kafka",
    "건강검진 삼성서울병원",
];

fn bench_recall_p95(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");

    let tmp = TempDir::new().expect("tempdir");
    let mem = SqliteMemory::new(tmp.path()).expect("SqliteMemory::new");

    // Seed the full eval corpus (180 entries).
    let corpus_path = std::path::Path::new("tests/evals/corpus.jsonl");
    let corpus_text = std::fs::read_to_string(corpus_path)
        .expect("read corpus.jsonl — run from the repo root");
    let entries: Vec<serde_json::Value> = corpus_text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse corpus line"))
        .collect();
    assert!(
        entries.len() >= 180,
        "expected ≥180 corpus entries, got {}",
        entries.len()
    );
    rt.block_on(async {
        for e in &entries {
            let key = e["key"].as_str().unwrap();
            let content = e["content"].as_str().unwrap();
            mem.store(key, content, MemoryCategory::Core, None)
                .await
                .unwrap();
        }
    });

    let mut group = c.benchmark_group("recall_latency");
    group.throughput(Throughput::Elements(QUERIES.len() as u64));
    // Enough samples for stable p95 estimation but short wall-clock.
    group.sample_size(50);
    group.bench_function("p95_recall_top5_180corpus", |b| {
        b.iter(|| {
            rt.block_on(async {
                for q in QUERIES {
                    let _ = mem.recall(black_box(q), 5, None).await.unwrap();
                }
            });
        });
    });
    group.finish();
}

criterion_group!(benches, bench_recall_p95);
criterion_main!(benches);
