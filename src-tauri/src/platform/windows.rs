//! Windows OS metric provider for AI Engine Room.
//!
//! Acquires exactly one platform-native observation: physical memory currently
//! available to the operating system, exposed as `os.ram.available`. The real
//! Win32 probe is cfg-gated to Windows; deterministic tests use the narrow probe
//! seam and never call the operating system API.

use aer_core::{
    AcquisitionError, LimitationCode, MetricDescriptor, MetricId, MetricResult, MetricSample,
    OsMetricsProvider, Provenance, SampleValue, SourceAvailability, SourceClass, Unit,
};
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use std::mem::size_of;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

const RAM_AVAILABLE_ID: &str = "os.ram.available";
const RAM_TOTAL_ID: &str = "os.ram.total";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WindowsMemoryStatus {
    pub(crate) available_physical_bytes: u64,
    pub(crate) total_physical_bytes: u64,
}

pub(crate) trait WindowsMemoryProbe: Send + Sync {
    fn memory_status(&self) -> Result<WindowsMemoryStatus, ()>;
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GlobalMemoryStatusProbe;

#[cfg(target_os = "windows")]
fn query_global_memory_status(status: &mut MEMORYSTATUSEX) -> bool {
    // SAFETY: `status` is the exact windows-sys `MEMORYSTATUSEX` binding,
    // is initialized with the required `dwLength` by the caller, and is
    // exclusively borrowed through a valid mutable pointer for the duration
    // of this call. The API writes only within that structure.
    unsafe { GlobalMemoryStatusEx(status) != 0 }
}

#[cfg(target_os = "windows")]
impl WindowsMemoryProbe for GlobalMemoryStatusProbe {
    fn memory_status(&self) -> Result<WindowsMemoryStatus, ()> {
        let mut status = MEMORYSTATUSEX {
            dwLength: size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };

        if !query_global_memory_status(&mut status) {
            return Err(());
        }

        Ok(WindowsMemoryStatus {
            available_physical_bytes: status.ullAvailPhys,
            total_physical_bytes: status.ullTotalPhys,
        })
    }
}

/// Windows provider for the single approved operating-system metric.
pub(crate) struct WindowsMetricsProvider<P> {
    probe: P,
    cached_status: Mutex<Option<Result<WindowsMemoryStatus, ()>>>,
}

impl<P> WindowsMetricsProvider<P> {
    pub(crate) fn from_probe(probe: P) -> Self {
        Self {
            probe,
            cached_status: Mutex::new(None),
        }
    }
}

#[cfg(target_os = "windows")]
impl WindowsMetricsProvider<GlobalMemoryStatusProbe> {
    pub(crate) fn new() -> Self {
        Self::from_probe(GlobalMemoryStatusProbe)
    }
}

impl<P: WindowsMemoryProbe> OsMetricsProvider for WindowsMetricsProvider<P> {
    fn source_availability(&self) -> SourceAvailability {
        SourceAvailability::Ready
    }

    fn list_metrics(&self) -> Vec<MetricDescriptor> {
        vec![
            MetricDescriptor {
                id: MetricId::new(RAM_AVAILABLE_ID),
                display_name: String::from("Available memory"),
                unit: Unit::Bytes,
                source_class: SourceClass::OperatingSystem,
            },
            MetricDescriptor {
                id: MetricId::new(RAM_TOTAL_ID),
                display_name: String::from("Total memory"),
                unit: Unit::Bytes,
                source_class: SourceClass::OperatingSystem,
            },
        ]
    }

    fn acquire(&self, id: &MetricId) -> MetricResult {
        if id.as_str() != RAM_AVAILABLE_ID && id.as_str() != RAM_TOTAL_ID {
            return MetricResult::unsupported(LimitationCode::NotImplementedOnThisPlatform);
        }

        let status = match self.cached_status.lock() {
            Ok(mut cache) => *cache.get_or_insert_with(|| self.probe.memory_status()),
            Err(_) => Err(()),
        };
        match status {
            Ok(status) => MetricResult::ok(MetricSample {
                value: SampleValue::Count(if id.as_str() == RAM_AVAILABLE_ID {
                    status.available_physical_bytes
                } else {
                    status.total_physical_bytes
                }),
                unit: Unit::Bytes,
                provenance: Provenance::OperatingSystemReported,
                timestamp: aer_core::Timestamp::now(),
                limitations: Vec::new(),
            }),
            Err(()) => MetricResult::failed(AcquisitionError::Other),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use aer_core::{explain, MetricAvailability, Outcome, Support};

    use super::*;

    #[derive(Clone)]
    struct FakeMemoryProbe {
        result: Result<WindowsMemoryStatus, ()>,
        calls: Arc<AtomicUsize>,
    }

    impl FakeMemoryProbe {
        fn new(result: Result<WindowsMemoryStatus, ()>) -> (Self, Arc<AtomicUsize>) {
            let calls = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    result,
                    calls: Arc::clone(&calls),
                },
                calls,
            )
        }
    }

