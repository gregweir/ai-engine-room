//! Isolated acquisition and sanitization primitives for a future bounded probe.
//!
//! This crate is intentionally not linked to AI Engine Room. It has no binary,
//! scheduler, persistence, logging, or product interface, and its native
//! adapters are never called by this implementation slice.

#[allow(dead_code)]
mod adapter;
pub mod classify;
pub mod model;
mod platform;
pub mod sanitize;

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::adapter::{AdapterError, FakeAdapter, SnapshotAdapter, SnapshotRequest};
    use crate::model::{
        AcquisitionCompleteness, AddressClass, AddressFamily, FixtureLabel, FixtureOutcome,
        ProcessAssociation, SampleIndex, SanitizedFixtureResult, TcpState,
    };
    use crate::sanitize::{sanitize, ArtificialObservation, SanitizeError};

    #[test]
    fn fake_adapter_is_a_pure_in_memory_seam() {
        let adapter = FakeAdapter::failed(AdapterError::PermissionLimited);
        let request = SnapshotRequest::artificial(SampleIndex::new(0).unwrap());
        assert_eq!(
            adapter.snapshot(&request),
            Err(AdapterError::PermissionLimited)
        );
    }

    #[test]
    fn retained_result_has_only_closed_sanitized_fields() {
        let raw = ArtificialObservation::tcp(
            FixtureLabel::LoopbackIpv4Long,
            SampleIndex::new(3).unwrap(),
            TcpState::Established,
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            ProcessAssociation::SupportedProcess,
        );
        let batch = sanitize(
            &[raw],
            AcquisitionCompleteness::Complete,
            &[FixtureLabel::LoopbackIpv4Long],
        )
        .unwrap();

        assert_eq!(batch.results().len(), 1);
        assert_eq!(
            batch.results()[0],
            SanitizedFixtureResult::new_for_test(
                FixtureLabel::LoopbackIpv4Long,
                AddressFamily::Ipv4,
                TcpState::Established,
                AddressClass::SameMachineLoopback,
                ProcessAssociation::SupportedProcess,
                FixtureOutcome::Observed,
                AcquisitionCompleteness::Complete,
                &[SampleIndex::new(3).unwrap()],
                1,
            )
        );
    }

    #[test]
    fn sanitizer_rejects_out_of_contract_inputs() {
        assert_eq!(SampleIndex::new(11), Err(crate::model::ModelError::SampleOutOfRange));
        assert_eq!(
            sanitize(
                &[ArtificialObservation::listener(FixtureLabel::LoopbackIpv4Long)],
                AcquisitionCompleteness::Complete,
                &[FixtureLabel::LoopbackIpv4Long],
            ),
            Err(SanitizeError::ListenerRejected)
        );
        assert_eq!(
            sanitize(
                &[ArtificialObservation::udp(FixtureLabel::LoopbackIpv4Long)],
                AcquisitionCompleteness::Complete,
                &[FixtureLabel::LoopbackIpv4Long],
            ),
            Err(SanitizeError::NonTcpRejected)
        );
        assert_eq!(
            sanitize(
                &[ArtificialObservation::unknown_fixture()],
                AcquisitionCompleteness::Complete,
                &[FixtureLabel::LoopbackIpv4Long],
            ),
            Err(SanitizeError::UnknownFixture)
        );
    }
}
