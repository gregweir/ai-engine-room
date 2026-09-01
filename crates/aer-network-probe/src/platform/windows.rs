use std::ffi::c_void;
use std::mem::size_of;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};
use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
};
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::adapter::{
    AdapterError, AllowListedProcess, NumericEndpoint, PlatformOwner, RawProtocol, RawRow,
    RawSnapshot, SnapshotAdapter, SnapshotRequest, MAX_RAW_ROWS,
};
use crate::model::{FixtureLabel, ProcessAssociation, TcpState};

pub(crate) struct WindowsAdapter;

impl SnapshotAdapter for WindowsAdapter {
    fn snapshot(&self, request: &SnapshotRequest) -> Result<RawSnapshot, AdapterError> {
        let mut rows = Vec::new();
        collect_ipv4(request, &mut rows)?;
        collect_ipv6(request, &mut rows)?;
        RawSnapshot::new(rows)
    }
}

fn collect_ipv4(request: &SnapshotRequest, output: &mut Vec<RawRow>) -> Result<(), AdapterError> {
    let buffer = tcp_table(AF_INET as u32)?;
    parse_table::<MIB_TCPROW_OWNER_PID, _>(&buffer, |row| {
        let Some((fixture, expected_path)) = windows_identity(request, row.dwOwningPid) else {
            return Ok(None);
        };
        if row.dwState == 2 {
            return Ok(None);
        }
        let (image_path, association) = validate_image(row.dwOwningPid, expected_path);
        Ok(Some(RawRow {
            fixture: Some(fixture),
            sample_index: request.sample_index,
            protocol: RawProtocol::Tcp,
            state: tcp_state(row.dwState),
            local: NumericEndpoint { address: IpAddr::V4(Ipv4Addr::from(row.dwLocalAddr.to_ne_bytes())), port: port(row.dwLocalPort) },
            remote: NumericEndpoint { address: IpAddr::V4(Ipv4Addr::from(row.dwRemoteAddr.to_ne_bytes())), port: port(row.dwRemotePort) },
            owner: PlatformOwner::Windows { pid: row.dwOwningPid, image_path },
            association,
        }))
    }, output)
}

fn collect_ipv6(request: &SnapshotRequest, output: &mut Vec<RawRow>) -> Result<(), AdapterError> {
    let buffer = tcp_table(AF_INET6 as u32)?;
    parse_table::<MIB_TCP6ROW_OWNER_PID, _>(&buffer, |row| {
        let Some((fixture, expected_path)) = windows_identity(request, row.dwOwningPid) else {
            return Ok(None);
        };
        if row.dwState == 2 {
            return Ok(None);
        }
        let (image_path, association) = validate_image(row.dwOwningPid, expected_path);
        Ok(Some(RawRow {
            fixture: Some(fixture),
            sample_index: request.sample_index,
            protocol: RawProtocol::Tcp,
            state: tcp_state(row.dwState),
            local: NumericEndpoint { address: IpAddr::V6(Ipv6Addr::from(row.ucLocalAddr)), port: port(row.dwLocalPort) },
            remote: NumericEndpoint { address: IpAddr::V6(Ipv6Addr::from(row.ucRemoteAddr)), port: port(row.dwRemotePort) },
            owner: PlatformOwner::Windows { pid: row.dwOwningPid, image_path },
            association,
        }))
    }, output)
}

fn tcp_table(family: u32) -> Result<Vec<u8>, AdapterError> {
    let mut length = 0_u32;
    // SAFETY: the null first buffer is the documented size query; `length`
    // points to an initialized u32 owned for the call and no returned pointer
    // is retained.
    let status = unsafe { GetExtendedTcpTable(ptr::null_mut(), &mut length, 0, family, TCP_TABLE_OWNER_PID_ALL, 0) };
    if status != ERROR_INSUFFICIENT_BUFFER || length < size_of::<u32>() as u32 {
        return Err(AdapterError::PlatformFailure);
    }
    let mut buffer = vec![0_u8; length as usize];
    // SAFETY: `buffer` owns exactly `length` initialized bytes, its mutable
    // pointer is valid for the call, and the API may reduce but not exceed the
    // supplied length. A changing table is rejected rather than retried.
    let status = unsafe { GetExtendedTcpTable(buffer.as_mut_ptr().cast::<c_void>(), &mut length, 0, family, TCP_TABLE_OWNER_PID_ALL, 0) };
    if status == ERROR_INSUFFICIENT_BUFFER { return Err(AdapterError::BufferChanged); }
    if status != NO_ERROR || length as usize > buffer.len() { return Err(AdapterError::PlatformFailure); }
    buffer.truncate(length as usize);
    Ok(buffer)
}

