#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_esprit::spi::{Spi as SpiTrait, SpiConfig, SpiMode, SpiBitOrder, SpiRxHandler, SpiTxHandler};

pub struct SpiWrapped {
    raw: *mut ln_spi_c,
}

impl SpiWrapped {
    pub fn new(instance: u32, pin_cs: i32) -> Self {
        let raw = unsafe { lnspi_create(instance as cty::c_uint, pin_cs as cty::c_int) };
        assert!(!raw.is_null(), "lnspi_create returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_spi_c {
        self.raw
    }
}

impl Drop for SpiWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                lnspi_delete(self.raw);
            }
        }
    }
}

impl SpiTrait for SpiWrapped {
    fn configure(&mut self, config: &SpiConfig) -> bool {
        unsafe {
            lnspi_begin(self.raw, config.data_size as cty::c_uint);
            lnspi_set_speed(self.raw, config.speed_hz as cty::c_uint);
            let mode = match config.mode {
                SpiMode::Mode0 => spiDataMode_SPI_MODE0,
                SpiMode::Mode1 => spiDataMode_SPI_MODE1,
                SpiMode::Mode2 => spiDataMode_SPI_MODE2,
                SpiMode::Mode3 => spiDataMode_SPI_MODE3,
            };
            lnspi_set_data_mode(self.raw, mode);
            let order = match config.bit_order {
                SpiBitOrder::MsbFirst => spiBitOrder_SPI_MSBFIRST,
                SpiBitOrder::LsbFirst => spiBitOrder_SPI_LSBFIRST,
            };
            lnspi_set_bit_order(self.raw, order);
        }
        true
    }

    fn transfer8(&mut self, val: u8) -> u8 {
        let tx = [val];
        let mut rx = [0u8];
        unsafe {
            lnspi_transfer8(self.raw, 1, tx.as_ptr(), rx.as_mut_ptr());
        }
        rx[0]
    }

    fn write_async(&mut self, data: &[u8]) -> bool {
        unsafe {
            lnspi_block_write8(self.raw, data.len() as cty::c_uint, data.as_ptr())
        }
    }

    fn set_rx_handler(&mut self, _handler: Option<&'static dyn SpiRxHandler>) {}

    fn set_tx_handler(&mut self, _handler: Option<&'static dyn SpiTxHandler>) {}

    fn write_slice8(&mut self, data: &[u8]) -> bool {
        unsafe {
            lnspi_block_write8(self.raw, data.len() as cty::c_uint, data.as_ptr())
        }
    }

    fn write_slice16(&mut self, data: &[u16]) -> bool {
        unsafe {
            lnspi_block_write16(self.raw, data.len() as cty::c_uint, data.as_ptr())
        }
    }

    fn fill8(&mut self, count: u32, val: u8) -> bool {
        unsafe {
            lnspi_block_write8_repeat(self.raw, count as cty::c_uint, val)
        }
    }

    fn fill16(&mut self, count: u32, val: u16) -> bool {
        unsafe {
            lnspi_block_write16_repeat(self.raw, count as cty::c_uint, val)
        }
    }

    fn transfer_exact(&mut self, tx: &[u8], rx: &mut [u8]) -> bool {
        if tx.len() != rx.len() {
            return false;
        }
        unsafe {
            lnspi_transfer8(self.raw, tx.len() as cty::c_uint, tx.as_ptr(), rx.as_mut_ptr())
        }
    }

    fn set_speed(&mut self, speed_hz: u32) {
        unsafe {
            lnspi_set_speed(self.raw, speed_hz as cty::c_uint);
        }
    }

    fn set_dma_mode(&mut self, enable: bool) {
        unsafe {
            lnspi_set_dma_mode(self.raw, enable);
        }
    }

    fn set_ssel(&mut self, ssel: i32) {
        unsafe {
            lnspi_set_ssel(self.raw, ssel as cty::c_int);
        }
    }

    fn wait_done(&mut self) -> bool {
        unsafe {
            lnspi_wait_for_completion(self.raw)
        }
    }
}
