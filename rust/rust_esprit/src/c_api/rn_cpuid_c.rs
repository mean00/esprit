#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

unsafe extern "C" {
    pub fn ln_cpuid_vendor() -> u32;
    pub fn ln_cpuid_mcu() -> u32;
    pub fn ln_cpuid_mcu_as_string() -> *const cty::c_char;
    pub fn ln_cpuid_flash_size() -> cty::c_int;
    pub fn ln_cpuid_ram_size() -> cty::c_int;
    pub fn ln_cpuid_id_as_string() -> *const cty::c_char;
    pub fn ln_cpuid_clock_speed() -> cty::c_int;
    pub fn ln_cpuid_serial_id() -> u32;
}
