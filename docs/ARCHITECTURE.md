# MoA — Architecture & Product Vision

> **Date**: 2026-03-01
> **Status**: Living document — updated with each major feature milestone
> **Audience**: AI reviewers (Gemini, Claude), human contributors, future maintainers

---

## 1. Product Vision

### What is MoA?

**MoA (Mixture of Agents)** is a cross-platform AI personal assistant
application that runs **independently on each user's device** — desktop
(Windows, macOS, Linux via Tauri) and mobile (iOS, Android). Each MoA app
instance contains a full **ZeroClaw autonomous agent runtime** with its own
local SQLite database for long-term memory. Multiple devices owned by the
same user **synchronize their long-term memories in real-time** via a
lightweight relay server, without ever persistently storing memory on the
server (patent: server-non-storage E2E encrypted memory sync).

MoA combines multiple AI models collaboratively to deliver results across
seven task categories — with particular emphasis on **real-time simultaneous
interpretation** and **AI-collaborative coding**.

### Core Thesis

> Single-model AI is limited. The best results come from multiple
> specialized AI models **collaborating, reviewing, and refining each
> other's work** — much like a team of human experts.

This "mixture of agents" philosophy applies everywhere:
- **Coding**: Claude Opus 4.6 writes code → Gemini 3.1 Pro reviews
  architecture → Claude validates Gemini's feedback → consensus-driven
  quality
- **Interpretation**: Gemini Live processes audio in real-time →
  segmentation engine commits phrase-level chunks → translation streams
  continuously
- **General tasks**: Local SLM (gatekeeper) handles simple queries → cloud
  LLM handles complex ones → routing optimizes cost/latency
- **Memory**: Each device runs independently but all memories converge via
  delta-based E2E encrypted sync

---

## 2. Deployment Architecture

### Per-User, Per-Device, Independent App

```
┌─────────────────────────────────────────────────────────────────┐
│                        User "Alice"                             │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Desktop App  │  │  Mobile App  │  │  Mobile App          │  │
│  │  (Tauri/Win)  │  │  (Android)   │  │  (iOS)               │  │
│  │              │  │              │  │                      │  │
│  │  ZeroClaw    │  │  ZeroClaw    │  │  ZeroClaw            │  │
│  │  + SQLite    │  │  + SQLite    │  │  + SQLite            │  │
│  │  + sqlite-vec│  │  + sqlite-vec│  │  + sqlite-vec        │  │
│  │  + FTS5      │  │  + FTS5      │  │  + FTS5              │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                      │              │
│         └────────┬────────┴──────────────────────┘              │
│                  │ E2E encrypted delta sync                     │
│                  ▼                                              │
│         ┌────────────────┐                                     │
│         │ Railway Relay   │  ← 5-minute TTL buffer only        │
│         │ Server          │  ← no persistent memory storage    │
│         └────────────────┘                                     │
└─────────────────────────────────────────────────────────────────┘
```

**Key principles:**
1. Each MoA app instance **works independently** — no server required for
   normal AI operations
2. Each device has its **own SQLite with long-term memory** (sqlite-vec for
   embeddings, FTS5 for full-text search)
3. Memory sync happens **peer-to-peer via relay** — the relay server holds
   data for at most **5 minutes** then deletes it
4. A user can install MoA on **multiple devices** — all share the same
   memory through real-time sync
5. **Normal AI operations do NOT go through the relay server** — the app
   calls LLM APIs directly from the device

### LLM API Key Model

```
┌─────────────────────────────────────────────────────────────┐
│ User has own API keys?                                      │
│                                                             │
│   YES → App uses user's keys directly                       │
│         → Calls the most powerful latest model available     │
│         → No credit deduction                               │
│         → No server involvement                             │
│                                                             │
│   NO  → App fetches operator's keys from Railway server     │
│         → User must pre-purchase credits                    │
│         → Each API call costs 2x the actual API cost        │
│         → Default model: Gemini 3.0 Flash (cost-effective)  │
│         → Voice/interpretation: Gemini 2.5 Flash Live API   │
└─────────────────────────────────────────────────────────────┘
```

| Scenario | API Key Source | Model Used | Billing |
|----------|---------------|------------|---------|
| User has key | User's own | Latest top-tier model for that provider | Free (user pays provider directly) |
| User has no key | Operator's key (from Railway env) | Gemini 3.0 Flash (default) | 2x actual API cost in credits |
| Voice interpretation | User's or operator's | Gemini 2.5 Flash Live API | Same rules as above |

