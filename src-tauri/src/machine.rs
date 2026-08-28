//! Narrow categorical machine context. CPU architecture is metadata, not a
//! numeric metric, and deliberately has no Report projection.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineContextState {
    Available,
    NotExposed,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct MachineContextView {
    pub state: MachineContextState,
    pub native_cpu_architecture: Option<String>,
    pub interpretation: String,
    pub why_it_matters: String,
    pub qualification: String,
}

fn view(state: MachineContextState, architecture: Option<&str>) -> MachineContextView {
    MachineContextView {
        state,
        native_cpu_architecture: architecture.map(str::to_string),
        interpretation: match architecture {
            Some(value) => format!("The operating system reports the native CPU architecture as {value}."),
            None => "The native CPU architecture is not available right now.".to_string(),
        },
        why_it_matters: "Native architecture provides compatibility context for software built for a particular processor family.".to_string(),
        qualification: "Architecture alone does not establish model compatibility, acceleration, performance, or compute placement.".to_string(),
    }
}

#[cfg(any(target_os = "linux", test))]
fn map_architecture(value: &str) -> &str {
    match value {
        "x86_64" | "amd64" => "x86_64",
        "x86" | "i386" | "i486" | "i586" | "i686" => "x86",
        "aarch64" | "arm64" => "arm64",
        value if value.starts_with("arm") => "arm",
        _ => "unknown",
    }
}

#[cfg(any(target_os = "windows", test))]
fn map_windows_architecture(code: u16) -> &'static str {
    // Documented PROCESSOR_ARCHITECTURE_* values from SYSTEM_INFO.
    match code {
        9 => "x86_64", // AMD64
        0 => "x86",    // INTEL
        12 => "arm64", // ARM64
        5 => "arm",    // ARM
        _ => "unknown",
    }
}

#[cfg(target_os = "linux")]
pub fn current_machine_context() -> MachineContextView {
    use std::ffi::CStr;
    use std::os::raw::{c_char, c_int};

    #[repr(C)]
    struct UtsName {
        sysname: [c_char; 65],
        nodename: [c_char; 65],
        release: [c_char; 65],
        version: [c_char; 65],
        machine: [c_char; 65],
        domainname: [c_char; 65],
    }

    extern "C" {
        fn uname(name: *mut UtsName) -> c_int;
    }

    let mut name = std::mem::MaybeUninit::<UtsName>::uninit();
    // SAFETY: `uname` receives writable storage for the exact Linux utsname
    // layout. The value is read only after a successful return. Only the
    // bounded machine field is mapped; all other fields are discarded.
    if unsafe { uname(name.as_mut_ptr()) } != 0 {
        return view(MachineContextState::Failed, None);
    }
    // SAFETY: a successful `uname` initializes the structure and terminates
    // each field. `machine` is read only to perform a controlled mapping.
    let name = unsafe { name.assume_init() };
    let raw = unsafe { CStr::from_ptr(name.machine.as_ptr()) };
    match raw.to_str() {
        Ok(value) => view(
            MachineContextState::Available,
            Some(map_architecture(value)),
        ),
        Err(_) => view(MachineContextState::Failed, None),
    }
}

#[cfg(target_os = "windows")]
pub fn current_machine_context() -> MachineContextView {
    use windows_sys::Win32::System::SystemInformation::{GetNativeSystemInfo, SYSTEM_INFO};

    let mut info = std::mem::MaybeUninit::<SYSTEM_INFO>::zeroed();
    // SAFETY: GetNativeSystemInfo writes one SYSTEM_INFO into valid storage.
    unsafe { GetNativeSystemInfo(info.as_mut_ptr()) };
    let info = unsafe { info.assume_init() };
    let code = unsafe { info.Anonymous.Anonymous.wProcessorArchitecture };
    let architecture = map_windows_architecture(code);
    view(MachineContextState::Available, Some(architecture))
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn current_machine_context() -> MachineContextView {
    view(MachineContextState::NotExposed, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architecture_mapping_is_bounded() {
        assert_eq!(map_architecture("x86_64"), "x86_64");
        assert_eq!(map_architecture("i686"), "x86");
        assert_eq!(map_architecture("aarch64"), "arm64");
        assert_eq!(map_architecture("armv7l"), "arm");
        assert_eq!(map_architecture("private-host-value"), "unknown");
        assert_eq!(map_windows_architecture(9), "x86_64");
        assert_eq!(map_windows_architecture(0), "x86");
        assert_eq!(map_windows_architecture(12), "arm64");
        assert_eq!(map_windows_architecture(5), "arm");
        assert_eq!(map_windows_architecture(u16::MAX), "unknown");
    }

    #[test]
    fn view_never_makes_capability_or_placement_claims() {
        let context = view(MachineContextState::Available, Some("arm64"));
        assert_eq!(context.native_cpu_architecture.as_deref(), Some("arm64"));
        assert!(context.qualification.contains("does not establish"));
        assert!(!context.qualification.contains("ready"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn native_linux_architecture_is_reduced_to_the_controlled_set() {
        let context = current_machine_context();
        assert_eq!(context.state, MachineContextState::Available);
        assert!(matches!(
            context.native_cpu_architecture.as_deref(),
            Some("x86_64" | "x86" | "arm64" | "arm" | "unknown")
        ));
    }
}
