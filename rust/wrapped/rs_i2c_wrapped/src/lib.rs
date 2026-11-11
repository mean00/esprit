#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::i2c::{self, MAX_CHUNKS};

/// One handler slot: the wrapped C ABI has a single I²C bus per instance.
const I2C_INSTANCES: usize = 2;

static HANDLERS: HandlerTable<dyn i2c::I2cHandler, I2C_INSTANCES> = HandlerTable::new();

pub struct I2cWrapped {
    raw: *mut ln_i2c_c,
}

impl I2cWrapped {
    pub fn new(instance: u32, speed_hz: u32) -> Self {
        let raw = unsafe { lni2c_create(instance as cty::c_uint, speed_hz as cty::c_uint) };
        assert!(!raw.is_null(), "lni2c_create returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_i2c_c {
        self.raw
    }
}

impl Drop for I2cWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                lni2c_delete(self.raw);
            }
        }
    }
}

impl i2c::I2c for I2cWrapped {
    fn configure(&mut self, config: &i2c::I2cConfig) -> rs_mcu_traits::Result<()> {
        unsafe {
            lni2c_setSpeed(self.raw, config.speed_hz as cty::c_uint);
            lni2c_setAddress(self.raw, config.own_address as cty::c_uint);
        }
        Ok(())
    }

    fn set_speed(&mut self, speed_hz: u32) -> rs_mcu_traits::Result<()> {
        unsafe {
            lni2c_setSpeed(self.raw, speed_hz as cty::c_uint);
        }
        Ok(())
    }

    /// START, the operations in order, STOP — the shape the C driver implements.
    fn transfer(&mut self, target: u8, operations: &mut [i2c::Operation<'_>]) -> rs_mcu_traits::Result<()> {
        let started = unsafe { lni2c_begin(self.raw, target as cty::c_uint) };
        if !started {
            return Err(rs_mcu_traits::Error::Io);
        }
        for operation in operations.iter_mut() {
            let ok = match operation {
                i2c::Operation::Write(data) => unsafe {
                    lni2c_write(self.raw, data.len() as cty::c_uint, data.as_ptr())
                },
                i2c::Operation::Read(buffer) => unsafe {
                    lni2c_read(self.raw, buffer.len() as cty::c_uint, buffer.as_mut_ptr())
                },
            };
            if !ok {
                return Err(rs_mcu_traits::Error::Io);
            }
        }
        Ok(())
    }

    fn set_handler(&mut self, handler: Option<&'static dyn i2c::I2cHandler>) {
        HANDLERS.set(0, handler);
    }

    /// Asynchronous multi-chunk write over the C ABI (`lni2c_multi_write_to`):
    /// the transaction runs from the bus interrupt and reports through
    /// `on_tx_complete`.
    fn start_write_chunks(&mut self, target: u8, chunks: &[&[u8]]) -> rs_mcu_traits::Result<()> {
        if chunks.is_empty() || chunks.len() > MAX_CHUNKS {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        }
        let mut lengths = [0u32; MAX_CHUNKS];
        let mut pointers = [core::ptr::null::<u8>(); MAX_CHUNKS];
        for (index, chunk) in chunks.iter().enumerate() {
            lengths[index] = chunk.len() as cty::c_uint;
            pointers[index] = chunk.as_ptr();
        }
        let started = unsafe {
            lni2c_multi_write_to(
                self.raw,
                target as cty::c_uint,
                chunks.len() as cty::c_uint,
                lengths.as_ptr(),
                pointers.as_mut_ptr(),
            )
        };
        if started {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    /// Block until the queued sequence is out. The wrapped C ABI exposes no
    /// completion poll (its `begin` blocks instead), so this keeps the trait's
    /// default and the completion is reported through `on_tx_complete`.
    fn wait_idle(&mut self) -> rs_mcu_traits::Result<()> {
        Ok(())
    }
}