### Remote Access via Channels

Users can interact with their MoA app from **any device** (even without
MoA installed) through messaging channels:

```
┌────────────────┐     ┌────────────┐     ┌──────────────────┐
│ Any device     │────▸│  Channel   │────▸│  User's MoA app  │
│ (no MoA app)  │◂────│  (relay)   │◂────│  (on home device)│
└────────────────┘     └────────────┘     └──────────────────┘
```

**Supported channels:**
- **KakaoTalk** (MoA addition — not in upstream ZeroClaw)
- Telegram
- Discord
- Slack
- LINE
- Web chat (homepage)

Users send messages through these channels to their remote MoA device,
which processes the request and sends back the response through the same
channel.

### Web Chat Access

A web-based chat interface on the MoA homepage allows users to:
- Send commands to their remote MoA app instance
- Receive responses in real-time
- No MoA app installation required on the browsing device
- Authenticated connection to the user's registered MoA devices

---

## 3. Patent: Server-Non-Storage E2E Encrypted Memory Sync

### Title (발명의 명칭)

**서버 비저장 방식의 다중 기기 간 종단간 암호화 메모리 동기화 시스템 및 방법**

(Server-Non-Storage Multi-Device End-to-End Encrypted Memory
Synchronization System and Method)

### Problem Statement

Conventional cloud-sync approaches store user data persistently on a
central server, creating:
- Privacy risk (server breach exposes all user data)
- Single point of failure
- Regulatory compliance burden (GDPR, data residency)
- Server storage cost scaling with user count

### Invention Summary

A system where **each user device maintains its own authoritative copy**
of long-term memory in a local SQLite database, and **synchronizes changes
(deltas) with other devices via a relay server that never persistently
stores the data**.

### Architecture

```
Device A                    Relay Server              Device B
┌──────────┐               ┌──────────────┐          ┌──────────┐
│ SQLite   │               │              │          │ SQLite   │
│ (full    │──encrypt──▸   │  TTL buffer  │   ◂──────│ (full    │
│  memory) │  delta        │  (5 min max) │  fetch   │  memory) │
│          │               │              │  + apply │          │
│ vec+FTS5 │               │  No persist  │          │ vec+FTS5 │
└──────────┘               └──────────────┘          └──────────┘
```

### Core Mechanisms

#### 1. Delta-Based Sync (델타 기반 동기화)

- When a memory entry is created/updated/deleted on any device, only the
  **delta (change)** is transmitted — not the entire memory store
- Deltas include: operation type (insert/update/delete), entry ID, content
  hash, timestamp, vector embedding diff
- This minimizes bandwidth and enables efficient sync even on slow
  mobile networks

#### 2. End-to-End Encryption (종단간 암호화)

- All deltas are encrypted on the **sending device** before transmission
- The relay server **cannot read** the content — it only stores opaque
  encrypted blobs
- Decryption happens only on the **receiving device**
- Key derivation: device-specific keys derived from user's master secret
  via HKDF (see `src/security/device_binding.rs`)

#### 3. Server TTL Buffer (서버 임시 보관 — 5분 TTL)

- The relay server (Railway) holds encrypted deltas for a **maximum of
  5 minutes**
- If the receiving device is online, it fetches and applies deltas
  immediately
- If the receiving device comes online within 5 minutes, it picks up
  buffered deltas
- After 5 minutes, undelivered deltas are **permanently deleted** from
  the server
- The server **never has persistent storage of any user memory**

#### 4. Offline Reconciliation (오프라인 기기 동기화)

When a device comes online after being offline for more than 5 minutes:
- It cannot rely on the relay server buffer (TTL expired)
- Instead, it performs **peer-to-peer full reconciliation** with another
  online device of the same user
- Reconciliation uses vector clock / timestamp comparison to resolve
  conflicts
- Last-write-wins with semantic merge for non-conflicting concurrent edits

#### 5. Conflict Resolution (충돌 해결)

| Scenario | Resolution Strategy |
|----------|-------------------|
| Same entry edited on two devices | Last-write-wins (by timestamp) |
| Entry deleted on A, edited on B | Delete wins (tombstone preserved) |
| New entries on both devices | Both kept (no conflict) |
| Embedding vectors diverged | Re-compute from merged text content |

### Implementation in MoA

