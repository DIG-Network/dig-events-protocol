//! The two shape traits the engine implements and apps depend on (SPEC §7).
//!
//! These define the SHAPE of emit + catch-up without any machinery: [`EventEmitter`] is what the
//! engine's event bus offers to producers, and [`CatchUp`] is what a subscriber calls once after a
//! gap to backfill the missed range before resuming the live stream. The concrete bus, persisted
//! delta store, and live subscription streams live in the engine (dig-wallet-backend) — this crate
//! only fixes the interface so a second implementation is interchangeable.

use async_trait::async_trait;
use enumset::EnumSet;

use crate::cursor::{Cursor, EmittedEvent};
use crate::event::WalletEvent;
use crate::kind::EventKind;

/// A sink that accepts emitted events and stamps each with a monotonic [`Cursor`].
///
/// The engine implements this over its event bus; a producer calls [`publish`](EventEmitter::publish)
/// and gets back the cursor the event was assigned (so it can correlate or persist a checkpoint).
pub trait EventEmitter {
    /// Publish an event, returning the monotonic [`Cursor`] stamped on it.
    fn publish(&self, event: WalletEvent) -> Cursor;
}

/// The backfill half of the subscription contract: a subscriber that fell behind calls this ONCE to
/// fetch the events it missed, then resumes the live stream.
///
/// The associated [`Error`](CatchUp::Error) type is generic so this leaf crate needs no error
/// dependency — the engine picks its own error type when it implements the trait.
#[async_trait]
pub trait CatchUp {
    /// The implementer's backfill error type.
    type Error;

    /// Return every [`EmittedEvent`] with a cursor STRICTLY GREATER than `since`, in cursor order,
    /// optionally narrowed to the subscriber's `filter`. Passing an empty filter is the caller's
    /// choice to receive nothing; pass [`EnumSet::all`] to backfill every kind.
    async fn catch_up(
        &self,
        since: Cursor,
        filter: EnumSet<EventKind>,
    ) -> Result<Vec<EmittedEvent>, Self::Error>;
}

/// Retain only the [`EmittedEvent`]s whose event kind passes `filter`, preserving cursor order.
///
/// The shared, drift-free filtering rule used on BOTH sides: the engine narrows a live stream with
/// it, and a [`CatchUp`] implementer applies the same rule to its backfill so live and catch-up
/// deliver an identical filtered view.
pub fn filter_events(
    events: impl IntoIterator<Item = EmittedEvent>,
    filter: EnumSet<EventKind>,
) -> Vec<EmittedEvent> {
    events
        .into_iter()
        .filter(|e| e.event.matches(filter))
        .collect()
}
