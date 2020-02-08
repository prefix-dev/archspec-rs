mod alias;
mod detect;
mod microarchitecture;
mod schema;

pub use microarchitecture::{
    generic_microarchitecture, UnsupportedMicroarchitecture, version_components,
    Microarchitecture, TARGETS,
};
pub use detect::host;