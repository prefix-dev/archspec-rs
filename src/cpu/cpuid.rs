#![allow(dead_code)]

use crate::schema::{CpuIdSchema, CpuRegister};
use std::arch::x86_64::__cpuid_count;
use std::collections::HashSet;
use std::ffi::CStr;
use std::sync::OnceLock;

#[derive(Debug)]
pub(crate) struct CpuId {
    /// The vendor name of the CPU
    pub vendor: String,

    /// Optional brand name of the CPU
    pub brand: Option<String>,

    /// The supported features of the CPU.
    pub features: HashSet<String>,
}

impl CpuId {
    fn detect() -> Self {
        let schema = CpuIdSchema::schema();

        // Read the vendor information
        let registers = unsafe { __cpuid_count(schema.vendor.input.eax, schema.vendor.input.ecx) };
        let highest_basic_support = registers.eax;
        let vendor_bytes: [u8; 12] =
            unsafe { std::mem::transmute([registers.ebx, registers.edx, registers.ecx]) };
        let vendor = String::from_utf8_lossy(&vendor_bytes).into_owned();

        // Read the highest_extension_support
        let registers = unsafe {
            __cpuid_count(
                schema.highest_extension_support.input.eax,
                schema.highest_extension_support.input.ecx,
            )
        };
        let highest_extension_support = registers.eax;

        // Read feature flags
        let mut features = HashSet::new();
        let supported_flags = schema
            .flags
            .iter()
            .filter(|flags| flags.input.eax <= highest_basic_support);
        let supported_extensions = schema
            .extension_flags
            .iter()
            .filter(|flags| flags.input.eax <= highest_extension_support);
        for flags in supported_flags.chain(supported_extensions) {
            let registers = unsafe { __cpuid_count(flags.input.eax, flags.input.ecx) };
            for bits in &flags.bits {
                let register = match bits.register {
                    CpuRegister::Eax => registers.eax,
                    CpuRegister::Ebx => registers.ebx,
                    CpuRegister::Ecx => registers.ecx,
                    CpuRegister::Edx => registers.edx,
                };
                if register & (1 << bits.bit) != 0 {
                    features.insert(bits.name.clone());
                }
            }
        }

        // Read brand name if supported.
        let brand = if highest_extension_support >= 0x80000004 {
            let registers = unsafe {
                (
                    __cpuid_count(0x80000002, 0),
                    __cpuid_count(0x80000003, 0),
                    __cpuid_count(0x80000004, 0),
                )
            };

            let vendor_bytes: [u8; 48] = unsafe {
                std::mem::transmute([
                    registers.0.eax,
                    registers.0.ebx,
                    registers.0.ecx,
                    registers.0.edx,
                    registers.1.eax,
                    registers.1.ebx,
                    registers.1.ecx,
                    registers.1.edx,
                    registers.2.eax,
                    registers.2.ebx,
                    registers.2.ecx,
                    registers.2.edx,
                ])
            };
            let brand_string = match CStr::from_bytes_until_nul(&vendor_bytes) {
                Ok(cstr) => cstr.to_string_lossy(),
                Err(_) => String::from_utf8_lossy(&vendor_bytes),
            };
            Some(brand_string.trim().to_string())
        } else {
            None
        };

        Self {
            vendor,
            features,
            brand,
        }
    }

    /// Returns the CPUID information for the host CPU.
    pub fn host() -> &'static Self {
        static CPUID: OnceLock<CpuId> = OnceLock::new();
        CPUID.get_or_init(CpuId::detect)
    }
}
