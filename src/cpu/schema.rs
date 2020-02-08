//! Types and functions to manipulate the contents microarchitecture data file.
//!
//! These are encoding the rules of the corresponding schema as Rust data types
//! with the help of `serde` deserialization.

use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Schema for microarchitecture definitions and feature aliases.
#[derive(Debug, Deserialize)]
pub struct Schema {
    pub(crate) microarchitectures: HashMap<String, Microarchitecture>,
    pub(crate) feature_aliases: HashMap<String, FeatureAlias>,
    pub(crate) conversions: Conversions,
}

/// Defines the attributes and requirements of a microarchitecture.
#[derive(Debug, Deserialize)]
pub struct Microarchitecture {
    /// A list of the immediate microarchitectures that this one is considered
    /// to be derived from.
    #[serde(deserialize_with = "zero_one_many_string")]
    pub(crate) from: Vec<String>,

    /// Human-readable vendor name.
    pub(crate) vendor: String,

    /// The CPU features that are required to exist on the system for it to be
    /// compatible with this microarchitecture.
    pub(crate) features: Vec<String>,

    /// Optional information on how to tell different compilers how to optimize
    /// for this microarchitecture.
    pub(crate) compilers: Option<HashMap<String, CompilerSet>>,
}

/// Compiler optimization for a particular compiler, either one for all flavours
/// of the compiler or several indicating how to do it for particular version ranges.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CompilerSet {
    /// Multiple entries (Compiler change options across versions).
    Several(Vec<Compiler>),

    /// Single entry (Compiler didn't change options across versions).
    Single(Compiler),
}

/// Indicates how to tell a particular compiler flavour how to optimize
/// for an microarchitecture.
#[derive(Debug, Deserialize)]
pub struct Compiler {
    /// Indicates the versions of the compiler this applies to.
    pub(crate) versions: String,

    /// Command line argument to pass to the compiler to optimize for this architecture.
    /// May contain `{name}` placeholders.
    pub(crate) flags: String,

    /// Architecture name, for use in the optimization flags.
    pub(crate) name: Option<String>,
}

/// Synthesised feature aliases derived from existing features or families.
#[derive(Debug, Deserialize)]
pub struct FeatureAlias {
    /// The reason for why this alias is defined.
    pub(crate) reason: Option<String>,

    /// The alias is valid if any of the items are a feature of the target.
    pub(crate) any_of: Option<Vec<String>>,

    /// The alias is valid if the family of the target is in this list.
    pub(crate) families: Option<Vec<String>>,
}

/// Conversions that map some platform specific value to canonical values.
#[derive(Debug, Deserialize)]
pub struct Conversions {
    pub(crate) description: String,

    /// Maps from ARM vendor hex-values to actual vendor names.
    pub(crate) arm_vendors: HashMap<String, String>,

    /// Maps from macOS feature flags to the expected feature names.
    pub(crate) darwin_flags: HashMap<String, String>,
}

/// Deserialization helper to map {null, string, [string]} to a sequence of strings.
fn zero_one_many_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Vtor;

    impl<'de> de::Visitor<'de> for Vtor {
        type Value = Vec<String>;

        fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str("a null or a loose element or a sequence")
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![])
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![v.to_string()])
        }

        fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut v = Vec::with_capacity(access.size_hint().unwrap_or(0));
            while let Some(a) = access.next_element()? {
                v.push(a);
            }

            Ok(v)
        }
    }

    deserializer.deserialize_any(Vtor)
}

/// Deserialization helper to map from a single object or a sequence of objects to a sequence.
fn one_many_object<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Vtor<T> {
        marker: PhantomData<fn() -> Vec<T>>,
    };

    impl<T> Vtor<T> {
        fn new() -> Self {
            Vtor {
                marker: PhantomData,
            }
        }
    }

    impl<'de, T> de::Visitor<'de> for Vtor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            fmt.write_str("a loose element or a sequence")
        }

        fn visit_map<A>(self, access: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let obj: T = Deserialize::deserialize(de::value::MapAccessDeserializer::new(access))?;
            Ok(vec![obj])
        }

        fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut v = Vec::with_capacity(access.size_hint().unwrap_or(0));
            while let Some(a) = access.next_element()? {
                v.push(a);
            }

            Ok(v)
        }
    }

    deserializer.deserialize_any(Vtor::new())
}

lazy_static! {
    /// Underlying dataset from the archspec JSON file.
    pub static ref TARGETS_JSON: Schema = {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/json/cpu/microarchitectures.json"
        )))
        .expect("Failed to load microarchitectures.json")
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn show_json() {
        let json: &super::Schema = &super::TARGETS_JSON;
        println!("{:#?}", json);
    }
}
