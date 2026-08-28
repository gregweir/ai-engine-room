//! Mock/null implementations for 1A testing and browser-preview development.
//!
//! All fixture data here is deliberately artificial: fictional metric ids,
//! round-number values, and `demo-*` model names. No real machine/model,
//! path, address, username, hostname, or serial data appears. The mocks are
//! gated behind the `mock` Cargo feature so production/default builds cannot
//! depend on mock behaviour; tests and browser-preview development enable it
//! explicitly. No randomness — the fixtures are deterministic.

use crate::availability::{MetricAvailability, SourceAvailability};
use crate::identity::{MetricDescriptor, MetricId, SourceClass, Unit};
use crate::outcome::AcquisitionError;
use crate::provenance::{MetricRef, Provenance};
use crate::providers::{AiRuntimeAdapter, OsMetricsProvider, RuntimeModel};
use crate::result::MetricResult;
use crate::sample::MetricSample;
use crate::value::{Formula, LimitationCode, Model, SampleValue, Timestamp};

/// A fixed, deterministic table-backed mock OS provider. Construct with
/// [`MockOsProvider::default`]. The source is always `Ready`; `acquire` returns
/// a pre-baked `MetricResult` per metric id, covering every provenance category
/// and the unavailable/failed/unsupported states.
pub struct MockOsProvider {
    descriptors: Vec<MetricDescriptor>,
}

impl Default for MockOsProvider {
    fn default() -> Self {
        Self {
            descriptors: os_descriptors(),
        }
    }
}

impl OsMetricsProvider for MockOsProvider {
    fn source_availability(&self) -> SourceAvailability {
        SourceAvailability::Ready
    }

    fn list_metrics(&self) -> Vec<MetricDescriptor> {
        self.descriptors.clone()
    }

    fn acquire(&self, id: &MetricId) -> MetricResult {
        match id.as_str() {
            "os.ram.used" => MetricResult::ok(sample(
                SampleValue::Count(8_589_934_592),
                Unit::Bytes,
                Provenance::Calculated {
                    inputs: vec![
                        MetricRef {
                            id: MetricId::new("os.ram.total"),
                            provenance: Provenance::OperatingSystemReported,
                        },
                        MetricRef {
                            id: MetricId::new("os.ram.free"),
                            provenance: Provenance::OperatingSystemReported,
                        },
                    ],
                    formula: Formula::new("used = total - free", "total - free"),
                },
            )),
            "os.ram.total" => MetricResult::ok(sample(
                SampleValue::Count(17_179_869_184),
                Unit::Bytes,
                Provenance::OperatingSystemReported,
            )),
            "os.cpu.load" => MetricResult::ok(sample(
                SampleValue::Float(0.42),
                Unit::Percent,
                Provenance::ApplicationMeasured,
            )),
            "os.disk.used" => MetricResult::ok(sample(
                SampleValue::Count(500_000_000_000),
                Unit::Bytes,
                Provenance::Estimated {
                    inputs: vec![MetricRef {
                        id: MetricId::new("os.disk.read"),
                        provenance: Provenance::OperatingSystemReported,
                    }],
                    assumptions: Model::new("uniform growth from last sample"),
                    limitations: vec![LimitationCode::Approximation],
                },
            )),
            // Clearly-mock GPU fixture: a driver-reported value with no real
            // driver behind it.
            "gpu.mem.used" => MetricResult::ok(sample(
                SampleValue::Count(2_147_483_648),
                Unit::Bytes,
                Provenance::DriverReported,
            )),
            // Mock-only fixture exercising the RuntimeReported category from a
            // deterministic table; not real OS-provider behaviour.
            "rt.demo.reported" => MetricResult::ok(sample(
                SampleValue::Count(4_096),
                Unit::Count,
                Provenance::RuntimeReported,
            )),
            "os.temp" => MetricResult::metric_not_available(MetricAvailability::NotExposed),
            "os.swap.transient" => {
                MetricResult::metric_not_available(MetricAvailability::TransientlyUnavailable)
            }
            "os.net.errors" => MetricResult::failed(AcquisitionError::Timeout),
            "gpu.fan.rpm" => MetricResult::unsupported(LimitationCode::RequiresDriver),
            // Unknown id on a ready source: the metric is not exposed.
            _ => MetricResult::metric_not_available(MetricAvailability::NotExposed),
        }
    }
}

/// A mock AI runtime adapter. Built in one of two deterministic modes:
/// [`MockRuntimeAdapter::not_detected`] (runtime absent) or
/// [`MockRuntimeAdapter::ready`] (runtime present, with fictional demo models).
pub struct MockRuntimeAdapter {
    source: SourceAvailability,
    models: Vec<RuntimeModel>,
    loaded: Vec<RuntimeModel>,
}

