//! Task 3 — support, source/metric availability, acquisition outcome, and
//! the §4.5 legal-combination rules, including the timeout rule (§4.6).
//!
//! These four concepts are distinct and must not collapse into one value.

use aer_core::{
    AcquisitionError, LimitationCode, MetricAvailability, MetricResult, MetricSample, Outcome,
    Provenance, SampleValue, SourceAvailability, Support, Timestamp, Unit,
};

#[test]
fn constructors_are_legal_and_match_their_rows() {
    let unsupported = MetricResult::unsupported(LimitationCode::NotImplementedOnThisPlatform);
    assert!(unsupported.is_legal());
    assert!(matches!(unsupported.support, Support::Unsupported { .. }));
    assert!(unsupported.source_availability.is_none());
    assert_eq!(
        unsupported.metric_availability,
        Some(MetricAvailability::NotApplicable)
    );
    assert!(matches!(unsupported.outcome, Outcome::NoValue));
    assert!(!unsupported.is_failure());

    let not_ready = MetricResult::source_not_ready(SourceAvailability::NotRunning);
    assert!(not_ready.is_legal());
    assert!(matches!(not_ready.support, Support::Supported));
    assert_eq!(
        not_ready.source_availability,
        Some(SourceAvailability::NotRunning)
    );
    assert!(not_ready.metric_availability.is_none());
    assert!(matches!(not_ready.outcome, Outcome::NoValue));
    assert!(!not_ready.is_failure());

    let not_exposed = MetricResult::metric_not_available(MetricAvailability::NotExposed);
    assert!(not_exposed.is_legal());
    assert_eq!(
        not_exposed.source_availability,
        Some(SourceAvailability::Ready)
    );
    assert_eq!(
        not_exposed.metric_availability,
        Some(MetricAvailability::NotExposed)
    );
    assert!(matches!(not_exposed.outcome, Outcome::NoValue));
    assert!(!not_exposed.is_failure());
}

#[test]
fn impossible_ready_available_no_value_is_illegal() {
    // §4.5 has no row for Ready + Available + NoValue: a ready, exposed source
    // must yield Ok or Failed, never a bare NoValue.
    let bad = MetricResult {
        support: Support::Supported,
        source_availability: Some(SourceAvailability::Ready),
        metric_availability: Some(MetricAvailability::Available),
        outcome: Outcome::NoValue,
    };
    assert!(!bad.is_legal());
}

#[test]
fn not_applicable_paired_with_supported_is_illegal() {
    // NotApplicable is reserved for Unsupported (§4.5, §4.3).
    let bad = MetricResult {
        support: Support::Supported,
        source_availability: Some(SourceAvailability::Ready),
        metric_availability: Some(MetricAvailability::NotApplicable),
        outcome: Outcome::NoValue,
    };
    assert!(!bad.is_legal());
}

#[test]
fn source_not_ready_leaves_metric_availability_none() {
    let r = MetricResult::source_not_ready(SourceAvailability::NotDetected);
    assert!(r.metric_availability.is_none());
    assert!(r.is_legal());
    assert!(!r.is_failure());
}

#[test]
fn metric_not_available_is_no_value_and_not_a_failure() {
    for metric in [
        MetricAvailability::NotExposed,
        MetricAvailability::TransientlyUnavailable,
    ] {
        let r = MetricResult::metric_not_available(metric.clone());
        assert!(
            matches!(r.outcome, Outcome::NoValue),
            "expected NoValue for {metric:?}"
        );
        assert!(!r.is_failure(), "unavailability must not read as failure");
        assert!(r.is_legal());
    }
}

#[test]
fn failed_timeout_is_a_failure() {
    // A timeout is a failure, not normal unavailability (§4.6).
    let r = MetricResult::failed(AcquisitionError::Timeout);
    assert!(r.is_failure());
    assert!(matches!(
        r.outcome,
        Outcome::Failed(AcquisitionError::Timeout)
    ));
    assert!(r.is_legal());
}

#[test]
fn ok_carries_sample_and_sets_ready_available() {
    let sample = MetricSample {
        value: SampleValue::Count(7),
        unit: Unit::Count,
        provenance: Provenance::RuntimeReported,
        timestamp: Timestamp::from_millis(1_700_000_000_000),
        limitations: vec![],
    };
    let r = MetricResult::ok(sample);
    assert_eq!(r.source_availability, Some(SourceAvailability::Ready));
    assert_eq!(r.metric_availability, Some(MetricAvailability::Available));
    match &r.outcome {
        Outcome::Ok(s) => assert!(matches!(s.value, SampleValue::Count(7))),
        other => panic!("expected Ok, got {other:?}"),
    }
    assert!(r.is_legal());
    assert!(!r.is_failure());
}

#[test]
fn acquisition_error_messages_are_controlled() {
    let err = AcquisitionError::Timeout;
    let msg = err.message();
    assert!(!msg.is_empty());
    assert_eq!(msg, err.message());
    assert_ne!(
        AcquisitionError::Connection.message(),
        AcquisitionError::Timeout.message()
    );
    // category() is a stable lowercase code used by the report/IPC boundary.
    assert_eq!(AcquisitionError::Timeout.category(), "timeout");
}
