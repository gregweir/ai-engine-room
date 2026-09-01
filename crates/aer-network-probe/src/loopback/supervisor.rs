use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use super::fixture::{READY_TOKEN, STOP_TOKEN};
use super::record::{CandidateRecord, MAX_RECORD_BYTES};

const OUTER_TIMEOUT: Duration = Duration::from_secs(15);
const CAPTURE_LIMIT: u64 = MAX_RECORD_BYTES as u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SupervisorError {
    Child,
    Identity,
    Io,
    Privilege,
    Record,
    Timeout,
}

struct FixtureChild {
    child: Child,
    stdin: Option<ChildStdin>,
    ready: Receiver<Result<(), SupervisorError>>,
    tail: Receiver<Result<Vec<u8>, SupervisorError>>,
    stderr: Receiver<Result<Vec<u8>, SupervisorError>>,
}

struct SamplerChild {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Receiver<Result<Vec<u8>, SupervisorError>>,
    stderr: Receiver<Result<Vec<u8>, SupervisorError>>,
}

pub(crate) fn run() -> Result<(), SupervisorError> {
    ordinary_user_preflight()?;
    let deadline = Instant::now()
        .checked_add(OUTER_TIMEOUT)
        .ok_or(SupervisorError::Timeout)?;
    let mut ipv4 = spawn_fixture("ipv4")?;
    let mut ipv6 = match spawn_fixture("ipv6") {
        Ok(child) => child,
        Err(error) => {
            terminate_fixture(&mut ipv4);
            return Err(error);
        }
    };

    let result = run_children(&mut ipv4, &mut ipv6, deadline);
    if result.is_err() {
        terminate_fixture(&mut ipv4);
        terminate_fixture(&mut ipv6);
    }
    result
}

fn run_children(
    ipv4: &mut FixtureChild,
    ipv6: &mut FixtureChild,
    deadline: Instant,
) -> Result<(), SupervisorError> {
    wait_ready(&ipv4.ready, deadline)?;
    wait_ready(&ipv6.ready, deadline)?;
    ensure_running(&mut ipv4.child)?;
    ensure_running(&mut ipv6.child)?;

    let identities = format!(
        "fixture=loopback_ipv4_long;pid={}\nfixture=loopback_ipv6_long;pid={}\n",
        ipv4.child.id(),
        ipv6.child.id(),
    );
    let mut sampler = spawn_sampler()?;
    let sampler_result = (|| {
        sampler
            .stdin
            .as_mut()
            .ok_or(SupervisorError::Io)?
            .write_all(identities.as_bytes())
            .map_err(|_| SupervisorError::Io)?;
        sampler.stdin.take();
        let status = wait_status(&mut sampler.child, deadline)?;
        let stdout = receive_capture(&sampler.stdout, deadline)?;
        let stderr = receive_capture(&sampler.stderr, deadline)?;
        validate_clean_exit(status, &stderr)?;
        let candidate_text = one_line(&stdout)?;
        let candidate = CandidateRecord::parse_candidate(candidate_text)
            .map_err(|_| SupervisorError::Record)?;

        request_stop(ipv4)?;
        request_stop(ipv6)?;
        validate_fixture_exit(ipv4, deadline)?;
        validate_fixture_exit(ipv6, deadline)?;

        let final_record = candidate
            .encode_final()
            .map_err(|_| SupervisorError::Record)?;
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(final_record.as_bytes())
            .and_then(|_| stdout.write_all(b"\n"))
            .and_then(|_| stdout.flush())
            .map_err(|_| SupervisorError::Io)
    })();
    if sampler_result.is_err() {
        terminate_sampler(&mut sampler);
    }
    sampler_result
}

fn spawn_fixture(family: &str) -> Result<FixtureChild, SupervisorError> {
    let executable = std::env::current_exe().map_err(|_| SupervisorError::Identity)?;
    let mut child = Command::new(executable)
        .args(["fixture", family])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| SupervisorError::Child)?;
    let stdout = child.stdout.take().ok_or(SupervisorError::Child)?;
    let stderr = child.stderr.take().ok_or(SupervisorError::Child)?;
    let (ready_tx, ready) = mpsc::channel();
    let (tail_tx, tail) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let readiness = reader
            .read_line(&mut line)
            .map_err(|_| SupervisorError::Io)
            .and_then(|_| {
                if line == format!("{READY_TOKEN}\n") {
                    Ok(())
                } else {
                    Err(SupervisorError::Child)
                }
            });
        let _ = ready_tx.send(readiness);
        let _ = tail_tx.send(read_bounded(reader));
    });
    Ok(FixtureChild {
        stdin: child.stdin.take(),
        child,
        ready,
        tail,
        stderr: spawn_capture(stderr),
    })
}

fn spawn_sampler() -> Result<SamplerChild, SupervisorError> {
    let executable = std::env::current_exe().map_err(|_| SupervisorError::Identity)?;
    let mut child = Command::new(executable)
        .arg("sampler")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| SupervisorError::Child)?;
    let stdout = child.stdout.take().ok_or(SupervisorError::Child)?;
    let stderr = child.stderr.take().ok_or(SupervisorError::Child)?;
    Ok(SamplerChild {
        stdin: child.stdin.take(),
        child,
        stdout: spawn_capture(stdout),
        stderr: spawn_capture(stderr),
    })
}

fn spawn_capture<R: Read + Send + 'static>(
    reader: R,
) -> Receiver<Result<Vec<u8>, SupervisorError>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(read_bounded(reader));
    });
    receiver
}

