use crate::model::{
    AcquisitionCompleteness, AddressClass, AddressFamily, FixtureLabel, FixtureOutcome,
    ProcessAssociation, SampleIndex, SanitizedBatch, TcpState,
};

pub(crate) const MAX_RECORD_BYTES: usize = 32_768;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateRecord {
    platform: &'static str,
    actual_samples: u8,
    max_actual_interval_ms: u16,
    actual_window_ms: u16,
    acquisition: AcquisitionCompleteness,
    expected_missed: u8,
    results: Vec<ResultRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResultRecord {
    fixture: FixtureLabel,
    family: AddressFamily,
    state: TcpState,
    scope: AddressClass,
    association: ProcessAssociation,
    outcome: FixtureOutcome,
    samples: [bool; 11],
    deduplicated_count: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecordError {
    InvalidGrammar,
    InvalidValue,
    TooLarge,
}

impl CandidateRecord {
    pub(crate) fn from_batch(
        platform: &'static str,
        actual_samples: u8,
        max_actual_interval_ms: u16,
        actual_window_ms: u16,
        acquisition: AcquisitionCompleteness,
        batch: &SanitizedBatch,
    ) -> Result<Self, RecordError> {
        let mut results = Vec::with_capacity(batch.results().len());
        let mut seen = [false; 2];
        for result in batch.results() {
            let fixture_index = loopback_fixture_index(result.fixture())?;
            seen[fixture_index] = true;
            let mut samples = [false; 11];
            for index in 0..=10 {
                let sample = SampleIndex::new(index).map_err(|_| RecordError::InvalidValue)?;
                samples[usize::from(index)] = result.observed(sample);
            }
            results.push(ResultRecord {
                fixture: result.fixture(),
                family: result.family(),
                state: result.state(),
                scope: result.scope(),
                association: result.association(),
                outcome: result.outcome(),
                samples,
                deduplicated_count: result.deduplicated_count(),
            });
        }
        let expected_missed = seen.iter().filter(|value| !**value).count() as u8;
        let record = Self {
            platform,
            actual_samples,
            max_actual_interval_ms,
            actual_window_ms,
            acquisition,
            expected_missed,
            results,
        };
        record.validate()?;
        Ok(record)
    }

    pub(crate) fn encode_candidate(&self) -> Result<String, RecordError> {
        let mut output = format!(
            "candidate|platform={}|planned_samples=11|actual_samples={}|planned_interval_ms=500|max_actual_interval_ms={}|planned_window_ms=5000|actual_window_ms={}|outer_timeout_s=15|acquisition={}|expected_missed={}",
            self.platform,
            self.actual_samples,
            self.max_actual_interval_ms,
            self.actual_window_ms,
            acquisition_token(self.acquisition),
            self.expected_missed,
        );
        for result in &self.results {
            output.push_str("|result=");
            output.push_str(fixture_token(result.fixture));
            output.push(',');
            output.push_str(family_token(result.family));
            output.push(',');
            output.push_str(state_token(result.state));
            output.push(',');
            output.push_str(scope_token(result.scope));
            output.push(',');
            output.push_str(association_token(result.association));
            output.push(',');
            output.push_str(outcome_token(result.outcome));
            output.push(',');
            for observed in result.samples {
                output.push(if observed { '1' } else { '0' });
            }
            output.push(',');
            output.push_str(&result.deduplicated_count.to_string());
        }
        if output.len() > MAX_RECORD_BYTES {
            return Err(RecordError::TooLarge);
        }
        Ok(output)
    }

    pub(crate) fn parse_candidate(input: &str) -> Result<Self, RecordError> {
        if input.is_empty()
            || input.len() > MAX_RECORD_BYTES
            || input.contains(['\r', '\n'])
            || input.contains(['/', '\\', ':'])
        {
            return Err(RecordError::InvalidGrammar);
        }
        let mut fields = input.split('|');
        if fields.next() != Some("candidate") {
            return Err(RecordError::InvalidGrammar);
        }
        let platform = parse_platform(exact_value(fields.next(), "platform=")?)?;
        if fields.next() != Some("planned_samples=11") {
            return Err(RecordError::InvalidGrammar);
        }
        let actual_samples = parse_bounded_u8(exact_value(fields.next(), "actual_samples=")?, 11)?;
        if fields.next() != Some("planned_interval_ms=500") {
            return Err(RecordError::InvalidGrammar);
        }
        let max_actual_interval_ms = parse_bounded_u16(
            exact_value(fields.next(), "max_actual_interval_ms=")?,
            5_000,
        )?;
        if fields.next() != Some("planned_window_ms=5000") {
            return Err(RecordError::InvalidGrammar);
        }
        let actual_window_ms =
            parse_bounded_u16(exact_value(fields.next(), "actual_window_ms=")?, 5_000)?;
        if fields.next() != Some("outer_timeout_s=15") {
            return Err(RecordError::InvalidGrammar);
        }
        let acquisition = parse_acquisition(exact_value(fields.next(), "acquisition=")?)?;
        let expected_missed = parse_bounded_u8(exact_value(fields.next(), "expected_missed=")?, 2)?;
        let mut results = Vec::new();
        for field in fields {
            let encoded = exact_value(Some(field), "result=")?;
            results.push(parse_result(encoded)?);
            if results.len() > 256 {
                return Err(RecordError::TooLarge);
            }
        }
        let record = Self {
            platform,
            actual_samples,
            max_actual_interval_ms,
            actual_window_ms,
            acquisition,
            expected_missed,
            results,
        };
        record.validate()?;
        if record.encode_candidate()? != input {
            return Err(RecordError::InvalidGrammar);
        }
        Ok(record)
    }

    pub(crate) fn encode_final(&self) -> Result<String, RecordError> {
        let candidate = self.encode_candidate()?;
        let body = candidate
            .strip_prefix("candidate")
            .ok_or(RecordError::InvalidGrammar)?;
        let output = format!(
            "record{body}|fixture_termination=clean|sampler_termination=clean|supervisor_termination=pending_exit|non_claims=bounded_loopback_observation_only"
        );
        if output.len() > MAX_RECORD_BYTES {
            return Err(RecordError::TooLarge);
        }
        Ok(output)
    }

    fn validate(&self) -> Result<(), RecordError> {
        if !approved_platform(self.platform)
            || self.actual_samples > 11
            || self.max_actual_interval_ms > 5_000
            || self.actual_window_ms > 5_000
            || (self.actual_samples <= 1
                && (self.max_actual_interval_ms != 0 || self.actual_window_ms != 0))
            || ((self.acquisition == AcquisitionCompleteness::Complete)
                != (self.actual_samples == 11))
        {
            return Err(RecordError::InvalidValue);
        }
        let mut seen = [false; 2];
        for result in &self.results {
            let fixture_index = loopback_fixture_index(result.fixture)?;
            seen[fixture_index] = true;
            if result.family != result.fixture.family()
                || result.scope != AddressClass::SameMachineLoopback
                || result.state == TcpState::Listen
                || result.outcome != FixtureOutcome::Observed
                || result.deduplicated_count == 0
                || result
                    .samples
                    .iter()
                    .enumerate()
                    .any(|(index, observed)| *observed && index >= usize::from(self.actual_samples))
            {
                return Err(RecordError::InvalidValue);
            }
        }
        let missed = seen.iter().filter(|value| !**value).count() as u8;
        if missed != self.expected_missed {
            return Err(RecordError::InvalidValue);
        }
        Ok(())
    }
}

fn parse_result(input: &str) -> Result<ResultRecord, RecordError> {
    let parts: Vec<_> = input.split(',').collect();
    if parts.len() != 8 {
        return Err(RecordError::InvalidGrammar);
    }
    let fixture = parse_fixture(parts[0])?;
    let family = parse_family(parts[1])?;
    if fixture.family() != family {
        return Err(RecordError::InvalidValue);
    }
    let state = parse_state(parts[2])?;
    let scope = parse_scope(parts[3])?;
    let association = parse_association(parts[4])?;
    let outcome = parse_outcome(parts[5])?;
    if parts[6].len() != 11 || !parts[6].bytes().all(|byte| matches!(byte, b'0' | b'1')) {
        return Err(RecordError::InvalidValue);
    }
    let mut samples = [false; 11];
    for (index, byte) in parts[6].bytes().enumerate() {
        samples[index] = byte == b'1';
    }
    let deduplicated_count = parts[7]
        .parse::<u16>()
        .map_err(|_| RecordError::InvalidValue)?;
    if (outcome == FixtureOutcome::Observed) != (deduplicated_count > 0) {
        return Err(RecordError::InvalidValue);
    }
    Ok(ResultRecord {
        fixture,
        family,
        state,
        scope,
        association,
        outcome,
        samples,
        deduplicated_count,
    })
}

fn exact_value<'a>(field: Option<&'a str>, prefix: &str) -> Result<&'a str, RecordError> {
    field
        .and_then(|value| value.strip_prefix(prefix))
        .filter(|value| !value.is_empty())
        .ok_or(RecordError::InvalidGrammar)
}

