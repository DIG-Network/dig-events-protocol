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
/// Serializes ALWAYS as a decimal string on the wire — every value, small or large — so the
/// JavaScript/TypeScript binding maps it to a single `bigint` via `BigInt(str)`: one code path,
/// no `typeof` branch, and never a silent precision loss past `Number.MAX_SAFE_INTEGER` (SPEC §3).
/// The Rust representation stays `u64` because a Chia mojo amount always fits in 64 bits.
///
/// Deserialization accepts the canonical decimal string and — for leniency towards hand-written
/// or legacy JSON — a bare JSON number, but serialization ALWAYS emits a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(pub u64);

impl Amount {
    /// The raw value in the smallest unit (mojos).
    pub fn mojos(self) -> u64 {
        self.0
    }
}

impl Serialize for Amount {
    /// Always emit the value as a decimal string so a JS consumer reads it as one `bigint`.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Amount {
    /// Accept the canonical decimal string, and — leniently — a bare JSON number.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// A decimal-string amount (canonical) or, for leniency, a bare JSON number.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrNumber {
            Text(String),
            Number(u64),
        }
        match StringOrNumber::deserialize(deserializer)? {
            StringOrNumber::Text(s) => s.parse().map(Amount).map_err(serde::de::Error::custom),
            StringOrNumber::Number(n) => Ok(Amount(n)),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The largest integer a JavaScript `number` (IEEE-754 double) holds exactly: 2^53 − 1.
    /// The always-string wire form means values above it survive the JS boundary via `BigInt`.
    const MAX_JS_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

    #[test]
    fn small_amount_serializes_as_string() {
        assert_eq!(serde_json::to_string(&Amount(5)).unwrap(), "\"5\"");
        assert_eq!(serde_json::to_string(&Amount(0)).unwrap(), "\"0\"");
    }

    #[test]
    fn large_amount_serializes_as_string() {
        let big = Amount(MAX_JS_SAFE_INTEGER + 1);
        assert_eq!(serde_json::to_string(&big).unwrap(), "\"9007199254740992\"");
    }

    /// KAT proving a value past the JS-safe threshold serializes to the SAME decimal string
    /// that dig-wallet-backend's original `Amount` (the extraction source) emits — byte-identical
    /// across the two crates for the large-value case (#1112).
    #[test]
    fn large_amount_matches_extraction_source_byte_for_byte() {
        // dig-wallet-backend's original test asserts Amount(2^53) -> "\"9007199254740992\"".
        let value = MAX_JS_SAFE_INTEGER + 1;
        assert_eq!(
            serde_json::to_string(&Amount(value)).unwrap(),
            "\"9007199254740992\"",
        );
        // And at the far end of the u64 range.
        assert_eq!(
            serde_json::to_string(&Amount(u64::MAX)).unwrap(),
            "\"18446744073709551615\"",
        );
    }

    #[test]
    fn amount_deserializes_from_string() {
        let from_str: Amount = serde_json::from_str("\"9007199254740992\"").unwrap();
        assert_eq!(from_str, Amount(9_007_199_254_740_992));
    }

    #[test]
    fn amount_leniently_deserializes_from_bare_number() {
        let from_num: Amount = serde_json::from_str("42").unwrap();
        assert_eq!(from_num, Amount(42));
    }

    #[test]
    fn amount_round_trips_across_the_threshold() {
        for value in [
            0u64,
            1,
            MAX_JS_SAFE_INTEGER,
            MAX_JS_SAFE_INTEGER + 1,
            u64::MAX,
        ] {
            let json = serde_json::to_string(&Amount(value)).unwrap();
            let back: Amount = serde_json::from_str(&json).unwrap();
            assert_eq!(back, Amount(value), "round-trip failed for {value}");
        }
    }

    #[test]
    fn amount_bad_string_is_an_error() {
        assert!(serde_json::from_str::<Amount>("\"not-a-number\"").is_err());
    }

    #[test]
    fn mojos_accessor_returns_raw() {
        assert_eq!(Amount(555).mojos(), 555);
    }
}
