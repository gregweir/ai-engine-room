//! Task 2 — value model, provenance categories, and sample.
//!
//! Semantics per design §4.1. Provenance attaches only to a value that exists;
//! `MetricSample` carries no per-sample evidence field.

use aer_core::{
    Formula, LimitationCode, MetricId, MetricRef, MetricSample, Model, Provenance, SampleValue,
    Timestamp, Unit,
};

#[test]
fn every_provenance_category_is_constructible() {
    let input = MetricRef {
        id: MetricId::new("os.ram.used"),
        provenance: Provenance::OperatingSystemReported,
    };
    let _ = Provenance::OperatingSystemReported;
    let _ = Provenance::DriverReported;
    let _ = Provenance::RuntimeReported;
    let _ = Provenance::ApplicationMeasured;
    let _ = Provenance::Calculated {
        inputs: vec![input.clone()],
        formula: Formula::new("used = total - free", "total - free"),
    };
    let _ = Provenance::Estimated {
        inputs: vec![input],
        assumptions: Model::new("uniform distribution across SMs"),
        limitations: vec![LimitationCode::Approximation, LimitationCode::Sampling],
    };
}

#[test]
fn calculated_preserves_inputs_and_formula() {
    let input = MetricRef {
        id: MetricId::new("os.ram.used"),
        provenance: Provenance::OperatingSystemReported,
    };
    let formula = Formula::new("used = total - free", "total - free");
    let provenance = Provenance::Calculated {
        inputs: vec![input.clone()],
        formula: formula.clone(),
    };
    match provenance {
        Provenance::Calculated { inputs, formula } => {
            assert_eq!(inputs.len(), 1);
            assert_eq!(inputs[0].id.as_str(), "os.ram.used");
            assert_eq!(inputs[0].provenance, Provenance::OperatingSystemReported);
            assert_eq!(formula.description(), "used = total - free");
            assert_eq!(formula.expression(), "total - free");
        }
        _ => panic!("expected Calculated"),
    }
}

#[test]
fn estimated_preserves_assumptions_and_limitations() {
    let input = MetricRef {
        id: MetricId::new("gpu.mem.used"),
        provenance: Provenance::DriverReported,
    };
    let assumptions = Model::new("uniform distribution across SMs");
    let limitations = vec![LimitationCode::Approximation, LimitationCode::Sampling];
    let provenance = Provenance::Estimated {
        inputs: vec![input],
        assumptions: assumptions.clone(),
        limitations: limitations.clone(),
    };
    match provenance {
        Provenance::Estimated {
            inputs,
            assumptions,
            limitations,
        } => {
            assert_eq!(inputs.len(), 1);
            assert_eq!(assumptions.description(), "uniform distribution across SMs");
            assert_eq!(limitations.len(), 2);
            assert_eq!(limitations[0], LimitationCode::Approximation);
        }
        _ => panic!("expected Estimated"),
    }
}

#[test]
fn metric_sample_has_exactly_five_fields_and_no_evidence() {
    // Struct-literal construction locks the field set: adding any further
    // field (e.g. a per-sample evidence field) would break compilation here.
    let sample = MetricSample {
        value: SampleValue::Count(42),
        unit: Unit::Count,
        provenance: Provenance::ApplicationMeasured,
        timestamp: Timestamp::from_millis(1_700_000_000_000),
        limitations: vec![],
    };
    assert!(matches!(sample.value, SampleValue::Count(42)));
    assert!(matches!(sample.unit, Unit::Count));
    assert!(matches!(sample.provenance, Provenance::ApplicationMeasured));
    assert_eq!(sample.timestamp.millis_since_epoch(), 1_700_000_000_000);
    assert!(sample.limitations.is_empty());
}

#[test]
fn limitation_code_returns_controlled_messages() {
    // message() returns a fixed, developer-authored string per code — never
    // raw system/provider text. Two lookups of the same code are identical.
    let code = LimitationCode::Approximation;
    let msg = code.message();
    assert!(!msg.is_empty());
    assert_eq!(msg, code.message());
    // A distinct code yields a distinct message.
    assert_ne!(
        LimitationCode::Sampling.message(),
        LimitationCode::Approximation.message()
    );
}

#[test]
fn timestamp_from_millis_is_deterministic() {
    let t = Timestamp::from_millis(1_234_567_890_000);
    assert_eq!(t.millis_since_epoch(), 1_234_567_890_000);
}
