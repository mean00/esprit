use crate::*;

pub type lnEdge = Edge;
pub type lnExtiCallback = Callback;

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiAttachInterrupt_c(
    pin: rs_gpio_bluepill::lnPin,
    edge: lnEdge,
    cb: lnExtiCallback,
    cookie: *mut core::ffi::c_void,
) {
    attach_interrupt(pin, edge, cb, cookie);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiDetachInterrupt_c(pin: rs_gpio_bluepill::lnPin) {
    detach_interrupt(pin);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiEnableInterrupt_c(pin: rs_gpio_bluepill::lnPin) {
    enable_interrupt(pin);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnExtiDisableInterrupt_c(pin: rs_gpio_bluepill::lnPin) {
    disable_interrupt(pin);
}
