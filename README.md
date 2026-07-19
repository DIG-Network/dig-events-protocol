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

## Full Protocol Interface (at-a-glance LLM reference)

### WalletEvent Variants (11 total)

Serialized as JSON with `#[serde(tag = "type", rename_all = "snake_case")]` — type discriminant is the variant name in snake_case.

| Variant | Wire Type | Fields | Summary |
|---|---|---|---|
| `FundsReceived` | `funds_received` | `wallet_id: WalletId`, `asset: Option<AssetId>`, `amount: Amount`, `coin_id: String`, `confirmed_height: u32` | Inbound value landed for wallet |
| `FundsSent` | `funds_sent` | `wallet_id: WalletId`, `asset: Option<AssetId>`, `amount: Amount`, `tx_id: String`, `confirmed_height: u32` | Outbound value confirmed for wallet |
| `CoinStateChanged` | `coin_state_changed` | `coin_id: String`, `spent: bool`, `created_height: Option<u32>`, `spent_height: Option<u32>` | Tracked coin's spent/created state changed |
| `Confirmation` | `confirmation` | `tx_id: String`, `height: u32` | Submitted transaction confirmed on-chain |
| `TransactionFailed` | `transaction_failed` | `tx_id: String`, `error: String` | Submitted transaction failed (rejected/never confirmed) |
| `NewTip` | `new_tip` | `height: u32`, `header_hash: String` | New chain tip observed |
| `SyncProgress` | `sync_progress` | `wallet_id: WalletId`, `state: SyncLifecycle`, `peak_height: u32`, `target_height: u32` | Sync progress advanced for wallet |
| `CatInfo` | `cat_info` | `asset_id: AssetId`, `name: Option<String>` | CAT metadata became available |
| `DidInfo` | `did_info` | `launcher_id: String` | DID metadata became available |
| `NftData` | `nft_data` | `launcher_id: String` | NFT data became available |
| `Derivation` | `derivation` | `wallet_id: WalletId`, `index: u32` | New HD receive address became active |

### EventKind — Subscription Filter (11 kinds)

`EnumSet<EventKind>` is the subscription filter; serializes as a snake_case list on the wire (e.g., `["funds_received","funds_sent"]`):

```rust
pub enum EventKind {
    FundsReceived,      // inbound value
    FundsSent,          // outbound value confirmed
    CoinStateChanged,   // coin state (spent/created) changed
    Confirmation,       // transaction confirmed
    TransactionFailed,  // transaction failed
    NewTip,             // new chain tip
    SyncProgress,       // sync progress advanced
    CatInfo,            // CAT metadata resolved
    DidInfo,            // DID metadata resolved
    NftData,            // NFT metadata resolved
    Derivation,         // new HD derivation active
}
```

**Usage:** `event.kind()` returns the event's `EventKind`; `event.matches(filter)` checks if it passes a filter.

### Delivery & Backfill: Cursor + EmittedEvent

```rust
/// Monotonic per-instance sequence number stamped on delivered events.
pub struct Cursor(pub u64);

/// The envelope that flows over live stream AND catch-up backfill.
pub struct EmittedEvent {
    pub cursor: Cursor,      // monotonic sequence (u64 on wire)
    pub event: WalletEvent,  // the payload
}
```

**Subscription Model:**
1. **Live stream:** Engine emits `EmittedEvent`s in real-time; subscriber remembers the last `Cursor`.
2. **On gap (disconnect/lag):** Subscriber calls `catch_up(since: Cursor, filter: EventKind)` ONCE to backfill missed events.
3. **Resume live:** After backfill, resume the live stream from the last backfill cursor.

### Sync State: SyncLifecycle + SyncStatus

```rust
/// Tri-state sync snapshot (wire: "idle" / "syncing" / "synced").
pub enum SyncLifecycle {
    Idle,      // not started / no peer
    Syncing,   // actively catching up
    Synced,    // caught up and tracking live
}

pub struct SyncStatus {
    pub state: SyncLifecycle,     // current state
    pub peak_height: u32,         // height processed to
    pub target_height: u32,       // chain tip being synced toward
}
```

Delivered via `WalletEvent::SyncProgress`.

### Payload Newtypes

Three thin newtypes so the wire is a bare value, not a tagged object:

```rust
pub struct WalletId(pub u32);     // wallet identifier (wire: bare u32)
pub struct Amount(pub u64);        // on-chain value in mojos (wire: bare u64)
pub struct AssetId(pub String);    // CAT tail hash, hex (wire: bare string)
```

**Asset field semantics:**
- `Some(AssetId(…))` = CAT (colored coin)
- `None` = native XCH

### Shape Traits: EventEmitter + CatchUp

**`EventEmitter`** — what the engine implements; producers call `publish()`:
```rust
pub trait EventEmitter {
    fn publish(&self, event: WalletEvent) -> Cursor;
}
```

**`CatchUp`** — what a subscriber calls once to backfill:
```rust
pub trait CatchUp {
    type Error;
    
    async fn catch_up(
        &self,
        since: Cursor,
        filter: EnumSet<EventKind>,
    ) -> Result<Vec<EmittedEvent>, Self::Error>;
}
```

Returns every `EmittedEvent` with cursor strictly > `since`, optionally narrowed to `filter`, in cursor order.

### Filtering: filter_events

Shared, drift-free rule applied on BOTH live and catch-up:

```rust
pub fn filter_events(
    events: impl IntoIterator<Item = EmittedEvent>,
    filter: EnumSet<EventKind>,
) -> Vec<EmittedEvent>
```

Retains only events whose `kind()` is in the filter, preserving cursor order. Ensures live and catch-up deliver identical filtered views.

### Wire Characteristics

- **Serialization:** JSON via `serde`
- **Event tagging:** `#[serde(tag = "type", rename_all = "snake_case")]`
- **Kind filter list:** `["event_kind_one","event_kind_two"]` (snake_case, sorted)
- **Cursor:** Bare `u64` on wire, monotonic increasing per instance
- **Amounts:** `u64` in mojos (smallest indivisible unit)
- **Timestamps:** None on the wire itself (events carry heights, not absolute times; time mapping is app-layer)

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
