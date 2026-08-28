//! Task 6 — report-safe boundary: allow-list projection, defence-in-depth
//! sanitizer, and preview.
//!
//! Two test classes, kept distinct:
//! 1. unsafe/internal fields never enter the report-safe model through the
//!    normal projection (the allow-list is the boundary);
//! 2. the sanitizer still catches deliberately injected unsafe-looking content
//!    if it reaches a report-safe textual field unexpectedly (defence-in-depth).

use aer_core::{
    explain, project, render_preview, sanitize, AcquisitionError, LimitationCode, MetricDescriptor,
    MetricId, MetricResult, MetricSample, Provenance, ReportEntry, ReportProvenance,
    ReportSnapshot, ReportStatus, ReportValue, SampleValue, Snapshot, SnapshotEntry, SourceClass,
    Timestamp, Unit,
};

fn descriptor(id: &str, name: &str) -> MetricDescriptor {
    MetricDescriptor {
        id: MetricId::new(id),
        display_name: String::from(name),
        unit: Unit::Bytes,
        source_class: SourceClass::OperatingSystem,
    }
}

fn entry(id: &str, name: &str, result: MetricResult) -> SnapshotEntry {
    let d = descriptor(id, name);
    let explanation = explain(&d, &result);
    SnapshotEntry {
        descriptor: d,
        result,
        explanation,
    }
}

fn snapshot(entries: Vec<SnapshotEntry>) -> Snapshot {
    Snapshot {
        at: Timestamp::from_millis(1),
        entries,
    }
}

fn ok_sample(value: u64) -> MetricSample {
    MetricSample {
        value: SampleValue::Count(value),
        unit: Unit::Bytes,
        provenance: Provenance::OperatingSystemReported,
        timestamp: Timestamp::from_millis(0),
        limitations: vec![],
    }
}

// --- Test class 1: allow-list is the boundary --------------------------------

#[test]
fn non_allowlisted_metric_with_private_path_is_absent() {
    let allowed = entry(
        "os.ram.used",
        "Memory in use",
        MetricResult::ok(ok_sample(1024)),
    );
    let private = entry(
        "internal.debug.path",
        "/home/alice/secret",
        MetricResult::ok(ok_sample(1)),
    );
    let report = project(&snapshot(vec![allowed, private]));
    // Only the allow-listed metric crosses into the report-safe model.
    assert_eq!(report.entries.len(), 1);
    assert_eq!(report.entries[0].display_name, "Memory in use");
    // The private path never reaches any report-safe field.
    for e in &report.entries {
        assert!(!e.display_name.contains("/home/"));
        assert!(!e.display_name.contains("alice"));
    }
}

#[test]
fn total_memory_remains_outside_the_report_allow_list() {
    let total = entry(
        "os.ram.total",
        "Total memory",
        MetricResult::ok(ok_sample(16_000_000_000)),
    );
    let report = project(&snapshot(vec![total]));
    assert!(report.entries.is_empty());
}

#[test]
fn failed_entry_has_no_value_and_no_raw_error() {
    let e = entry(
        "os.ram.used",
        "Memory in use",
        MetricResult::failed(AcquisitionError::Timeout),
    );
    let report = project(&snapshot(vec![e]));
    assert_eq!(report.entries.len(), 1);
    let re = &report.entries[0];
    assert_eq!(re.status, ReportStatus::Failed);
    assert!(re.value.is_none());
    assert!(re.provenance.is_none());
    // ReportEntry has no error field; the preview must not leak the raw
    // acquisition error message.
    let preview = render_preview(&report);
    assert!(!preview.to_lowercase().contains("timed out"));
    assert!(!preview
        .to_lowercase()
        .contains(&AcquisitionError::Timeout.message().to_lowercase()));
}

#[test]
fn ok_projects_to_available_with_value_and_provenance() {
    let mut sample = ok_sample(1024);
    sample.limitations = vec![LimitationCode::Sampling];
    let e = entry("os.ram.used", "Memory in use", MetricResult::ok(sample));
    let report = project(&snapshot(vec![e]));
    let re = &report.entries[0];
    assert_eq!(re.status, ReportStatus::Available);
    let v = re.value.as_ref().expect("available entry carries a value");
    assert_eq!(v.amount, 1024.0);
    assert_eq!(v.unit, Unit::Bytes);
    assert_eq!(
        re.provenance,
        Some(ReportProvenance::OperatingSystemReported)
    );
    assert_eq!(re.limitation_codes, vec![LimitationCode::Sampling]);
}

#[test]
fn unsupported_projects_to_not_available_with_limitation_code() {
    let e = entry(
        "os.ram.used",
        "Memory in use",
        MetricResult::unsupported(LimitationCode::RequiresDriver),
    );
    let report = project(&snapshot(vec![e]));
    let re = &report.entries[0];
    assert_eq!(re.status, ReportStatus::NotAvailable);
    assert!(re.value.is_none());
    assert_eq!(re.limitation_codes, vec![LimitationCode::RequiresDriver]);
}

