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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
        let tail: String = obs
            .stdout
            .lines()
            .rev()
            .take(20)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...[truncated]", &s[..max])
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
}
