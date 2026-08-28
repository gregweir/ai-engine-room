//! Task 5 — in-memory session, snapshot, and event boundary.
//!
//! Minimal in-memory model: timestamped snapshots of descriptor/result/
//! explanation, and one representative event proving later runtime/model state
//! changes can be represented without redesigning Session. No persistence.

use aer_core::{
    explain, AcquisitionError, Event, MetricDescriptor, MetricId, MetricResult, Session, Snapshot,
    SnapshotEntry, SourceAvailability, SourceClass, Timestamp, Unit,
};

fn descriptor(name: &str) -> MetricDescriptor {
    MetricDescriptor {
        id: MetricId::new(name),
        display_name: String::from(name),
        unit: Unit::Count,
        source_class: SourceClass::OperatingSystem,
    }
}

fn entry(name: &str, result: MetricResult) -> SnapshotEntry {
    let d = descriptor(name);
    let explanation = explain(&d, &result);
    SnapshotEntry {
        descriptor: d,
        result,
        explanation,
    }
}

fn sample_snapshot() -> Snapshot {
    Snapshot {
        at: Timestamp::from_millis(1),
        entries: vec![
            entry(
                "os.ram.used",
                MetricResult::failed(AcquisitionError::Timeout),
            ),
            entry(
                "os.cpu.count",
                MetricResult::unsupported(aer_core::LimitationCode::NotImplementedOnThisPlatform),
            ),
        ],
    }
}

#[test]
fn new_session_starts_empty() {
    let s = Session::new();
    assert!(s.snapshots.is_empty());
    assert!(s.events.is_empty());
}

#[test]
fn snapshots_accumulate_and_update_count() {
    let mut s = Session::new();
    s.add_snapshot(sample_snapshot());
    assert_eq!(s.snapshots.len(), 1);
    s.add_snapshot(sample_snapshot());
    assert_eq!(s.snapshots.len(), 2);
    // Entries survive intact inside the snapshot.
    assert_eq!(s.snapshots[0].entries.len(), 2);
}

#[test]
fn events_accumulate_and_update_count() {
    let mut s = Session::new();
    let e = Event::SourceAvailabilityChanged {
        metric: MetricId::new("os.ram.used"),
        from: SourceAvailability::NotRunning,
        to: SourceAvailability::Ready,
        at: Timestamp::from_millis(2),
    };
    s.record_event(e.clone());
    assert_eq!(s.events.len(), 1);
    s.record_event(e);
    assert_eq!(s.events.len(), 2);
    // The recorded event carries the transition.
    assert!(matches!(
        &s.events[0],
        Event::SourceAvailabilityChanged {
            to: SourceAvailability::Ready,
            ..
        }
    ));
}

#[test]
fn session_records_started_time() {
    let s = Session::new();
    // started is a real, non-zero timestamp taken from the system clock at
    // session creation — the lightweight session-start context for 1A.
    assert!(s.started.millis_since_epoch() > 0);
}

#[test]
fn snapshot_entry_couples_descriptor_result_and_explanation() {
    let e = entry(
        "os.ram.used",
        MetricResult::failed(AcquisitionError::Timeout),
    );
    // The explanation was derived from the same result it is stored beside.
    assert_eq!(e.explanation, explain(&e.descriptor, &e.result));
}
