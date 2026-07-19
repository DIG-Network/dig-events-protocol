//! # dig-events-protocol — the canonical blockchain→app event contract
//!
//! The ONE ecosystem definition of the wallet/chain event taxonomy that the DIG node/wallet engine
//! EMITS and apps (dig-app) SUBSCRIBE to. This crate owns only the CONTRACT — the wire types and the
//! two shape traits — so a second implementation matches the first byte-for-byte. The machinery that
//! produces, persists, and streams events (the event bus, the catch-up delta store, live
//! subscriptions) STAYS in the engine (dig-wallet-backend); moving it here would couple every
//! consumer to the runtime.
//!
//! ## The contract
//!
//! - [`WalletEvent`] — the event enum the engine emits (tagged `type` snake_case on the wire), with
//!   [`WalletEvent::kind`] and [`WalletEvent::matches`] for subscription filtering.
//! - [`EventKind`] — the kind discriminant; an `EnumSet<EventKind>` is the subscription FILTER
//!   (serialized as a snake_case list).
//! - [`Cursor`] + [`EmittedEvent`] — the monotonic delivery cursor and the envelope that flows over
//!   the live stream and is returned by catch-up.
//! - [`SyncLifecycle`] / [`SyncStatus`] — the tri-state sync snapshot.
//! - [`WalletId`] / [`Amount`] / [`AssetId`] — the payload newtypes.
//! - [`EventEmitter`] + [`CatchUp`] + [`filter_events`] — the emit/backfill shape traits and the
//!   shared filtering rule.
//!
//! ## The drift-freeze
//!
//! The wire format is frozen by golden-JSON conformance KATs (`tests/conformance.rs`): every
//! [`WalletEvent`] variant and the [`EventKind`] list round-trip against a byte-stable fixture. A
//! change that alters the wire shape breaks a KAT — that is the guardrail against silent drift.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cursor;
mod event;
mod kind;
mod sync;
mod traits;
mod value;

pub use cursor::{Cursor, EmittedEvent};
pub use event::WalletEvent;
pub use kind::EventKind;
pub use sync::{SyncLifecycle, SyncStatus};
pub use traits::{filter_events, CatchUp, EventEmitter};
pub use value::{Amount, AssetId, WalletId};

// Re-export enumset so consumers can name `EnumSet<EventKind>` (the subscription filter) without
// depending on a matching enumset version themselves.
pub use enumset::{enum_set, EnumSet};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_advances_monotonically() {
        assert_eq!(Cursor(0).next(), Cursor(1));
        assert!(Cursor(1) > Cursor(0));
    }

    #[test]
    fn event_is_tagged_snake_case() {
        let e = WalletEvent::Confirmation {
            tx_id: "ab".into(),
            height: 100,
        };
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("\"type\":\"confirmation\""), "{json}");
        let back: WalletEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e);
    }

    #[test]
    fn kind_maps_each_variant() {
        let e = WalletEvent::FundsReceived {
            wallet_id: WalletId(1),
            asset: None,
            amount: Amount(5),
            coin_id: "c".into(),
            confirmed_height: 10,
        };
        assert_eq!(e.kind(), EventKind::FundsReceived);
    }

    #[test]
    fn filter_admits_only_matching_kinds() {
        let received = WalletEvent::FundsReceived {
            wallet_id: WalletId(1),
            asset: None,
            amount: Amount(5),
            coin_id: "c".into(),
            confirmed_height: 10,
        };
        let tip = WalletEvent::NewTip {
            height: 9,
            header_hash: "hh".into(),
        };

        let funds_only = EventKind::FundsReceived | EventKind::FundsSent;
        assert!(received.matches(funds_only));
        assert!(!tip.matches(funds_only));
    }

    #[test]
    fn filter_events_retains_only_matching_in_order() {
        let events = vec![
            EmittedEvent {
                cursor: Cursor(1),
                event: WalletEvent::NewTip {
                    height: 1,
                    header_hash: "a".into(),
                },
            },
            EmittedEvent {
                cursor: Cursor(2),
                event: WalletEvent::FundsReceived {
                    wallet_id: WalletId(1),
                    asset: None,
                    amount: Amount(5),
                    coin_id: "c".into(),
                    confirmed_height: 10,
                },
            },
        ];
        let kept = filter_events(events, EventKind::FundsReceived.into());
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].cursor, Cursor(2));
    }

    #[test]
    fn amount_exposes_mojos() {
        assert_eq!(Amount(42).mojos(), 42);
    }

    #[test]
    fn newtypes_display() {
        assert_eq!(WalletId(3).to_string(), "3");
        assert_eq!(Amount(9).to_string(), "9");
        assert_eq!(AssetId("tail".into()).to_string(), "tail");
    }

    #[test]
    fn sync_status_round_trips() {
        let s = SyncStatus {
            state: SyncLifecycle::Synced,
            peak_height: 100,
            target_height: 100,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("\"state\":\"synced\""), "{json}");
        let back: SyncStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }
}
