//! Coding sandbox: run → observe → fix autonomous loop.
//!
//! Implements a Replit-like (but more intelligent) iterative coding loop:
//!
//! 1. **Run** — execute code / start dev server in an isolated environment
//! 2. **Observe** — collect stdout, stderr, exit codes, server logs, and
//!    optionally DOM snapshots from a web preview
//! 3. **Analyse** — classify errors, detect recurring patterns, assess severity
//! 4. **Fix** — generate a targeted patch and apply it
//! 5. **Repeat** until success or budget exhaustion
//!
//! Key improvements over Replit's approach:
//!
//! - **Error classification** — categorizes errors (syntax, runtime, type,
//!   dependency, network, timeout) for targeted fix strategies
//! - **Recurring-error detection** — if the same error class appears 3+ times,
//!   switches strategy (rollback, alternative approach, or escalation)
//! - **Rollback checkpoints** — snapshots working state so a bad patch can
//!   be reverted instantly instead of accumulating breakage
//! - **Multi-signal observation** — combines exit code + stderr patterns +
//!   server health-check + optional DOM snapshot for richer context
//! - **Budget-aware** — hard limits on iterations and wall-clock time to
//!   prevent runaway loops
//! - **Severity scoring** — prioritises critical errors (crash, compile
//!   failure) over warnings, avoiding infinite cosmetic-fix loops

pub mod layout;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════
// Part 1 — Coding Workflow (structured methodology)
// ═══════════════════════════════════════════════════════════════════

/// The six-phase workflow that drives expert-level coding.
///
/// Each phase has a clear purpose, entry criteria, outputs, and
/// transition rules.  The sandbox loop (Part 2) is embedded inside
/// Phase 4 (Implement → Verify).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodingPhase {
    /// Phase 1 — Read and understand the existing codebase before touching it.
    Comprehend,
    /// Phase 2 — Define the precise scope: what changes, what doesn't, what the
    /// acceptance criteria are.
    Plan,
    /// Phase 3 — Prepare the environment: install deps, create branches,
    /// snapshot working state.
    Prepare,
    /// Phase 4 — Write code + run→observe→fix loop until all checks pass.
    Implement,
    /// Phase 5 — Final validation: full test suite, lint, type-check, build.
    Validate,
    /// Phase 6 — Deliver: commit with clear message, report to user.
    Deliver,
}

impl CodingPhase {
    pub const ALL: &'static [CodingPhase] = &[
        CodingPhase::Comprehend,
        CodingPhase::Plan,
        CodingPhase::Prepare,
        CodingPhase::Implement,
        CodingPhase::Validate,
        CodingPhase::Deliver,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            CodingPhase::Comprehend => "Comprehend",
            CodingPhase::Plan => "Plan",
            CodingPhase::Prepare => "Prepare",
            CodingPhase::Implement => "Implement",
            CodingPhase::Validate => "Validate",
            CodingPhase::Deliver => "Deliver",
        }
    }

    /// Next phase in the normal forward flow.
    pub fn next(self) -> Option<Self> {
        match self {
            CodingPhase::Comprehend => Some(CodingPhase::Plan),
            CodingPhase::Plan => Some(CodingPhase::Prepare),
            CodingPhase::Prepare => Some(CodingPhase::Implement),
            CodingPhase::Implement => Some(CodingPhase::Validate),
            CodingPhase::Validate => Some(CodingPhase::Deliver),
            CodingPhase::Deliver => None,
        }
    }
}

/// Detailed instructions for each coding phase, derived from the actual
/// methodology that makes expert coding agents effective.
pub fn phase_instructions(phase: CodingPhase) -> &'static str {
    match phase {
        CodingPhase::Comprehend => PHASE_COMPREHEND,
        CodingPhase::Plan => PHASE_PLAN,
        CodingPhase::Prepare => PHASE_PREPARE,
        CodingPhase::Implement => PHASE_IMPLEMENT,
        CodingPhase::Validate => PHASE_VALIDATE,
        CodingPhase::Deliver => PHASE_DELIVER,
    }
}

const PHASE_COMPREHEND: &str = "\
## Phase 1: Comprehend (Read Before Write)

Before writing ANY code, you MUST fully understand the existing codebase:

### Required Actions
1. **Read the target files** — Open and read every file you plan to modify.
   Never propose changes to code you haven't read.
2. **Understand the architecture** — Identify:
   - Module boundaries and dependency direction
   - Naming conventions and code style already in use
   - How similar features are implemented elsewhere
   - Extension points (traits, interfaces, factories)
3. **Search for related code** — Use grep/search to find:
   - All callers of functions you'll change
   - All tests that cover the target code
   - Configuration or schema that references the target
4. **Identify constraints** — Note:
   - What must NOT break (public API, backward compat)
   - Security-sensitive surfaces
   - Performance-critical paths

### Exit Criteria
- You can describe the current behavior of the code you'll change
- You know which files will be modified and which won't
- You've identified existing tests and conventions

### Anti-Patterns to Avoid
- Starting to write code before reading the existing implementation
- Guessing at module structure instead of exploring it
- Ignoring existing patterns and inventing incompatible ones";

const PHASE_PLAN: &str = "\
## Phase 2: Plan (Define Scope and Strategy)

Define a precise, minimal plan before implementation:

### Required Actions
1. **State the change** — One sentence: what will be different after this change?
2. **List files to modify** — Explicit list with what changes in each.
3. **Define acceptance criteria** — What must pass for this to be done?
   - Specific test cases (existing or new)
   - Build/lint/type-check requirements
   - Runtime behavior expectations
4. **Identify risk** — What could go wrong?
   - Breaking changes
   - Edge cases
   - Cross-module side effects
5. **Choose the minimal approach** — Apply:
   - KISS: simplest control flow that works
   - YAGNI: no speculative features
   - DRY (rule of three): don't extract until pattern repeats

### Exit Criteria
- Clear scope boundary: one concern, no mixed refactor+feature
- Each planned change maps to a specific acceptance criterion

### Decision Framework
- If extending behavior: implement existing traits/interfaces first
- If fixing a bug: write a failing test first, then fix
- If refactoring: ensure identical behavior via existing tests
- If uncertain about approach: try the simplest thing first";

const PHASE_PREPARE: &str = "\
## Phase 3: Prepare (Environment Setup)

