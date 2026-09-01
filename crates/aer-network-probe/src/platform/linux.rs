use std::collections::BTreeMap;
use std::fs;
use std::mem::{size_of, zeroed};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::ptr;

use crate::adapter::{
    AdapterError, AllowListedProcess, NumericEndpoint, PlatformOwner, RawProtocol, RawRow,
    RawSnapshot, SnapshotAdapter, SnapshotRequest, MAX_RAW_ROWS,
};
use crate::model::{AddressFamily, FixtureLabel, ProcessAssociation, TcpState};

const SOCK_DIAG_BY_FAMILY: u16 = 20;
const NLMSG_DONE: u16 = 3;
const NLMSG_ERROR: u16 = 2;
const NLM_F_REQUEST: u16 = 1;
const NLM_F_DUMP: u16 = 0x300;
const TCP_ALL_STATES: u32 = 0x0fff;
const NL_HEADER: usize = 16;
const DIAG_MESSAGE: usize = 72;

#[repr(C)]
#[derive(Clone, Copy)]
struct Request {
    header: libc::nlmsghdr,
    family: u8,
    protocol: u8,
    extensions: u8,
    pad: u8,
    states: u32,
    socket_id: [u32; 12],
}

pub(crate) struct LinuxAdapter;

impl SnapshotAdapter for LinuxAdapter {
    fn snapshot(&self, request: &SnapshotRequest) -> Result<RawSnapshot, AdapterError> {
        let owners = allowed_socket_inodes(request)?;
        let mut rows = Vec::new();
        rows.extend(snapshot_family(request, AddressFamily::Ipv4, &owners)?);
        rows.extend(snapshot_family(request, AddressFamily::Ipv6, &owners)?);
        RawSnapshot::new(rows)
    }
}

fn snapshot_family(
    request: &SnapshotRequest,
    family: AddressFamily,
    owners: &BTreeMap<u64, (FixtureLabel, u32)>,
) -> Result<Vec<RawRow>, AdapterError> {
    // SAFETY: socket returns a new descriptor or -1; a successful descriptor
    // is immediately transferred to one OwnedFd and closed exactly once.
    let raw_fd = unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, libc::NETLINK_SOCK_DIAG) };
    if raw_fd < 0 { return Err(AdapterError::PermissionLimited); }
    // SAFETY: `raw_fd` is a newly owned successful socket descriptor and no
    // other owner exists after this conversion.
    let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

    // SAFETY: zero is a valid initialization for this owned stack sockaddr_nl;
    // no pointer exists yet, and every bind-visible field is assigned below.
    let mut local: libc::sockaddr_nl = unsafe { zeroed() };
    local.nl_family = libc::AF_NETLINK as u16;
    // SAFETY: `local` is fully initialized for its declared size and borrowed
    // only for the duration of bind; the owned socket remains valid.
    let bound = unsafe { libc::bind(fd.as_raw_fd(), ptr::addr_of!(local).cast(), size_of::<libc::sockaddr_nl>() as libc::socklen_t) };
    if bound != 0 { return Err(AdapterError::PlatformFailure); }

    let sequence = match family { AddressFamily::Ipv4 => 0x1a4_u32, AddressFamily::Ipv6 => 0x1a6_u32 };
    let mut socket_id = [0_u32; 12];
    socket_id[10] = u32::MAX;
    socket_id[11] = u32::MAX;
    let request_wire = Request {
        header: libc::nlmsghdr { nlmsg_len: size_of::<Request>() as u32, nlmsg_type: SOCK_DIAG_BY_FAMILY, nlmsg_flags: NLM_F_REQUEST | NLM_F_DUMP, nlmsg_seq: sequence, nlmsg_pid: 0 },
        family: match family { AddressFamily::Ipv4 => libc::AF_INET as u8, AddressFamily::Ipv6 => libc::AF_INET6 as u8 },
        protocol: libc::IPPROTO_TCP as u8,
        extensions: 0,
        pad: 0,
        states: TCP_ALL_STATES,
        socket_id,
    };
    // SAFETY: zero is valid initialization for this owned stack kernel address;
    // no pointer exists yet, family is assigned, and pid/groups stay zero.
    let mut kernel: libc::sockaddr_nl = unsafe { zeroed() };
    kernel.nl_family = libc::AF_NETLINK as u16;
    // SAFETY: `request_wire` and `kernel` are fully initialized, both pointers
    // are valid for their owned sizes, and sendto copies bytes before return.
    let sent = unsafe {
        libc::sendto(
            fd.as_raw_fd(),
            ptr::addr_of!(request_wire).cast(),
            size_of::<Request>(),
            0,
            ptr::addr_of!(kernel).cast(),
            size_of::<libc::sockaddr_nl>() as libc::socklen_t,
        )
    };
    if sent != size_of::<Request>() as isize { return Err(AdapterError::PlatformFailure); }

    let mut output = Vec::new();
    loop {
        let mut buffer = [0_u8; 65_536];
        // SAFETY: `buffer` owns 65,536 writable initialized bytes for recv;
        // the returned length is checked before any parsing or slicing.
        let received = unsafe { libc::recv(fd.as_raw_fd(), buffer.as_mut_ptr().cast(), buffer.len(), 0) };
        if received <= 0 { return Err(AdapterError::PlatformFailure); }
        let done = parse_netlink(&buffer[..received as usize], sequence, family, request, owners, &mut output)?;
        if done { break; }
        if output.len() > MAX_RAW_ROWS { return Err(AdapterError::ResourceLimit); }
    }
    Ok(output)
}