fn parse_bounded_u8(value: &str, maximum: u8) -> Result<u8, RecordError> {
    let parsed = value.parse::<u8>().map_err(|_| RecordError::InvalidValue)?;
    if parsed > maximum || parsed.to_string() != value {
        return Err(RecordError::InvalidValue);
    }
    Ok(parsed)
}

fn parse_bounded_u16(value: &str, maximum: u16) -> Result<u16, RecordError> {
    let parsed = value
        .parse::<u16>()
        .map_err(|_| RecordError::InvalidValue)?;
    if parsed > maximum || parsed.to_string() != value {
        return Err(RecordError::InvalidValue);
    }
    Ok(parsed)
}

fn approved_platform(value: &str) -> bool {
    matches!(value, "ubuntu_24_04_x86_64" | "windows_11_25h2_26200_x64")
}

fn parse_platform(value: &str) -> Result<&'static str, RecordError> {
    match value {
        "ubuntu_24_04_x86_64" => Ok("ubuntu_24_04_x86_64"),
        "windows_11_25h2_26200_x64" => Ok("windows_11_25h2_26200_x64"),
        _ => Err(RecordError::InvalidValue),
    }
}

fn loopback_fixture_index(value: FixtureLabel) -> Result<usize, RecordError> {
    match value {
        FixtureLabel::LoopbackIpv4Long => Ok(0),
        FixtureLabel::LoopbackIpv6Long => Ok(1),
        _ => Err(RecordError::InvalidValue),
    }
}

