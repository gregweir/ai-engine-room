//! Bounded live verification for Milestone 1B.
//!
//! This integration test exercises the real LinuxMetricsProvider against the
//! live `/proc/meminfo` on the development machine. It is gated to Linux and
//! is intentionally read-only: it does not benchmark, allocate memory, or
//! modify system state.
//!
//! This test is ignored by default because it depends on the live system
//! interface and is not deterministic across machines. Run it explicitly with
//! `cargo test --test linux_ram_live -- --ignored` when native graphical
//! verification is not available.

#[cfg(target_os = "linux")]
use std::fs;

/// Live environment verification for `os.ram.available`.
///
/// Run explicitly; it is ignored by default so ordinary `cargo test` stays
/// deterministic and does not depend on the current machine state.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "live environment verification: reads real /proc/meminfo and is not deterministic across machines"]
fn live_memavailable_is_plausible() {
    use aer_core::{LimitationCode, OsMetricsProvider, Outcome, Provenance, SampleValue, Unit};
    use aiengineroom_lib::platform::linux::LinuxMetricsProvider;

    let provider = LinuxMetricsProvider::new();
    assert_eq!(
        provider.source_availability(),
        aer_core::SourceAvailability::Ready,
        "/proc/meminfo should be readable on this Linux machine"
    );

    let result = provider.acquire(&aer_core::MetricId::new("os.ram.available"));
    assert!(
        result.is_legal(),
        "live acquisition result must be a legal domain state"
    );
    assert!(
        !result.is_failure(),
        "live acquisition must not fail on this machine"
    );

    let sample = match &result.outcome {
        Outcome::Ok(sample) => sample,
        other => panic!("expected Ok live sample, got {other:?}"),
    };

    let mem_total_kib = extract_memtotal_kib();
    let mem_available_kib = extract_memavailable_kib();

    assert!(
        mem_available_kib > 0,
        "live MemAvailable must be greater than zero"
    );
    assert!(
        mem_available_kib <= mem_total_kib,
        "live MemAvailable ({mem_available_kib} kB) must not exceed MemTotal ({mem_total_kib} kB)"
    );

    match &sample.value {
        SampleValue::Count(bytes) => {
            assert!(*bytes > 0, "provider value must be greater than zero bytes");
            assert_eq!(
                *bytes % 1024,
                0,
                "provider value must be a whole-kB multiple expressed as bytes"
            );
        }
        other => panic!("expected Count value, got {other:?}"),
    }

    assert_eq!(sample.unit, Unit::Bytes);
    assert_eq!(sample.provenance, Provenance::OperatingSystemReported);
    assert!(
        sample.limitations.contains(&LimitationCode::KernelEstimate),
        "live sample must carry the kernel-estimate limitation"
    );
}

#[cfg(target_os = "linux")]
fn extract_memavailable_kib() -> u64 {
    extract_meminfo_field_kib("MemAvailable:")
}

#[cfg(target_os = "linux")]
fn extract_memtotal_kib() -> u64 {
    extract_meminfo_field_kib("MemTotal:")
}

#[cfg(target_os = "linux")]
fn extract_meminfo_field_kib(prefix: &str) -> u64 {
    let content = fs::read_to_string("/proc/meminfo").expect("read /proc/meminfo");
    let line = content
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("{prefix} field present on this kernel"));
    let prefix_len = prefix.len();
    let mut parts = line[prefix_len..].split_whitespace();
    let value = parts.next().unwrap_or_else(|| panic!("{prefix} value"));
    let unit = parts.next().unwrap_or_else(|| panic!("{prefix} unit"));
    assert_eq!(unit, "kB", "{prefix} unit must be kB");
    value
        .parse()
        .unwrap_or_else(|_| panic!("{prefix} numeric value"))
}
