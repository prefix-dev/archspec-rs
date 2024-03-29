use super::microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
use crate::cpu::cpuid::CpuId;
use itertools::Itertools;
use std::cmp::Ordering;
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
