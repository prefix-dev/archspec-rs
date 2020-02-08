use super::microarchitecture::{Microarchitecture, UnsupportedMicroarchitecture};
use std::collections::HashMap;
use std::sync::Arc;

/// A type for a mapping of the information for the current host.
struct RawInfoMap;

/// Returns a map with information on the CPU of the current host.
fn raw_info_map() -> RawInfoMap {
    RawInfoMap
}

type CompatibilityCheckBox = Box<dyn Fn(&RawInfoMap, &str) -> bool + Send + Sync>;

lazy_static! {
    static ref COMPATIBILITY_CHECKS: HashMap<String, CompatibilityCheckBox> = {
        let mut m = HashMap::new();
        
        m
    };
}

/// Returns an unordered sequence of known micro-architectures that are
/// compatible with the info map passed as argument.
fn compatible_microarchitectures(info_map: RawInfoMap) -> Vec<Arc<Microarchitecture>> {
    let architecture_family = uname::uname().unwrap().machine;
    if let Some(tester) = COMPATIBILITY_CHECKS.get(&architecture_family) {
        super::microarchitecture::TARGETS
            .iter()
            .filter_map(|(name, march)| {
                if tester(&info_map, name) {
                    Some(march.clone())
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![super::microarchitecture::generic_microarchitecture(&architecture_family).into()]
    }
}

/// Detects the host micro-architecture and returns it.
pub fn host() -> Result<Arc<Microarchitecture>, UnsupportedMicroarchitecture> {
    let info = raw_info_map();
    let candidates = compatible_microarchitectures(info);
    candidates
        .iter()
        .min_by_key(|cand| cand.ancestors().len())
        .cloned()
        .ok_or(UnsupportedMicroarchitecture)
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
