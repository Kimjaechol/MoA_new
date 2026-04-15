# MoA retrieval evaluation harness (PR #8)

RAGAS-style golden-set evaluation for the retrieval pipeline. Detects
regressions in recall quality *before* they reach users.

> Data files live under `tests/evals/` to comply with the repo's
> Claude-harness path gate (it allows `tests/` but not a top-level
> `evals/`). The runner binary lives at `src/bin/moa_eval.rs`.

## Metrics shipped

The Rust-native runner covers the metrics we can compute without an LLM
judge:

- **context_precision@k** — fraction of top-k retrieved that match a
  gold context key.
- **context_recall@k** — fraction of gold keys that appear in top-k.
- **mrr** — mean reciprocal rank of the first gold hit.

LLM-judged metrics (`faithfulness`, `answer_relevance`) from full RAGAS
need an LLM in the eval path. Left as a follow-up hook
(`scripts/eval_rag_llm.py`, not in this commit) — run the Rust harness
today, bolt on LLM metrics when a CI judge is wired up.

## Data layout

```
tests/evals/
├── README.md           — this file
├── thresholds.toml     — per-domain pass/fail thresholds (CI reads)
├── corpus.jsonl        — documents to seed the memory DB
├── golden_ko.jsonl     — Korean queries + expected keys
├── golden_en.jsonl     — English queries + expected keys
└── golden_law.jsonl    — Korean legal-domain queries
```

### Corpus schema

```json
{"key": "corpus_001", "content": "대법원 판결은 …", "category": "core"}
```

### Gold query schema

```json
{"query": "대법원 2023다12345 핵심 쟁점", "gold_keys": ["corpus_037"], "domain": "law"}
```

## Running locally

```bash
# All sets, stdout report
cargo run --bin moa_eval

# Single set, JSON report to file
cargo run --bin moa_eval -- --set law --output /tmp/report.json

# Tighter recall window
cargo run --bin moa_eval -- --top-k 5
```

The binary rebuilds a fresh SqliteMemory in a temp dir on every run —
local state never contaminates scores.

## Thresholds and CI

`tests/evals/thresholds.toml` is the source of truth. CI
(`.github/workflows/eval.yml`) reads it and fails the job when any
`_min` metric is below spec.

Default thresholds are intentionally lenient because the corpus is
seed-sized (5–10 queries per domain). Raise them as the corpus grows
past the 30-per-domain target in the roadmap.

## Adding cases

1. Append a corpus entry to `corpus.jsonl` with a stable `key`.
2. Append a query JSON to the right `golden_*.jsonl`, listing every key
   that would be a correct retrieval.
3. Run `cargo run --bin moa_eval` — commit when green.
