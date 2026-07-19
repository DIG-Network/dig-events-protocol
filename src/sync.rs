//! Sync-state contract: where the sync loop sits relative to the chain tip (SPEC §6).

use serde::{Deserialize, Serialize};

/// Where the sync loop is relative to the chain tip (a tri-state, pushed via a `SyncProgress`
/// event).
///
/// Serializes as a snake_case string on the wire (`"idle"` / `"syncing"` / `"synced"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncLifecycle {
    /// Not yet started / no peer.
    Idle,
    /// Actively catching up to the tip.
    Syncing,
    /// Caught up to the tip and tracking live.
    Synced,
}

/// A snapshot of sync state for a wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncStatus {
    /// The tri-state lifecycle.
    pub state: SyncLifecycle,
    /// The height the wallet has processed up to.
    pub peak_height: u32,
    /// The chain tip height the wallet is syncing toward.
    pub target_height: u32,
}
