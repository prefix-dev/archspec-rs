use super::microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
use crate::cpu::cpuid::CpuId;
use std::sync::Arc;

/// Detects the host micro-architecture and returns it.
pub fn host() -> Result<Arc<Microarchitecture>, UnsupportedMicroarchitecture> {
    let cpuid = CpuId::host();
    let architecture = Microarchitecture {
        name: String::new(),
        parents: vec![],
        vendor: cpuid.vendor.clone(),
        features: cpuid.features.clone(),
        compilers: Default::default(),
        generation: 0,
    };

    Ok(Arc::new(architecture))
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_host() {
        let host = super::host();
        eprintln!("{:?}", &host);
        host.expect("host() should return something");
    }
}
