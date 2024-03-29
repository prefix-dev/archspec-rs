mod detect;
mod microarchitecture;

#[cfg(target_os = "windows")]
mod cpuid;

pub use detect::host;
pub use microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