fn fixture_token(value: FixtureLabel) -> &'static str {
    match value {
        FixtureLabel::LoopbackIpv4Long => "loopback_ipv4_long",
        FixtureLabel::LoopbackIpv6Long => "loopback_ipv6_long",
        _ => "invalid_fixture",
    }
}

fn parse_fixture(value: &str) -> Result<FixtureLabel, RecordError> {
    match value {
        "loopback_ipv4_long" => Ok(FixtureLabel::LoopbackIpv4Long),
        "loopback_ipv6_long" => Ok(FixtureLabel::LoopbackIpv6Long),
        _ => Err(RecordError::InvalidValue),
    }
}

macro_rules! closed_tokens {
    ($encode:ident, $decode:ident, $type:ty, { $($variant:path => $token:literal),+ $(,)? }) => {
        fn $encode(value: $type) -> &'static str {
            match value { $($variant => $token),+ }
        }
        fn $decode(value: &str) -> Result<$type, RecordError> {
            match value { $($token => Ok($variant)),+, _ => Err(RecordError::InvalidValue) }
        }
    };
}

closed_tokens!(family_token, parse_family, AddressFamily, {
    AddressFamily::Ipv4 => "ipv4",
    AddressFamily::Ipv6 => "ipv6",
});
closed_tokens!(state_token, parse_state, TcpState, {
    TcpState::Established => "established",
    TcpState::SynSent => "syn_sent",
    TcpState::SynReceived => "syn_received",
    TcpState::FinWait1 => "fin_wait_1",
    TcpState::FinWait2 => "fin_wait_2",
    TcpState::TimeWait => "time_wait",
    TcpState::Closed => "closed",
    TcpState::CloseWait => "close_wait",
    TcpState::LastAck => "last_ack",
    TcpState::Listen => "listen",
    TcpState::Closing => "closing",
    TcpState::DeleteTcb => "delete_tcb",
    TcpState::Unknown => "unknown",
});
closed_tokens!(scope_token, parse_scope, AddressClass, {
    AddressClass::SameMachineLoopback => "same_machine_loopback",
    AddressClass::LocalOrPrivateScope => "local_or_private_scope",
    AddressClass::ExternallyAddressed => "externally_addressed",
    AddressClass::SpecialOrUnresolved => "special_or_unresolved",
});
closed_tokens!(association_token, parse_association, ProcessAssociation, {
    ProcessAssociation::SupportedProcess => "supported_process",
    ProcessAssociation::HelperDescendant => "helper_descendant",
    ProcessAssociation::Unattributed => "unattributed",
    ProcessAssociation::PermissionLimited => "permission_limited",
    ProcessAssociation::Stale => "stale",
});
closed_tokens!(outcome_token, parse_outcome, FixtureOutcome, {
    FixtureOutcome::Observed => "observed",
    FixtureOutcome::ExpectedButMissed => "expected_but_missed",
    FixtureOutcome::FixtureUnavailable => "fixture_unavailable",
    FixtureOutcome::AssociationUnavailable => "association_unavailable",
    FixtureOutcome::Invalidated => "invalidated",
});
closed_tokens!(acquisition_token, parse_acquisition, AcquisitionCompleteness, {
    AcquisitionCompleteness::Complete => "complete",
    AcquisitionCompleteness::Partial => "partial",
    AcquisitionCompleteness::PermissionLimited => "permission_limited",
    AcquisitionCompleteness::Failed => "failed",
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sanitize::{sanitize, ArtificialObservation};
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn candidate_round_trip_uses_only_the_closed_grammar() {
        let batch = sanitize(
            &[ArtificialObservation::tcp(
                FixtureLabel::LoopbackIpv4Long,
                SampleIndex::new(0).unwrap(),
                TcpState::Established,
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                ProcessAssociation::SupportedProcess,
            )],
            AcquisitionCompleteness::Partial,
            &[
                FixtureLabel::LoopbackIpv4Long,
                FixtureLabel::LoopbackIpv6Long,
            ],
        )
        .unwrap();
        let record = CandidateRecord::from_batch(
            "ubuntu_24_04_x86_64",
            1,
            0,
            0,
            AcquisitionCompleteness::Partial,
            &batch,
        )
        .unwrap();
        let encoded = record.encode_candidate().unwrap();
        assert_eq!(CandidateRecord::parse_candidate(&encoded), Ok(record));
        assert!(!encoded.contains(['/', '\\', ':', '\r', '\n']));
    }

    #[test]
    fn parser_rejects_reordered_unknown_and_path_like_fields() {
        for invalid in [
            "candidate|planned_samples=11|platform=ubuntu_24_04_x86_64",
            "candidate|platform=other|planned_samples=11|actual_samples=0|planned_interval_ms=500|max_actual_interval_ms=0|planned_window_ms=5000|actual_window_ms=0|outer_timeout_s=15|acquisition=failed|expected_missed=2",
            "candidate|platform=ubuntu_24_04_x86_64/path|planned_samples=11|actual_samples=0|planned_interval_ms=500|max_actual_interval_ms=0|planned_window_ms=5000|actual_window_ms=0|outer_timeout_s=15|acquisition=failed|expected_missed=2",
        ] {
            assert!(CandidateRecord::parse_candidate(invalid).is_err());
        }
    }
}
