//! The delivery cursor and its event envelope (SPEC §4).
//!
//! Every delivered event is stamped with a monotonic, per-instance [`Cursor`]. A subscriber
//! remembers the last cursor it saw; on a gap (reconnect or lag) it calls `catch_up(since)` ONCE to
//! backfill the missed range, then resumes the live stream. [`EmittedEvent`] is the envelope that
//! flows over the live stream AND is returned by catch-up backfill.

use serde::{Deserialize, Serialize};

use crate::event::WalletEvent;

/// A monotonic, per-instance sequence number stamped on delivered events.
///
/// A subscriber remembers the last cursor it saw; on a gap (reconnect or lag) it calls
/// `catch_up(since)` ONCE to backfill the missed range, then resumes the live stream.
/// Serializes as a bare `u64` on the wire.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct Cursor(pub u64);

impl Cursor {
    /// The next cursor in sequence.
    pub fn next(self) -> Cursor {
        Cursor(self.0 + 1)
    }
}

/// A delivered event paired with its monotonic [`Cursor`].
///
/// The engine stamps a per-instance cursor on every event as it is emitted; subscribers remember
/// the last cursor and pass it to `catch_up` after a gap. This envelope is what flows over the
/// subscription stream (live) and what catch-up returns (backfill).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmittedEvent {
    /// The monotonic delivery cursor.
    pub cursor: Cursor,
    /// The event payload.
    pub event: WalletEvent,
}
