use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct CpuIdSchema {
    pub vendor: CpuIdProperty,
    pub highest_extension_support: CpuIdProperty,
    pub flags: Vec<CpuIdFlags>,
    #[serde(rename = "extension-flags")]
    pub extension_flags: Vec<CpuIdFlags>,
}

impl CpuIdSchema {
    pub fn schema() -> &'static CpuIdSchema {
        static SCHEMA: OnceLock<CpuIdSchema> = OnceLock::new();
        SCHEMA.get_or_init(|| {
            serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/json/cpu/cpuid.json"
            )))
            .expect("Failed to load cpuid.json")
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CpuIdProperty {
    pub description: String,
    pub input: CpuIdInput,
}

#[derive(Debug, Deserialize)]
pub struct CpuIdFlags {
    pub description: String,
    pub input: CpuIdInput,
    pub bits: Vec<CpuIdBits>,
}

#[derive(Debug, Deserialize)]
pub struct CpuIdBits {
    pub name: String,
    pub register: CpuRegister,
    pub bit: u8,
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CpuRegister {
    Eax,
    Ebx,
    Ecx,
    Edx,
}

#[derive(Debug, Deserialize)]
pub struct CpuIdInput {
    pub eax: u32,
    pub ecx: u32,
}