#[test]
fn projection_copies_display_name_not_internal_id() {
    let e = entry(
        "os.ram.used",
        "Memory in use",
        MetricResult::ok(ok_sample(1)),
    );
    let report = project(&snapshot(vec![e]));
    // ReportEntry has no id field at all; the preview uses the display name.
    let preview = render_preview(&report);
    assert!(preview.contains("Memory in use"));
    assert!(!preview.contains("os.ram.used"));
}

#[test]
fn ram_available_is_allowlisted() {
    let ram = entry(
        "os.ram.available",
        "Available memory",
        MetricResult::ok(ok_sample(8_589_934_592)),
    );
    let report = project(&snapshot(vec![ram]));
    assert_eq!(report.entries.len(), 1);
    assert_eq!(report.entries[0].display_name, "Available memory");
    assert_eq!(report.entries[0].status, ReportStatus::Available);
    assert_eq!(
        report.entries[0].value,
        Some(ReportValue {
            amount: 8_589_934_592.0,
            unit: Unit::Bytes,
        })
    );
    assert_eq!(
        report.entries[0].provenance,
        Some(ReportProvenance::OperatingSystemReported)
    );
}

// --- Test class 2: sanitizer is defence-in-depth -----------------------------

#[test]
fn sanitizer_redacts_injected_unsafe_display_name() {
    let mut report = ReportSnapshot {
        at: Timestamp::from_millis(0),
        entries: vec![ReportEntry {
            display_name: String::from("/home/alice/secret"),
            status: ReportStatus::Available,
            value: Some(ReportValue {
                amount: 1.0,
                unit: Unit::Bytes,
            }),
            provenance: None,
            limitation_codes: vec![],
        }],
    };
    sanitize(&mut report);
    assert_eq!(report.entries[0].display_name, "[redacted]");
}

#[test]
fn sanitizer_catches_hostile_patterns_but_leaves_safe_names() {
    let hostile = [
        "/home/alice/secret",
        "/var/log/app",
        "alice@nowhere.test",
        "192.168.0.5",
        "10.0.0.7",
        "build-agent.local",
        "machine.internal",
        "C:\\Users\\bob",
    ];
    for raw in hostile {
        let mut report = ReportSnapshot {
            at: Timestamp::from_millis(0),
            entries: vec![ReportEntry {
                display_name: String::from(raw),
                status: ReportStatus::Available,
                value: None,
                provenance: None,
                limitation_codes: vec![],
            }],
        };
        sanitize(&mut report);
        assert_eq!(
            report.entries[0].display_name, "[redacted]",
            "failed to redact {raw}"
        );
    }
    // A safe, catalogue-style display name is left untouched.
    let mut report = ReportSnapshot {
        at: Timestamp::from_millis(0),
        entries: vec![ReportEntry {
            display_name: String::from("Memory in use"),
            status: ReportStatus::Available,
            value: None,
            provenance: None,
            limitation_codes: vec![],
        }],
    };
    sanitize(&mut report);
    assert_eq!(report.entries[0].display_name, "Memory in use");
}

#[test]
fn render_preview_after_sanitize_is_path_free() {
    let e = entry(
        "os.ram.used",
        "Memory in use",
        MetricResult::ok(ok_sample(1024)),
    );
    let mut report = project(&snapshot(vec![e]));
    sanitize(&mut report);
    let preview = render_preview(&report);
    assert!(preview.contains("Memory in use"));
    assert!(preview.contains("Status: Available"));
    assert!(preview.contains("1.02 KB"));
    assert!(preview.contains("1,024 bytes"));
    assert!(!preview.contains("/home/"));
    assert!(!preview.contains('@'));
    assert!(!preview.contains("192.168"));
}

#[test]
fn available_memory_preview_is_structured_human_readable_and_has_no_timestamp() {
    let mut sample = ok_sample(12_961_325_056);
    sample.limitations = vec![LimitationCode::KernelEstimate];
    let report = project(&snapshot(vec![entry(
        "os.ram.available",
        "Available memory",
        MetricResult::ok(sample),
    )]));

    assert_eq!(
        render_preview(&report),
        concat!(
            "AI Engine Room — Observation Report\n",
            "\n",
            "Observation\n",
            "Available memory\n",
            "Status: Available\n",
            "Value: 12.96 GB\n",
            "Exact value: 12,961,325,056 bytes\n",
            "Source: Operating system\n",
            "\n",
            "Qualifications\n",
            "- Available memory is an operating-system observation.\n",
            "- Operating systems define available memory differently; values from different platforms are not claimed to be numerically equivalent.\n",
            "- This value is an estimate supplied by the operating system, not a direct physical measurement.\n",
            "\n",
            "Scope\n",
            "- This report contains only allow-listed report-safe observations.\n",
            "- Available memory does not establish memory pressure, model fit, headroom, or compute placement.\n",
        )
    );
}