Set up a safe environment for implementation:

### Required Actions
1. **Snapshot current state** — Create a checkpoint you can rollback to:
   - `git stash` or `git commit` the current working state
   - Note the commit hash or stash ID
2. **Install dependencies** — If new packages are needed:
   - Add them explicitly (don't rely on auto-install)
   - Pin versions when stability matters
   - Verify they install cleanly before proceeding
3. **Set up validation** — Identify and prepare:
   - The test command (`cargo test`, `npm test`, `pytest`, etc.)
   - The lint/format command
   - The build command
   - The dev server start command (if web app)
4. **Verify baseline** — Run the test suite BEFORE making changes:
   - If tests already fail, note which ones and why
   - This prevents blaming your changes for pre-existing failures

### Exit Criteria
- Rollback checkpoint exists
- You know the exact commands to validate your changes
- Baseline test results are recorded";

const PHASE_IMPLEMENT: &str = "\
## Phase 4: Implement (Write Code + Run→Observe→Fix Loop)

Write code in small, verifiable increments:

### Core Methodology: Incremental Implementation
1. **Make one logical change at a time** — Don't write 500 lines then test.
   Write a function, test it. Add an endpoint, test it.
2. **Run after every change** — Execute the relevant test/build/run command
   immediately after each modification.
3. **Read error messages completely** — Don't skim. The error message tells
   you exactly what's wrong. Parse it systematically:
   - File and line number
   - Error type (syntax, type, runtime, dependency)
   - The actual vs expected value/type
4. **Fix the root cause, not the symptom** — When you see an error:
   - Trace it back to the source (not just the line that crashed)
   - Understand WHY it happened before writing a fix
   - Don't add try/catch around everything as a band-aid

### Run → Observe → Fix Loop
For each iteration:
```
1. EXECUTE: Run build/test/server
2. OBSERVE: Collect stdout, stderr, exit code, server response
3. CLASSIFY: What type of error? (syntax/type/runtime/dep/network)
4. ANALYSE: What is the root cause? Read the FULL error message.
5. FIX: Apply the minimal targeted patch
6. CHECKPOINT: If working state improved, save a checkpoint
7. REPEAT: Go back to step 1
```

### Intelligent Error Resolution
- **Syntax errors**: Fix the exact line indicated. Check brackets, quotes, semicolons.
- **Type errors**: Check function signatures, imports, variable declarations.
  Don't just cast — fix the type mismatch at its source.
- **Dependency errors**: Install the package. If version conflict, check
  compatibility matrix. If unavailable, use an alternative.
- **Runtime errors**: Add logging before the crash point to understand state.
  Check for null/undefined, out-of-bounds, division by zero.
- **Test failures**: Compare expected vs actual. The test is usually right —
  fix the implementation, not the test (unless the test is wrong).
- **Same error 3+ times**: STOP. The current approach is wrong.
  Step back and try a fundamentally different solution.

### Web App Specific
When building web applications:
1. Start the dev server and capture its output
2. Make HTTP requests or browser-check the preview URL
3. Check both server logs AND client response
4. If server returns 500, read the server-side stack trace
5. If UI is wrong, check the DOM structure / API response

### Anti-Patterns to Avoid
- Writing all the code then testing at the very end
- Ignoring warnings (they often become errors later)
- Adding complexity to work around a misunderstood problem
- Copy-pasting error messages into fixes without understanding them";

const PHASE_VALIDATE: &str = "\
## Phase 5: Validate (Full Verification)

Run the complete validation suite:

### Required Checks (in order)
1. **Format check** — Auto-format or verify formatting
2. **Lint check** — All warnings resolved (not suppressed without reason)
3. **Type check** — Full type verification passes
4. **Unit tests** — All existing tests pass + new tests for new code
5. **Build** — Production build succeeds
6. **Integration/E2E** — If applicable, verify end-to-end behavior

### Regression Detection
- Compare test results against the Phase 3 baseline
- Any NEW failures must be fixed before delivery
- If you cannot fix a regression, document it and report to user

### Quality Checks
- No hardcoded secrets, passwords, or tokens
- No debug print statements left in code
- No commented-out code blocks (delete unused code)
- No TODO comments without clear context
- Error messages are meaningful (not just 'error occurred')

### Exit Criteria
- All validation commands pass
- No regressions from baseline
- Code is clean and production-ready";

const PHASE_DELIVER: &str = "\
## Phase 6: Deliver (Report Results)

Communicate clearly what was done:

### Required Actions
1. **Summarize changes** — What was changed and why
2. **List modified files** — Explicit file list
3. **Report validation results** — Which checks passed
4. **Note any caveats** — Edge cases, known limitations, follow-up items
5. **Commit** — With a clear, descriptive commit message (if user wants it)

### Communication Style
- Lead with the outcome, not the process
- Be specific: 'Added X to Y' not 'Made some changes'
- If something didn't work as planned, say so upfront";

/// The complete coding methodology as a single system prompt.
/// This replaces the short `category_system_prompt` for Coding mode.
pub fn coding_system_prompt() -> String {
    let mut prompt = String::from(
        "## Active Mode: Coding (Expert Sandbox)\n\n\
         You are an expert software engineer. Follow this structured methodology \
         for every coding task. Do NOT skip phases.\n\n",
    );

    // ── Principles ──
    prompt.push_str(
        "### Core Principles\n\n\
         1. **Read before write** — Never modify code you haven't read and understood.\n\
         2. **Incremental verification** — Test after every small change, not at the end.\n\
         3. **Root-cause fixing** — Understand WHY an error occurs before writing a fix.\n\
         4. **Minimal changes** — The best patch is the smallest one that works.\n\
         5. **Fail fast, rollback fast** — If an approach isn't working after 3 attempts, \
            try a different strategy. Don't accumulate broken patches.\n\
         6. **Always have a checkpoint** — Before risky changes, ensure you can revert.\n\
         7. **Read the FULL error** — Error messages contain the answer. Don't skim.\n\
         8. **One concern per change** — Don't mix feature work with refactoring.\n\n",
    );

    // ── Phase instructions ──
    for phase in CodingPhase::ALL {
        prompt.push_str(phase_instructions(*phase));
        prompt.push_str("\n\n---\n\n");
    }

    // ── Meta-cognitive rules ──
    prompt.push_str(
        "### Meta-Cognitive Rules (Self-Monitoring)\n\n\
         While coding, continuously check yourself:\n\n\
         - **Am I reading the error message completely?** Don't just see 'error' and guess.\n\
         - **Am I fixing the root cause?** Or am I papering over a symptom?\n\
         - **Have I been stuck on the same error?** If 3+ attempts on the same issue, \
           step back and reconsider the approach.\n\
         - **Am I making the code more complex than necessary?** Three lines of clear code \
           beat a premature abstraction.\n\
         - **Did I test after my last change?** Never batch multiple untested changes.\n\
         - **Would I be able to revert this?** If not, create a checkpoint first.\n\
         - **Am I respecting existing patterns?** Don't invent new conventions when the \
           codebase has established ones.\n",
    );

    prompt
}

/// Tracks which coding phase the workflow is in.
#[derive(Debug)]
pub struct CodingWorkflow {
    current_phase: CodingPhase,
    phase_history: Vec<(CodingPhase, PhaseOutcome)>,
    baseline_test_result: Option<BaselineResult>,
    checkpoint_ref: Option<String>,
    files_read: Vec<String>,
    files_modified: Vec<String>,
    validation_commands: Vec<String>,
}

/// Outcome recorded when a phase completes.
///
/// The associated `CodingPhase` is stored externally in the tuple
/// `(CodingPhase, PhaseOutcome)` inside `CodingWorkflow::phase_history`,
/// avoiding redundant duplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseOutcome {
    pub status: PhaseStatus,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Completed,
    Skipped,
    Failed,
}

