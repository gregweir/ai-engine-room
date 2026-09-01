use std::net::IpAddr;
use std::path::PathBuf;

use crate::model::{FixtureLabel, ProcessAssociation, SampleIndex, TcpState};

pub(crate) const MAX_ALLOW_LIST: usize = 16;
pub(crate) const MAX_RAW_ROWS: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdapterError {
    InvalidRequest,
    PermissionLimited,
    BufferChanged,
    MalformedResponse,
    ResourceLimit,
    PlatformFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RawProtocol {
    Tcp,
    Udp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericEndpoint {
    pub(crate) address: IpAddr,
    pub(crate) port: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlatformOwner {
    Windows { pid: u32, image_path: PathBuf },
    Linux { uid: u32, inode: u64 },
    Artificial,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawRow {
    pub(crate) fixture: Option<FixtureLabel>,
    pub(crate) sample_index: SampleIndex,
    pub(crate) protocol: RawProtocol,
    pub(crate) state: TcpState,
    pub(crate) local: NumericEndpoint,
    pub(crate) remote: NumericEndpoint,
    pub(crate) owner: PlatformOwner,
    pub(crate) association: ProcessAssociation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AllowListedProcess {
    Windows {
        fixture: FixtureLabel,
        pid: u32,
        expected_image_path: PathBuf,
    },
    Linux {
        fixture: FixtureLabel,
        pid: u32,
        uid: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotRequest {
    pub(crate) sample_index: SampleIndex,
    pub(crate) processes: Vec<AllowListedProcess>,
}

impl SnapshotRequest {
    pub(crate) fn new(
        sample_index: SampleIndex,
        processes: Vec<AllowListedProcess>,
    ) -> Result<Self, AdapterError> {
        if processes.len() > MAX_ALLOW_LIST {
            return Err(AdapterError::InvalidRequest);
        }
        Ok(Self { sample_index, processes })
    }

    #[cfg(test)]
    pub(crate) fn artificial(sample_index: SampleIndex) -> Self {
        Self { sample_index, processes: Vec::new() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawSnapshot {
    rows: Vec<RawRow>,
}

impl RawSnapshot {
    pub(crate) fn new(rows: Vec<RawRow>) -> Result<Self, AdapterError> {
        if rows.len() > MAX_RAW_ROWS {
            return Err(AdapterError::ResourceLimit);
        }
        Ok(Self { rows })
    }

    pub(crate) fn rows(&self) -> &[RawRow] {
        &self.rows
    }
}

pub(crate) trait SnapshotAdapter {
    fn snapshot(&self, request: &SnapshotRequest) -> Result<RawSnapshot, AdapterError>;
}

#[cfg(test)]
pub(crate) struct FakeAdapter(Result<RawSnapshot, AdapterError>);

#[cfg(test)]
impl FakeAdapter {
    pub(crate) fn failed(error: AdapterError) -> Self { Self(Err(error)) }
}

#[cfg(test)]
impl SnapshotAdapter for FakeAdapter {
    fn snapshot(&self, _request: &SnapshotRequest) -> Result<RawSnapshot, AdapterError> {
        self.0.clone()
    }
}