| Component | Module | Description |
|-----------|--------|-------------|
| Local memory store | `src/memory/` | SQLite + sqlite-vec + FTS5 per device |
| Sync engine | `src/sync/` | Delta generation, encryption, relay communication |
| E2E encryption | `src/security/` | HKDF key derivation, ChaCha20-Poly1305 encryption |
| Relay client | `src/sync/` | WebSocket connection to Railway relay server |
| Conflict resolver | `src/sync/coordinator.rs` | Vector clock comparison, merge strategies |
| Device binding | `src/security/device_binding.rs` | Device identity, key pairing |

### Security Properties

1. **Zero-knowledge relay**: Server cannot decrypt any data
2. **Forward secrecy**: Key rotation per sync session
3. **Device compromise isolation**: Compromising one device does not
   expose keys of other devices
4. **Deletion guarantee**: Server data is ephemeral (5-minute TTL)
5. **No server-side backup**: There is no "cloud copy" of user data

---

## 4. Target Users

| User type | Primary use case |
|-----------|-----------------|
| **Korean business professionals** | Real-time Korean ↔ English/Japanese/Chinese interpretation for meetings, calls |
| **Developers** | AI-assisted coding with Claude + Gemini self-checking review |
| **Content creators** | Document drafting, image/video/music generation |
| **General users** | Web search, Q&A, daily tasks with multi-model intelligence |
| **Multi-device users** | Seamless AI assistant across desktop + mobile with synced memory |
| **Channel users** | Interact with MoA via KakaoTalk, Telegram, Discord, web chat without installing the app |

---

## 5. Task Categories

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
| **Channels** | 채널 | KakaoTalk, Telegram, Discord, Slack, LINE, Web chat management |
| **Billing** | 결제 | Credits, usage, payment |
| **MyPage** | 마이페이지 | User profile, API key settings, device management |

---

## 6. System Architecture

### High-Level Module Map

```
src/
├── main.rs              # CLI entrypoint, command routing
├── lib.rs               # Module exports, shared enums
├── config/              # Schema + config loading/merging
├── agent/               # Orchestration loop
├── gateway/             # Webhook/gateway server
├── security/            # Policy, pairing, secret store, E2E encryption
├── memory/              # SQLite + sqlite-vec + FTS5 long-term memory
├── providers/           # Model providers (Gemini, Claude, OpenAI, Ollama, etc.)
├── channels/            # KakaoTalk, Telegram, Discord, Slack, LINE, Web chat
├── tools/               # Tool execution (shell, file, memory, browser)
├── coding/              # Multi-model code review pipeline ← MoA addition
├── voice/               # Real-time voice interpretation  ← MoA addition
├── sandbox/             # Coding sandbox (run→observe→fix loop)
├── task_category.rs     # Category definitions + tool routing ← MoA addition
├── gatekeeper/          # Local SLM intent classification  ← MoA addition
├── billing/             # Credit-based billing system      ← MoA addition
├── sync/                # E2E encrypted memory sync engine (patent impl)
├── peripherals/         # Hardware peripherals (STM32, RPi GPIO)
├── runtime/             # Runtime adapters
├── observability/       # Tracing, metrics
├── telemetry/           # Telemetry collection
├── plugins/             # Plugin loader
└── ...                  # (auth, hooks, rag, etc.)
```

### Platform Targets

| Platform | Technology | ZeroClaw Runtime | SQLite |
|----------|-----------|-----------------|--------|
| **Windows** | Tauri 2.x | Native Rust binary | Local file |
| **macOS** | Tauri 2.x | Native Rust binary | Local file |
| **Linux** | Tauri 2.x | Native Rust binary | Local file |
| **Android** | Tauri 2.x Mobile | Native Rust (NDK) | Local file |
| **iOS** | Tauri 2.x Mobile | Native Rust (static lib) | Local file |

Every platform runs the **same ZeroClaw Rust core** — the app is not a
thin client. Each device is a fully autonomous AI agent.

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

## 7. Voice / Simultaneous Interpretation

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
Client mic ─▸ audio_chunk ─▸ SimulSession ─▸ Gemini 2.5 Flash Live API
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

### WebSocket Event Protocol (`src/voice/events.rs`)

Client ↔ Server messages use JSON text frames:

**Client → Server**: `SessionStart`, `SessionStop`, `AudioChunk`,
`ActivitySignal`

**Server → Client**: `SessionReady`, `PartialSrc`, `CommitSrc`,
`PartialTgt`, `CommitTgt`, `AudioOut`, `TurnComplete`, `Interrupted`,
`Error`, `SessionEnded`

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

## 8. Coding / Multi-Model Review Pipeline

### Goal

