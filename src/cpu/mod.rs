mod cpuid;
mod detect;
mod microarchitecture;
mod schema;

pub use detect::host;
pub use microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
