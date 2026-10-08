#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

pub type lnPin = cty::c_int;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum lnGpioMode {
    lnFLOATING = 0,
    lnINPUT_FLOATING = 1,
    lnINPUT_PULLUP = 2,
    lnINPUT_PULLDOWN = 3,
    lnOUTPUT = 4,
    lnOUTPUT_OPEN_DRAIN = 5,
    lnALTERNATE_PP = 6,
    lnALTERNATE_OD = 7,
    lnPWM = 8,
    lnADC_MODE = 9,
    lnDAC_MODE = 10,
    lnUART = 11,
    lnSPI_MODE = 12,
    lnUART_Alt = 13,
}

unsafe extern "C" {
    pub fn lnPinMode(pin: lnPin, mode: lnGpioMode, speedInMhz: cty::c_uint);
    pub fn lnPinMode_c(pin: lnPin, mode: lnGpioMode, speedInMhz: cty::c_uint);
    pub fn lnDigitalWrite(pin: lnPin, value: bool);
    pub fn lnDigitalRead(pin: lnPin) -> bool;
    pub fn lnDigitalToggle(pin: lnPin);
    pub fn lnOpenDrainClose(pin: lnPin, close: bool);
    pub fn lnGetGpioToggleRegister(port: cty::c_uint) -> *mut cty::c_uint;
    pub fn lnGetGpioDirectionRegister(port: cty::c_uint) -> *mut cty::c_uint;
    pub fn lnGetGpioValueRegister(port: cty::c_uint) -> *mut cty::c_uint;
    pub fn lnGetGpioOnRegister(port: cty::c_uint) -> *mut cty::c_uint;
    pub fn lnGetGpioOffRegister(port: cty::c_uint) -> *mut cty::c_uint;
    pub fn lnReadPort(port: cty::c_uint) -> cty::c_uint;
}
