#![allow(dead_code)]
use crate::rn_gpio::{pinMode, rnPin};
use crate::rn_gpio_bp_c as gpio;
use gpio::{lnGetGpioToggleRegister, lnGpioMode as rnGpioMode};

pub struct rnFastIO {
    onoff_adr: *mut u32,
    on_bit: u32,
    off_bit: u32,
}

impl rnFastIO {
    pub fn new(pin: rnPin) -> Self {
        let bit: u32 = 1_u32 << ((pin as u32) & 0x0f);
        pinMode(pin, rnGpioMode::lnOUTPUT);
        let r: rnFastIO;
        unsafe {
            r = rnFastIO {
                onoff_adr: lnGetGpioToggleRegister(((pin as usize) >> 4) as u32),
                on_bit: bit,
                off_bit: bit << 16,
            };
        }
        r
    }

    #[inline(always)]
    pub fn on(&mut self) {
        unsafe {
            *self.onoff_adr = self.on_bit;
        }
    }

    #[inline(always)]
    pub fn off(&mut self) {
        unsafe {
            *self.onoff_adr = self.off_bit;
        }
    }
}