#[test]
fn preview_omits_report_timestamp_even_when_it_is_distinctive() {
    let report = ReportSnapshot {
        at: Timestamp::from_millis(1_700_123_456_789),
        entries: vec![],
    };
    let preview = render_preview(&report);
    assert!(!preview.contains("1700123456789"));
    assert_eq!(
        preview,
        concat!(
            "AI Engine Room — Observation Report\n",
            "\n",
            "Scope\n",
            "- This report contains only allow-listed report-safe observations.\n",
            "- Available memory does not establish memory pressure, model fit, headroom, or compute placement.\n",
        )
    );
}

#[test]
fn byte_formatter_covers_zero_singular_and_approved_decimal_cases() {
    let cases = [
        (0.0, "Value: 0 B\nExact value: 0 bytes"),
        (1.0, "Value: 1 B\nExact value: 1 byte"),
        (1024.0, "Value: 1.02 KB\nExact value: 1,024 bytes"),
        (1_048_576.0, "Value: 1.05 MB\nExact value: 1,048,576 bytes"),
        (
            13_175_230_464.0,
            "Value: 13.18 GB\nExact value: 13,175,230,464 bytes",
        ),
    ];
    for (amount, expected) in cases {
        let report = ReportSnapshot {
            at: Timestamp::from_millis(0),
            entries: vec![ReportEntry {
                display_name: String::from("Available memory"),
                status: ReportStatus::Available,
                value: Some(ReportValue {
                    amount,
                    unit: Unit::Bytes,
                }),
                provenance: Some(ReportProvenance::OperatingSystemReported),
                limitation_codes: vec![],
            }],
        };
        assert!(
            render_preview(&report).contains(expected),
            "amount {amount}"
        );
    }
}

#[test]
fn unsafe_or_fractional_bytes_are_approximate_and_never_printed_as_exact() {
    for amount in [9_007_199_254_740_992.0, u64::MAX as f64, 1.5] {
        let report = ReportSnapshot {
            at: Timestamp::from_millis(0),
            entries: vec![ReportEntry {
                display_name: String::from("Available memory"),
                status: ReportStatus::Available,
                value: Some(ReportValue {
                    amount,
                    unit: Unit::Bytes,
                }),
                provenance: Some(ReportProvenance::OperatingSystemReported),
                limitation_codes: vec![],
            }],
        };
        let preview = render_preview(&report);
        assert!(preview.contains("Value: ≈"));
        assert!(preview.contains("Exact value: Not representable exactly in this presentation"));
        assert!(!preview.contains(&format!("Exact value: {amount:.0} bytes")));
        if amount == u64::MAX as f64 {
            assert!(preview.contains("Value: ≈18.45 EB"));
        }
    }
}

#[test]
fn invalid_byte_values_and_absent_values_never_fabricate_zero() {
    for (status, value) in [
        (ReportStatus::Available, Some(f64::NAN)),
        (ReportStatus::Available, Some(f64::INFINITY)),
        (ReportStatus::Available, Some(-1.0)),
        (ReportStatus::NotAvailable, None),
        (ReportStatus::Failed, None),
    ] {
        let report = ReportSnapshot {
            at: Timestamp::from_millis(0),
            entries: vec![ReportEntry {
                display_name: String::from("Available memory"),
                status,
                value: value.map(|amount| ReportValue {
                    amount,
                    unit: Unit::Bytes,
                }),
                provenance: None,
                limitation_codes: vec![],
            }],
        };
        let preview = render_preview(&report);
        assert!(preview.contains("Value: No value reported"));
        assert!(!preview.contains("Value: 0 B"));
    }
}

#[test]
fn every_payload_free_provenance_has_a_controlled_source_label() {
    let provenances = [
        (
            ReportProvenance::OperatingSystemReported,
            "Operating system",
        ),
        (ReportProvenance::DriverReported, "Hardware driver"),
        (ReportProvenance::RuntimeReported, "AI runtime"),
        (ReportProvenance::ApplicationMeasured, "AI Engine Room"),
        (
            ReportProvenance::Calculated,
            "Calculated from other metrics",
        ),
        (ReportProvenance::Estimated, "Estimate"),
    ];
    for (provenance, label) in provenances {
        let report = ReportSnapshot {
            at: Timestamp::from_millis(0),
            entries: vec![ReportEntry {
                display_name: String::from("Memory in use"),
                status: ReportStatus::Available,
                value: Some(ReportValue {
                    amount: 1.0,
                    unit: Unit::Bytes,
                }),
                provenance: Some(provenance),
                limitation_codes: vec![],
            }],
        };
        assert!(render_preview(&report).contains(&format!("Source: {label}")));
    }
}

#[test]
fn controlled_limitation_message_is_rendered_without_a_new_text_field() {
    let report = ReportSnapshot {
        at: Timestamp::from_millis(0),
        entries: vec![ReportEntry {
            display_name: String::from("Memory in use"),
            status: ReportStatus::NotAvailable,
            value: None,
            provenance: None,
            limitation_codes: vec![LimitationCode::RequiresDriver],
        }],
    };
    let preview = render_preview(&report);
    assert!(preview.contains("Status: Not available"));
    assert!(preview.contains("Value: No value reported"));
    assert!(preview.contains(LimitationCode::RequiresDriver.message()));
}
