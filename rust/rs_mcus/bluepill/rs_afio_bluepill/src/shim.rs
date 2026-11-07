use crate::*;

// C-compatibility shims for AFIO
#[inline(always)]
#[allow(non_snake_case)]
pub fn lnAfio_enable() {
    enable_afio();
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnAfio_releaseJtagPins() {
    release_jtag_pins();
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnAfio_remapTimer2Partial() {
    remap_timer2_partial();
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnAfio_selectExtiSource(port: u32, source: u32) {
    select_exti_source(port, source);
}