fn parse_netlink(
    buffer: &[u8],
    sequence: u32,
    family: AddressFamily,
    request: &SnapshotRequest,
    owners: &BTreeMap<u64, (FixtureLabel, u32)>,
    output: &mut Vec<RawRow>,
) -> Result<bool, AdapterError> {
    let mut offset = 0_usize;
    while offset < buffer.len() {
        if buffer.len() - offset < NL_HEADER { return Err(AdapterError::MalformedResponse); }
        let length = read_u32_ne(buffer, offset)? as usize;
        let message_type = read_u16_ne(buffer, offset + 4)?;
        let message_sequence = read_u32_ne(buffer, offset + 8)?;
        let sender_pid = read_u32_ne(buffer, offset + 12)?;
        if length < NL_HEADER || length > buffer.len() - offset || message_sequence != sequence || sender_pid != 0 {
            return Err(AdapterError::MalformedResponse);
        }
        if message_type == NLMSG_DONE { return Ok(true); }
        if message_type == NLMSG_ERROR { return Err(AdapterError::PlatformFailure); }
        if message_type != SOCK_DIAG_BY_FAMILY || length < NL_HEADER + DIAG_MESSAGE { return Err(AdapterError::MalformedResponse); }
        let payload = &buffer[offset + NL_HEADER..offset + length];
        if let Some(row) = parse_diag(payload, family, request, owners)? { output.push(row); }
        offset = align4(offset.checked_add(length).ok_or(AdapterError::MalformedResponse)?)?;
        if offset > buffer.len() { return Err(AdapterError::MalformedResponse); }
    }
    Ok(false)
}

fn parse_diag(
    payload: &[u8],
    family: AddressFamily,
    request: &SnapshotRequest,
    owners: &BTreeMap<u64, (FixtureLabel, u32)>,
) -> Result<Option<RawRow>, AdapterError> {
    if payload.len() < DIAG_MESSAGE { return Err(AdapterError::MalformedResponse); }
    let expected_family = match family { AddressFamily::Ipv4 => libc::AF_INET as u8, AddressFamily::Ipv6 => libc::AF_INET6 as u8 };
    if payload[0] != expected_family { return Err(AdapterError::MalformedResponse); }
    let state = linux_state(payload[1]);
    if state == TcpState::Listen { return Ok(None); }
    let local_port = read_u16_be(payload, 4)?;
    let remote_port = read_u16_be(payload, 6)?;
    let uid = read_u32_ne(payload, 64)?;
    let inode = read_u32_ne(payload, 68)? as u64;
    let Some((fixture, allowed_uid)) = owners.get(&inode).copied() else { return Ok(None); };
    let association = if uid == allowed_uid { ProcessAssociation::SupportedProcess } else { ProcessAssociation::Stale };
    let (local_address, remote_address) = match family {
        AddressFamily::Ipv4 => (
            IpAddr::V4(Ipv4Addr::from(read_array::<4>(payload, 8)?)),
            IpAddr::V4(Ipv4Addr::from(read_array::<4>(payload, 24)?)),
        ),
        AddressFamily::Ipv6 => (
            IpAddr::V6(Ipv6Addr::from(read_array::<16>(payload, 8)?)),
            IpAddr::V6(Ipv6Addr::from(read_array::<16>(payload, 24)?)),
        ),
    };
    Ok(Some(RawRow {
        fixture: Some(fixture), sample_index: request.sample_index, protocol: RawProtocol::Tcp, state,
        local: NumericEndpoint { address: local_address, port: local_port },
        remote: NumericEndpoint { address: remote_address, port: remote_port },
        owner: PlatformOwner::Linux { uid, inode }, association,
    }))
}