impl MockRuntimeAdapter {
    /// Runtime not detected: no models, every metric is `NoValue`.
    pub fn not_detected() -> Self {
        Self {
            source: SourceAvailability::NotDetected,
            models: Vec::new(),
            loaded: Vec::new(),
        }
    }

    /// Runtime ready, with clearly-artificial `demo-*` models.
    pub fn ready() -> Self {
        let models = vec![
            RuntimeModel {
                id: String::from("demo-llama"),
                display_name: String::from("Demo Llama (mock)"),
            },
            RuntimeModel {
                id: String::from("demo-mistral"),
                display_name: String::from("Demo Mistral (mock)"),
            },
            RuntimeModel {
                id: String::from("demo-phi"),
                display_name: String::from("Demo Phi (mock)"),
            },
        ];
        let loaded = vec![RuntimeModel {
            id: String::from("demo-llama"),
            display_name: String::from("Demo Llama (mock)"),
        }];
        Self {
            source: SourceAvailability::Ready,
            models,
            loaded,
        }
    }
}

impl AiRuntimeAdapter for MockRuntimeAdapter {
    fn detect(&self) -> SourceAvailability {
        self.source.clone()
    }

    fn list_models(&self) -> Vec<RuntimeModel> {
        self.models.clone()
    }

    fn loaded_models(&self) -> Vec<RuntimeModel> {
        self.loaded.clone()
    }

    fn runtime_metrics(&self, id: &MetricId) -> MetricResult {
        match self.source {
            SourceAvailability::NotDetected
            | SourceAvailability::NotRunning
            | SourceAvailability::Unreachable => {
                MetricResult::source_not_ready(self.source.clone())
            }
            SourceAvailability::Ready => match id.as_str() {
                "rt.context.tokens" => MetricResult::ok(sample(
                    SampleValue::Count(4_096),
                    Unit::Count,
                    Provenance::RuntimeReported,
                )),
                _ => MetricResult::metric_not_available(MetricAvailability::NotExposed),
            },
        }
    }
}

fn sample(value: SampleValue, unit: Unit, provenance: Provenance) -> MetricSample {
    MetricSample {
        value,
        unit,
        provenance,
        timestamp: Timestamp::from_millis(0),
        limitations: Vec::new(),
    }
}

