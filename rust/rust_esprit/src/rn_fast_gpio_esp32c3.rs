#![allow(dead_code)]

use crate::rn_gpio::{pinMode, rnPin};
use crate::rn_gpio_esp32_c as gpio;
use gpio::{lnGetGpioOffRegister, lnGetGpioOnRegister, lnGpioMode as rnGpioMode};
//use crate::rn_gpio_esp32_c ;
//
//
pub struct rnFastIO {
    on_adr: *mut u32,
    off_adr: *mut u32,
    on_bit: u32,
}

impl rnFastIO {
    pub fn new(pin: rnPin) -> Self {
        let bit: u32 = 1_u32 << ((pin as u32) & 0x0f);
        pinMode(pin, rnGpioMode::lnOUTPUT);
        let r: rnFastIO;
        unsafe {
            r = rnFastIO {
                on_adr: lnGetGpioOnRegister(0),
                off_adr: lnGetGpioOffRegister(0),
                on_bit: bit,
            };
        }
        r
    }
    #[inline(always)]
    pub fn on(&mut self) {
        unsafe {
            *self.on_adr = self.on_bit;
        }
    }
    #[inline(always)]
    pub fn off(&mut self) {
        unsafe {
            *self.off_adr = self.on_bit;
        }
    }
}
