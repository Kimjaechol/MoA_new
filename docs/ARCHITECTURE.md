# MoA — Architecture & Product Vision

> **Date**: 2026-03-01
> **Status**: Living document — updated with each major feature milestone
> **Audience**: AI reviewers (Gemini, Claude), human contributors, future maintainers

---

## 1. Product Vision

### What is MoA?

**MoA (Mixture of Agents)** is an AI-powered personal assistant application
built on the ZeroClaw autonomous agent runtime. It combines multiple AI
models collaboratively to deliver high-quality results across seven task
categories — with particular emphasis on **real-time simultaneous
interpretation** and **AI-collaborative coding**.

### Core Thesis

> Single-model AI is limited. The best results come from multiple
> specialized AI models **collaborating, reviewing, and refining each
> other's work** — much like a team of human experts.

This "mixture of agents" philosophy applies everywhere:
- **Coding**: Claude writes code → Gemini reviews architecture → Claude
  validates Gemini's feedback → consensus-driven quality
- **Interpretation**: Gemini Live processes audio in real-time → segmentation
  engine commits phrase-level chunks → translation streams continuously
- **General tasks**: Local SLM (gatekeeper) handles simple queries → cloud
  LLM handles complex ones → routing optimizes cost/latency

### Patent-Relevant Innovation Areas

1. **Commit-point segmentation for simultaneous interpretation**:
   Real-time phrase-level audio translation using a three-pointer
   architecture (committed | stable-uncommitted | unstable) with hybrid
   boundary detection (punctuation, silence, length-cap)

2. **Multi-model consensus code review pipeline**:
   Automated code quality assurance where Model A generates code, Model B
   reviews for architecture alignment, Model A validates Model B's findings,
   and a pipeline merges findings with severity-weighted deduplication

3. **Task-category-aware tool routing**:
   Dynamic tool availability per task category — each category exposes only
   the tools relevant to its domain, reducing attack surface and improving
   model focus

4. **Six-phase structured coding methodology with autonomous repair loop**:
   Comprehend → Plan → Prepare → Implement (run→observe→fix) → Validate →
   Deliver, with error classification, recurring-error detection, and
   rollback checkpoints

---

## 2. Target Users

| User type | Primary use case |
|-----------|-----------------|
| **Korean business professionals** | Real-time Korean ↔ English/Japanese/Chinese interpretation for meetings, calls |
| **Developers** | AI-assisted coding with self-checking multi-model review |
| **Content creators** | Document drafting, image/video/music generation |
| **General users** | Web search, Q&A, daily tasks with multi-model intelligence |

---

## 3. Task Categories

MoA organizes all user interactions into **7 top-bar categories** and
**3 sidebar navigation items**:

### Top-Bar (Task Modes)

| Category | Korean | UI Mode | Tool Scope |
|----------|--------|---------|------------|
| **WebGeneral** | 웹/일반 | default chat | BASE + VISION |
| **Document** | 문서 | default chat | BASE |
| **Coding** | 코딩 | `sandbox` | ALL tools (unrestricted) |
| **Image** | 이미지 | default chat | BASE + VISION |
| **Music** | 음악 | default chat | BASE |
| **Video** | 비디오 | default chat | BASE + VISION |
| **Translation** | 통역 | `voice_interpret` | MINIMAL (memory + browser + file I/O) |

### Sidebar (Navigation)

| Item | Korean | Purpose |
|------|--------|---------|
| **Channels** | 채널 | Telegram, Discord, Slack management |
| **Billing** | 결제 | Credits, usage, payment |
| **MyPage** | 마이페이지 | User profile, settings |

---

## 4. System Architecture

### High-Level Module Map

