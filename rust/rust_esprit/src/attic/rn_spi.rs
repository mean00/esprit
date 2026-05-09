#![allow(dead_code)]

use crate::rn_gpio::rnPin;
use crate::rn_spi_c as rc;
pub use rc::ln_spi_c as lnSPI;
use rc::*;

type cint = cty::c_int;
type uint = cty::c_uint;

#[derive(Copy, Clone)]
pub enum rnSpiBitOrder {
    SPI_LSBFIRST = 0,
    SPI_MSBFIRST = 1,
}

pub struct rnSPISettings {
    pub pinCS: rnPin,          //CS pin associated to the configuration
    pub speed: u32,            //specifies the spi bus maximum clock speed
    pub bOrder: rnSpiBitOrder, //bit order (MSBFirst or LSBFirst)
    pub dMode: usize,          //one of the data mode
}

pub struct rnSPI {
    ln: *mut lnSPI,
    setting: rnSPISettings,
}

/*
 *
 *
 */
impl rnSPI {
    pub fn new(instance: u32, speed: u32) -> Self {
        unsafe {
            let t: rnSPI = rnSPI {
                ln: lnspi_create(instance as cint, -1 as cint),
                setting: rnSPISettings {
                    pinCS: rnPin::NoPin,
                    speed,
                    bOrder: rnSpiBitOrder::SPI_LSBFIRST,
                    dMode: 0,
                },
            };
            t
        }
    }
    pub fn rn2ln(rn: &rnSPISettings) -> rc::lnSPISettings {
        rc::lnSPISettings {
            pinCS: rn.pinCS as cty::c_int,
            speed: rn.speed as cty::c_uint,
            bOrder: rn.bOrder as rc::spiBitOrder,
            dMode: rn.dMode as rc::spiDataMode,
        }
    }
    pub fn wait_for_completion(&mut self) {
        unsafe {
            rc::lnspi_wait_for_completion(self.ln);
        }
    }
    pub fn set(&mut self, st: &rnSPISettings) {
        unsafe { rc::lnspi_set(self.ln, &rnSPI::rn2ln(st)) }
    }
    pub fn begin(&mut self, sz: usize) {
        unsafe { rc::lnspi_begin(self.ln, sz as cty::c_uint) }
    }
    pub fn end(&mut self) {
        unsafe { rc::lnspi_end(self.ln) }
    }
    pub fn set_speed(&mut self, speed: u32) {
        unsafe {
            rc::lnspi_set_speed(self.ln, speed as cty::c_uint);
        }
    }
    pub fn set_data_mode(&mut self, mode: u8) {
        unsafe {
            rc::lnspi_set_data_mode(self.ln, mode as u32);
        }
    }

    pub fn write8(&mut self, data: u8) -> bool {
        unsafe { rc::lnspi_write8(self.ln, data) }
    }
    pub fn write16(&mut self, data: u16) -> bool {
        unsafe { rc::lnspi_write16(self.ln, data) }
    }
    pub fn block_write16(&mut self, data: &[u16]) -> bool {
        unsafe { rc::lnspi_block_write16(self.ln, data.len() as u32, data.as_ptr()) }
    }
    pub fn block_write8(&mut self, data: &[u8]) -> bool {
        unsafe { rc::lnspi_block_write8(self.ln, data.len() as u32, data.as_ptr()) }
    }
    pub fn block_write16_repeat(&mut self, nb: usize, data: u16) -> bool {
        unsafe { rc::lnspi_block_write16_repeat(self.ln, nb as u32, data) }
    }
    pub fn block_write8_repeat(&mut self, nb: usize, data: u8) -> bool {
        unsafe { rc::lnspi_block_write8_repeat(self.ln, nb as u32, data) }
    }
    pub fn transfer8(&mut self, tx: &[u8], rx: &mut [u8]) -> bool {
        unsafe { rc::lnspi_transfer8(self.ln, tx.len() as u32, tx.as_ptr(), rx.as_mut_ptr()) }
    }
}
// eof