fn os_descriptors() -> Vec<MetricDescriptor> {
    fn desc(id: &str, name: &str, unit: Unit) -> MetricDescriptor {
        MetricDescriptor {
            id: MetricId::new(id),
            display_name: String::from(name),
            unit,
            source_class: SourceClass::OperatingSystem,
        }
    }
    vec![
        desc("os.ram.used", "Memory in use", Unit::Bytes),
        desc("os.ram.total", "Total memory", Unit::Bytes),
        desc("os.cpu.load", "CPU load", Unit::Percent),
        desc("os.disk.used", "Disk in use", Unit::Bytes),
        desc("gpu.mem.used", "GPU memory in use", Unit::Bytes),
        desc("rt.demo.reported", "Demo runtime value", Unit::Count),
        desc("os.temp", "Thermal sensor", Unit::Count),
        desc("os.swap.transient", "Swap (transient)", Unit::Bytes),
        desc("os.net.errors", "Network errors", Unit::Count),
        desc("gpu.fan.rpm", "GPU fan speed", Unit::Count),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outcome::Outcome;
    use crate::support::Support;

    #[test]
    fn os_mock_source_is_ready_and_lists_metrics() {
        let p = MockOsProvider::default();
        assert_eq!(p.source_availability(), SourceAvailability::Ready);
        assert!(!p.list_metrics().is_empty());
    }

    #[test]
    fn os_mock_covers_states_and_yields_calculated_for_ram_used() {
        let p = MockOsProvider::default();
        // Ok + Calculated for os.ram.used (the plan's required assertion).
        let ram = p.acquire(&MetricId::new("os.ram.used"));
        assert!(ram.is_legal());
        assert!(!ram.is_failure());
        match &ram.outcome {
            Outcome::Ok(s) => assert!(matches!(s.provenance, Provenance::Calculated { .. })),
            other => panic!("expected Ok for os.ram.used, got {other:?}"),
        }

        // Other Ok provenance categories.
        assert!(matches!(
            p.acquire(&MetricId::new("os.ram.total")).outcome,
            Outcome::Ok(MetricSample {
                provenance: Provenance::OperatingSystemReported,
                ..
            })
        ));
        assert!(matches!(
            p.acquire(&MetricId::new("os.cpu.load")).outcome,
            Outcome::Ok(MetricSample {
                provenance: Provenance::ApplicationMeasured,
                ..
            })
        ));
        assert!(matches!(
            p.acquire(&MetricId::new("os.disk.used")).outcome,
            Outcome::Ok(MetricSample {
                provenance: Provenance::Estimated { .. },
                ..
            })
        ));
        // Clearly-mock DriverReported fixture.
        assert!(matches!(
            p.acquire(&MetricId::new("gpu.mem.used")).outcome,
            Outcome::Ok(MetricSample {
                provenance: Provenance::DriverReported,
                ..
            })
        ));
        assert!(matches!(
            p.acquire(&MetricId::new("rt.demo.reported")).outcome,
            Outcome::Ok(MetricSample {
                provenance: Provenance::RuntimeReported,
                ..
            })
        ));

        // NotExposed and TransientlyUnavailable are NoValue and not failures.
        let not_exposed = p.acquire(&MetricId::new("os.temp"));
        assert!(matches!(not_exposed.outcome, Outcome::NoValue));
        assert_eq!(
            not_exposed.metric_availability,
            Some(MetricAvailability::NotExposed)
        );
        assert!(!not_exposed.is_failure());

        let transient = p.acquire(&MetricId::new("os.swap.transient"));
        assert!(matches!(transient.outcome, Outcome::NoValue));
        assert_eq!(
            transient.metric_availability,
            Some(MetricAvailability::TransientlyUnavailable)
        );
        assert!(!transient.is_failure());

        // Failed timeout is a failure.
        let failed = p.acquire(&MetricId::new("os.net.errors"));
        assert!(failed.is_failure());
        assert!(matches!(
            failed.outcome,
            Outcome::Failed(AcquisitionError::Timeout)
        ));

        // Unsupported capability is not a failure.
        let unsupported = p.acquire(&MetricId::new("gpu.fan.rpm"));
        assert!(matches!(unsupported.support, Support::Unsupported { .. }));
        assert!(!unsupported.is_failure());
    }

    #[test]
    fn runtime_not_detected_yields_no_value_with_unevaluated_metric() {
        let rt = MockRuntimeAdapter::not_detected();
        assert_eq!(rt.detect(), SourceAvailability::NotDetected);
        assert!(rt.list_models().is_empty());
        assert!(rt.loaded_models().is_empty());
        let r = rt.runtime_metrics(&MetricId::new("rt.context.tokens"));
        assert!(matches!(r.outcome, Outcome::NoValue));
        assert!(r.metric_availability.is_none());
        assert_eq!(r.source_availability, Some(SourceAvailability::NotDetected));
        assert!(!r.is_failure());
    }

    #[test]
    fn runtime_ready_yields_ok_with_runtime_reported() {
        let rt = MockRuntimeAdapter::ready();
        assert_eq!(rt.detect(), SourceAvailability::Ready);
        assert!(!rt.list_models().is_empty());
        assert!(!rt.loaded_models().is_empty());
        let r = rt.runtime_metrics(&MetricId::new("rt.context.tokens"));
        assert!(r.is_legal());
        assert!(!r.is_failure());
        match &r.outcome {
            Outcome::Ok(s) => assert!(matches!(s.provenance, Provenance::RuntimeReported)),
            other => panic!("expected Ok, got {other:?}"),
        }
        // An unknown runtime metric is normally unavailable, not a failure.
        let unknown = rt.runtime_metrics(&MetricId::new("rt.unknown.metric"));
        assert!(matches!(unknown.outcome, Outcome::NoValue));
        assert!(!unknown.is_failure());
    }

    #[test]
    fn mock_fixtures_contain_no_private_data() {
        let os = MockOsProvider::default();
        for d in os.list_metrics() {
            let name = d.display_name.as_str();
            assert!(
                !name.contains('/'),
                "display name must not look like a path: {name}"
            );
            assert!(
                !name.contains('@'),
                "display name must not look like an email: {name}"
            );
            assert!(
                !name.contains("192.168."),
                "display name must not contain a private IP: {name}"
            );
        }
        let rt = MockRuntimeAdapter::ready();
        for m in rt.list_models().iter().chain(rt.loaded_models().iter()) {
            // Mock model names are clearly artificial (demo-* prefix).
            assert!(
                m.id.starts_with("demo-"),
                "mock model id should be clearly artificial: {}",
                m.id
            );
        }
    }
}
