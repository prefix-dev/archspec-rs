use super::schema::{Compiler, CompilerSet};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq)]
pub struct Microarchitecture {
    pub(crate) name: String,
    pub(crate) parents: Vec<Arc<Microarchitecture>>,
    pub(crate) vendor: String,
    pub(crate) features: HashSet<String>,
    pub(crate) compilers: HashMap<String, Vec<Compiler>>,
    pub(crate) generation: usize,
}

impl Microarchitecture {
    pub(crate) fn new(
        name: String,
        parents: Vec<Arc<Microarchitecture>>,
        vendor: String,
        features: HashSet<String>,
        compilers: HashMap<String, Vec<Compiler>>,
    ) -> Self {
        Microarchitecture::new_generation(name, parents, vendor, features, compilers, 0)
    }

    pub(crate) fn new_generation(
        name: String,
        parents: Vec<Arc<Microarchitecture>>,
        vendor: String,
        features: HashSet<String>,
        compilers: HashMap<String, Vec<Compiler>>,
        generation: usize,
    ) -> Self {
        Microarchitecture {
            name,
            parents,
            vendor,
            features,
            compilers,
            generation,
        }
    }

    pub fn generic(name: &str) -> Microarchitecture {
        Microarchitecture::new(
            name.to_string(),
            vec![],
            "generic".to_string(),
            HashSet::new(),
            HashMap::new(),
        )
    }

    pub fn ancestors(&self) -> Vec<Arc<Microarchitecture>> {
        let mut v = self.parents.clone();
        for parent in &self.parents {
            let new_ancestors = parent
                .ancestors()
                .into_iter()
                .filter(|a| v.contains(a))
                .collect::<Vec<_>>();
            v.extend(new_ancestors);
        }
        v
    }
}

#[derive(Debug)]
pub struct UnsupportedMicroarchitecture;

fn known_microarchitectures() -> HashMap<String, Arc<Microarchitecture>> {
    let mut known_targets: HashMap<String, Arc<Microarchitecture>> = HashMap::new();
    let schema = &super::schema::TARGETS_JSON;

    fn fill_target_from_map(
        name: &str,
        schema: &super::schema::MicroarchitecturesSchema,
        targets: &mut HashMap<String, Arc<Microarchitecture>>,
    ) {
        let data = &schema.microarchitectures;
        let values = &data[name];
        let parent_names = &values.from;
        for parent in parent_names {
            if !targets.contains_key(parent) {
                fill_target_from_map(parent, schema, targets);
            }
        }
        let parents = parent_names
            .iter()
            .map(|parent| targets[parent].clone())
            .collect::<Vec<Arc<Microarchitecture>>>();

        let vendor = values.vendor.clone();
        let features: HashSet<String> = values.features.iter().cloned().collect();
        let compilers: HashMap<String, Vec<Compiler>> = values
            .compilers
            .as_ref()
            .map(|compilers| {
                compilers
                    .iter()
                    .map(|(vendor, set)| {
                        (
                            vendor.clone(),
                            // normalize to a sequence of compiler definitions
                            match set {
                                CompilerSet::Several(cs) => cs.clone(),
                                CompilerSet::Single(c) => vec![c.clone()],
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_else(HashMap::new);
        let generation = values.generation.unwrap_or(0);

        targets.insert(
            name.to_string(),
            Arc::new(Microarchitecture::new_generation(
                name.to_string(),
                parents,
                vendor,
                features,
                compilers,
                generation,
            )),
        );
    }

    for name in schema.microarchitectures.keys() {
        if !known_targets.contains_key(name) {
            fill_target_from_map(name, schema, &mut known_targets);
        }
    }

    // let host_platform = uname::uname().unwrap().machine;
    // let generic_ma = generic_microarchitecture(&host_platform).into();
    // known_targets.entry(host_platform).or_insert(generic_ma);

    known_targets
}

pub fn version_components(version: &str) -> Option<(String, String)> {
    let re = regex::Regex::new(r"([\d.]*)(-?)(.*)").unwrap();
    let caps = re.captures(version)?;
    let version_number = caps.get(1)?.as_str().to_string();
    let suffix = caps.get(3)?.as_str().to_string();

    Some((version_number, suffix))
}

lazy_static! {
    pub static ref TARGETS: HashMap<String, Arc<Microarchitecture>> = known_microarchitectures();
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_components() {
        fn ref_tup(t: &(String, String)) -> (&str, &str) {
            (&t.0, &t.1)
        }

        use super::version_components;
        for (version, truth) in &[("1.2.3-hi.ho", Some(("1.2.3", "hi.ho")))] {
            assert_eq!(version_components(version).as_ref().map(ref_tup), *truth);
        }
    }
}
