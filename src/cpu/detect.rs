use super::microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
use crate::cpu::cpuid::CpuId;
use crate::cpu::schema::TARGETS_JSON;
use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ffi::CStr;
use std::io;
use std::io::{BufRead, BufReader};
use std::mem::MaybeUninit;
use std::sync::Arc;

#[cfg(target_os = "windows")]
fn detect() -> Result<Microarchitecture, UnsupportedMicroarchitecture> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "x86_64")] {
            let cpuid = CpuId::host();
            return Ok(Microarchitecture {
                name: String::new(),
                parents: vec![],
                vendor: cpuid.vendor.clone(),
                features: cpuid.features.clone(),
                compilers: Default::default(),
                generation: 0,
                ancestors: Default::default(),
            });
        } else {
            detect_generic_arch()
        }
    }
}

#[cfg(not(any(target_os = "windows")))]
fn uname_machine() -> std::io::Result<String> {
    let mut utsname = MaybeUninit::zeroed();
    let r = unsafe { libc::uname(utsname.as_mut_ptr()) };
    if r != 0 {
        return Err(io::Error::last_os_error());
    }

    let utsname = unsafe { utsname.assume_init() };
    let machine = unsafe { CStr::from_ptr(utsname.machine.as_ptr()) };
    Ok(machine.to_string_lossy().into_owned())
}

#[cfg(target_os = "linux")]
fn detect() -> Result<Microarchitecture, UnsupportedMicroarchitecture> {
    // Read the CPU information from /proc/cpuinfo
    let mut data = HashMap::new();
    let mut lines = std::fs::File::open("/proc/cpuinfo")
        .map(BufReader::new)
        .map_err(|_| UnsupportedMicroarchitecture)?
        .lines();
    for line in lines {
        let Ok(line) = line else {
            continue;
        };
        let Some((key, value)) = line.split_once(':') else {
            // If there is no seperator and info was already populated, break because we are on a
            // blank line seperating CPUs.
            if !data.is_empty() {
                break;
            }
            continue;
        };
        data.insert(key.trim().to_string(), value.trim().to_string());
    }

    let architecture = uname_machine().map_err(|_| UnsupportedMicroarchitecture)?;
    if architecture == "x86_64" {
        return Ok(Microarchitecture {
            vendor: data
                .remove("vendor_id")
                .unwrap_or_else(|| String::from("generic")),
            features: data
                .remove("flags")
                .unwrap_or_default()
                .split(' ')
                .map(|s| s.to_string())
                .collect(),
            ..Microarchitecture::generic("")
        });
    }

    if architecture == "aarch64" {
        let vendor = if let Some(implementer) = data.get("CPU implementer") {
            // Mapping numeric codes to vendor (ARM). This list is a merge from
            // different sources:
            //
            // https://github.com/karelzak/util-linux/blob/master/sys-utils/lscpu-arm.c
            // https://developer.arm.com/docs/ddi0487/latest/arm-architecture-reference-manual-armv8-for-armv8-a-architecture-profile
            // https://github.com/gcc-mirror/gcc/blob/master/gcc/config/aarch64/aarch64-cores.def
            // https://patchwork.kernel.org/patch/10524949/
            TARGETS_JSON
                .conversions
                .arm_vendors
                .get(implementer)
                .cloned()
                .unwrap_or_else(|| "generic".to_string())
        } else {
            String::from("generic")
        };

        return Ok(Microarchitecture {
            vendor,
            features: data
                .remove("Features")
                .unwrap_or_default()
                .split(' ')
                .map(|s| s.to_string())
                .collect(),
            ..Microarchitecture::generic("")
        });
    }

    if architecture == "ppc64" || architecture == "ppc64le" {
        let cpu = data.remove("cpu").unwrap_or_default();
        let generation = cpu
            .strip_prefix("POWER")
            .and_then(|rest| rest.parse().ok())
            .unwrap_or(0);
        return Ok(Microarchitecture {
            generation,
            ..Microarchitecture::generic("")
        });
    }

    if architecture == "riscv64" {
        let uarch = match data.get("uarch").map(String::as_str) {
            Some("sifive,u74-mc") => "u74mc",
            Some(uarch) => uarch,
            None => "riscv64",
        };
        return Ok(Microarchitecture::generic(uarch));
    }

    Ok(Microarchitecture::generic(&architecture))
}

/// Construct a generic [`Microarchitecture`] based on the architecture of the host.
pub(crate) fn detect_generic_arch() -> Result<Microarchitecture, UnsupportedMicroarchitecture> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "aarch64")] {
            return Ok(Microarchitecture::generic("aarch64"));
        } else if #[cfg(target_arch = "powerpc64le")] {
            return Ok(Microarchitecture::generic("ppc64le"));
        } else if #[cfg(target_arch = "powerpc64")] {
            return Ok(Microarchitecture::generic("ppc64"));
        } else if #[cfg(target_arch = "riscv64")] {
            return Ok(Microarchitecture::generic("riscv64"));
        } else if #[cfg(target_arch = "x86_64")] {
            return Ok(Microarchitecture::generic("x86_64"));
        } else {
            return Err(UnsupportedMicroarchitecture);
        }
    }
}

fn compare_microarchitectures(a: &Microarchitecture, b: &Microarchitecture) -> Ordering {
    let ancestors_a = a.ancestors().len();
    let ancestors_b = b.ancestors().len();

    let features_a = a.features.len();
    let features_b = b.features.len();

    ancestors_a
        .cmp(&ancestors_b)
        .then(features_a.cmp(&features_b))
}

