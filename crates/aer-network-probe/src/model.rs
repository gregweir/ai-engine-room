use std::fmt;

pub const MAX_SAMPLE_INDEX: u8 = 10;
pub const MAX_RETAINED_RESULTS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AddressFamily {
    Ipv4,
    Ipv6,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TcpState {
    Established,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    TimeWait,
    Closed,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    DeleteTcb,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AcquisitionCompleteness {
    Complete,
    Partial,
    PermissionLimited,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AddressClass {
    SameMachineLoopback,
    LocalOrPrivateScope,
    ExternallyAddressed,
    SpecialOrUnresolved,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProcessAssociation {
    SupportedProcess,
    HelperDescendant,
    Unattributed,
    PermissionLimited,
    Stale,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FixtureOutcome {
    Observed,
    ExpectedButMissed,
    FixtureUnavailable,
    AssociationUnavailable,
    Invalidated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FixtureLabel {
    LoopbackIpv4Long,
    LoopbackIpv6Long,
    ExternalIpv4Long,
    ExternalIpv6Long,
    LoopbackIpv4Unattributed,
    LoopbackIpv6Unattributed,
    LoopbackIpv4Short,
    LoopbackIpv6Short,
}

impl FixtureLabel {
    pub const ALL: [Self; 8] = [
        Self::LoopbackIpv4Long,
        Self::LoopbackIpv6Long,
        Self::ExternalIpv4Long,
        Self::ExternalIpv6Long,
        Self::LoopbackIpv4Unattributed,
        Self::LoopbackIpv6Unattributed,
        Self::LoopbackIpv4Short,
        Self::LoopbackIpv6Short,
    ];

    pub const fn family(self) -> AddressFamily {
        match self {
            Self::LoopbackIpv4Long
            | Self::ExternalIpv4Long
            | Self::LoopbackIpv4Unattributed
            | Self::LoopbackIpv4Short => AddressFamily::Ipv4,
            Self::LoopbackIpv6Long
            | Self::ExternalIpv6Long
            | Self::LoopbackIpv6Unattributed
            | Self::LoopbackIpv6Short => AddressFamily::Ipv6,
        }
    }

    pub const fn expected_class(self) -> AddressClass {
        match self {
            Self::ExternalIpv4Long | Self::ExternalIpv6Long => AddressClass::ExternallyAddressed,
            _ => AddressClass::SameMachineLoopback,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelError {
    SampleOutOfRange,
    TooManyResults,
    InvalidCount,
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SampleIndex(u8);

impl SampleIndex {
    pub const fn new(value: u8) -> Result<Self, ModelError> {
        if value <= MAX_SAMPLE_INDEX {
            Ok(Self(value))
        } else {
            Err(ModelError::SampleOutOfRange)
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl fmt::Debug for SampleIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("SampleIndex").field(&self.0).finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanitizedFixtureResult {
    fixture: FixtureLabel,
    family: AddressFamily,
    state: TcpState,
    scope: AddressClass,
    association: ProcessAssociation,
    outcome: FixtureOutcome,
    acquisition: AcquisitionCompleteness,
    samples: [bool; 11],
    deduplicated_count: u16,
}

impl SanitizedFixtureResult {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create(
        fixture: FixtureLabel,
        family: AddressFamily,
        state: TcpState,
        scope: AddressClass,
        association: ProcessAssociation,
        outcome: FixtureOutcome,
        acquisition: AcquisitionCompleteness,
        sample_indexes: &[SampleIndex],
        deduplicated_count: u16,
    ) -> Result<Self, ModelError> {
        if (outcome == FixtureOutcome::Observed) != (deduplicated_count > 0) {
            return Err(ModelError::InvalidCount);
        }
        let mut samples = [false; 11];
        for index in sample_indexes {
            samples[usize::from(index.get())] = true;
        }
        Ok(Self {
            fixture,
            family,
            state,
            scope,
            association,
            outcome,
            acquisition,
            samples,
            deduplicated_count,
        })
    }

    pub fn fixture(&self) -> FixtureLabel { self.fixture }
    pub fn family(&self) -> AddressFamily { self.family }
    pub fn state(&self) -> TcpState { self.state }
    pub fn scope(&self) -> AddressClass { self.scope }
    pub fn association(&self) -> ProcessAssociation { self.association }
    pub fn outcome(&self) -> FixtureOutcome { self.outcome }
    pub fn acquisition(&self) -> AcquisitionCompleteness { self.acquisition }
    pub fn observed(&self, index: SampleIndex) -> bool { self.samples[usize::from(index.get())] }
    pub fn deduplicated_count(&self) -> u16 { self.deduplicated_count }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_for_test(
        fixture: FixtureLabel,
        family: AddressFamily,
        state: TcpState,
        scope: AddressClass,
        association: ProcessAssociation,
        outcome: FixtureOutcome,
        acquisition: AcquisitionCompleteness,
        sample_indexes: &[SampleIndex],
        deduplicated_count: u16,
    ) -> Self {
        Self::create(fixture, family, state, scope, association, outcome, acquisition, sample_indexes, deduplicated_count).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_completeness_outcome_and_association_states_fit_the_closed_model() {
        let completeness = [
            AcquisitionCompleteness::Complete,
            AcquisitionCompleteness::Partial,
            AcquisitionCompleteness::PermissionLimited,
            AcquisitionCompleteness::Failed,
        ];
        let outcomes = [
            FixtureOutcome::Observed,
            FixtureOutcome::ExpectedButMissed,
            FixtureOutcome::FixtureUnavailable,
            FixtureOutcome::AssociationUnavailable,
            FixtureOutcome::Invalidated,
        ];
        let associations = [
            ProcessAssociation::SupportedProcess,
            ProcessAssociation::HelperDescendant,
            ProcessAssociation::Unattributed,
            ProcessAssociation::PermissionLimited,
            ProcessAssociation::Stale,
        ];
        for acquisition in completeness {
            for outcome in outcomes {
                for association in associations {
                    let (samples, count) = if outcome == FixtureOutcome::Observed {
                        (vec![SampleIndex::new(0).unwrap()], 1)
                    } else {
                        (Vec::new(), 0)
                    };
                    assert!(SanitizedFixtureResult::create(
                        FixtureLabel::LoopbackIpv4Long,
                        AddressFamily::Ipv4,
                        TcpState::Established,
                        AddressClass::SameMachineLoopback,
                        association,
                        outcome,
                        acquisition,
                        &samples,
                        count,
                    )
                    .is_ok());
                }
            }
        }
    }

    #[test]
    fn observed_and_non_observed_counts_are_consistent() {
        assert_eq!(
            SanitizedFixtureResult::create(
                FixtureLabel::LoopbackIpv4Long,
                AddressFamily::Ipv4,
                TcpState::Established,
                AddressClass::SameMachineLoopback,
                ProcessAssociation::SupportedProcess,
                FixtureOutcome::Observed,
                AcquisitionCompleteness::Complete,
                &[],
                0,
            ),
            Err(ModelError::InvalidCount)
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanitizedBatch {
    results: Vec<SanitizedFixtureResult>,
}

impl SanitizedBatch {
    pub(crate) fn create(results: Vec<SanitizedFixtureResult>) -> Result<Self, ModelError> {
        if results.len() > MAX_RETAINED_RESULTS {
            return Err(ModelError::TooManyResults);
        }
        Ok(Self { results })
    }

    pub fn results(&self) -> &[SanitizedFixtureResult] {
        &self.results
    }
}