```
src/
├── main.rs              # CLI entrypoint, command routing
├── lib.rs               # Module exports, shared enums
├── config/              # Schema + config loading/merging
├── agent/               # Orchestration loop
├── gateway/             # Webhook/gateway server
├── security/            # Policy, pairing, secret store
├── memory/              # Markdown/SQLite memory + embeddings
├── providers/           # Model providers (OpenAI, Anthropic, Gemini, Ollama, etc.)
├── channels/            # Telegram, Discord, Slack, LINE, etc.
├── tools/               # Tool execution (shell, file, memory, browser)
├── coding/              # Multi-model code review pipeline ← MoA addition
├── voice/               # Real-time voice interpretation  ← MoA addition
├── sandbox/             # Coding sandbox (run→observe→fix loop)
├── task_category.rs     # Category definitions + tool routing ← MoA addition
├── gatekeeper/          # Local SLM intent classification  ← MoA addition
├── billing/             # Credit-based billing system      ← MoA addition
├── peripherals/         # Hardware peripherals (STM32, RPi GPIO)
├── runtime/             # Runtime adapters
├── observability/       # Tracing, metrics
├── sync/                # P2P sync engine
├── telemetry/           # Telemetry collection
├── plugins/             # Plugin loader
└── ...                  # (auth, hooks, rag, etc.)
```

### Trait-Driven Extension Points

| Trait | Location | Purpose |
|-------|----------|---------|
| `Provider` | `src/providers/traits.rs` | Model API abstraction |
| `Channel` | `src/channels/traits.rs` | Messaging platform abstraction |
| `Tool` | `src/tools/traits.rs` | Tool execution interface |
| `Memory` | `src/memory/traits.rs` | Memory backend abstraction |
| `Observer` | `src/observability/traits.rs` | Observability sink |
| `RuntimeAdapter` | `src/runtime/traits.rs` | Runtime environment abstraction |
| `Peripheral` | `src/peripherals/traits.rs` | Hardware board abstraction |
| `VoiceProvider` | `src/voice/pipeline.rs` | Voice API streaming |
| `CodeReviewer` | `src/coding/traits.rs` | AI code review agent |

**Rule**: New capabilities are added by implementing traits + factory
registration, NOT by cross-module rewrites.

---

## 5. Voice / Simultaneous Interpretation

### Goal

Deliver **real-time simultaneous interpretation** that translates speech
*while the speaker is still talking*, at phrase-level granularity — not
waiting for complete sentences.

### Why This Matters

Traditional interpretation apps wait for the speaker to finish a sentence
before translating. This creates unnatural pauses and loses the speaker's
pacing and intent. MoA's simultaneous interpretation:

- Translates **phrase by phrase** as the speaker talks
- Preserves the speaker's **deliberate pauses and pacing**
- Handles **25 languages** with bidirectional auto-detection
- Supports **domain specialization** (business, medical, legal, technical)

### Architecture

```
Client mic ─▸ audio_chunk ─▸ SimulSession ─▸ Gemini Live API
                                   │
                                   ├─ InputTranscript ─▸ SegmentationEngine
                                   │                         │
                                   │            commit_src / partial_src
                                   │                         │
                                   ├─ Audio (translated) ──▸ audio_out ──▸ Client speaker
                                   └─ OutputTranscript ────▸ commit_tgt ──▸ Client subtitles
```

### Commit-Point Segmentation Engine (`src/voice/simul.rs`)

The core innovation: a **three-pointer segmentation** architecture.

```
|---committed---|---stable-uncommitted---|---unstable (may change)---|
0        last_committed      stable_end              partial_end
```

- **Committed**: Text already sent for translation. Never re-sent.
- **Stable-uncommitted**: High confidence text, not yet committed.
- **Unstable**: Trailing N characters that ASR may still revise.

#### Commit Decision Strategy (hybrid)

| Strategy | Trigger | Purpose |
|----------|---------|---------|
| **Boundary** | Punctuation (`.` `!` `?` `。` `,` `、`) | Natural language breaks |
| **Silence** | No input for `silence_commit_ms` | Speaker pauses |
| **Length cap** | Stable text > `max_uncommitted_chars` | Prevent unbounded buffering |

#### Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| `unstable_tail_chars` | 8 | Trailing chars considered mutable |
| `min_commit_chars` | 10 | Minimum text before committing |
| `max_uncommitted_chars` | 80 | Force commit threshold |
| `silence_commit_ms` | 600 | Silence detection threshold |

### WebSocket Event Protocol (`src/voice/events.rs`)

Client ↔ Server messages use JSON text frames:

**Client → Server**: `SessionStart`, `SessionStop`, `AudioChunk`, `ActivitySignal`
**Server → Client**: `SessionReady`, `PartialSrc`, `CommitSrc`, `PartialTgt`,
`CommitTgt`, `AudioOut`, `TurnComplete`, `Interrupted`, `Error`, `SessionEnded`

### Interpretation Modes

