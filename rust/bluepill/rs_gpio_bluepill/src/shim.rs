use crate::*;

pub type lnPin = Pin;
pub type lnGpioMode = Mode;

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnDigitalWrite(pin: lnPin, value: bool) {
    write(pin, value);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnDigitalRead(pin: lnPin) -> bool {
    read(pin)
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnDigitalToggle(pin: lnPin) {
    toggle(pin);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnOpenDrainClose(pin: lnPin, close: bool) {
    open_drain_close(pin, close);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnReadPort(port: u32) -> u32 {
    read_port(port)
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnPinMode_c(pin: lnPin, mode: lnGpioMode, speed_in_mhz: u32) {
    set_mode(pin, mode, speed_in_mhz);
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnGetGpioToggleRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).bop) }
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnGetGpioDirectionRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).ctl0) }
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnGetGpioValueRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).istat) }
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnGetGpioOnRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).bop) }
}

#[inline(always)]
#[allow(non_snake_case)]
pub fn lnGetGpioOffRegister(port: u32) -> *mut u32 {
    unsafe { core::ptr::addr_of_mut!((*get_port_ptr(port)).bc) }
}