/// Baseline test result captured in Phase 3 (Prepare).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineResult {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub pre_existing_failures: Vec<String>,
}

impl CodingWorkflow {
    pub fn new() -> Self {
        Self {
            current_phase: CodingPhase::Comprehend,
            phase_history: Vec::new(),
            baseline_test_result: None,
            checkpoint_ref: None,
            files_read: Vec::new(),
            files_modified: Vec::new(),
            validation_commands: Vec::new(),
        }
    }

    pub fn current_phase(&self) -> CodingPhase {
        self.current_phase
    }

    /// Record a file as having been read (Phase 1).
    pub fn record_file_read(&mut self, path: &str) {
        if !self.files_read.contains(&path.to_string()) {
            self.files_read.push(path.to_string());
        }
    }

    /// Record a file as having been modified (Phase 4).
    pub fn record_file_modified(&mut self, path: &str) {
        if !self.files_modified.contains(&path.to_string()) {
            self.files_modified.push(path.to_string());
        }
    }

    /// Record the baseline test result (Phase 3).
    pub fn record_baseline(&mut self, result: BaselineResult) {
        self.baseline_test_result = Some(result);
    }

    /// Record a rollback checkpoint reference (Phase 3).
    pub fn record_checkpoint(&mut self, reference: &str) {
        self.checkpoint_ref = Some(reference.to_string());
    }

    /// Record which validation commands to use (Phase 3).
    pub fn record_validation_commands(&mut self, commands: Vec<String>) {
        self.validation_commands = commands;
    }

    /// Check if a file was read before being modified (Phase 1 rule).
    pub fn was_read_before_modify(&self, path: &str) -> bool {
        self.files_read.iter().any(|f| f == path)
    }

    /// Complete the current phase and advance to the next.
    pub fn advance(&mut self, notes: &str) -> Option<CodingPhase> {
        let outcome = PhaseOutcome {
            status: PhaseStatus::Completed,
            notes: notes.to_string(),
        };
        self.phase_history.push((self.current_phase, outcome));

        if let Some(next) = self.current_phase.next() {
            self.current_phase = next;
            Some(next)
        } else {
            None
        }
    }

    /// Skip the current phase (e.g., trivial task doesn't need full Prepare).
    pub fn skip(&mut self, reason: &str) -> Option<CodingPhase> {
        let outcome = PhaseOutcome {
            status: PhaseStatus::Skipped,
            notes: reason.to_string(),
        };
        self.phase_history.push((self.current_phase, outcome));

        if let Some(next) = self.current_phase.next() {
            self.current_phase = next;
            Some(next)
        } else {
            None
        }
    }

    /// Go back to a previous phase (e.g., Validate fails → back to Implement).
    pub fn rewind_to(&mut self, phase: CodingPhase, reason: &str) {
        let outcome = PhaseOutcome {
            status: PhaseStatus::Failed,
            notes: reason.to_string(),
        };
        self.phase_history.push((self.current_phase, outcome));
        self.current_phase = phase;
    }

    /// Build a status summary for progress reporting.
    pub fn status_summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push("### Coding Workflow Status".to_string());
        for phase in CodingPhase::ALL {
            let marker = if *phase == self.current_phase {
                "▶"
            } else if self.phase_history.iter().any(|(p, o)| {
                *p == *phase && o.status == PhaseStatus::Completed
            }) {
                "✓"
            } else if self.phase_history.iter().any(|(p, o)| {
                *p == *phase && o.status == PhaseStatus::Skipped
            }) {
                "⊘"
            } else {
                "○"
            };
            lines.push(format!("{marker} {}", phase.label()));
        }
        if !self.files_read.is_empty() {
            lines.push(format!("\nFiles read: {}", self.files_read.len()));
        }
        if !self.files_modified.is_empty() {
            lines.push(format!("Files modified: {}", self.files_modified.len()));
        }
        if let Some(ref baseline) = self.baseline_test_result {
            lines.push(format!(
                "Baseline: {}/{} tests passed",
                baseline.passed, baseline.total_tests
            ));
        }
        lines.join("\n")
    }

    pub fn files_read(&self) -> &[String] {
        &self.files_read
    }

    pub fn files_modified(&self) -> &[String] {
        &self.files_modified
    }

    pub fn baseline(&self) -> Option<&BaselineResult> {
        self.baseline_test_result.as_ref()
    }

    pub fn checkpoint_ref(&self) -> Option<&str> {
        self.checkpoint_ref.as_deref()
    }

    pub fn validation_commands(&self) -> &[String] {
        &self.validation_commands
    }

    pub fn phase_history(&self) -> &[(CodingPhase, PhaseOutcome)] {
        &self.phase_history
    }
}

// ── Error classification ───────────────────────────────────────────

