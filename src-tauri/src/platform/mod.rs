//! Platform-specific OS metric providers.
//!
//! Real provider implementations are gated by `#[cfg(target_os = ...)]` so that
//! only the relevant platform is compiled into the native binary. `aer-core`
//! remains platform-free; this module lives in the Tauri application crate.

#[cfg(target_os = "linux")]
pub mod linux;

// The platform-neutral provider seam is also compiled for deterministic tests;
// the actual Win32 binding and FFI probe remain Windows-only.
#[cfg(any(target_os = "windows", test))]
pub mod windows;
