use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq)]
pub struct Microarchitecture {
    name: String,
    parents: Vec<Arc<Microarchitecture>>,
    vendor: String,
    features: Vec<String>,
    compilers: HashMap<String, Compiler>,
    generation: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Compiler {
    version: String,
    name: Option<String>,
    flags: String,
}

impl Microarchitecture {
    pub(crate) fn new(
        name: String,
        parents: Vec<Arc<Microarchitecture>>,
        vendor: String,
        features: Vec<String>,
        compilers: HashMap<String, Compiler>,
    ) -> Self {
        Microarchitecture::new_generation(name, parents, vendor, features, compilers, 0)
    }

    pub(crate) fn new_generation(
        name: String,
        parents: Vec<Arc<Microarchitecture>>,
        vendor: String,
        features: Vec<String>,
        compilers: HashMap<String, Compiler>,
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
        schema: &super::schema::Schema,
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
    };

    for name in schema.microarchitectures.keys() {
        if !known_targets.contains_key(name) {
            fill_target_from_map(name, schema, &mut known_targets);
        }
    }

    let host_platform = uname::uname().unwrap().machine;
    let generic_ma = generic_microarchitecture(&host_platform).into();
    known_targets.entry(host_platform).or_insert(generic_ma);

    known_targets
}

pub fn generic_microarchitecture(name: &str) -> Microarchitecture {
    Microarchitecture::new(
        name.to_string(),
        vec![],
        "generic".to_string(),
        vec![],
        HashMap::new(),
    )
}

pub fn version_components(version: &str) -> Option<(String, String)> {
    let re = regex::Regex::new(r"([\d.]*)(-?)(.*)").unwrap();
    let caps = re.captures(version)?;
    let version_number = caps.get(1)?.as_str().to_string();
    let suffix = caps.get(3)?.as_str().to_string();

    Some((version_number, suffix))
}

lazy_static! {
    pub static ref TARGETS: HashMap<String, Arc<Microarchitecture>> =
        { known_microarchitectures() };
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
