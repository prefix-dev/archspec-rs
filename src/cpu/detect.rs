use super::microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
use crate::cpu::cpuid::CpuId;
use std::sync::Arc;

#[cfg(target_os = "windows")]
fn detect() -> Result<Microarchitecture, UnsupportedMicroarchitecture> {
    #[cfg(target_arch = "x86_64")]
    {
        let cpuid = CpuId::host();
        return Ok(Microarchitecture {
            name: String::new(),
            parents: vec![],
            vendor: cpuid.vendor.clone(),
            features: cpuid.features.clone(),
            compilers: Default::default(),
            generation: 0,
        });
    }

    detect_generic_arch()
}

/// Construct a generic [`Microarchitecture`] based on the architecture of the host.
fn detect_generic_arch() -> Result<Microarchitecture, UnsupportedMicroarchitecture> {
    #[cfg(target_arch = "aarch64")]
    {
        return Ok(Microarchitecture::generic("aarch64"));
    }

    #[cfg(target_arch = "powerpc64le")]
    {
        return Ok(Microarchitecture::generic("ppc64le"));
    }
    #[cfg(target_arch = "powerpc64")]
    {
        return Ok(Microarchitecture::generic("ppc64"));
    }
    #[cfg(target_arch = "riscv64")]
    {
        return Ok(Microarchitecture::generic("riscv64"));
    }

    Err(UnsupportedMicroarchitecture)
}

/// Detects the host micro-architecture and returns it.
pub fn host() -> Result<Microarchitecture, UnsupportedMicroarchitecture> {
    // Detect the host micro-architecture.
    let detected_info = detect()?;

    // Get a list of possible candidates that are compatible with the hosts micro-architecture.
    
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