/// Coarse error category used to select fix strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    /// Syntax / parse error — fix: patch the offending line.
    Syntax,
    /// Type or compile error — fix: adjust types, imports, signatures.
    Type,
    /// Runtime panic / exception — fix: add guards, fix logic.
    Runtime,
    /// Missing dependency or import — fix: install package or add import.
    Dependency,
    /// Network / connection error — fix: retry, check config, or skip.
    Network,
    /// Process timed out — fix: optimize or increase timeout.
    Timeout,
    /// Test assertion failure — fix: adjust logic to match spec.
    TestFailure,
    /// Lint / format warning — fix: auto-format.
    Lint,
    /// Unknown / unclassified.
    Unknown,
}

impl ErrorClass {
    /// Severity score (higher = more critical, fix first).
    pub fn severity(&self) -> u8 {
        match self {
            ErrorClass::Syntax => 90,
            ErrorClass::Type => 85,
            ErrorClass::Dependency => 80,
            ErrorClass::Runtime => 75,
            ErrorClass::TestFailure => 70,
            ErrorClass::Timeout => 60,
            ErrorClass::Network => 50,
            ErrorClass::Lint => 20,
            ErrorClass::Unknown => 40,
        }
    }

    /// Classify from raw stderr / log output using pattern matching.
    pub fn classify(output: &str) -> Self {
        let lower = output.to_lowercase();

        // Order matters: more specific patterns first.
        if lower.contains("syntaxerror")
            || lower.contains("syntax error")
            || lower.contains("unexpected token")
            || lower.contains("parsing error")
            || lower.contains("parse error")
        {
            return ErrorClass::Syntax;
        }

        if lower.contains("typeerror")
            || lower.contains("type error")
            || lower.contains("cannot find type")
            || lower.contains("expected type")
            || lower.contains("mismatched types")
            || lower.contains("type mismatch")
            || lower.contains("ts(")
        {
            return ErrorClass::Type;
        }

        if lower.contains("module not found")
            || lower.contains("cannot find module")
            || lower.contains("no such module")
            || lower.contains("modulenotfounderror")
            || lower.contains("importerror")
            || lower.contains("unresolved import")
            || lower.contains("package not found")
            || lower.contains("could not resolve")
        {
            return ErrorClass::Dependency;
        }

        if lower.contains("timeout")
            || lower.contains("timed out")
            || lower.contains("deadline exceeded")
        {
            return ErrorClass::Timeout;
        }

        if lower.contains("econnrefused")
            || lower.contains("enotfound")
            || lower.contains("network error")
            || lower.contains("connection refused")
            || lower.contains("dns resolution")
        {
            return ErrorClass::Network;
        }

        if lower.contains("assertion")
            || lower.contains("expect(")
            || lower.contains("assert_eq")
            || lower.contains("assert!")
            || lower.contains("test failed")
        {
            return ErrorClass::TestFailure;
        }

        if lower.contains("warning")
            || lower.contains("lint")
            || lower.contains("clippy")
            || lower.contains("eslint")
            || lower.contains("prettier")
        {
            return ErrorClass::Lint;
        }

        if lower.contains("error")
            || lower.contains("exception")
            || lower.contains("panic")
            || lower.contains("traceback")
            || lower.contains("failed")
        {
            return ErrorClass::Runtime;
        }

        ErrorClass::Unknown
    }
}

// ── Observation snapshot ───────────────────────────────────────────

/// A single observation captured after running code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Shell exit code (0 = success).
    pub exit_code: i32,
    /// Combined stdout.
    pub stdout: String,
    /// Combined stderr.
    pub stderr: String,
    /// Classified error (if exit_code != 0 or stderr is non-empty).
    pub error_class: Option<ErrorClass>,
    /// Severity score (0–100); 0 if no error.
    pub severity: u8,
    /// Optional server health-check result (HTTP status or error).
    pub server_health: Option<ServerHealth>,
    /// Optional DOM snapshot (simplified accessibility tree).
    pub dom_snapshot: Option<String>,
    /// Wall-clock duration of the execution.
    pub duration: Duration,
}

/// Result of probing the dev server after starting it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub url: String,
    pub status_code: Option<u16>,
    pub error: Option<String>,
    pub response_time_ms: u64,
}

// ── Sandbox iteration record ───────────────────────────────────────

/// One cycle of the run→observe→fix loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxIteration {
    pub iteration: usize,
    pub action: String,
    pub observation: Observation,
    pub fix_applied: Option<String>,
    pub strategy_note: Option<String>,
}

// ── Sandbox loop configuration ─────────────────────────────────────

/// Limits and tunables for the sandbox loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum number of run→observe→fix iterations.
    pub max_iterations: usize,
    /// Maximum wall-clock time for the entire sandbox session.
    pub max_duration: Duration,
    /// How many times the same ErrorClass can recur before strategy switch.
    pub max_same_error_retries: usize,
    /// Whether to create git checkpoints for rollback.
    pub enable_checkpoints: bool,
    /// Dev-server probe URL (if applicable).
    pub preview_url: Option<String>,
    /// Working directory for sandbox execution.
    pub work_dir: String,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_iterations: 25,
            max_duration: Duration::from_secs(600), // 10 min
            max_same_error_retries: 3,
            enable_checkpoints: true,
            preview_url: None,
            work_dir: ".".to_string(),
        }
    }
}

impl From<&crate::config::schema::SandboxSettings> for SandboxConfig {
    fn from(settings: &crate::config::schema::SandboxSettings) -> Self {
        Self {
            max_iterations: settings.max_iterations,
            max_duration: Duration::from_secs(settings.max_duration_secs),
            max_same_error_retries: settings.max_same_error_retries,
            enable_checkpoints: settings.enable_checkpoints,
            preview_url: None,
            work_dir: ".".to_string(),
        }
    }
}

// ── Sandbox loop state machine ─────────────────────────────────────

/// Outcome of a single sandbox step, telling the agent what to do next.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxAction {
    /// Code executed successfully — no errors detected.
    Success { summary: String },
    /// Error detected — LLM should generate a fix.
    NeedsFix {
        error_class: ErrorClass,
        severity: u8,
        context: String,
        suggestion: String,
    },
    /// Same error recurring too many times — try alternative approach.
    SwitchStrategy {
        stuck_error: ErrorClass,
        attempts: usize,
        recommendation: String,
    },
    /// Rollback to last checkpoint — bad patch made things worse.
    Rollback { reason: String },
    /// Budget exhausted — hand back to user.
    BudgetExhausted {
        iterations_used: usize,
        elapsed: Duration,
        last_error: Option<String>,
    },
}

