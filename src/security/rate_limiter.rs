//! Rate limiting with 3-Strike escalation for ZeroClaw.
//!
//! Implements a sliding-window rate limiter with progressive penalties:
//! - Strike 1: 30-minute cooldown + warning
//! - Strike 2: 1-hour cooldown + final warning
//! - Strike 3: Permanent ban (admin reset required)
//!
//! ## Design
//! - In-memory sliding window per user+channel key
//! - Stale entry cleanup every 2 hours
//! - Admin reset/unban functions

use std::collections::HashMap;

/// Default: 30 requests per minute.
const DEFAULT_LIMIT: u32 = 30;

/// Default window: 60 seconds (1 minute).
const DEFAULT_WINDOW_SECS: u64 = 60;

/// Strike 1 cooldown: 30 minutes.
const STRIKE1_COOLDOWN_SECS: u64 = 30 * 60;

/// Strike 2 cooldown: 1 hour.
const STRIKE2_COOLDOWN_SECS: u64 = 60 * 60;

/// Stale entry cleanup threshold: 2 hours.
const STALE_CLEANUP_SECS: u64 = 2 * 60 * 60;

/// Current epoch seconds.
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Strike level for a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrikeLevel {
    /// No strikes.
    None,
    /// First offense: 30-minute cooldown.
    Strike1,
    /// Second offense: 1-hour cooldown.
    Strike2,
    /// Third offense: permanent ban.
    Banned,
}

/// Result of a rate limit check.
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Current strike level.
    pub strike: StrikeLevel,
    /// Remaining requests in the current window (0 if blocked).
    pub remaining: u32,
    /// Seconds until cooldown expires (0 if not in cooldown).
    pub cooldown_remaining_secs: u64,
    /// Human-readable message for the user.
    pub message: Option<String>,
}

/// Per-user rate tracking state.
#[derive(Debug, Clone)]
struct UserState {
    /// Timestamps of recent requests (within window).
    request_timestamps: Vec<u64>,
    /// Current strike level.
    strike: StrikeLevel,
    /// When the current cooldown expires (0 if none).
    cooldown_until: u64,
    /// Last activity timestamp (for stale cleanup).
    last_active: u64,
}

/// Rate limiter with 3-Strike escalation.
pub struct RateLimiter {
    /// Maximum requests per window.
    limit: u32,
    /// Window size in seconds.
    window_secs: u64,
    /// Per-key user states.
    states: HashMap<String, UserState>,
    /// Whether rate limiting is enabled.
    enabled: bool,
}

