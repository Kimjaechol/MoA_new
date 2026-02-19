//! Multi-device synchronization protocol module.
//!
//! Implements the 3-tier hierarchical sync system from the patent:
//!
//! - **Layer 1**: Temporary relay with TTL — real-time sync via server relay
//! - **Layer 2**: Local delta journal + version vectors — offline catch-up
//! - **Layer 3**: Manifest-based full sync — long-offline recovery
//!
//! The core sync engine lives in `memory::sync`. This module adds:
//! - WebSocket broadcast channel protocol
//! - Broadcast message types
//! - Order buffer for sequence guarantees
//! - Manifest comparison for full sync

pub mod protocol;
pub mod relay;

pub use relay::SyncRelay;