/// Tracks recurring error patterns to detect "stuck" loops.
#[derive(Debug, Default)]
pub struct ErrorTracker {
    counts: HashMap<ErrorClass, usize>,
    history: Vec<ErrorClass>,
}

impl ErrorTracker {
    pub fn record(&mut self, class: ErrorClass) {
        *self.counts.entry(class).or_insert(0) += 1;
        self.history.push(class);
    }

    pub fn count(&self, class: &ErrorClass) -> usize {
        self.counts.get(class).copied().unwrap_or(0)
    }

    /// Returns the error class that is stuck (exceeds max retries), if any.
    pub fn stuck_error(&self, max_retries: usize) -> Option<(ErrorClass, usize)> {
        self.counts
            .iter()
            .find(|(_, &count)| count >= max_retries)
            .map(|(&class, &count)| (class, count))
    }

    /// Check if severity has worsened compared to previous observation.
    pub fn severity_worsened(&self, prev_severity: u8, curr_severity: u8) -> bool {
        curr_severity > prev_severity && self.history.len() >= 2
    }

    pub fn total_errors(&self) -> usize {
        self.counts.values().sum()
    }

    pub fn reset(&mut self) {
        self.counts.clear();
        self.history.clear();
    }
}

/// Drives the sandbox loop state machine.
///
/// The actual tool execution and LLM calls happen outside this struct —
/// this is a pure state machine that determines the *next action* given
/// the current observation.
#[derive(Debug)]
pub struct SandboxLoop {
    config: SandboxConfig,
    tracker: ErrorTracker,
    iterations: Vec<SandboxIteration>,
    started_at: Instant,
    last_severity: u8,
    checkpoint_available: bool,
}