impl RateLimiter {
    /// Create a new rate limiter with default settings.
    pub fn new(enabled: bool) -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            window_secs: DEFAULT_WINDOW_SECS,
            states: HashMap::new(),
            enabled,
        }
    }

    /// Create a new rate limiter with custom limits.
    pub fn with_limits(enabled: bool, limit: u32, window_secs: u64) -> Self {
        Self {
            limit,
            window_secs,
            states: HashMap::new(),
            enabled,
        }
    }

    /// Check if a request from the given key is allowed.
    /// Key format: "{channel}:{user_id}".
    pub fn check(&mut self, key: &str) -> RateLimitResult {
        if !self.enabled {
            return RateLimitResult {
                allowed: true,
                strike: StrikeLevel::None,
                remaining: self.limit,
                cooldown_remaining_secs: 0,
                message: None,
            };
        }

        let now = now_secs();

        // Initialize state if new user
        let state = self.states.entry(key.to_string()).or_insert(UserState {
            request_timestamps: Vec::new(),
            strike: StrikeLevel::None,
            cooldown_until: 0,
            last_active: now,
        });

        state.last_active = now;

        // Check permanent ban
        if state.strike == StrikeLevel::Banned {
            return RateLimitResult {
                allowed: false,
                strike: StrikeLevel::Banned,
                remaining: 0,
                cooldown_remaining_secs: 0,
                message: Some("Permanently banned due to repeated rate limit violations. Contact admin to unban.".into()),
            };
        }

        // Check active cooldown
        if now < state.cooldown_until {
            let remaining_secs = state.cooldown_until - now;

            // Escalate strike if they retry during cooldown
            let new_strike = match state.strike {
                StrikeLevel::Strike1 => {
                    state.strike = StrikeLevel::Strike2;
                    state.cooldown_until = now + STRIKE2_COOLDOWN_SECS;
                    StrikeLevel::Strike2
                }
                StrikeLevel::Strike2 => {
                    state.strike = StrikeLevel::Banned;
                    state.cooldown_until = 0;
                    StrikeLevel::Banned
                }
                _ => state.strike,
            };

            let message = match new_strike {
                StrikeLevel::Strike2 => Some(
                    "Strike 2: Rate limit cooldown escalated to 1 hour. Final warning.".into(),
                ),
                StrikeLevel::Banned => Some(
                    "Strike 3: Permanently banned. Contact admin to restore access.".into(),
                ),
                _ => Some(format!(
                    "Rate limited. Please wait {} seconds.",
                    remaining_secs
                )),
            };

            return RateLimitResult {
                allowed: false,
                strike: new_strike,
                remaining: 0,
                cooldown_remaining_secs: if new_strike == StrikeLevel::Banned {
                    0
                } else {
                    state.cooldown_until.saturating_sub(now)
                },
                message,
            };
        }

        // Reset cooldown if it has expired
        if state.cooldown_until > 0 && now >= state.cooldown_until {
            state.cooldown_until = 0;
        }

        // Clean old timestamps outside the window
        let window_start = now.saturating_sub(self.window_secs);
        state
            .request_timestamps
            .retain(|&ts| ts >= window_start);

        // Check rate limit
        if u32::try_from(state.request_timestamps.len()).unwrap_or(u32::MAX) >= self.limit {
            // Rate limit exceeded — issue first strike or escalate
            state.strike = match state.strike {
                StrikeLevel::None => {
                    state.cooldown_until = now + STRIKE1_COOLDOWN_SECS;
                    StrikeLevel::Strike1
                }
                StrikeLevel::Strike1 => {
                    state.cooldown_until = now + STRIKE2_COOLDOWN_SECS;
                    StrikeLevel::Strike2
                }
                StrikeLevel::Strike2 => {
                    state.cooldown_until = 0;
                    StrikeLevel::Banned
                }
                StrikeLevel::Banned => StrikeLevel::Banned,
            };

            let message = match state.strike {
                StrikeLevel::Strike1 => Some(format!(
                    "Strike 1: Rate limit exceeded ({} requests/{}s). 30-minute cooldown.",
                    self.limit, self.window_secs,
                )),
                StrikeLevel::Strike2 => Some(
                    "Strike 2: Rate limit exceeded again. 1-hour cooldown. Final warning."
                        .into(),
                ),
                StrikeLevel::Banned => Some(
                    "Strike 3: Permanently banned. Contact admin to restore access."
                        .into(),
                ),
                StrikeLevel::None => None,
            };

            return RateLimitResult {
                allowed: false,
                strike: state.strike,
                remaining: 0,
                cooldown_remaining_secs: state.cooldown_until.saturating_sub(now),
                message,
            };
        }

        // Request allowed — record timestamp
        state.request_timestamps.push(now);
        let remaining = self.limit - u32::try_from(state.request_timestamps.len()).unwrap_or(u32::MAX);

        RateLimitResult {
            allowed: true,
            strike: state.strike,
            remaining,
            cooldown_remaining_secs: 0,
            message: None,
        }
    }

    /// Admin function: reset a user's strikes and cooldown.
    pub fn reset(&mut self, key: &str) {
        self.states.remove(key);
    }

    /// Admin function: unban a permanently banned user.
    pub fn unban(&mut self, key: &str) {
        if let Some(state) = self.states.get_mut(key) {
            state.strike = StrikeLevel::None;
            state.cooldown_until = 0;
            state.request_timestamps.clear();
        }
    }

    /// Clean up stale entries (users inactive for >2 hours).
    pub fn cleanup_stale(&mut self) {
        let cutoff = now_secs().saturating_sub(STALE_CLEANUP_SECS);
        self.states.retain(|_, state| {
            // Keep banned users even if stale
            state.strike == StrikeLevel::Banned || state.last_active >= cutoff
        });
    }

    /// Get the number of tracked users.
    pub fn tracked_users(&self) -> usize {
        self.states.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_within_limit() {
        let mut limiter = RateLimiter::with_limits(true, 5, 60);
        for _ in 0..5 {
            let result = limiter.check("kakao:zeroclaw_user");
            assert!(result.allowed);
        }
    }

    #[test]
    fn blocks_at_limit() {
        let mut limiter = RateLimiter::with_limits(true, 3, 60);
        for _ in 0..3 {
            limiter.check("kakao:zeroclaw_user");
        }
        let result = limiter.check("kakao:zeroclaw_user");
        assert!(!result.allowed);
        assert_eq!(result.strike, StrikeLevel::Strike1);
        assert!(result.cooldown_remaining_secs > 0);
    }

    #[test]
    fn strike_escalation() {
        let mut limiter = RateLimiter::with_limits(true, 1, 60);

        // First request OK
        let r = limiter.check("test:user");
        assert!(r.allowed);

        // Second triggers strike 1
        let r = limiter.check("test:user");
        assert!(!r.allowed);
        assert_eq!(r.strike, StrikeLevel::Strike1);

        // Retry during cooldown → escalate to strike 2
        let r = limiter.check("test:user");
        assert!(!r.allowed);
        assert_eq!(r.strike, StrikeLevel::Strike2);

        // Retry during cooldown → permanent ban
        let r = limiter.check("test:user");
        assert!(!r.allowed);
        assert_eq!(r.strike, StrikeLevel::Banned);

        // Subsequent attempts still banned
        let r = limiter.check("test:user");
        assert!(!r.allowed);
        assert_eq!(r.strike, StrikeLevel::Banned);
    }

    #[test]
    fn admin_reset_clears_strikes() {
        let mut limiter = RateLimiter::with_limits(true, 1, 60);

        limiter.check("test:user"); // OK
        limiter.check("test:user"); // Strike 1

        let r = limiter.check("test:user");
        assert!(!r.allowed);

        // Admin reset
        limiter.reset("test:user");

        // User can make requests again
        let r = limiter.check("test:user");
        assert!(r.allowed);
    }

    #[test]
    fn admin_unban_restores_access() {
        let mut limiter = RateLimiter::with_limits(true, 1, 60);

        // Drive to ban
        limiter.check("test:user"); // OK
        limiter.check("test:user"); // Strike 1
        limiter.check("test:user"); // Strike 2
        limiter.check("test:user"); // Banned

        let r = limiter.check("test:user");
        assert!(!r.allowed);
        assert_eq!(r.strike, StrikeLevel::Banned);

        // Unban
        limiter.unban("test:user");

        let r = limiter.check("test:user");
        assert!(r.allowed);
        assert_eq!(r.strike, StrikeLevel::None);
    }

    #[test]
    fn disabled_limiter_allows_all() {
        let mut limiter = RateLimiter::with_limits(false, 1, 60);
        for _ in 0..100 {
            let r = limiter.check("test:user");
            assert!(r.allowed);
        }
    }

    #[test]
    fn separate_keys_tracked_independently() {
        let mut limiter = RateLimiter::with_limits(true, 2, 60);

        limiter.check("kakao:user_a");
        limiter.check("kakao:user_a");
        let r = limiter.check("kakao:user_a");
        assert!(!r.allowed); // user_a at limit

        let r = limiter.check("kakao:user_b");
        assert!(r.allowed); // user_b still has capacity
    }

    #[test]
    fn cleanup_removes_stale() {
        let mut limiter = RateLimiter::with_limits(true, 10, 60);
        limiter.check("test:stale_user");
        assert_eq!(limiter.tracked_users(), 1);

        // Artificially make the entry stale
        if let Some(state) = limiter.states.get_mut("test:stale_user") {
            state.last_active = 0; // Very old
        }

        limiter.cleanup_stale();
        assert_eq!(limiter.tracked_users(), 0);
    }

    #[test]
    fn banned_users_survive_cleanup() {
        let mut limiter = RateLimiter::with_limits(true, 1, 60);

        // Drive to ban
        limiter.check("test:user");
        limiter.check("test:user"); // Strike 1
        limiter.check("test:user"); // Strike 2
        limiter.check("test:user"); // Banned

        // Make stale
        if let Some(state) = limiter.states.get_mut("test:user") {
            state.last_active = 0;
        }

        limiter.cleanup_stale();
        assert_eq!(limiter.tracked_users(), 1); // Banned user survives
    }

    #[test]
    fn remaining_decreases() {
        let mut limiter = RateLimiter::with_limits(true, 5, 60);

        let r = limiter.check("test:user");
        assert_eq!(r.remaining, 4);

        let r = limiter.check("test:user");
        assert_eq!(r.remaining, 3);
    }
}
