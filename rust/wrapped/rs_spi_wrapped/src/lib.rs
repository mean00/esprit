#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use core::ffi::c_void;
use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::spi;

/// One handler slot per SPI instance (the C ABI is instance-addressed).
const SPI_INSTANCES: usize = 2;

static HANDLERS: HandlerTable<dyn spi::SpiHandler, SPI_INSTANCES> = HandlerTable::new();

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

    /// One byte exchange over the C ABI.
    pub fn transfer_byte(&mut self, val: u8) -> u8 {
        let tx = [val];
        let mut rx = [0u8; 1];
        unsafe {
            lnspi_transfer8(self.raw, 1, tx.as_ptr(), rx.as_mut_ptr());
        }
        rx[0]
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

impl spi::Spi for SpiWrapped {
    fn configure(&mut self, config: &spi::SpiConfig) -> rs_mcu_traits::Result<()> {
        unsafe {
            lnspi_begin(self.raw, config.data_size as cty::c_uint);
            lnspi_set_speed(self.raw, config.speed_hz as cty::c_uint);
            lnspi_set_data_mode(
                self.raw,
                match config.mode {
                    spi::SpiMode::Mode0 => spiDataMode_SPI_MODE0,
                    spi::SpiMode::Mode1 => spiDataMode_SPI_MODE1,
                    spi::SpiMode::Mode2 => spiDataMode_SPI_MODE2,
                    spi::SpiMode::Mode3 => spiDataMode_SPI_MODE3,
                },
            );
            lnspi_set_bit_order(
                self.raw,
                match config.bit_order {
                    spi::BitOrder::MsbFirst => spiBitOrder_SPI_MSBFIRST,
                    spi::BitOrder::LsbFirst => spiBitOrder_SPI_LSBFIRST,
                },
            );
        }
        Ok(())
    }

    fn set_speed(&mut self, speed_hz: u32) -> rs_mcu_traits::Result<()> {
        unsafe {
            lnspi_set_speed(self.raw, speed_hz as cty::c_uint);
        }
        Ok(())
    }

    fn transfer(&mut self, byte: u8) -> u8 {
        self.transfer_byte(byte)
    }

    // ---- blocking blocks: the C driver's block paths -------------------------

    fn write_slice(&mut self, data: &[u8]) -> rs_mcu_traits::Result<()> {
        let ok = unsafe {
            lnspi_block_write8(self.raw, data.len() as cty::c_uint, data.as_ptr())
        };
        if ok { Ok(()) } else { Err(rs_mcu_traits::Error::Io) }
    }

    fn write_slice16(&mut self, data: &[u16]) -> rs_mcu_traits::Result<()> {
        let ok = unsafe {
            lnspi_block_write16(self.raw, data.len() as cty::c_uint, data.as_ptr())
        };
        if ok { Ok(()) } else { Err(rs_mcu_traits::Error::Io) }
    }

    fn fill(&mut self, count: u32, byte: u8) -> rs_mcu_traits::Result<()> {
        let ok = unsafe {
            lnspi_block_write8_repeat(self.raw, count as cty::c_uint, byte)
        };
        if ok { Ok(()) } else { Err(rs_mcu_traits::Error::Io) }
    }

    fn fill16(&mut self, count: u32, word: u16) -> rs_mcu_traits::Result<()> {
        let ok = unsafe {
            lnspi_block_write16_repeat(self.raw, count as cty::c_uint, word)
        };
        if ok { Ok(()) } else { Err(rs_mcu_traits::Error::Io) }
    }

    // ---- asynchronous: DMA, completion in the interrupt ----------------------

    fn start_write(&mut self, data: &[u8]) -> rs_mcu_traits::Result<()> {
        let callback = HANDLERS
            .get(0)
            .map(|_| tx_complete_trampoline as unsafe extern "C" fn(*mut c_void));
        let ok = unsafe {
            lnspi_asyncWrite8(
                self.raw,
                data.len() as cty::c_uint,
                data.as_ptr(),
                callback,
                core::ptr::null_mut(),
                false,
            )
        };
        if ok { Ok(()) } else { Err(rs_mcu_traits::Error::Io) }
    }

    fn start_write16(&mut self, data: &[u16]) -> rs_mcu_traits::Result<()> {
        let callback = HANDLERS
            .get(0)
            .map(|_| tx_complete_trampoline as unsafe extern "C" fn(*mut c_void));
        let ok = unsafe {
            lnspi_asyncWrite16(
                self.raw,
                data.len() as cty::c_uint,
                data.as_ptr(),
                callback,
                core::ptr::null_mut(),
                false,
            )
        };
        if ok { Ok(()) } else { Err(rs_mcu_traits::Error::Io) }
    }

    fn wait_idle(&mut self) -> rs_mcu_traits::Result<()> {
        if unsafe { lnspi_waitForAsync(self.raw) } {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Timeout)
        }
    }

    fn set_handler(&mut self, handler: Option<&'static dyn spi::SpiHandler>) {
        // The C ABI has no per-instance handler slot; index 0 keeps the trait's
        // contract for the single-bus case a wrapped target uses.
        HANDLERS.set(0, handler);
    }
}

/// Second step of the async completion: the DMA callback is a raw `extern "C"`
/// function, so the typed handler is looked up by the table above.
unsafe extern "C" fn tx_complete_trampoline(_cookie: *mut core::ffi::c_void) {
    // SAFETY: the handler was installed before the transfer was started.
    if let Some(handler) = unsafe { HANDLERS.isr_get(0) } {
        handler.on_tx_complete();
    }
}

