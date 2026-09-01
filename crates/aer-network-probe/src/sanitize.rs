use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr};

use crate::adapter::{
    NumericEndpoint, PlatformOwner, RawProtocol, RawRow, MAX_RAW_ROWS,
};
use crate::classify::classify;
use crate::model::{
    AcquisitionCompleteness, AddressClass, AddressFamily, FixtureLabel, FixtureOutcome,
    ModelError, ProcessAssociation, SampleIndex, SanitizedBatch, SanitizedFixtureResult, TcpState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SanitizeError {
    UnknownFixture,
    ListenerRejected,
    NonTcpRejected,
    FamilyMismatch,
    ClassificationMismatch,
    ResultLimit,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Key {
    fixture: FixtureLabel,
    family: AddressFamily,
    state: TcpState,
    scope: AddressClass,
    association: ProcessAssociation,
}

#[derive(Clone, Debug)]
pub struct ArtificialObservation {
    row: RawRow,
    unknown_fixture: bool,
}

impl ArtificialObservation {
    pub fn tcp(
        fixture: FixtureLabel,
        sample_index: SampleIndex,
        state: TcpState,
        remote_address: IpAddr,
        association: ProcessAssociation,
    ) -> Self {
        Self {
            row: RawRow {
                fixture: Some(fixture),
                sample_index,
                protocol: RawProtocol::Tcp,
                state,
                local: NumericEndpoint {
                    address: IpAddr::V4(Ipv4Addr::LOCALHOST),
                    port: 41_001,
                },
                remote: NumericEndpoint { address: remote_address, port: 41_002 },
                owner: PlatformOwner::Artificial,
                association,
            },
            unknown_fixture: false,
        }
    }

    pub fn listener(fixture: FixtureLabel) -> Self {
        Self::tcp(
            fixture,
            SampleIndex::new(0).expect("artificial index is bounded"),
            TcpState::Listen,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            ProcessAssociation::SupportedProcess,
        )
    }

    pub fn udp(fixture: FixtureLabel) -> Self {
        let mut observation = Self::tcp(
            fixture,
            SampleIndex::new(0).expect("artificial index is bounded"),
            TcpState::Established,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            ProcessAssociation::SupportedProcess,
        );
        observation.row.protocol = RawProtocol::Udp;
        observation
    }

    pub fn unknown_fixture() -> Self {
        let mut observation = Self::listener(FixtureLabel::LoopbackIpv4Long);
        observation.row.state = TcpState::Established;
        observation.unknown_fixture = true;
        observation
    }
}

pub fn sanitize(
    observations: &[ArtificialObservation],
    acquisition: AcquisitionCompleteness,
    allowed_fixtures: &[FixtureLabel],
) -> Result<SanitizedBatch, SanitizeError> {
    if observations.len() > MAX_RAW_ROWS {
        return Err(SanitizeError::ResultLimit);
    }
    let allowed: BTreeSet<_> = allowed_fixtures.iter().copied().collect();
    let mut groups: BTreeMap<Key, BTreeSet<SampleIndex>> = BTreeMap::new();
    let mut counts: BTreeMap<Key, u16> = BTreeMap::new();

    for artificial in observations {
        if artificial.unknown_fixture {
            return Err(SanitizeError::UnknownFixture);
        }
        let row = &artificial.row;
        let fixture = row.fixture.ok_or(SanitizeError::UnknownFixture)?;
        if !allowed.contains(&fixture) {
            return Err(SanitizeError::UnknownFixture);
        }
        if row.protocol != RawProtocol::Tcp {
            return Err(SanitizeError::NonTcpRejected);
        }
        if row.state == TcpState::Listen {
            return Err(SanitizeError::ListenerRejected);
        }
        let family = match row.remote.address {
            IpAddr::V4(_) => AddressFamily::Ipv4,
            IpAddr::V6(_) => AddressFamily::Ipv6,
        };
        if family != fixture.family() {
            return Err(SanitizeError::FamilyMismatch);
        }
        let scope = classify(row.remote.address);
        if scope != fixture.expected_class() {
            return Err(SanitizeError::ClassificationMismatch);
        }

        // Raw local/remote ports, local address, and platform ownership are
        // deliberately read only inside this projection and have no retained
        // field. This makes omission structural rather than text filtering.
        let _discarded_raw = (&row.local, row.remote.port, &row.owner);
        let key = Key { fixture, family, state: row.state, scope, association: row.association };
        groups.entry(key).or_default().insert(row.sample_index);
        let count = counts.entry(key).or_default();
        *count = count.checked_add(1).ok_or(SanitizeError::ResultLimit)?;
    }

    let mut results = Vec::with_capacity(groups.len());
    for (key, samples) in groups {
        let indexes: Vec<_> = samples.into_iter().collect();
        let count = counts
            .get(&key)
            .copied()
            .ok_or(SanitizeError::ResultLimit)?;
        results.push(
            SanitizedFixtureResult::create(
                key.fixture,
                key.family,
                key.state,
                key.scope,
                key.association,
                FixtureOutcome::Observed,
                acquisition,
                &indexes,
                count,
            )
            .map_err(map_model_error)?,
        );
    }
    SanitizedBatch::create(results).map_err(map_model_error)
}

fn map_model_error(_error: ModelError) -> SanitizeError {
    SanitizeError::ResultLimit
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn deduplicates_only_the_contract_key_and_sample_index() {
        let one = ArtificialObservation::tcp(
            FixtureLabel::LoopbackIpv4Long,
            SampleIndex::new(2).unwrap(),
            TcpState::Established,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            ProcessAssociation::SupportedProcess,
        );
        let batch = sanitize(
            &[one.clone(), one],
            AcquisitionCompleteness::Partial,
            &[FixtureLabel::LoopbackIpv4Long],
        ).unwrap();
        assert_eq!(batch.results().len(), 1);
        assert!(batch.results()[0].observed(SampleIndex::new(2).unwrap()));
        assert_eq!(batch.results()[0].deduplicated_count(), 2);
    }

    #[test]
    fn rejects_family_and_expected_class_inconsistency() {
        let wrong_family = ArtificialObservation::tcp(
            FixtureLabel::LoopbackIpv4Long,
            SampleIndex::new(0).unwrap(),
            TcpState::Established,
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            ProcessAssociation::SupportedProcess,
        );
        assert_eq!(sanitize(&[wrong_family], AcquisitionCompleteness::Complete, &[FixtureLabel::LoopbackIpv4Long]), Err(SanitizeError::FamilyMismatch));

        let wrong_scope = ArtificialObservation::tcp(
            FixtureLabel::ExternalIpv4Long,
            SampleIndex::new(0).unwrap(),
            TcpState::Established,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            ProcessAssociation::SupportedProcess,
        );
        assert_eq!(sanitize(&[wrong_scope], AcquisitionCompleteness::Complete, &[FixtureLabel::ExternalIpv4Long]), Err(SanitizeError::ClassificationMismatch));
    }
}
