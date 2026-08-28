//! Task 4 — deterministic explanation boundary.
//!
//! Proves that explanation logic distinguishes a successful available value,
//! a normal unavailable/no-value state, an unsupported capability, and an
//! actual acquisition failure (timeout) — without alarming the user over
//! value magnitude or inferring hardware/safety/performance claims.

use aer_core::{
    explain, AcquisitionError, LimitationCode, MetricAvailability, MetricDescriptor, MetricId,
    MetricResult, MetricSample, Provenance, SampleValue, SourceAvailability, SourceClass,
    Timestamp, Unit,
};

fn os_descriptor() -> MetricDescriptor {
    MetricDescriptor {
        id: MetricId::new("os.ram.used"),
        display_name: String::from("Memory in use"),
        unit: Unit::Bytes,
        source_class: SourceClass::OperatingSystem,
    }
}

fn sample(provenance: Provenance, value: u64) -> MetricSample {
    MetricSample {
        value: SampleValue::Count(value),
        unit: Unit::Count,
        provenance,
        timestamp: Timestamp::from_millis(1_700_000_000_000),
        limitations: vec![],
    }
}

#[test]
fn no_value_and_failure_explain_differently() {
    let d = os_descriptor();
    let no_value = MetricResult::metric_not_available(MetricAvailability::NotExposed);
    let failed = MetricResult::failed(AcquisitionError::Timeout);

    let e_nv = explain(&d, &no_value);
    let e_fail = explain(&d, &failed);

    assert_ne!(
        e_nv.interpretation.as_str(),
        e_fail.interpretation.as_str(),
        "unavailable and failure must not share an interpretation"
    );
    assert!(
        e_fail
            .interpretation
            .as_str()
            .to_lowercase()
            .contains("failed"),
        "a failure explanation must say it failed"
    );
    assert!(
        !e_nv
            .interpretation
            .as_str()
            .to_lowercase()
            .contains("failed"),
        "a normal unavailable state is not a failure"
    );
}

#[test]
fn ok_high_value_is_not_alarmist() {
    let d = os_descriptor();
    let r = MetricResult::ok(sample(Provenance::OperatingSystemReported, u64::MAX));
    let e = explain(&d, &r);
    let text = e.interpretation.as_str().to_lowercase();
    assert!(!text.contains("danger"));
    assert!(!text.contains("critical"));
    assert!(!text.contains("warning"));
}

#[test]
fn explain_is_deterministic() {
    let d = os_descriptor();
    let r = MetricResult::ok(sample(Provenance::OperatingSystemReported, 42));
    let a = explain(&d, &r);
    let b = explain(&d, &r);
    assert_eq!(a, b);
}

#[test]
fn total_memory_explanation_preserves_platform_semantics_and_non_claims() {
    let descriptor = MetricDescriptor {
        id: MetricId::new("os.ram.total"),
        display_name: String::from("Total memory"),
        unit: Unit::Bytes,
        source_class: SourceClass::OperatingSystem,
    };
    let result = MetricResult::ok(sample(Provenance::OperatingSystemReported, 16_000_000_000));
    let explanation = explain(&descriptor, &result);
    let text = format!(
        "{} {} {}",
        explanation.interpretation.as_str(),
        explanation.why_it_matters.as_str(),
        explanation.deeper.expect("qualified").as_str()
    );
    assert!(text.contains("Linux"));
    assert!(text.contains("Windows"));
    assert!(text.contains("does not establish memory pressure"));
}

#[test]
fn unsupported_differs_from_unavailable_and_failure() {
    let d = os_descriptor();
    let unsupported = MetricResult::unsupported(LimitationCode::NotImplementedOnThisPlatform);
    let no_value = MetricResult::metric_not_available(MetricAvailability::NotExposed);
    let failed = MetricResult::failed(AcquisitionError::Timeout);

    let uns = explain(&d, &unsupported).interpretation.as_str();
    let nv = explain(&d, &no_value).interpretation.as_str();
    let fail = explain(&d, &failed).interpretation.as_str();

    assert_ne!(uns, nv);
    assert_ne!(uns, fail);
    assert_ne!(nv, fail);
    // Unsupported is a capability statement, not an acquisition failure.
    assert!(!uns.to_lowercase().contains("failed"));
}

#[test]
fn unsupported_deeper_carries_controlled_limitation() {
    let d = os_descriptor();
    let r = MetricResult::unsupported(LimitationCode::RequiresDriver);
    let e = explain(&d, &r);
    let deeper = e
        .deeper
        .expect("unsupported should carry a deeper limitation note");
    assert_eq!(deeper.as_str(), LimitationCode::RequiresDriver.message());
}

#[test]
fn ok_interpretation_reflects_provenance() {
    let d = os_descriptor();
    let os_e = explain(
        &d,
        &MetricResult::ok(sample(Provenance::OperatingSystemReported, 1)),
    );
    let rt_e = explain(
        &d,
        &MetricResult::ok(sample(Provenance::RuntimeReported, 1)),
    );
    assert!(os_e
        .interpretation
        .as_str()
        .to_lowercase()
        .contains("operating system"));
    assert!(rt_e
        .interpretation
        .as_str()
        .to_lowercase()
        .contains("runtime"));
    assert_ne!(os_e.interpretation.as_str(), rt_e.interpretation.as_str());
}

#[test]
fn source_not_ready_no_value_is_explained_by_source_state() {
    let d = os_descriptor();
    let r = MetricResult::source_not_ready(SourceAvailability::NotRunning);
    let e = explain(&d, &r);
    // A normal unavailable state explains the source, never claims failure.
    let text = e.interpretation.as_str().to_lowercase();
    assert!(!text.contains("failed"));
}

#[test]
fn ram_available_explanation_mentions_kernel_estimate() {
    let d = MetricDescriptor {
        id: MetricId::new("os.ram.available"),
        display_name: String::from("Available memory"),
        unit: Unit::Bytes,
        source_class: SourceClass::OperatingSystem,
    };
    let r = MetricResult::ok(MetricSample {
        value: SampleValue::Count(8_589_934_592),
        unit: Unit::Bytes,
        provenance: Provenance::OperatingSystemReported,
        timestamp: Timestamp::from_millis(1_700_000_000_000),
        limitations: vec![LimitationCode::KernelEstimate],
    });
    let e = explain(&d, &r);
    let interp = e.interpretation.as_str().to_lowercase();
    let why = e.why_it_matters.as_str().to_lowercase();
    let deeper = e.deeper.expect("ram available should have a deeper note");
    assert!(interp.contains("linux"));
    assert!(interp.contains("estimate"));
    assert!(interp.contains("memory"));
    assert!(why.contains("local ai") || why.contains("inference"));
    assert!(deeper.as_str().contains("/proc/meminfo"));
    assert!(deeper.as_str().contains("estimate"));
}