    impl WindowsMemoryProbe for FakeMemoryProbe {
        fn memory_status(&self) -> Result<WindowsMemoryStatus, ()> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.result
        }
    }

    fn status(available: u64, total: u64) -> WindowsMemoryStatus {
        WindowsMemoryStatus {
            available_physical_bytes: available,
            total_physical_bytes: total,
        }
    }

    #[test]
    fn descriptor_is_the_existing_available_memory_metric() {
        let (probe, _) = FakeMemoryProbe::new(Ok(status(1, 2)));
        let provider = WindowsMetricsProvider::from_probe(probe);
        let descriptors = provider.list_metrics();

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].id.as_str(), RAM_AVAILABLE_ID);
        assert_eq!(descriptors[0].display_name, "Available memory");
        assert_eq!(descriptors[0].unit, Unit::Bytes);
        assert_eq!(descriptors[0].source_class, SourceClass::OperatingSystem);
        assert_eq!(descriptors[1].id.as_str(), RAM_TOTAL_ID);
        assert_eq!(descriptors[1].display_name, "Total memory");
    }

    #[test]
    fn exact_available_byte_value_is_reported_once() {
        let (probe, calls) = FakeMemoryProbe::new(Ok(status(12_345_678, 34_567_890)));
        let provider = WindowsMetricsProvider::from_probe(probe);

        let result = provider.acquire(&MetricId::new(RAM_AVAILABLE_ID));

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(result.source_availability, Some(SourceAvailability::Ready));
        assert_eq!(
            result.metric_availability,
            Some(MetricAvailability::Available)
        );
        match result.outcome {
            Outcome::Ok(sample) => {
                assert_eq!(sample.value, SampleValue::Count(12_345_678));
                assert_eq!(sample.unit, Unit::Bytes);
                assert_eq!(sample.provenance, Provenance::OperatingSystemReported);
                assert!(sample.limitations.is_empty());
            }
            other => panic!("expected an available Windows memory sample, got {other:?}"),
        }
    }

    #[test]
    fn exact_total_physical_byte_value_is_reported() {
        let (probe, _) = FakeMemoryProbe::new(Ok(status(12_345_678, 34_567_890)));
        let provider = WindowsMetricsProvider::from_probe(probe);
        let result = provider.acquire(&MetricId::new(RAM_TOTAL_ID));
        match result.outcome {
            Outcome::Ok(sample) => assert_eq!(sample.value, SampleValue::Count(34_567_890)),
            other => panic!("expected total physical memory, got {other:?}"),
        }
    }

    #[test]
    fn successful_zero_is_a_real_observation() {
        let (probe, calls) = FakeMemoryProbe::new(Ok(status(0, 0)));
        let provider = WindowsMetricsProvider::from_probe(probe);

        let result = provider.acquire(&MetricId::new(RAM_AVAILABLE_ID));

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        match result.outcome {
            Outcome::Ok(sample) => assert_eq!(sample.value, SampleValue::Count(0)),
            other => panic!("expected a zero-valued observation, got {other:?}"),
        }
    }

    #[test]
    fn probe_failure_is_controlled_and_never_fabricates_zero() {
        let (probe, calls) = FakeMemoryProbe::new(Err(()));
        let provider = WindowsMetricsProvider::from_probe(probe);

        let result = provider.acquire(&MetricId::new(RAM_AVAILABLE_ID));

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(result.is_legal());
        assert_eq!(result.outcome, Outcome::Failed(AcquisitionError::Other));
    }

    #[test]
    fn unknown_metric_is_controlled_and_does_not_probe() {
        let (probe, calls) = FakeMemoryProbe::new(Ok(status(9, 10)));
        let provider = WindowsMetricsProvider::from_probe(probe);

        let result = provider.acquire(&MetricId::new("os.ram.used"));

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            result.support,
            Support::Unsupported {
                limitation: LimitationCode::NotImplementedOnThisPlatform
            }
        );
        assert_eq!(result.outcome, Outcome::NoValue);
    }

    #[test]
    fn explanation_is_truthful_for_the_platform_native_observation() {
        let (probe, _) = FakeMemoryProbe::new(Ok(status(2048, 4096)));
        let provider = WindowsMetricsProvider::from_probe(probe);
        let descriptor = provider.list_metrics().remove(0);
        let result = provider.acquire(&descriptor.id);

        let explanation = explain(&descriptor, &result);

        assert!(explanation
            .interpretation
            .as_str()
            .contains("operating system"));
        assert!(explanation
            .deeper
            .expect("available memory has a qualification")
            .as_str()
            .contains("define available memory differently"));
        assert!(!explanation.interpretation.as_str().contains("Linux"));
    }
}
