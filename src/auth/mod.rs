//! User authentication module for multi-user, multi-device support.
//!
//! Provides:
//! - User registration with username/password (iterated SHA-256, 100k rounds + per-user salt)
//! - Session token management (opaque hex tokens, SHA-256 hashed for storage, time-limited)
//! - Device registry per user
//! - SQLite-backed persistent storage
//!
//! ## Design Decisions
//! - No external JWT dependency — sessions use opaque random tokens with
//!   server-side SHA-256 hashed lookup, consistent with existing pairing patterns.
//! - Password hashing uses iterated SHA-256 (100k rounds) + per-user salt (using
//!   existing `sha2` crate) to avoid new dependencies while maintaining security.
//! - Session tokens are opaque hex strings; server-side lookup for validation.

pub mod store;

pub use store::{AuthStore, DeviceWithStatus, Session};
