//! The event-kind discriminant used as the subscription filter (SPEC §5).

use enumset::EnumSetType;
use serde::{Deserialize, Serialize};

/// The kind discriminant of a [`WalletEvent`](crate::WalletEvent), used as the subscription FILTER.
///
/// A subscriber passes an `EnumSet<EventKind>`; the engine delivers only matching events (e.g. funds
/// notifications subscribe `FundsReceived | FundsSent`; chain-watch subscribes
/// `CoinStateChanged | NewTip`). An `EnumSet<EventKind>` serializes as a snake_case LIST on the wire
/// (`["funds_received","funds_sent"]`).
#[derive(Debug, Serialize, Deserialize, EnumSetType)]
#[enumset(serialize_repr = "list")]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// Inbound value landed.
    FundsReceived,
    /// Outbound value confirmed.
    FundsSent,
    /// A tracked coin's spent/created state changed.
    CoinStateChanged,
    /// A submitted transaction confirmed.
    Confirmation,
    /// A submitted transaction failed.
    TransactionFailed,
    /// A new chain tip was observed.
    NewTip,
    /// Sync progress advanced.
    SyncProgress,
    /// CAT metadata became available.
    CatInfo,
    /// DID metadata became available.
    DidInfo,
    /// NFT data became available.
    NftData,
    /// A new HD receive address became active.
    Derivation,
}