Create an autonomous coding assistant where **Claude Opus 4.6 writes code**
and **Gemini 3.1 Pro reviews it for architecture alignment**, then Claude
validates Gemini's findings — producing self-checked, high-quality code
through AI-to-AI collaboration.

### The Pipeline

```
Code diff ──┬─▸ GeminiReviewer ─▸ ReviewReport ─┐
            │   (Architecture Gatekeeper)        │
            │   Gemini 3.1 Pro                   ▼
            └─▸ ClaudeReviewer ──────────────────┼─▸ ConsensusReport
                (Sees Gemini's findings,         │
                 validates or refutes them)       │
                Claude Opus 4.6                  ▼
                               merge findings + consensus verdict
```

### Reviewer Roles

| Reviewer | Model | Role |
|----------|-------|------|
| **GeminiReviewer** | Gemini 3.1 Pro | Architecture gatekeeper: design alignment, structural issues, efficiency |
| **ClaudeReviewer** | Claude Opus 4.6 | Implementation quality: correctness, efficiency, validates/refutes Gemini's findings |

### How It Works

1. Claude Opus 4.6 writes code and self-reviews for errors
2. Code is pushed as a PR
3. GitHub Actions triggers Gemini review automatically
4. Gemini 3.1 Pro reviews against `docs/ARCHITECTURE.md` and `CLAUDE.md`
5. Gemini posts structured findings on the PR as a comment
6. Claude reads Gemini's review → accepts valid points → pushes fixes
7. Cycle repeats until consensus is reached

### Consensus Rules

- If **any** reviewer says `REQUEST_CHANGES` → overall verdict =
  `REQUEST_CHANGES`
- If **all** reviewers say `APPROVE` → overall verdict = `APPROVE`
- Otherwise → `COMMENT`

### Severity Levels

| Level | Meaning | Example |
|-------|---------|---------|
| `CRITICAL` | Must fix: correctness/security/architecture violation | SQL injection, unsafe unwrap |
| `HIGH` | Should fix before merge | Missing error handling, SRP violation |
| `MEDIUM` | Good to fix, not blocking | Inefficient algorithm |
| `LOW` | Informational suggestion | Minor style preference |

### GitHub Actions Integration

`.github/workflows/gemini-pr-review.yml`:

