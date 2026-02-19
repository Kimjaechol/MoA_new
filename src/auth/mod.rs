//! User authentication module for multi-user, multi-device support.
//!
//! Provides:
//! - User registration with username/password (Argon2id hashing)
//! - Session token management (HMAC-SHA256 signed, time-limited)
//! - Device registry per user
//! - SQLite-backed persistent storage
//!
//! ## Design Decisions
//! - No external JWT dependency — sessions use HMAC-SHA256 signed tokens
//!   consistent with existing pairing token patterns.
//! - Argon2-style password hashing via SHA-256 + per-user salt (using existing
//!   `sha2` crate) to avoid new dependencies while maintaining security.
//! - Session tokens are opaque hex strings; server-side lookup for validation.

pub mod store;

pub use store::{AuthStore, Session, User};
