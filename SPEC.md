# dig-events-protocol — SPEC

Normative contract for the DIG Network blockchain→app event stream. An independent implementation
built to this spec is interchangeable with the reference. The wire shapes here are FROZEN — a change
that alters any serialized form is a breaking (major) protocol event, guarded by the conformance KATs
in `tests/conformance.rs`.

## §1. Scope

This crate defines ONLY the CONTRACT: the event types, the subscription-filter discriminant, the
delivery cursor/envelope, the sync-state snapshot, the payload newtypes, and the two shape traits
(`EventEmitter`, `CatchUp`) plus the shared `filter_events` rule. The MACHINERY that produces,
persists, streams, and catches up events (the event bus, the persisted delta store, live
subscriptions) lives in the engine (dig-wallet-backend) and is OUT OF SCOPE here. This split lets
every consumer depend on the contract without pulling in the runtime.

## §2. Dependencies + purity

A pure leaf crate. Runtime dependencies are `serde`, `enumset`, and `async-trait` ONLY — NO `dig-*`
dependency, NO `tokio` (or any runtime), NO transport/error crate. `CatchUp::Error` is a generic
associated type so the leaf carries no error dependency. `#![forbid(unsafe_code)]` and
`#![warn(missing_docs)]` hold.

## §3. Payload newtypes

- `WalletId(pub u32)` — wallet identifier. Wire: bare JSON number.
- `Amount(pub u64)` — value in the smallest unit (mojos); `.mojos()` reads the raw value. Wire: bare
  JSON number.
- `AssetId(pub String)` — a CAT TAIL hash, hex. On an event's `asset` field, `Some` = a CAT and
  `None` = native XCH. Wire: bare JSON string.

## §4. Cursor + envelope

- `Cursor(pub u64)` — a monotonic, per-instance delivery sequence number; `Cursor::next()` returns
  the successor. Wire: bare JSON number.
- `EmittedEvent { cursor: Cursor, event: WalletEvent }` — the envelope carried over the live stream
  and returned by catch-up backfill. Wire: `{"cursor":<n>,"event":<WalletEvent>}`.

A subscriber remembers the last cursor it saw; on a gap it calls `catch_up(since)` ONCE to backfill
every event with a cursor STRICTLY GREATER than `since`, then resumes the live stream.

## §5. WalletEvent taxonomy

`WalletEvent` is a serde enum tagged by `type` in snake_case (`{"type":"funds_received",…}`). The 11
variants and their fields (all field names are wire contract; order is as declared):

| Variant | Fields |
|---|---|
| `FundsReceived` | `wallet_id: WalletId`, `asset: Option<AssetId>`, `amount: Amount`, `coin_id: String`, `confirmed_height: u32` |
| `FundsSent` | `wallet_id: WalletId`, `asset: Option<AssetId>`, `amount: Amount`, `tx_id: String`, `confirmed_height: u32` |
| `CoinStateChanged` | `coin_id: String`, `spent: bool`, `created_height: Option<u32>`, `spent_height: Option<u32>` |
| `Confirmation` | `tx_id: String`, `height: u32` |
| `TransactionFailed` | `tx_id: String`, `error: String` |
| `NewTip` | `height: u32`, `header_hash: String` |
| `SyncProgress` | `wallet_id: WalletId`, `state: SyncLifecycle`, `peak_height: u32`, `target_height: u32` |
| `CatInfo` | `asset_id: AssetId`, `name: Option<String>` |
| `DidInfo` | `launcher_id: String` |
| `NftData` | `launcher_id: String` |
| `Derivation` | `wallet_id: WalletId`, `index: u32` |

`EventKind` is the matching discriminant enum (same 11 names, snake_case). `WalletEvent::kind()`
returns the variant's `EventKind`; `WalletEvent::matches(filter)` is `filter.contains(self.kind())`.

An `EnumSet<EventKind>` is the subscription FILTER and serializes as a snake_case LIST
(`["funds_received","funds_sent"]`) — never a bitmask integer.

## §6. Sync state

- `SyncLifecycle` — `Idle` / `Syncing` / `Synced`, snake_case on the wire.
- `SyncStatus { state: SyncLifecycle, peak_height: u32, target_height: u32 }` — a sync snapshot.

## §7. Shape traits

- `EventEmitter::publish(&self, WalletEvent) -> Cursor` — accepts an event, returns the monotonic
  cursor stamped on it. Implemented by the engine's event bus.
- `CatchUp` (async, via `async-trait`): `type Error;` and `async fn catch_up(&self, since: Cursor,
  filter: EnumSet<EventKind>) -> Result<Vec<EmittedEvent>, Self::Error>`. Returns every
  `EmittedEvent` with `cursor > since`, in cursor order, narrowed to `filter`.
- `filter_events(events, filter) -> Vec<EmittedEvent>` — retains only events whose kind passes
  `filter`, preserving cursor order. Both the live path and a `CatchUp` implementer apply this SAME
  rule so live and catch-up deliver an identical filtered view.

## §8. Conformance — the drift-freeze

`tests/conformance.rs` pins the EXACT serialized JSON of every `WalletEvent` variant, the
`EmittedEvent` envelope, and the `EventKind` list (including the full-set order) to golden strings,
and round-trips each. Any change that renames/reorders a field, changes a tag, or alters the null/list
representation breaks a golden assertion. Fixture values are derived from a hashed seed (no literal
constants). An implementation is conformant iff it produces byte-identical wire output for the same
inputs.
