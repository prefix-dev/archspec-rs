//! Types and functions to manipulate the contents microarchitecture data file.
//!
//! These are encoding the rules of the corresponding schema as Rust data types
//! with the help of `serde` deserialization.

use serde::de;
use serde::{Deserialize, Deserializer};
use std::marker::PhantomData;

mod cpuid;
mod microarchitecture;

pub use cpuid::*;
pub use microarchitecture::*;

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
#[allow(dead_code)]
fn one_many_object<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Vtor<T> {
        marker: PhantomData<fn() -> Vec<T>>,
    }

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

#[cfg(test)]
mod tests {
    use crate::schema::cpuid::CpuIdSchema;
    use crate::schema::microarchitecture::MicroarchitecturesSchema;

    #[test]
    fn show_microarchitecture_json() {
        println!("{:#?}", MicroarchitecturesSchema::schema());
    }

    #[test]
    fn show_cpuid_json() {
        println!("{:#?}", CpuIdSchema::schema());
    }
}