fn read_bounded<R: Read>(reader: R) -> Result<Vec<u8>, SupervisorError> {
    let mut output = Vec::new();
    reader
        .take(CAPTURE_LIMIT + 1)
        .read_to_end(&mut output)
        .map_err(|_| SupervisorError::Io)?;
    if output.len() as u64 > CAPTURE_LIMIT {
        return Err(SupervisorError::Record);
    }
    Ok(output)
}

fn wait_ready(
    receiver: &Receiver<Result<(), SupervisorError>>,
    deadline: Instant,
) -> Result<(), SupervisorError> {
    receiver
        .recv_timeout(remaining(deadline)?)
        .map_err(|_| SupervisorError::Timeout)?
}

fn receive_capture(
    receiver: &Receiver<Result<Vec<u8>, SupervisorError>>,
    deadline: Instant,
) -> Result<Vec<u8>, SupervisorError> {
    receiver
        .recv_timeout(remaining(deadline)?)
        .map_err(|_| SupervisorError::Timeout)?
}

fn wait_status(child: &mut Child, deadline: Instant) -> Result<ExitStatus, SupervisorError> {
    loop {
        if let Some(status) = child.try_wait().map_err(|_| SupervisorError::Child)? {
            return Ok(status);
        }
        let left = remaining(deadline)?;
        std::thread::sleep(left.min(Duration::from_millis(10)));
    }
}

fn ensure_running(child: &mut Child) -> Result<(), SupervisorError> {
    match child.try_wait().map_err(|_| SupervisorError::Child)? {
        None => Ok(()),
        Some(_) => Err(SupervisorError::Child),
    }
}

fn request_stop(fixture: &mut FixtureChild) -> Result<(), SupervisorError> {
    fixture
        .stdin
        .as_mut()
        .ok_or(SupervisorError::Io)?
        .write_all(format!("{STOP_TOKEN}\n").as_bytes())
        .map_err(|_| SupervisorError::Io)?;
    fixture.stdin.take();
    Ok(())
}

fn validate_fixture_exit(
    fixture: &mut FixtureChild,
    deadline: Instant,
) -> Result<(), SupervisorError> {
    let status = wait_status(&mut fixture.child, deadline)?;
    let tail = receive_capture(&fixture.tail, deadline)?;
    let stderr = receive_capture(&fixture.stderr, deadline)?;
    validate_clean_exit(status, &stderr)?;
    if !tail.is_empty() {
        return Err(SupervisorError::Record);
    }
    Ok(())
}

fn validate_clean_exit(status: ExitStatus, stderr: &[u8]) -> Result<(), SupervisorError> {
    if !status.success() || !stderr.is_empty() {
        return Err(SupervisorError::Child);
    }
    Ok(())
}

fn one_line(bytes: &[u8]) -> Result<&str, SupervisorError> {
    let text = std::str::from_utf8(bytes).map_err(|_| SupervisorError::Record)?;
    let line = text.strip_suffix('\n').ok_or(SupervisorError::Record)?;
    if line.is_empty() || line.contains(['\r', '\n']) {
        return Err(SupervisorError::Record);
    }
    Ok(line)
}

fn remaining(deadline: Instant) -> Result<Duration, SupervisorError> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or(SupervisorError::Timeout)
}

fn terminate_fixture(fixture: &mut FixtureChild) {
    fixture.stdin.take();
    let _ = fixture.child.kill();
    let _ = fixture.child.wait();
}

fn terminate_sampler(sampler: &mut SamplerChild) {
    sampler.stdin.take();
    let _ = sampler.child.kill();
    let _ = sampler.child.wait();
}

#[cfg(target_os = "linux")]
fn ordinary_user_preflight() -> Result<(), SupervisorError> {
    // SAFETY: geteuid has no arguments, returns the caller's effective UID,
    // and has no ownership or lifetime contract.
    if unsafe { libc::geteuid() } == 0 {
        Err(SupervisorError::Privilege)
    } else {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn ordinary_user_preflight() -> Result<(), SupervisorError> {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token = ptr::null_mut();
    // SAFETY: GetCurrentProcess returns a pseudo-handle valid in this process;
    // `token` points to one writable handle and is closed below on success.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(SupervisorError::Privilege);
    }
    let mut elevation = TOKEN_ELEVATION::default();
    let mut returned = 0_u32;
    // SAFETY: `elevation` is writable for its declared size, `returned` is an
    // initialized out parameter, and the owned token remains valid for call.
    let queried = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast::<c_void>(),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
    };
    // SAFETY: token is the non-null owned handle returned by OpenProcessToken
    // and is closed exactly once here.
    unsafe { CloseHandle(token) };
    if queried == 0
        || returned != size_of::<TOKEN_ELEVATION>() as u32
        || elevation.TokenIsElevated != 0
    {
        Err(SupervisorError::Privilege)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_capture_requires_exactly_one_newline_terminated_line() {
        assert_eq!(one_line(b"candidate\n"), Ok("candidate"));
        for invalid in [b"candidate".as_slice(), b"candidate\nextra\n", b"\n"] {
            assert_eq!(one_line(invalid), Err(SupervisorError::Record));
        }
    }

    #[test]
    fn timeout_budget_is_fixed_at_fifteen_seconds() {
        assert_eq!(OUTER_TIMEOUT, Duration::from_secs(15));
        assert_eq!(CAPTURE_LIMIT, MAX_RECORD_BYTES as u64);
    }
}
