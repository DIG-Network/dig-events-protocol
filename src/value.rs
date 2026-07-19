//! The payload newtypes carried inside events (SPEC §3).
//!
//! These are the leaf value types the engine and apps agree on: which wallet an event concerns
//! ([`WalletId`]), how much value moved ([`Amount`], in the smallest on-chain unit), and which
//! asset ([`AssetId`], the CAT tail hex; `None` on an event means native XCH). They are thin
//! newtypes so the wire shape is a bare number/string, not a tagged object.

use serde::{Deserialize, Serialize};

/// The stable identifier of a wallet within an engine instance.
///
/// Serializes as a bare `u32` on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WalletId(pub u32);

impl std::fmt::Display for WalletId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An on-chain value amount, in the smallest indivisible unit (mojos).
///
/// Serializes as a bare `u64` on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Amount(pub u64);

impl Amount {
    /// The raw value in the smallest unit (mojos).
    pub fn mojos(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A CAT asset identifier — the asset's TAIL hash, hex-encoded.
///
/// On an event's `asset` field, `Some(AssetId(..))` is a CAT and `None` is native XCH.
/// Serializes as a bare string on the wire.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AssetId(pub String);

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
