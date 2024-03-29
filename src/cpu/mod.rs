mod detect;
mod microarchitecture;

pub use detect::host;
pub use microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