1. PR opened/updated → workflow triggers
2. Extracts diff + reads `CLAUDE.md`, `docs/ARCHITECTURE.md`
3. Calls Gemini API with architecture-aware review prompt
4. Posts structured review comment on the PR
5. Comment is idempotent (updates existing, doesn't duplicate)

**Required secret**: `GEMINI_API_KEY` in repository Actions secrets.

---

## 9. Coding Sandbox (Run → Observe → Fix)

### Six-Phase Methodology

| Phase | Purpose | Key Actions |
|-------|---------|-------------|
| **1. Comprehend** | Understand before changing | Read existing code, identify patterns |
| **2. Plan** | Define scope | Acceptance criteria, minimal approach |
| **3. Prepare** | Set up environment | Snapshot working state, install deps |
| **4. Implement** | Write + verify | Code → run → observe → classify errors → fix → repeat |
| **5. Validate** | Final checks | Format, lint, type-check, build, full test suite |
| **6. Deliver** | Ship | Commit with clear message, report results |

### Recurring Error Detection

If the same error class appears **3+ times**, the sandbox:
1. **Rolls back** to last checkpoint
2. **Switches strategy** (alternative approach)
3. **Escalates** to user if strategies exhausted

---

## 10. Configuration Reference

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
gemini_model = "gemini-2.5-flash"  # Upgrade to gemini-3.1-pro when available
claude_model = "claude-sonnet-4-6"
enable_secondary_review = true     # Claude validates Gemini's findings
max_diff_chars = 120000
# gemini_api_key = "..."           # or GEMINI_API_KEY env var
# claude_api_key = "..."           # or ANTHROPIC_API_KEY env var
```

---

## 11. Patent-Relevant Innovation Areas

### Innovation 1: Server-Non-Storage E2E Encrypted Memory Sync

See [Section 3](#3-patent-server-non-storage-e2e-encrypted-memory-sync)
for full specification.

**Claims**: Delta-based sync, 5-minute TTL relay, zero-knowledge server,
device-local authoritative storage, offline reconciliation.

### Innovation 2: Commit-Point Segmentation for Simultaneous Interpretation

Real-time phrase-level audio translation using a three-pointer architecture
(committed | stable-uncommitted | unstable) with hybrid boundary detection
(punctuation, silence, length-cap). Enables translation to begin **before
the speaker finishes a sentence**.

### Innovation 3: Multi-Model Consensus Code Review Pipeline

Automated code quality assurance where Model A (Claude) generates code,
Model B (Gemini) reviews for architecture alignment, Model A validates
Model B's findings, and a pipeline merges findings with severity-weighted
deduplication. AI models **autonomously discuss and refine** code quality.

### Innovation 4: Task-Category-Aware Tool Routing

Dynamic tool availability per task category — each category exposes only
the tools relevant to its domain, reducing attack surface and improving
model focus. The coding category gets all tools; the translation category
gets minimal tools.

### Innovation 5: Six-Phase Structured Coding with Autonomous Repair Loop

Comprehend → Plan → Prepare → Implement (run→observe→fix) → Validate →
Deliver, with error classification, recurring-error detection, rollback
checkpoints, and multi-signal observation (exit code + stderr + server
health + DOM snapshots).

---

## 12. Design Principles

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

## 13. Risk Tiers

| Tier | Scope | Review depth |
|------|-------|--------------|
| **Low** | docs, chore, tests-only | Lightweight checks |
| **Medium** | Most `src/**` behavior changes | Standard review |
| **High** | `src/security/**`, `src/runtime/**`, `src/gateway/**`, `src/tools/**`, `.github/workflows/**`, `src/sync/**` | Full validation + boundary testing |

---

## 14. Technology Stack

| Component | Technology |
|-----------|-----------|
| **Language** | Rust (edition 2021, MSRV 1.87) |
| **Async runtime** | Tokio |
| **App framework** | Tauri 2.x (desktop + mobile) |
| **HTTP client** | reqwest |
| **WebSocket** | tungstenite 0.28 |
| **Serialization** | serde + serde_json |
| **CLI** | clap |
| **Database** | SQLite (rusqlite) + sqlite-vec + FTS5 |
| **AI Models** | Gemini (Google), Claude (Anthropic), OpenAI, Ollama |
| **Default LLM** | Gemini 3.0 Flash (cost-effective default) |
| **Voice/Interp** | Gemini 2.5 Flash Native Audio (Live API) |
| **Coding review** | Claude Opus 4.6 + Gemini 3.1 Pro |
| **Relay server** | Railway (WebSocket relay, no persistent storage) |
| **Encryption** | ChaCha20-Poly1305, HKDF key derivation |
| **CI** | GitHub Actions |

---

## 15. Implementation Roadmap

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
- [x] Architecture documentation (this document)

### In Progress / Planned

- [ ] KakaoTalk channel implementation
- [ ] E2E encrypted memory sync (patent implementation)
- [ ] Railway relay server setup (5-minute TTL buffer)
- [ ] Offline reconciliation / peer-to-peer full sync
- [ ] Tauri desktop app (Windows, macOS, Linux)
- [ ] Tauri mobile app (iOS, Android)
- [ ] Web chat interface for remote MoA access
- [ ] User settings page (API key input, device management)
- [ ] Operator API key fallback with 2x credit billing
- [ ] WebSocket gateway endpoint for voice interpretation
- [ ] Gatekeeper SLM integration (Ollama-based local inference)
- [ ] Channel-specific voice features (KakaoTalk, Telegram, Discord)
- [ ] Multi-user simultaneous interpretation (conference mode)
- [ ] Coding sandbox integration with review pipeline
- [ ] Automated fix-apply from review findings
- [ ] Image/Video/Music generation tool integrations

---

## 16. For AI Reviewers

When reviewing a PR against this architecture:

1. **Check architecture alignment**: Does the change follow the trait-driven
   pattern? Does it belong in the right module?
2. **Check design principles**: KISS, YAGNI, SRP, fail-fast,
   secure-by-default
3. **Check MoA-specific contracts**: Voice segmentation parameters, event
   protocol compatibility, category tool routing, memory sync protocol
4. **Check risk tier**: High-risk paths (`security/`, `gateway/`, `tools/`,
   `workflows/`, `sync/`) need extra scrutiny
5. **Check backward compatibility**: Config keys are public API — changes
   need migration documentation
6. **Check platform independence**: Code must work on all 5 platforms
   (Windows, macOS, Linux, Android, iOS) — avoid platform-specific
   assumptions unless behind a `cfg` gate
7. **Check memory sync contract**: Any change to `memory/` or `sync/` must
   preserve the delta-based, E2E encrypted, server-non-storage invariants
8. **Check API key handling**: Never log API keys, never send them to the
   relay server, always handle both user-key and operator-key paths
