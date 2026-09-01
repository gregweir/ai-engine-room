use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

use crate::adapter::{AdapterError, AllowListedProcess, RawRow, SnapshotAdapter, SnapshotRequest};
use crate::model::{AcquisitionCompleteness, FixtureLabel, SampleIndex};
use crate::sanitize::sanitize_raw;

use super::record::CandidateRecord;

const IDENTITY_LIMIT: u64 = 4_096;
const SAMPLE_COUNT: u8 = 11;
const INTERVAL: Duration = Duration::from_millis(500);
const WINDOW: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SamplerError {
    Identity,
    Io,
    Record,
    Sanitization,
    ScheduleBound,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FixtureIdentity {
    fixture: FixtureLabel,
    pid: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Identities([FixtureIdentity; 2]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Schedule;

impl Schedule {
    fn offset(index: u8) -> Result<Duration, SamplerError> {
        if index >= SAMPLE_COUNT {
            return Err(SamplerError::ScheduleBound);
        }
        Ok(INTERVAL * u32::from(index))
    }
}

pub(crate) fn run() -> Result<(), SamplerError> {
    let identities = read_identities()?;
    let processes = platform_processes(&identities)?;
    let started = Instant::now();
    let mut rows = Vec::<RawRow>::new();
    let mut completions = Vec::with_capacity(usize::from(SAMPLE_COUNT));
    let mut acquisition = AcquisitionCompleteness::Complete;

    for raw_index in 0..SAMPLE_COUNT {
        let offset = Schedule::offset(raw_index)?;
        if offset > WINDOW {
            return Err(SamplerError::ScheduleBound);
        }
        let target = started
            .checked_add(offset)
            .ok_or(SamplerError::ScheduleBound)?;
        let now = Instant::now();
        if now < target {
            std::thread::sleep(target.duration_since(now));
        }
        let index = SampleIndex::new(raw_index).map_err(|_| SamplerError::ScheduleBound)?;
        let request =
            SnapshotRequest::new(index, processes.clone()).map_err(|_| SamplerError::Identity)?;
        match snapshot(&request) {
            Ok(snapshot) => {
                rows.extend(snapshot.rows().iter().cloned());
                completions.push(Instant::now());
            }
            Err(error) => {
                acquisition = acquisition_for_error(error);
                break;
            }
        }
    }

    let (max_interval_ms, actual_window_ms) = timing_summary(&completions)?;
    if actual_window_ms > 5_000 {
        return Err(SamplerError::ScheduleBound);
    }
    if completions.len() != usize::from(SAMPLE_COUNT)
        && acquisition == AcquisitionCompleteness::Complete
    {
        acquisition = AcquisitionCompleteness::Partial;
    }
    let fixtures = [
        FixtureLabel::LoopbackIpv4Long,
        FixtureLabel::LoopbackIpv6Long,
    ];
    let batch =
        sanitize_raw(&rows, acquisition, &fixtures).map_err(|_| SamplerError::Sanitization)?;
    let record = CandidateRecord::from_batch(
        platform_token(),
        completions.len() as u8,
        max_interval_ms,
        actual_window_ms,
        acquisition,
        &batch,
    )
    .map_err(|_| SamplerError::Record)?;
    let encoded = record
        .encode_candidate()
        .map_err(|_| SamplerError::Record)?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(encoded.as_bytes())
        .and_then(|_| stdout.write_all(b"\n"))
        .and_then(|_| stdout.flush())
        .map_err(|_| SamplerError::Io)
}

fn read_identities() -> Result<Identities, SamplerError> {
    let mut input = String::new();
    io::stdin()
        .lock()
        .take(IDENTITY_LIMIT + 1)
        .read_to_string(&mut input)
        .map_err(|_| SamplerError::Io)?;
    if input.len() as u64 > IDENTITY_LIMIT {
        return Err(SamplerError::Identity);
    }
    parse_identities(&input)
}

fn parse_identities(input: &str) -> Result<Identities, SamplerError> {
    if input.contains(['\r', '/', '\\', ':']) {
        return Err(SamplerError::Identity);
    }
    let lines: Vec<_> = input.lines().collect();
    if lines.len() != 2 || !input.ends_with('\n') {
        return Err(SamplerError::Identity);
    }
    let ipv4 = parse_identity(lines[0], "loopback_ipv4_long")?;
    let ipv6 = parse_identity(lines[1], "loopback_ipv6_long")?;
    if ipv4.pid == ipv6.pid {
        return Err(SamplerError::Identity);
    }
    Ok(Identities([ipv4, ipv6]))
}

fn parse_identity(input: &str, expected_fixture: &str) -> Result<FixtureIdentity, SamplerError> {
    let prefix = format!("fixture={expected_fixture};pid=");
    let pid_text = input
        .strip_prefix(&prefix)
        .filter(|value| !value.is_empty())
        .ok_or(SamplerError::Identity)?;
    let pid = pid_text
        .parse::<u32>()
        .map_err(|_| SamplerError::Identity)?;
    if pid == 0 || pid.to_string() != pid_text {
        return Err(SamplerError::Identity);
    }
    let fixture = match expected_fixture {
        "loopback_ipv4_long" => FixtureLabel::LoopbackIpv4Long,
        "loopback_ipv6_long" => FixtureLabel::LoopbackIpv6Long,
        _ => return Err(SamplerError::Identity),
    };
    Ok(FixtureIdentity { fixture, pid })
}

#[cfg(target_os = "linux")]
fn platform_processes(identities: &Identities) -> Result<Vec<AllowListedProcess>, SamplerError> {
    // SAFETY: geteuid has no arguments, returns the caller's effective UID,
    // and has no ownership or lifetime contract.
    let uid = unsafe { libc::geteuid() };
    Ok(identities
        .0
        .iter()
        .map(|identity| AllowListedProcess::Linux {
            fixture: identity.fixture,
            pid: identity.pid,
            uid,
        })
        .collect())
}

#[cfg(target_os = "windows")]
fn platform_processes(identities: &Identities) -> Result<Vec<AllowListedProcess>, SamplerError> {
    let image = std::env::current_exe().map_err(|_| SamplerError::Identity)?;
    Ok(identities
        .0
        .iter()
        .map(|identity| AllowListedProcess::Windows {
            fixture: identity.fixture,
            pid: identity.pid,
            expected_image_path: image.clone(),
        })
        .collect())
}

#[cfg(target_os = "linux")]
fn snapshot(request: &SnapshotRequest) -> Result<crate::adapter::RawSnapshot, AdapterError> {
    crate::platform::linux::LinuxAdapter.snapshot(request)
}

#[cfg(target_os = "windows")]
fn snapshot(request: &SnapshotRequest) -> Result<crate::adapter::RawSnapshot, AdapterError> {
    crate::platform::windows::WindowsAdapter.snapshot(request)
}

#[cfg(target_os = "linux")]
const fn platform_token() -> &'static str {
    "ubuntu_24_04_x86_64"
}

#[cfg(target_os = "windows")]
const fn platform_token() -> &'static str {
    "windows_11_25h2_26200_x64"
}

fn acquisition_for_error(error: AdapterError) -> AcquisitionCompleteness {
    match error {
        AdapterError::PermissionLimited => AcquisitionCompleteness::PermissionLimited,
        AdapterError::BufferChanged
        | AdapterError::InvalidRequest
        | AdapterError::MalformedResponse
        | AdapterError::PlatformFailure
        | AdapterError::ResourceLimit => AcquisitionCompleteness::Failed,
    }
}

fn timing_summary(completions: &[Instant]) -> Result<(u16, u16), SamplerError> {
    if completions.is_empty() {
        return Ok((0, 0));
    }
    let mut maximum = Duration::ZERO;
    for pair in completions.windows(2) {
        maximum = maximum.max(pair[1].duration_since(pair[0]));
    }
    let window = completions
        .last()
        .ok_or(SamplerError::ScheduleBound)?
        .duration_since(completions[0]);
    Ok((ceil_millis(maximum)?, ceil_millis(window)?))
}

fn ceil_millis(duration: Duration) -> Result<u16, SamplerError> {
    let nanos = duration.as_nanos();
    let millis = nanos
        .checked_add(999_999)
        .ok_or(SamplerError::ScheduleBound)?
        / 1_000_000;
    u16::try_from(millis).map_err(|_| SamplerError::ScheduleBound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_grammar_is_fixed_order_and_canonical() {
        let valid = "fixture=loopback_ipv4_long;pid=10\nfixture=loopback_ipv6_long;pid=11\n";
        assert_eq!(parse_identities(valid).unwrap().0[0].pid, 10);
        for invalid in [
            "fixture=loopback_ipv6_long;pid=11\nfixture=loopback_ipv4_long;pid=10\n",
            "fixture=loopback_ipv4_long;pid=010\nfixture=loopback_ipv6_long;pid=11\n",
            "fixture=loopback_ipv4_long;pid=10\nfixture=loopback_ipv6_long;pid=10\n",
            "fixture=loopback_ipv4_long;pid=10\n",
        ] {
            assert_eq!(parse_identities(invalid), Err(SamplerError::Identity));
        }
    }

    #[test]
    fn schedule_has_exactly_eleven_half_second_offsets() {
        let offsets: Vec<_> = (0..SAMPLE_COUNT)
            .map(|index| Schedule::offset(index).unwrap())
            .collect();
        assert_eq!(offsets.len(), 11);
        assert_eq!(offsets[0], Duration::ZERO);
        assert_eq!(offsets[10], Duration::from_secs(5));
        assert_eq!(Schedule::offset(11), Err(SamplerError::ScheduleBound));
    }

    #[test]
    fn millisecond_summary_rounds_up_without_using_a_timer() {
        assert_eq!(ceil_millis(Duration::from_nanos(1)).unwrap(), 1);
        assert_eq!(ceil_millis(Duration::from_millis(500)).unwrap(), 500);
        assert_eq!(ceil_millis(Duration::from_micros(500_001)).unwrap(), 501);
    }
}
