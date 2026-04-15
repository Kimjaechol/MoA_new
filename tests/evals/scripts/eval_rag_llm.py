#!/usr/bin/env python3
"""
eval_rag_llm.py — LLM-judged RAGAS metrics for MoA retrieval (PR #8 잔여).

The Rust-native harness in `src/bin/moa_eval.rs` handles the
embedding-based metrics (context_precision / context_recall / MRR). This
script adds the judged metrics — `faithfulness` and `answer_relevance` —
that require an LLM in the eval path.

Run manually until the corpus is large enough to justify a CI gate:

    pip install ragas datasets langchain-openai
    export OPENAI_API_KEY=sk-...
    python tests/evals/scripts/eval_rag_llm.py \\
        --goldens tests/evals/golden_ko.jsonl \\
        --corpus tests/evals/corpus.jsonl \\
        --judge-model gpt-4o-mini \\
        --output /tmp/llm-judge.json

The script never touches SqliteMemory directly; it feeds the JSONL
goldens into whatever retrieval endpoint the caller points at. This
means we can evaluate the *live* agent-loop pipeline (Phase 1→5) rather
than just the raw recall() call — closer to user-experienced quality.

Minimum viable flow:

    1. For each golden query, call the retrieval endpoint → top-k
       contexts + generated answer.
    2. Hand (query, answer, retrieved_contexts) to RAGAS:
         - faithfulness: "Is every claim in `answer` grounded in
           `retrieved_contexts`?"
         - answer_relevance: "Does `answer` actually address `query`?"
    3. Emit per-domain averages to `--output` JSON in the same shape
       as `moa_eval`'s report so the same PR-comment workflow can
       render both.

WHY THIS IS A SKELETON
----------------------

The full implementation requires:

  * A retrieval endpoint — today there is no stable HTTP surface
    for the agent loop's final top-k + answer; we'd need to add one
    or shell out to a `moa agent ask` CLI.
  * Cost controls — each golden query consumes one judge-model call
    per metric. At 200 goldens × 2 metrics × gpt-4o-mini pricing,
    one full pass is on the order of a few cents — fine for weekly
    CI, too expensive for every PR.
  * Baseline storage — PR diffs need the previous main-branch score
    to compute regressions. `tests/evals/thresholds.toml` already
    has `overall.max_regression_fraction`; the CI job needs to
    fetch the last main-branch eval-report.json artifact and diff.

Those are tracked under §6E-7 "PR #8 (잔여 확장)" in docs/ARCHITECTURE.md.
The skeleton exists so the contract (JSON shape + CLI args) is frozen
before the LLM wiring is built out.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


@dataclass
class GoldEntry:
    query: str
    gold_keys: list[str]
    domain: str


@dataclass
class CorpusEntry:
    key: str
    content: str


def load_jsonl(path: Path) -> Iterable[dict[str, Any]]:
    with path.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            yield json.loads(line)


def parse_goldens(path: Path) -> list[GoldEntry]:
    return [
        GoldEntry(
            query=e["query"],
            gold_keys=list(e.get("gold_keys", [])),
            domain=str(e.get("domain", "unknown")),
        )
        for e in load_jsonl(path)
    ]


def parse_corpus(path: Path) -> dict[str, CorpusEntry]:
    return {
        e["key"]: CorpusEntry(key=e["key"], content=e["content"])
        for e in load_jsonl(path)
    }


def run_llm_judge(
    goldens: list[GoldEntry],
    corpus: dict[str, CorpusEntry],
    judge_model: str,
    retrieval_endpoint: str | None,
) -> dict[str, Any]:
    """
    Returns a report in the same shape as `moa_eval`'s JSON so downstream
    CI tooling can render a unified table.

    Not yet implemented — emits a skeleton report with every metric set
    to `null` so the JSON consumer can still parse successfully.
    """
    # Group by domain for per-domain averages (same as the Rust harness).
    from collections import defaultdict

    per_domain: dict[str, dict[str, Any]] = defaultdict(
        lambda: {
            "faithfulness": None,
            "answer_relevance": None,
            "queries": 0,
        }
    )
    for g in goldens:
        per_domain[g.domain]["queries"] += 1

    sets = [
        {
            "domain": dom,
            "queries": info["queries"],
            "faithfulness": info["faithfulness"],
            "answer_relevance": info["answer_relevance"],
        }
        for dom, info in sorted(per_domain.items())
    ]
    return {
        "top_k": None,
        "sets": sets,
        "overall": {
            "domain": "overall",
            "queries": sum(s["queries"] for s in sets),
            "faithfulness": None,
            "answer_relevance": None,
        },
        "judge_model": judge_model,
        "retrieval_endpoint": retrieval_endpoint,
        "skeleton": True,  # flag so CI can detect an unfilled run
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="LLM-judged RAGAS evaluation for MoA retrieval (skeleton)."
    )
    parser.add_argument(
        "--goldens",
        type=Path,
        required=True,
        help="Path to a golden JSONL file (e.g. tests/evals/golden_ko.jsonl).",
    )
    parser.add_argument(
        "--corpus",
        type=Path,
        required=True,
        help="Path to the corpus JSONL file (tests/evals/corpus.jsonl).",
    )
    parser.add_argument(
        "--judge-model",
        default="gpt-4o-mini",
        help="LLM model used for faithfulness / answer_relevance judgement.",
    )
    parser.add_argument(
        "--retrieval-endpoint",
        default=None,
        help="HTTP endpoint that serves the retrieval pipeline (future work).",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Write JSON report to PATH (defaults to stdout).",
    )
    args = parser.parse_args()

    goldens = parse_goldens(args.goldens)
    corpus = parse_corpus(args.corpus)
    report = run_llm_judge(
        goldens=goldens,
        corpus=corpus,
        judge_model=args.judge_model,
        retrieval_endpoint=args.retrieval_endpoint,
    )
    payload = json.dumps(report, indent=2, ensure_ascii=False)

    if args.output:
        args.output.write_text(payload, encoding="utf-8")
        print(f"wrote LLM judge report → {args.output}")
    else:
        print(payload)
    return 0


if __name__ == "__main__":
    sys.exit(main())