| Mode | Description |
|------|-------------|
| `simul` | Simultaneous: translate while speaker talks |
| `consecutive` | Wait for speaker to finish, then translate |
| `bidirectional` | Auto-detect language and interpret both ways |

### Supported Languages (25)

Korean, Japanese, Chinese (Simplified & Traditional), Thai, Vietnamese,
Indonesian, Malay, Filipino, Hindi, English, Spanish, French, German,
Italian, Portuguese, Dutch, Polish, Czech, Swedish, Danish, Russian,
Ukrainian, Turkish, Arabic

---

## 6. Coding / Multi-Model Review Pipeline

### Goal

Create an autonomous coding assistant that doesn't just generate code,
but **verifies its own work through multi-model collaboration** — the
way a senior engineer writes code and a tech lead reviews it.

### The Pipeline

```
Code diff ──┬─▸ GeminiReviewer ─▸ ReviewReport ─┐
            │   (Architecture Gatekeeper)        │
            │                                    ▼
            └─▸ ClaudeReviewer ──────────────────┼─▸ ConsensusReport
                (Sees Gemini's findings,         │
                 validates or refutes them)       │
                                                 ▼
                               merge findings + consensus verdict
```

### Reviewer Roles

| Reviewer | Model | Role |
|----------|-------|------|
| **GeminiReviewer** | Gemini 2.5 Flash+ | Architecture gatekeeper: design alignment, structural issues |
| **ClaudeReviewer** | Claude Sonnet/Opus | Implementation quality: correctness, efficiency, prior review validation |

### Consensus Rules

- If **any** reviewer says `REQUEST_CHANGES` → overall verdict = `REQUEST_CHANGES`
- If **all** reviewers say `APPROVE` → overall verdict = `APPROVE`
- Otherwise → `COMMENT`

### Finding Deduplication

When multiple reviewers flag the same `(file, category)`:
- **Highest severity wins**
- Descriptions are preserved from the highest-severity report
- Final findings are sorted by severity (critical first)

### Severity Levels

| Level | Meaning | Example |
|-------|---------|---------|
| `CRITICAL` | Must fix: correctness/security/architecture violation | SQL injection, unsafe unwrap in production path |
| `HIGH` | Should fix before merge | Missing error handling, SRP violation |
| `MEDIUM` | Good to fix, not blocking | Inefficient algorithm, unclear naming |
| `LOW` | Informational suggestion | Minor style preference |

### GitHub Actions Integration

The pipeline runs automatically on every PR via
`.github/workflows/gemini-pr-review.yml`:

1. PR opened/updated → workflow triggers
2. Extracts diff + reads `CLAUDE.md`, `docs/ARCHITECTURE.md`
3. Calls Gemini API with architecture-aware review prompt
4. Posts structured review comment on the PR
5. Comment is idempotent (updates existing, doesn't duplicate)

**Required secret**: `GEMINI_API_KEY` in repository settings.

---

## 7. Coding Sandbox (Run → Observe → Fix)

### Six-Phase Methodology

| Phase | Purpose | Key Actions |
|-------|---------|-------------|
| **1. Comprehend** | Understand before changing | Read existing code, identify patterns |
| **2. Plan** | Define scope | Acceptance criteria, minimal approach |
| **3. Prepare** | Set up environment | Snapshot working state, install deps |
| **4. Implement** | Write + verify | Code → run → observe → classify errors → fix → repeat |
| **5. Validate** | Final checks | Format, lint, type-check, build, full test suite |
| **6. Deliver** | Ship | Commit with clear message, report results |

### Error Classification

| Type | Strategy |
|------|----------|
| Syntax | Direct fix from error message |
| Runtime | Trace execution path, add guards |
| Type | Align types, add conversions |
| Dependency | Install/update package |
| Network | Add retry/timeout handling |
| Timeout | Optimize or increase budget |

### Recurring Error Detection

If the same error class appears **3+ times**, the sandbox:
1. **Rolls back** to last checkpoint
2. **Switches strategy** (alternative approach)
3. **Escalates** to user if strategies exhausted

---

## 8. Configuration Reference

### VoiceConfig

```toml
[voice]
enabled = true
max_sessions_per_user = 5
default_source_language = "ko"
default_target_language = "en"
default_interp_mode = "simul"      # simul | consecutive | bidirectional
min_commit_chars = 10
max_uncommitted_chars = 80
silence_commit_ms = 600
silence_duration_ms = 300
prefix_padding_ms = 100
# gemini_api_key = "..."           # or GEMINI_API_KEY env var
# openai_api_key = "..."           # or OPENAI_API_KEY env var
# default_provider = "gemini"      # gemini | openai
```

### CodingConfig

```toml
[coding]
review_enabled = false             # Enable multi-model review
gemini_model = "gemini-2.5-flash"
claude_model = "claude-sonnet-4-6"
enable_secondary_review = true     # Claude validates Gemini's findings
max_diff_chars = 120000
# gemini_api_key = "..."           # or GEMINI_API_KEY env var
# claude_api_key = "..."           # or ANTHROPIC_API_KEY env var
```

---

## 9. Design Principles

These are **mandatory constraints**, not guidelines:

| Principle | Rule |
|-----------|------|
| **KISS** | Prefer straightforward control flow over clever meta-programming |
| **YAGNI** | No speculative features — concrete accepted use case required |
| **DRY + Rule of Three** | Extract shared logic only after 3+ repetitions |
| **SRP + ISP** | One concern per module, narrow trait interfaces |
| **Fail Fast** | Explicit errors for unsupported states, never silently broaden |
| **Secure by Default** | Deny-by-default, no secret logging, minimal exposure |
| **Determinism** | Reproducible behavior, no flaky tests |
| **Reversibility** | Small commits, clear rollback paths |

---

## 10. Risk Tiers

| Tier | Scope | Review depth |
|------|-------|--------------|
| **Low** | docs, chore, tests-only | Lightweight checks |
| **Medium** | Most `src/**` behavior changes | Standard review |
| **High** | `src/security/**`, `src/runtime/**`, `src/gateway/**`, `src/tools/**`, `.github/workflows/**` | Full validation + boundary testing |

---

## 11. Technology Stack

| Component | Technology |
|-----------|-----------|
| **Language** | Rust (edition 2021, MSRV 1.87) |
| **Async runtime** | Tokio |
| **HTTP client** | reqwest |
| **WebSocket** | tungstenite 0.28 |
| **Serialization** | serde + serde_json |
| **CLI** | clap |
| **Database** | SQLite (rusqlite) |
| **AI Models** | Gemini (Google), Claude (Anthropic), OpenAI, Ollama |
| **Voice** | Gemini 2.5 Flash Native Audio (Live API) |
| **CI** | GitHub Actions |

---

## 12. Implementation Roadmap

### Completed

- [x] ZeroClaw upstream sync (1692 commits merged)
- [x] Task category system with tool routing (7 categories)
- [x] Voice pipeline with 25-language support
- [x] Gemini Live WebSocket client with automatic VAD
- [x] Simultaneous interpretation segmentation engine
- [x] WebSocket event protocol for client-server communication
- [x] SimulSession manager (audio forwarding + event processing)
- [x] Multi-model code review pipeline (Gemini + Claude)
- [x] GitHub Actions Gemini PR review workflow
- [x] Coding sandbox 6-phase methodology
- [x] Translation UI manifest for frontend
- [x] Credit-based billing system

### In Progress / Planned

- [ ] Frontend web UI (Tauri-based desktop + web)
- [ ] WebSocket gateway endpoint for voice interpretation
- [ ] Gatekeeper SLM integration (Ollama-based local inference)
- [ ] Channel-specific voice features (Telegram, Discord)
- [ ] Multi-user simultaneous interpretation (conference mode)
- [ ] Coding sandbox integration with review pipeline
- [ ] Automated fix-apply from review findings
- [ ] Image/Video/Music generation tool integrations
- [ ] Mobile app (iOS/Android)

---

## 13. For AI Reviewers

When reviewing a PR against this architecture:

1. **Check architecture alignment**: Does the change follow the trait-driven
   pattern? Does it belong in the right module?
2. **Check design principles**: KISS, YAGNI, SRP, fail-fast, secure-by-default
3. **Check MoA-specific contracts**: Voice segmentation parameters, event
   protocol compatibility, category tool routing
4. **Check risk tier**: High-risk paths (`security/`, `gateway/`, `tools/`,
   `workflows/`) need extra scrutiny
5. **Check backward compatibility**: Config keys are public API — changes
   need migration documentation
