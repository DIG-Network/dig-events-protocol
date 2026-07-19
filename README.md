# dig-events-protocol

The canonical **blockchain→app event contract** for the DIG Network.

`dig-events-protocol` is the ONE ecosystem definition of the wallet/chain event taxonomy that the DIG
node/wallet engine EMITS and apps (dig-app) SUBSCRIBE to. It owns only the CONTRACT — the wire types
and the two shape traits — so a second implementation matches the first byte-for-byte. The machinery
that produces, persists, and streams events (the event bus, the catch-up delta store, live
subscriptions) stays in the engine ([dig-wallet-backend](https://github.com/DIG-Network/dig-wallet-backend)).

## What's in the contract

- **`WalletEvent`** — the event enum the engine emits (11 variants, serde `tag = "type"` snake_case),
  with `kind()` and `matches()` for subscription filtering.
- **`EventKind`** — the kind discriminant; an `EnumSet<EventKind>` is the subscription FILTER
  (serialized as a snake_case list).
- **`Cursor` + `EmittedEvent`** — the monotonic delivery cursor and the envelope that flows over the
  live stream and is returned by catch-up.
- **`SyncLifecycle` / `SyncStatus`** — the tri-state sync snapshot.
- **`WalletId` / `Amount` / `AssetId`** — the payload newtypes.
- **`EventEmitter` + `CatchUp` + `filter_events`** — the emit/backfill shape traits and the shared
  filtering rule.

## Usage

```rust
use dig_events_protocol::{EnumSet, EventKind, WalletEvent, WalletId, Amount};

// Build a subscription filter — only funds movement.
let filter = EventKind::FundsReceived | EventKind::FundsSent;

let event = WalletEvent::FundsReceived {
    wallet_id: WalletId(1),
    asset: None, // native XCH
    amount: Amount(5_000),
    coin_id: "abcd".into(),
    confirmed_height: 100,
};

assert!(event.matches(filter));
```

Implement `EventEmitter` on your event bus and `CatchUp` on your delta store to produce a conformant
event source; consumers depend only on this crate.

## Installation

```toml
[dependencies]
dig-events-protocol = "0.1"
```

## Purity

A pure leaf crate: `serde` + `enumset` + `async-trait` only. No `dig-*` dependency, no runtime.

## The drift-freeze

The wire format is frozen by golden-JSON conformance KATs (`tests/conformance.rs`): every event
variant, the envelope, and the kind list round-trip against byte-stable fixtures. A change that alters
the wire shape breaks a KAT.

## Spec

See [`SPEC.md`](./SPEC.md) for the normative contract.

## License

Apache-2.0 OR MIT.