fn parse_table<T: Copy, F>(buffer: &[u8], mut project: F, output: &mut Vec<RawRow>) -> Result<(), AdapterError>
where F: FnMut(T) -> Result<Option<RawRow>, AdapterError> {
    if buffer.len() < size_of::<u32>() { return Err(AdapterError::MalformedResponse); }
    // SAFETY: four checked bytes are present; unaligned access copies into an
    // owned u32 and does not borrow the native buffer.
    let count = unsafe { ptr::read_unaligned(buffer.as_ptr().cast::<u32>()) } as usize;
    let bytes = count.checked_mul(size_of::<T>()).and_then(|n| n.checked_add(size_of::<u32>())).ok_or(AdapterError::ResourceLimit)?;
    if count > MAX_RAW_ROWS || bytes > buffer.len() { return Err(AdapterError::MalformedResponse); }
    for index in 0..count {
        let offset = size_of::<u32>() + index * size_of::<T>();
        // SAFETY: the full row extent was checked above; unaligned access
        // copies the POD Windows row and no pointer outlives `buffer`.
        let row = unsafe { ptr::read_unaligned(buffer.as_ptr().add(offset).cast::<T>()) };
        if let Some(row) = project(row)? { output.push(row); }
    }
    Ok(())
}

fn windows_identity(request: &SnapshotRequest, pid: u32) -> Option<(FixtureLabel, &Path)> {
    request.processes.iter().find_map(|identity| match identity {
        AllowListedProcess::Windows { fixture, pid: allowed, expected_image_path } if *allowed == pid => Some((*fixture, expected_image_path.as_path())),
        _ => None,
    })
}

fn validate_image(pid: u32, expected: &Path) -> (PathBuf, ProcessAssociation) {
    match image_path(pid) {
        Ok(actual) if actual == expected => (actual, ProcessAssociation::SupportedProcess),
        Ok(actual) => (actual, ProcessAssociation::Stale),
        Err(AdapterError::PermissionLimited) => (PathBuf::new(), ProcessAssociation::PermissionLimited),
        Err(_) => (PathBuf::new(), ProcessAssociation::Stale),
    }
}

fn image_path(pid: u32) -> Result<PathBuf, AdapterError> {
    // SAFETY: no inherited handle is requested; the returned owned handle is
    // checked and closed on every path after creation.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() { return Err(AdapterError::PermissionLimited); }
    let mut buffer = vec![0_u16; 32_768];
    let mut length = buffer.len() as u32;
    // SAFETY: `buffer` has `length` writable UTF-16 elements, `length` is an
    // initialized in/out value, and the result is copied before the owned
    // handle and buffer are released.
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) };
    // SAFETY: `handle` is the non-null owned handle returned by OpenProcess and
    // is closed exactly once here.
    unsafe { CloseHandle(handle) };
    if ok == 0 || length as usize > buffer.len() { return Err(AdapterError::PlatformFailure); }
    buffer.truncate(length as usize);
    Ok(PathBuf::from(String::from_utf16(&buffer).map_err(|_| AdapterError::MalformedResponse)?))
}

fn port(value: u32) -> u16 { u16::from_be(value as u16) }

fn tcp_state(value: u32) -> TcpState {
    match value {
        1 => TcpState::Closed, 2 => TcpState::Listen, 3 => TcpState::SynSent,
        4 => TcpState::SynReceived, 5 => TcpState::Established, 6 => TcpState::FinWait1,
        7 => TcpState::FinWait2, 8 => TcpState::CloseWait, 9 => TcpState::Closing,
        10 => TcpState::LastAck, 11 => TcpState::TimeWait, 12 => TcpState::DeleteTcb,
        _ => TcpState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_parser_rejects_truncated_artificial_rows_without_native_io() {
        let mut buffer = vec![0_u8; size_of::<u32>()];
        buffer.copy_from_slice(&1_u32.to_ne_bytes());
        let mut output = Vec::new();
        let result = parse_table::<MIB_TCPROW_OWNER_PID, _>(
            &buffer,
            |_row| Ok(None),
            &mut output,
        );
        assert_eq!(result, Err(AdapterError::MalformedResponse));
    }
}