/// Detects the host micro-architecture and returns it.
pub fn host() -> Result<Arc<Microarchitecture>, UnsupportedMicroarchitecture> {
    // Detect the host micro-architecture.
    let detected_info = detect()?;

    // Get a list of possible candidates that are compatible with the hosts micro-architecture.
    let compatible_targets = compatible_microarchitectures_for_host(&detected_info);

    // Find the best generic candidates
    let Some(best_generic_candidate) = compatible_targets
        .iter()
        .filter(|target| target.vendor == "generic")
        .sorted_by(|a, b| compare_microarchitectures(&a, &b))
        .last()
    else {
        // If there is no matching generic candidate then
        return Err(UnsupportedMicroarchitecture);
    };

    // Filter the candidates to be descendant of the best generic candidate. This is to avoid that
    // the lack of a niche feature that can be disabled from e.g. BIOS prevents detection of a
    // reasonably performant architecture
    let best_candidates = compatible_targets
        .iter()
        .filter(|target| target.is_strict_superset(&best_generic_candidate))
        .collect_vec();

    // Resort the matching candidates and fall back to the best generic candidate if there is no
    // matching non-generic candidate.
    Ok(best_candidates
        .into_iter()
        .sorted_by(|a, b| compare_microarchitectures(&a, &b))
        .last()
        .unwrap_or(best_generic_candidate)
        .clone())
}

fn compatible_microarchitectures_for_aarch64(
    detected_info: &Microarchitecture,
    is_macos: bool,
) -> Vec<Arc<Microarchitecture>> {
    let targets = Microarchitecture::known_targets();

    // Get the root micro-architecture for aarch64.
    let Some(arch_root) = targets.get("aarch64") else {
        return vec![];
    };

    // On macOS it seems impossible to get all the CPU features with sysctl info, but for
    // ARM we can get the exact model
    let macos_model = if is_macos {
        match targets.get(&detected_info.name) {
            None => return vec![],
            model => model,
        }
    } else {
        None
    };

    // Find all targets that are decendants of the root architecture and are compatibile with the
    // detected micro-architecture.
    targets
        .values()
        .filter(|target| {
            // Must share the same architecture family and vendor.
            if arch_root.as_ref() != target.family()
                || !(target.vendor == "generic" || target.vendor != detected_info.vendor)
            {
                return false;
            }

            if let Some(macos_model) = macos_model {
                return target.as_ref() == macos_model.as_ref() || macos_model.decendent_of(target);
            } else {
                target.features.is_subset(&detected_info.features)
            }
        })
        .cloned()
        .collect()
}

fn compatible_microarchitectures_for_host(
    detected_info: &Microarchitecture,
) -> Vec<Arc<Microarchitecture>> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "aarch64")] {
            compatible_microarchitectures_for_aarch64(detected_info)
        } else if #[cfg(target_arch="powerpc64le")] {
            compatible_microarchitectures_for_ppc64(detected_info, PowerPc64::PowerPc64Le)
        } else if #[cfg(target_arch="powerpc64")] {
            compatible_microarchitectures_for_ppc64(detected_info, PowerPc64::PowerPc64)
        } else if #[cfg(target_arch="riscv64")] {
            compatible_microarchitectures_for_riscv64(detected_info)
        } else if #[cfg(target_arch="x86_64")] {
            compatible_microarchitectures_for_x86_64(detected_info)
        } else {
            vec![]
        }
    }
}

enum PowerPc64 {
    PowerPc64,
    PowerPc64Le,
}

fn compatible_microarchitectures_for_ppc64(
    detected_info: &Microarchitecture,
    power_pc64: PowerPc64,
) -> Vec<Arc<Microarchitecture>> {
    let targets = Microarchitecture::known_targets();

    let root_arch = match power_pc64 {
        PowerPc64::PowerPc64 => "ppc64",
        PowerPc64::PowerPc64Le => "ppc64le",
    };

    // Get the root micro-architecture.
    let Some(arch_root) = targets.get(root_arch) else {
        return vec![];
    };

    // Find all targets that are decendants of the root architecture and are compatibile with the
    // detected micro-architecture.
    targets
        .values()
        .filter(|target| {
            (target.as_ref() == arch_root.as_ref() || target.decendent_of(&arch_root))
                && target.generation <= detected_info.generation
        })
        .cloned()
        .collect()
}

fn compatible_microarchitectures_for_x86_64(
    detected_info: &Microarchitecture,
) -> Vec<Arc<Microarchitecture>> {
    let targets = Microarchitecture::known_targets();

    // Get the root micro-architecture for x86_64.
    let Some(arch_root) = targets.get("x86_64") else {
        return vec![];
    };

    // Find all targets that are decendants of the root architecture and are compatibile with the
    // detected micro-architecture.
    targets
        .values()
        .filter(|target| {
            (target.as_ref() == arch_root.as_ref() || target.decendent_of(&arch_root))
                && (target.vendor == detected_info.vendor || target.vendor == "generic")
                && target.features.is_subset(&detected_info.features)
        })
        .cloned()
        .collect()
}

fn compatible_microarchitectures_for_riscv64(
    detected_info: &Microarchitecture,
) -> Vec<Arc<Microarchitecture>> {
    let targets = Microarchitecture::known_targets();

    // Get the root micro-architecture for riscv64.
    let Some(arch_root) = targets.get("riscv64") else {
        return vec![];
    };

    // Find all targets that are descendants of the root architecture and are compatible with the
    // detected micro-architecture.
    targets
        .values()
        .filter(|target| {
            (target.as_ref() == arch_root.as_ref() || target.decendent_of(&arch_root))
                && (target.name == detected_info.name || target.vendor == "generic")
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn check_host() {
        let host = super::host();
        eprintln!("{:#?}", &host);
        host.expect("host() should return something");
    }
}
