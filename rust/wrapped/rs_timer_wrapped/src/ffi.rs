#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

pub type lnPin = cty::c_int;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_timer_c {
    pub dummy: *mut cty::c_void,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ln_delay_timer_c {
    pub dummy: *mut cty::c_void,
}

pub type DelayTimerCallback = Option<unsafe extern "C" fn(cookie: *mut cty::c_void)>;

unsafe extern "C" {
    pub fn ln_timer_create(timer: cty::c_uint, channel: cty::c_uint) -> *mut ln_timer_c;
    pub fn ln_timer_create_from_pin(pin: lnPin) -> *mut ln_timer_c;
    pub fn ln_timer_delete(timer: *mut ln_timer_c);
    pub fn ln_timer_single_shot(timer: *mut ln_timer_c, durationMs: cty::c_uint, up: bool);

    pub fn ln_delay_timer_create(timer: cty::c_int, channel: cty::c_int) -> *mut ln_delay_timer_c;
    pub fn ln_delay_timer_delete(timer: *mut ln_delay_timer_c);
    pub fn ln_delay_timer_arm(timer: *mut ln_delay_timer_c, delay_us: cty::c_int);
    pub fn ln_delay_timer_set_interrupt(
        timer: *mut ln_delay_timer_c,
        handler: DelayTimerCallback,
        cookie: *mut cty::c_void,
    );
    pub fn ln_delay_timer_enable_interrupt(timer: *mut ln_delay_timer_c);
    pub fn ln_delay_timer_disable_interrupt(timer: *mut ln_delay_timer_c);
}
