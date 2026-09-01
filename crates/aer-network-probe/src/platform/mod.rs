#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub(crate) mod linux;

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub(crate) mod windows;
