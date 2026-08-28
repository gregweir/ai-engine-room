//! Minimal in-memory session boundary.
//!
//! A session is a lightweight, in-memory sequence of timestamped snapshots and
//! events. There is no database, filesystem persistence, history retention,
//! cloud sync, analytics, or replay infrastructure here — only what the
//! in-memory model naturally supports. Semantics per design §7.
//!
//! Time is held as milliseconds since the Unix epoch (see [`Timestamp`]); no
//! external date/time dependency and no calendar/timezone formatting is added
//! to the core.

use crate::availability::SourceAvailability;
use crate::explain::Explanation;
use crate::identity::{MetricDescriptor, MetricId};
use crate::result::MetricResult;
use crate::value::Timestamp;

/// One metric's observed state within a snapshot: its identity, the acquired
/// result, and the deterministic explanation of that result.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapshotEntry {
    pub descriptor: MetricDescriptor,
    pub result: MetricResult,
    pub explanation: Explanation,
}

/// A timestamped set of metric observations.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub at: Timestamp,
    pub entries: Vec<SnapshotEntry>,
}

/// A minimal, extensible event type. One representative variant proves that
/// later runtime/model state changes can be recorded without redesigning
/// [`Session`]; further variants can be added here as needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A source's availability transitioned between two states.
    SourceAvailabilityChanged {
        metric: MetricId,
        from: SourceAvailability,
        to: SourceAvailability,
        at: Timestamp,
    },
}

/// An in-memory session: a start time and the snapshots and events accumulated
/// since. All state lives in memory and is dropped when the session is dropped.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub started: Timestamp,
    pub snapshots: Vec<Snapshot>,
    pub events: Vec<Event>,
}

impl Session {
    /// Create a new, empty session stamped at the current system time.
    pub fn new() -> Self {
        Self {
            started: Timestamp::now(),
            snapshots: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Record a timestamped snapshot of observations.
    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshots.push(snapshot);
    }

    /// Record a runtime/model state-change event.
    pub fn record_event(&mut self, event: Event) {
        self.events.push(event);
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