fn allowed_socket_inodes(request: &SnapshotRequest) -> Result<BTreeMap<u64, (FixtureLabel, u32)>, AdapterError> {
    let mut output = BTreeMap::new();
    for identity in &request.processes {
        let AllowListedProcess::Linux { fixture, pid, uid } = identity else { continue };
        let process = PathBuf::from(format!("/proc/{pid}"));
        let metadata = fs::metadata(&process).map_err(|error| if error.kind() == std::io::ErrorKind::PermissionDenied { AdapterError::PermissionLimited } else { AdapterError::PlatformFailure })?;
        if metadata.uid() != *uid { return Err(AdapterError::PermissionLimited); }
        let descriptors = fs::read_dir(process.join("fd")).map_err(|error| if error.kind() == std::io::ErrorKind::PermissionDenied { AdapterError::PermissionLimited } else { AdapterError::PlatformFailure })?;
        for descriptor in descriptors {
            let descriptor = descriptor.map_err(|_| AdapterError::BufferChanged)?;
            let target = fs::read_link(descriptor.path()).map_err(|_| AdapterError::BufferChanged)?;
            if let Some(inode) = parse_socket_link(&target) { output.insert(inode, (*fixture, *uid)); }
        }
    }
    Ok(output)
}

fn parse_socket_link(target: &std::path::Path) -> Option<u64> {
    let value = target.to_str()?;
    value.strip_prefix("socket:[")?.strip_suffix(']')?.parse().ok()
}

fn align4(value: usize) -> Result<usize, AdapterError> {
    Ok(value.checked_add(3).ok_or(AdapterError::MalformedResponse)? & !3)
}

fn read_u16_ne(buffer: &[u8], offset: usize) -> Result<u16, AdapterError> {
    Ok(u16::from_ne_bytes(read_array::<2>(buffer, offset)?))
}

fn read_u16_be(buffer: &[u8], offset: usize) -> Result<u16, AdapterError> {
    Ok(u16::from_be_bytes(read_array::<2>(buffer, offset)?))
}

fn read_u32_ne(buffer: &[u8], offset: usize) -> Result<u32, AdapterError> {
    Ok(u32::from_ne_bytes(read_array::<4>(buffer, offset)?))
}

fn read_array<const N: usize>(buffer: &[u8], offset: usize) -> Result<[u8; N], AdapterError> {
    let end = offset.checked_add(N).ok_or(AdapterError::MalformedResponse)?;
    let source = buffer.get(offset..end).ok_or(AdapterError::MalformedResponse)?;
    let mut result = [0_u8; N];
    result.copy_from_slice(source);
    Ok(result)
}

fn linux_state(value: u8) -> TcpState {
    match value {
        1 => TcpState::Established, 2 => TcpState::SynSent, 3 => TcpState::SynReceived,
        4 => TcpState::FinWait1, 5 => TcpState::FinWait2, 6 => TcpState::TimeWait,
        7 => TcpState::Closed, 8 => TcpState::CloseWait, 9 => TcpState::LastAck,
        10 => TcpState::Listen, 11 => TcpState::Closing, 12 => TcpState::DeleteTcb,
        _ => TcpState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rejects_bad_message_lengths_without_native_io() {
        let request = SnapshotRequest::artificial(crate::model::SampleIndex::new(0).unwrap());
        let mut output = Vec::new();
        let mut bytes = [0_u8; NL_HEADER];
        bytes[0..4].copy_from_slice(&(8_u32).to_ne_bytes());
        assert_eq!(parse_netlink(&bytes, 1, AddressFamily::Ipv4, &request, &BTreeMap::new(), &mut output), Err(AdapterError::MalformedResponse));
    }

    #[test]
    fn socket_link_parser_accepts_only_numeric_descriptor_targets() {
        assert_eq!(parse_socket_link(std::path::Path::new("socket:[42]")), Some(42));
        assert_eq!(parse_socket_link(std::path::Path::new("pipe:[42]")), None);
        assert_eq!(parse_socket_link(std::path::Path::new("socket:[name]")), None);
    }
}
