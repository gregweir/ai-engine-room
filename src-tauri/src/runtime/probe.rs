//! Transport-neutral probe abstraction and the pure detection mapping.
//!
//! `ProbeOutcome` is the controlled result of one local HTTP probe, free of any
//! `reqwest` type. The production transport (`ReqwestProbe`) and any test
//! transport (`FakeProbe`) both reduce to this enum, so the detection semantics
//! in [`interpret_probe`] — the deterministic heart of Milestone 1C — are
//! testable with fixtures and no socket, no `reqwest`, and no live Ollama.
//!
//! Detection mapping (design §3), a pure function:
//!
//! | Probe observation                       | `SourceAvailability` | version |
//! |-----------------------------------------|----------------------|---------|
//! | HTTP 200 + non-empty `version` string   | `Ready`              | `Some`  |
//! | HTTP 200 + malformed/missing/empty `version` | `Ready`          | `None`  |
//! | Any non-200 HTTP status                 | `Ready`              | `None`  |
//! | Connection refused                      | `NotDetected`        | `None`  |
//! | Timeout / unreachable                   | `Unreachable`        | `None`  |
//! | Other unexpected I/O                    | `Unreachable`        | `None`  |

use std::future::Future;

use aer_core::SourceAvailability;

use crate::runtime::status::{RuntimeStatus, RuntimeType, RuntimeVersion, VersionProvenance};

/// The controlled result of one local HTTP probe, free of transport types. A
/// returned `Response` reduces to `Responded { status, body }`; transport
/// failures reduce to the honest categories below. No raw error, URL, header,
/// or hostname crosses this boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// An HTTP response arrived. `status` is the HTTP status code; `body` is the
    /// raw response body text (parsed later, in the pure layer).
    Responded { status: u16, body: String },
    /// Nothing was listening at the local endpoint (connection refused).
    ConnectionRefused,
    /// The connect or read attempt exceeded its deadline.
    Timeout,
    /// Unexpected I/O not expected on localhost plain HTTP.
    Other,
}

/// A transport that performs one local runtime version probe and reduces the
/// result to a controlled [`ProbeOutcome`]. `Send + Sync` so an adapter can be
/// shared across the Tauri async boundary. Implementations keep all transport
/// types (e.g. `reqwest`) confined to themselves.
pub trait VersionProbe: Send + Sync {
    /// Perform one probe and return the controlled outcome.
    fn probe(&self) -> impl Future<Output = ProbeOutcome> + Send;
}

/// Map one probe outcome to a controlled runtime status. Pure: no I/O, no
/// transport types. `runtime_type` is carried through so the pure layer stays
/// runtime-neutral; the caller (the Ollama adapter) supplies it.
pub fn interpret_probe(outcome: ProbeOutcome, runtime_type: RuntimeType) -> RuntimeStatus {
    match outcome {
        ProbeOutcome::Responded { status, body } => {
            // A response arrived, so the endpoint was reached: the source is
            // `Ready`. The version is present only for a 200 with a valid,
            // non-empty `version` field; any other response is a controlled
            // "responded unexpectedly" state (`Ready` + `None`).
            let version = if status == 200 {
                extract_version(&body)
            } else {
                None
            };
            RuntimeStatus::new(
                runtime_type,
                SourceAvailability::Ready,
                version.map(|value| RuntimeVersion {
                    value,
                    provenance: VersionProvenance::RuntimeReported,
                }),
            )
        }
        ProbeOutcome::ConnectionRefused => {
            RuntimeStatus::new(runtime_type, SourceAvailability::NotDetected, None)
        }
        // Timeout and other transport failures are grouped: the connect-vs-read
        // distinction is not surfaced in 1C.
        ProbeOutcome::Timeout | ProbeOutcome::Other => {
            RuntimeStatus::new(runtime_type, SourceAvailability::Unreachable, None)
        }
    }
}

/// Extract a non-empty `version` string from an `/api/version` JSON body,
/// verbatim. Returns `None` for malformed JSON, a missing `version` field, a
/// non-string `version`, or an empty/whitespace-only value. The string is
/// preserved verbatim (no semver parsing, no normalisation, no trimming of
/// surrounding content beyond the empty/whitespace check).
fn extract_version(body: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    let version = value.get("version")?.as_str()?;
    if version.trim().is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}

#[cfg(test)]
mod tests {
    //! Deterministic coverage of `interpret_probe` over every design-§3 row,
    //! plus the honesty invariants. No socket, no `reqwest`, no live Ollama.

    use super::*;
    use crate::runtime::status::{RuntimeType, VersionProvenance};
    use aer_core::SourceAvailability;

    const OLLAMA: RuntimeType = RuntimeType::Ollama;

    fn responded(status: u16, body: &str) -> ProbeOutcome {
        ProbeOutcome::Responded {
            status,
            body: body.to_string(),
        }
    }

    // --- §3 rows: source availability + version presence -----------------------