impl SandboxLoop {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            tracker: ErrorTracker::default(),
            iterations: Vec::new(),
            started_at: Instant::now(),
            last_severity: 0,
            checkpoint_available: false,
        }
    }

    /// Current iteration count.
    pub fn iteration_count(&self) -> usize {
        self.iterations.len()
    }

    /// Mark that a checkpoint has been taken.
    pub fn mark_checkpoint(&mut self) {
        self.checkpoint_available = true;
    }

    /// Mark checkpoint consumed (after rollback).
    pub fn consume_checkpoint(&mut self) {
        self.checkpoint_available = false;
        self.tracker.reset();
    }

    /// Process an observation and decide the next action.
    pub fn step(&mut self, observation: Observation) -> SandboxAction {
        let elapsed = self.started_at.elapsed();
        let iteration_num = self.iterations.len() + 1;

        // ── Budget check ──
        if iteration_num > self.config.max_iterations || elapsed > self.config.max_duration {
            return SandboxAction::BudgetExhausted {
                iterations_used: self.iterations.len(),
                elapsed,
                last_error: if observation.stderr.is_empty() {
                    None
                } else {
                    Some(truncate(&observation.stderr, 500))
                },
            };
        }

        // ── Success? ──
        if observation.exit_code == 0 && observation.error_class.is_none() {
            let health_ok = observation
                .server_health
                .as_ref()
                .map_or(true, |h| h.status_code.map_or(false, |c| c < 400));

            if health_ok {
                self.iterations.push(SandboxIteration {
                    iteration: iteration_num,
                    action: "run".to_string(),
                    observation,
                    fix_applied: None,
                    strategy_note: Some("Success — all checks passed.".to_string()),
                });
                return SandboxAction::Success {
                    summary: format!(
                        "Completed in {} iteration(s), {:.1}s",
                        iteration_num,
                        elapsed.as_secs_f64()
                    ),
                };
            }
        }

        // ── Error processing ──
        let error_class = observation
            .error_class
            .unwrap_or_else(|| ErrorClass::classify(&observation.stderr));
        let severity = observation.severity.max(error_class.severity());

        self.tracker.record(error_class);

        // ── Severity worsened after a patch? Rollback. ──
        if self.checkpoint_available
            && self.tracker.severity_worsened(self.last_severity, severity)
            && !self.iterations.is_empty()
        {
            self.iterations.push(SandboxIteration {
                iteration: iteration_num,
                action: "observe".to_string(),
                observation,
                fix_applied: None,
                strategy_note: Some("Severity worsened — rolling back.".to_string()),
            });
            return SandboxAction::Rollback {
                reason: format!(
                    "Severity increased from {} to {} after last patch. \
                     Rolling back to checkpoint.",
                    self.last_severity, severity
                ),
            };
        }

        self.last_severity = severity;

        // ── Stuck on same error? Switch strategy. ──
        if let Some((stuck_class, attempts)) =
            self.tracker.stuck_error(self.config.max_same_error_retries)
        {
            let recommendation = match stuck_class {
                ErrorClass::Dependency => {
                    "Try installing the package with a different version, \
                     or use an alternative library."
                        .to_string()
                }
                ErrorClass::Syntax | ErrorClass::Type => {
                    "Rewrite the problematic section from scratch \
                     using a different approach."
                        .to_string()
                }
                ErrorClass::Runtime => {
                    "Add comprehensive error handling and logging, \
                     then re-examine the control flow."
                        .to_string()
                }
                ErrorClass::Network => {
                    "Check network configuration, add retry logic, \
                     or mock the external service."
                        .to_string()
                }
                ErrorClass::Timeout => {
                    "Optimise the slow path or increase the timeout limit.".to_string()
                }
                _ => "Try a fundamentally different implementation approach.".to_string(),
            };

            self.iterations.push(SandboxIteration {
                iteration: iteration_num,
                action: "observe".to_string(),
                observation,
                fix_applied: None,
                strategy_note: Some(format!("Stuck on {stuck_class:?} x{attempts}")),
            });

            // Reset tracker so the new strategy gets a fresh budget.
            self.tracker.reset();

            return SandboxAction::SwitchStrategy {
                stuck_error: stuck_class,
                attempts,
                recommendation,
            };
        }

        // ── Normal fix needed ──
        let context = build_error_context(&observation, error_class);
        let suggestion = suggest_fix(error_class);

        self.iterations.push(SandboxIteration {
            iteration: iteration_num,
            action: "observe".to_string(),
            observation,
            fix_applied: None,
            strategy_note: None,
        });

        SandboxAction::NeedsFix {
            error_class,
            severity,
            context,
            suggestion,
        }
    }

    /// Record that a fix was applied in the current iteration.
    pub fn record_fix(&mut self, fix_description: &str) {
        if let Some(last) = self.iterations.last_mut() {
            last.fix_applied = Some(fix_description.to_string());
        }
    }

    /// Full iteration history for reporting.
    pub fn history(&self) -> &[SandboxIteration] {
        &self.iterations
    }

    /// Generate a summary prompt for the LLM, including relevant history.
    pub fn build_fix_prompt(
        &self,
        action: &SandboxAction,
        code_context: &str,
    ) -> String {
        match action {
            SandboxAction::NeedsFix {
                error_class,
                severity,
                context,
                suggestion,
            } => {
                format!(
                    "## Sandbox Fix Request (iteration {})\n\n\
                     **Error class**: {error_class:?} (severity {severity}/100)\n\n\
                     **Error context**:\n```\n{context}\n```\n\n\
                     **Suggestion**: {suggestion}\n\n\
                     **Current code**:\n```\n{code_context}\n```\n\n\
                     Generate a minimal, targeted fix. Only change what is necessary.",
                    self.iteration_count()
                )
            }
            SandboxAction::SwitchStrategy {
                stuck_error,
                attempts,
                recommendation,
            } => {
                format!(
                    "## Strategy Switch Required (iteration {})\n\n\
                     The error `{stuck_error:?}` has persisted for {attempts} attempts.\n\
                     The previous approach is not working.\n\n\
                     **Recommendation**: {recommendation}\n\n\
                     **Current code**:\n```\n{code_context}\n```\n\n\
                     Rewrite the problematic section using a different approach.",
                    self.iteration_count()
                )
            }
            SandboxAction::Rollback { reason } => {
                format!(
                    "## Rollback (iteration {})\n\n\
                     {reason}\n\n\
                     The last patch made things worse. After rollback, \
                     try a more conservative fix.",
                    self.iteration_count()
                )
            }
            _ => String::new(),
        }
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn build_error_context(obs: &Observation, class: ErrorClass) -> String {
    let mut parts = Vec::new();

    if obs.exit_code != 0 {
        parts.push(format!("Exit code: {}", obs.exit_code));
    }
    if !obs.stderr.is_empty() {
        parts.push(format!("Stderr (truncated):\n{}", truncate(&obs.stderr, 1500)));
    }
    if !obs.stdout.is_empty() {
        // Include last 20 lines of stdout for context.
        let lines: Vec<&str> = obs.stdout.lines().collect();
        let start = lines.len().saturating_sub(20);
        let tail = lines[start..].join("\n");
        parts.push(format!("Stdout (last 20 lines):\n{tail}"));
    }
    if let Some(ref health) = obs.server_health {
        if let Some(err) = &health.error {
            parts.push(format!("Server health error: {err}"));
        } else if let Some(code) = health.status_code {
            parts.push(format!(
                "Server responded: {} ({}ms)",
                code, health.response_time_ms
            ));
        }
    }
    if let Some(ref dom) = obs.dom_snapshot {
        parts.push(format!("DOM snapshot:\n{}", truncate(dom, 1000)));
    }

    parts.push(format!("Error class: {class:?}"));
    parts.join("\n\n")
}

fn suggest_fix(class: ErrorClass) -> String {
    match class {
        ErrorClass::Syntax => {
            "Fix the syntax error on the indicated line. Check for missing \
             brackets, semicolons, or quotes."
                .to_string()
        }
        ErrorClass::Type => {
            "Resolve the type mismatch. Check function signatures, \
             variable declarations, and import paths."
                .to_string()
        }
        ErrorClass::Dependency => {
            "Install the missing dependency or fix the import path. \
             Run the appropriate package manager command."
                .to_string()
        }
        ErrorClass::Runtime => {
            "Add proper error handling or fix the logic that causes \
             the runtime error. Check null/undefined access and bounds."
                .to_string()
        }
        ErrorClass::Network => {
            "Verify network configuration. The target host may be \
             unreachable — add retry logic or check the URL."
                .to_string()
        }
        ErrorClass::Timeout => {
            "The operation took too long. Optimize the slow path, \
             add caching, or increase the timeout."
                .to_string()
        }
        ErrorClass::TestFailure => {
            "A test assertion failed. Compare expected vs actual \
             output and adjust the implementation logic."
                .to_string()
        }
        ErrorClass::Lint => "Run the auto-formatter and fix any remaining warnings.".to_string(),
        ErrorClass::Unknown => {
            "Analyse the error output carefully and apply the most \
             appropriate fix."
                .to_string()
        }
    }
}

fn truncate(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        // Find the last char boundary at or before `max_bytes` to avoid
        // panicking on multi-byte UTF-8 sequences.
        let end = s
            .char_indices()
            .take_while(|(i, _)| *i < max_bytes)
            .last()
            .map_or(0, |(i, c)| i + c.len_utf8());
        format!("{}...[truncated]", &s[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_success_obs() -> Observation {
        Observation {
            exit_code: 0,
            stdout: "All tests passed".to_string(),
            stderr: String::new(),
            error_class: None,
            severity: 0,
            server_health: None,
            dom_snapshot: None,
            duration: Duration::from_millis(500),
        }
    }

    fn make_error_obs(stderr: &str, exit_code: i32) -> Observation {
        let class = ErrorClass::classify(stderr);
        Observation {
            exit_code,
            stdout: String::new(),
            stderr: stderr.to_string(),
            error_class: Some(class),
            severity: class.severity(),
            server_health: None,
            dom_snapshot: None,
            duration: Duration::from_millis(200),
        }
    }

    #[test]
    fn classify_syntax_error() {
        assert_eq!(
            ErrorClass::classify("SyntaxError: Unexpected token '}'"),
            ErrorClass::Syntax
        );
    }

    #[test]
    fn classify_type_error() {
        assert_eq!(
            ErrorClass::classify("TypeError: undefined is not a function"),
            ErrorClass::Type
        );
    }

    #[test]
    fn classify_dependency_error() {
        assert_eq!(
            ErrorClass::classify("Module not found: Cannot resolve 'lodash'"),
            ErrorClass::Dependency
        );
    }

    #[test]
    fn classify_network_error() {
        assert_eq!(
            ErrorClass::classify("Error: connect ECONNREFUSED 127.0.0.1:3000"),
            ErrorClass::Network
        );
    }

    #[test]
    fn classify_timeout_error() {
        assert_eq!(
            ErrorClass::classify("Error: operation timed out after 30s"),
            ErrorClass::Timeout
        );
    }

    #[test]
    fn classify_test_failure() {
        assert_eq!(
            ErrorClass::classify("FAILED: assertion failed: expected 42, got 0"),
            ErrorClass::TestFailure
        );
    }

    #[test]
    fn classify_lint_warning() {
        assert_eq!(
            ErrorClass::classify("warning: unused variable `x` (clippy)"),
            ErrorClass::Lint
        );
    }

    #[test]
    fn classify_runtime_error() {
        assert_eq!(
            ErrorClass::classify("thread 'main' panicked at 'index out of bounds'"),
            ErrorClass::Runtime
        );
    }

    #[test]
    fn classify_unknown() {
        assert_eq!(ErrorClass::classify("something happened"), ErrorClass::Unknown);
    }

    #[test]
    fn sandbox_loop_success_on_first_try() {
        let mut sandbox = SandboxLoop::new(SandboxConfig::default());
        let obs = make_success_obs();
        let action = sandbox.step(obs);
        assert!(matches!(action, SandboxAction::Success { .. }));
        assert_eq!(sandbox.iteration_count(), 1);
    }

    #[test]
    fn sandbox_loop_needs_fix_on_error() {
        let mut sandbox = SandboxLoop::new(SandboxConfig::default());
        let obs = make_error_obs("SyntaxError: missing )", 1);
        let action = sandbox.step(obs);
        assert!(matches!(action, SandboxAction::NeedsFix { .. }));
        if let SandboxAction::NeedsFix { error_class, .. } = action {
            assert_eq!(error_class, ErrorClass::Syntax);
        }
    }

    #[test]
    fn sandbox_loop_switches_strategy_after_repeated_errors() {
        let mut sandbox = SandboxLoop::new(SandboxConfig {
            max_same_error_retries: 3,
            ..SandboxConfig::default()
        });

        // Feed the same error 3 times.
        for _ in 0..2 {
            let obs = make_error_obs("SyntaxError: unexpected end", 1);
            let action = sandbox.step(obs);
            assert!(matches!(action, SandboxAction::NeedsFix { .. }));
        }
        // Third time triggers strategy switch.
        let obs = make_error_obs("SyntaxError: unexpected end", 1);
        let action = sandbox.step(obs);
        assert!(matches!(action, SandboxAction::SwitchStrategy { .. }));
    }

    #[test]
    fn sandbox_loop_budget_exhaustion() {
        let mut sandbox = SandboxLoop::new(SandboxConfig {
            max_iterations: 2,
            ..SandboxConfig::default()
        });

        sandbox.step(make_error_obs("error", 1));
        sandbox.step(make_error_obs("error", 1));
        let action = sandbox.step(make_error_obs("error", 1));
        assert!(matches!(action, SandboxAction::BudgetExhausted { .. }));
    }

    #[test]
    fn error_tracker_counts_correctly() {
        let mut tracker = ErrorTracker::default();
        tracker.record(ErrorClass::Syntax);
        tracker.record(ErrorClass::Syntax);
        tracker.record(ErrorClass::Runtime);
        assert_eq!(tracker.count(&ErrorClass::Syntax), 2);
        assert_eq!(tracker.count(&ErrorClass::Runtime), 1);
        assert_eq!(tracker.total_errors(), 3);
    }

    #[test]
    fn error_tracker_stuck_detection() {
        let mut tracker = ErrorTracker::default();
        tracker.record(ErrorClass::Dependency);
        tracker.record(ErrorClass::Dependency);
        tracker.record(ErrorClass::Dependency);
        let stuck = tracker.stuck_error(3);
        assert!(stuck.is_some());
        assert_eq!(stuck.unwrap().0, ErrorClass::Dependency);
    }

    #[test]
    fn severity_ordering() {
        assert!(ErrorClass::Syntax.severity() > ErrorClass::Lint.severity());
        assert!(ErrorClass::Type.severity() > ErrorClass::Network.severity());
        assert!(ErrorClass::Runtime.severity() > ErrorClass::Unknown.severity());
    }

    #[test]
    fn build_fix_prompt_non_empty() {
        let sandbox = SandboxLoop::new(SandboxConfig::default());
        let action = SandboxAction::NeedsFix {
            error_class: ErrorClass::Syntax,
            severity: 90,
            context: "line 42: unexpected '}'".to_string(),
            suggestion: "Fix the bracket.".to_string(),
        };
        let prompt = sandbox.build_fix_prompt(&action, "fn main() { }");
        assert!(prompt.contains("Syntax"));
        assert!(prompt.contains("line 42"));
    }

    #[test]
    fn record_fix_updates_last_iteration() {
        let mut sandbox = SandboxLoop::new(SandboxConfig::default());
        sandbox.step(make_error_obs("SyntaxError", 1));
        sandbox.record_fix("Fixed missing bracket on line 10");
        assert!(sandbox.history().last().unwrap().fix_applied.is_some());
    }

    #[test]
    fn truncate_helper() {
        assert_eq!(truncate("hello", 10), "hello");
        assert!(truncate("hello world this is long", 10).contains("...[truncated]"));
    }

    #[test]
    fn truncate_multibyte_utf8_does_not_panic() {
        // Korean characters are 3 bytes each in UTF-8.
        // 안=0..3, 녕=3..6, 하=6..9, 세=9..12, 요=12..15, space=15, 세=16..19, 계=19..22
        let korean = "안녕하세요 세계";

        // Truncate at byte 2 — falls inside the 1st char (bytes 0..3).
        // Only chars whose start < 2: 안(0). End = 0 + 3 = 3 bytes kept.
        let result = truncate(korean, 2);
        assert!(result.contains("...[truncated]"));
        assert!(result.starts_with("안"));
        assert!(!result.starts_with("안녕"));

        // Truncate at byte 5 — 녕 starts at 3 (< 5), so it is included.
        // End = 3 + 3 = 6 bytes kept → "안녕".
        let result = truncate(korean, 5);
        assert!(result.starts_with("안녕"));
        assert!(!result.starts_with("안녕하")); // 하 starts at 6 (>= 5)
    }

    #[test]
    fn rollback_on_severity_increase() {
        let mut sandbox = SandboxLoop::new(SandboxConfig::default());
        sandbox.mark_checkpoint();

        // First error: low severity (lint).
        let obs1 = make_error_obs("warning: unused variable (lint)", 0);
        sandbox.step(obs1);

        // Second error: we need to create a worsening scenario.
        // After the first step, last_severity gets set.
        // Now create a much higher severity observation.
        let obs2 = Observation {
            exit_code: 1,
            stdout: String::new(),
            stderr: "SyntaxError: completely broken".to_string(),
            error_class: Some(ErrorClass::Syntax),
            severity: 95,
            server_health: None,
            dom_snapshot: None,
            duration: Duration::from_millis(100),
        };
        let action = sandbox.step(obs2);
        assert!(matches!(action, SandboxAction::Rollback { .. }));
    }

    #[test]
    fn sandbox_config_defaults_are_sane() {
        let cfg = SandboxConfig::default();
        assert_eq!(cfg.max_iterations, 25);
        assert_eq!(cfg.max_duration, Duration::from_secs(600));
        assert_eq!(cfg.max_same_error_retries, 3);
        assert!(cfg.enable_checkpoints);
    }

    // ── CodingWorkflow tests ──────────────────────────────────────

    #[test]
    fn coding_workflow_starts_at_comprehend() {
        let wf = CodingWorkflow::new();
        assert_eq!(wf.current_phase(), CodingPhase::Comprehend);
    }

    #[test]
    fn coding_workflow_advances_through_all_phases() {
        let mut wf = CodingWorkflow::new();
        let expected = [
            CodingPhase::Plan,
            CodingPhase::Prepare,
            CodingPhase::Implement,
            CodingPhase::Validate,
            CodingPhase::Deliver,
        ];
        for &exp in &expected {
            let next = wf.advance("done");
            assert_eq!(next, Some(exp));
            assert_eq!(wf.current_phase(), exp);
        }
        // After Deliver, no more phases.
        assert_eq!(wf.advance("done"), None);
    }

    #[test]
    fn coding_workflow_skip_phase() {
        let mut wf = CodingWorkflow::new();
        // Skip Comprehend (trivial task).
        let next = wf.skip("trivial one-liner");
        assert_eq!(next, Some(CodingPhase::Plan));
        assert_eq!(wf.current_phase(), CodingPhase::Plan);
    }

    #[test]
    fn coding_workflow_rewind() {
        let mut wf = CodingWorkflow::new();
        wf.advance("read files");
        wf.advance("planned");
        wf.advance("prepared");
        wf.advance("implemented");
        assert_eq!(wf.current_phase(), CodingPhase::Validate);

        // Validation fails, rewind to Implement.
        wf.rewind_to(CodingPhase::Implement, "tests failed");
        assert_eq!(wf.current_phase(), CodingPhase::Implement);
    }

    #[test]
    fn coding_workflow_tracks_files() {
        let mut wf = CodingWorkflow::new();
        wf.record_file_read("src/main.rs");
        wf.record_file_read("src/lib.rs");
        wf.record_file_read("src/main.rs"); // duplicate
        assert_eq!(wf.files_read().len(), 2);

        wf.record_file_modified("src/main.rs");
        assert!(wf.was_read_before_modify("src/main.rs"));
        assert!(!wf.was_read_before_modify("src/unknown.rs"));
    }

    #[test]
    fn coding_workflow_baseline_recording() {
        let mut wf = CodingWorkflow::new();
        wf.record_baseline(BaselineResult {
            total_tests: 100,
            passed: 98,
            failed: 2,
            pre_existing_failures: vec!["test_flaky".into(), "test_slow".into()],
        });
        let baseline = wf.baseline().unwrap();
        assert_eq!(baseline.total_tests, 100);
        assert_eq!(baseline.passed, 98);
        assert_eq!(baseline.pre_existing_failures.len(), 2);
    }

    #[test]
    fn coding_workflow_checkpoint() {
        let mut wf = CodingWorkflow::new();
        assert!(wf.checkpoint_ref().is_none());
        wf.record_checkpoint("abc123");
        assert_eq!(wf.checkpoint_ref(), Some("abc123"));
    }

    #[test]
    fn coding_workflow_validation_commands() {
        let mut wf = CodingWorkflow::new();
        wf.record_validation_commands(vec![
            "cargo test".into(),
            "cargo clippy".into(),
        ]);
        assert_eq!(wf.validation_commands().len(), 2);
    }

    #[test]
    fn coding_workflow_status_summary() {
        let mut wf = CodingWorkflow::new();
        wf.record_file_read("foo.rs");
        wf.advance("done");
        let summary = wf.status_summary();
        assert!(summary.contains("Comprehend"));
        assert!(summary.contains("Plan"));
        assert!(summary.contains("Files read: 1"));
    }

    #[test]
    fn coding_phase_all_has_six_phases() {
        assert_eq!(CodingPhase::ALL.len(), 6);
    }

    #[test]
    fn phase_instructions_non_empty() {
        for phase in CodingPhase::ALL {
            assert!(!phase_instructions(*phase).is_empty());
        }
    }

    #[test]
    fn coding_system_prompt_contains_all_phases() {
        let prompt = coding_system_prompt();
        assert!(prompt.contains("Phase 1: Comprehend"));
        assert!(prompt.contains("Phase 2: Plan"));
        assert!(prompt.contains("Phase 3: Prepare"));
        assert!(prompt.contains("Phase 4: Implement"));
        assert!(prompt.contains("Phase 5: Validate"));
        assert!(prompt.contains("Phase 6: Deliver"));
        assert!(prompt.contains("Meta-Cognitive Rules"));
        assert!(prompt.contains("Root-cause fixing"));
    }

    #[test]
    fn coding_phase_next_chain() {
        let mut phase = CodingPhase::Comprehend;
        let mut count = 0;
        while let Some(next) = phase.next() {
            phase = next;
            count += 1;
        }
        assert_eq!(count, 5); // 5 transitions from Comprehend → Deliver
        assert_eq!(phase, CodingPhase::Deliver);
    }
}
