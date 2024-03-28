mod detect;
mod microarchitecture;
mod schema;
mod cpuid;

pub use detect::host;
pub use microarchitecture::{
    generic_microarchitecture, version_components, Microarchitecture, UnsupportedMicroarchitecture,
    TARGETS,
};