    #[test]
    fn http_200_with_valid_version_is_ready_with_runtime_reported_version() {
        let s = interpret_probe(responded(200, r#"{"version":"0.1.32"}"#), OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert_eq!(s.runtime_type, OLLAMA);
        let v = s.version.expect("version present on valid 200 response");
        assert_eq!(v.value, "0.1.32");
        assert_eq!(v.provenance, VersionProvenance::RuntimeReported);
    }

    #[test]
    fn http_200_with_empty_version_string_is_ready_without_version() {
        let s = interpret_probe(responded(200, r#"{"version":""}"#), OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert!(s.version.is_none());
    }

    #[test]
    fn http_200_with_whitespace_only_version_is_ready_without_version() {
        let s = interpret_probe(responded(200, r#"{"version":"   "}"#), OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert!(s.version.is_none());
    }

    #[test]
    fn http_200_with_missing_version_field_is_ready_without_version() {
        let s = interpret_probe(responded(200, r#"{"build":"dev"}"#), OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert!(s.version.is_none());
    }

    #[test]
    fn http_200_with_non_string_version_is_ready_without_version() {
        let s = interpret_probe(responded(200, r#"{"version":42}"#), OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert!(s.version.is_none());
    }

    #[test]
    fn http_200_with_malformed_json_is_ready_without_version() {
        let s = interpret_probe(responded(200, "not-json-at-all"), OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert!(s.version.is_none());
    }

    #[test]
    fn non_200_http_status_is_ready_without_version() {
        let s = interpret_probe(responded(404, r#"{"version":"0.1.32"}"#), OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Ready);
        assert!(s.version.is_none());
    }

    #[test]
    fn connection_refused_is_not_detected_without_version() {
        let s = interpret_probe(ProbeOutcome::ConnectionRefused, OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::NotDetected);
        assert!(s.version.is_none());
    }

    #[test]
    fn timeout_is_unreachable_without_version() {
        let s = interpret_probe(ProbeOutcome::Timeout, OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Unreachable);
        assert!(s.version.is_none());
    }

    #[test]
    fn other_transport_failure_is_unreachable_without_version() {
        let s = interpret_probe(ProbeOutcome::Other, OLLAMA);
        assert_eq!(s.source_availability, SourceAvailability::Unreachable);
        assert!(s.version.is_none());
    }

    // --- Semantics / honesty invariants ----------------------------------------

    #[test]
    fn version_is_preserved_verbatim_without_parsing() {
        // An unusual but valid version string is kept exactly as reported.
        let s = interpret_probe(
            responded(200, r#"{"version":"0.1.32-rc1+deadbeef"}"#),
            OLLAMA,
        );
        let v = s.version.expect("version present");
        assert_eq!(v.value, "0.1.32-rc1+deadbeef");
    }

    #[test]
    fn version_with_surrounding_whitespace_is_preserved_verbatim() {
        // Non-empty after trim → present; the original string is kept verbatim.
        let s = interpret_probe(responded(200, r#"{"version":" 0.1.32 "}"#), OLLAMA);
        let v = s.version.expect("version present");
        assert_eq!(v.value, " 0.1.32 ");
    }

    #[test]
    fn never_produces_not_running() {
        // No ProbeOutcome maps to NotRunning: an HTTP probe cannot establish
        // installed-but-stopped, so 1C does not emit that state.
        for outcome in [
            responded(200, r#"{"version":"0.1.32"}"#),
            responded(200, r#"{"version":""}"#),
            responded(404, ""),
            ProbeOutcome::ConnectionRefused,
            ProbeOutcome::Timeout,
            ProbeOutcome::Other,
        ] {
            assert_ne!(
                interpret_probe(outcome, OLLAMA).source_availability,
                SourceAvailability::NotRunning,
                "NotRunning must never be produced from a probe outcome"
            );
        }
    }

    #[test]
    fn four_user_facing_states_are_distinguished() {
        // (source_availability, version.is_some()) fully distinguishes the four
        // intended user-facing states.
        let ready_version = interpret_probe(responded(200, r#"{"version":"0.1.32"}"#), OLLAMA);
        let ready_unexpected = interpret_probe(responded(200, r#"{"version":""}"#), OLLAMA);
        let not_detected = interpret_probe(ProbeOutcome::ConnectionRefused, OLLAMA);
        let unreachable = interpret_probe(ProbeOutcome::Timeout, OLLAMA);

        let key = |s: &RuntimeStatus| (s.source_availability.clone(), s.version.is_some());
        assert_eq!(key(&ready_version), (SourceAvailability::Ready, true));
        assert_eq!(key(&ready_unexpected), (SourceAvailability::Ready, false));
        assert_eq!(key(&not_detected), (SourceAvailability::NotDetected, false));
        assert_eq!(key(&unreachable), (SourceAvailability::Unreachable, false));

        // All four keys are mutually distinct.
        let keys = [
            key(&ready_version),
            key(&ready_unexpected),
            key(&not_detected),
            key(&unreachable),
        ];
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j], "states {i} and {j} collide");
            }
        }
    }
}
