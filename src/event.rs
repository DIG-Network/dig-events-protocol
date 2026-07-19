//! The wallet/chain event taxonomy — the heart of the contract (SPEC §5).
//!
//! The engine EMITS [`WalletEvent`]s; apps SUBSCRIBE to a FILTERED view of them (by [`EventKind`]).
//! Subscription is live and best-effort; a subscriber that falls behind uses a
//! [`Cursor`](crate::Cursor) to catch up from the engine's persisted delta, then resumes live. This
//! is the "event-driven, poll only on a gap" contract. This module owns ONLY the wire shape — the
//! bus/store/subscription machinery lives in the engine.

use enumset::EnumSet;
use serde::{Deserialize, Serialize};

use crate::kind::EventKind;
use crate::sync::SyncLifecycle;
use crate::value::{Amount, AssetId, WalletId};

/// The event the engine emits and apps consume.
///
/// Tagged by `type` in snake_case on the wire (`{"type":"funds_received",…}`), so a machine consumer
/// branches on a stable discriminant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WalletEvent {
    /// Inbound value landed for a wallet.
    FundsReceived {
        /// The wallet that received value.
        wallet_id: WalletId,
        /// The asset received; `None` = native XCH.
        asset: Option<AssetId>,
        /// The amount received.
        amount: Amount,
        /// The receiving coin id (hex).
        coin_id: String,
        /// The confirmation height.
        confirmed_height: u32,
    },
    /// Outbound value confirmed for a wallet.
    FundsSent {
        /// The wallet that sent value.
        wallet_id: WalletId,
        /// The asset sent; `None` = native XCH.
        asset: Option<AssetId>,
        /// The amount sent.
        amount: Amount,
        /// The transaction id (hex).
        tx_id: String,
        /// The confirmation height.
        confirmed_height: u32,
    },
    /// A tracked coin changed state.
    CoinStateChanged {
        /// The affected coin id (hex).
        coin_id: String,
        /// Whether the coin is now spent.
        spent: bool,
        /// The height it was created at, if known.
        created_height: Option<u32>,
        /// The height it was spent at, if spent.
        spent_height: Option<u32>,
    },
    /// A submitted transaction confirmed on-chain.
    Confirmation {
        /// The transaction id (hex).
        tx_id: String,
        /// The confirmation height.
        height: u32,
    },
    /// A submitted transaction failed (rejected or never confirmed).
    TransactionFailed {
        /// The transaction id (hex).
        tx_id: String,
        /// A human-readable failure reason.
        error: String,
    },
    /// A new chain tip was observed.
    NewTip {
        /// The tip height.
        height: u32,
        /// The tip header hash (hex).
        header_hash: String,
    },
    /// Sync progress advanced for a wallet.
    SyncProgress {
        /// The wallet whose sync advanced.
        wallet_id: WalletId,
        /// The current lifecycle state.
        state: SyncLifecycle,
        /// The processed height.
        peak_height: u32,
        /// The tip height being synced toward.
        target_height: u32,
    },
    /// CAT metadata for an asset became available.
    CatInfo {
        /// The CAT asset id.
        asset_id: AssetId,
        /// The resolved ticker/name.
        name: Option<String>,
    },
    /// DID metadata became available.
    DidInfo {
        /// The DID launcher id (hex).
        launcher_id: String,
    },
    /// NFT data became available.
    NftData {
        /// The NFT launcher id (hex).
        launcher_id: String,
    },
    /// A new HD receive address became active.
    Derivation {
        /// The wallet the address belongs to.
        wallet_id: WalletId,
        /// The newly-active derivation index.
        index: u32,
    },
}

impl WalletEvent {
    /// The [`EventKind`] discriminant used for subscription filtering.
    pub fn kind(&self) -> EventKind {
        match self {
            Self::FundsReceived { .. } => EventKind::FundsReceived,
            Self::FundsSent { .. } => EventKind::FundsSent,
            Self::CoinStateChanged { .. } => EventKind::CoinStateChanged,
            Self::Confirmation { .. } => EventKind::Confirmation,
            Self::TransactionFailed { .. } => EventKind::TransactionFailed,
            Self::NewTip { .. } => EventKind::NewTip,
            Self::SyncProgress { .. } => EventKind::SyncProgress,
            Self::CatInfo { .. } => EventKind::CatInfo,
            Self::DidInfo { .. } => EventKind::DidInfo,
            Self::NftData { .. } => EventKind::NftData,
            Self::Derivation { .. } => EventKind::Derivation,
        }
    }

    /// Whether this event passes a subscription filter (an `EnumSet` of kinds).
    pub fn matches(&self, filter: EnumSet<EventKind>) -> bool {
        filter.contains(self.kind())
    }
}
