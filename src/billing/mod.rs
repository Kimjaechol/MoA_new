//! Billing and credit tracking module for ZeroClaw.
//!
//! Tracks API usage costs across providers and channels, enforces
//! spending limits, and provides usage reporting.
//!
//! ## Design
//! - SQLite-based local cost ledger
//! - Per-provider, per-model cost tracking with token counts
//! - Configurable spending limits (daily/monthly) with alerts
//! - Usage summary export for billing reconciliation

pub mod payment;
pub mod tracker;

pub use payment::{CreditPackage, PaymentManager, PaymentRecord, PaymentStatus};
pub use tracker::{CostEntry, CostTracker, UsageSummary};
