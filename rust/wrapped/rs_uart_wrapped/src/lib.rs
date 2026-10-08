#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_esprit::uart::{UartTx as UartTxTrait, UartRx as UartRxTrait, UartConfig, UartTxHandler, UartRxHandler};

pub struct UartTxWrapped {
    raw: *mut ln_serial_tx_c,
}

impl UartTxWrapped {
    pub fn new(instance: u32, dma: bool, buffered: bool) -> Self {
        let raw = unsafe { lnserial_tx_create(instance as cty::c_uint, dma, buffered) };
        assert!(!raw.is_null(), "lnserial_tx_create returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_serial_tx_c {
        self.raw
    }
}

impl Drop for UartTxWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                lnserial_tx_delete(self.raw);
            }
        }
    }
}

impl UartTxTrait for UartTxWrapped {
    fn configure(&mut self, config: &UartConfig) -> bool {
        self.set_speed(config.baudrate)
    }

    fn set_speed(&mut self, baudrate: u32) -> bool {
        unsafe {
            lnserial_tx_set_speed(self.raw, baudrate as cty::c_uint)
        }
    }

    fn transmit(&mut self, buffer: &[u8]) -> bool {
        unsafe {
            lnserial_tx_transmit(self.raw, buffer.len() as cty::c_uint, buffer.as_ptr())
        }
    }

    fn transmit_no_block(&mut self, buffer: &[u8]) -> i32 {
        if self.transmit(buffer) {
            buffer.len() as i32
        } else {
            -1
        }
    }

    fn set_tx_handler(&mut self, _handler: Option<&'static dyn UartTxHandler>) {}
}

pub struct UartRxWrapped {
    raw: *mut ln_serial_rx_c,
}

impl UartRxWrapped {
    pub fn new(instance: u32, rx_buffer_size: u32, dma: bool) -> Self {
        let raw = unsafe { lnserial_rx_create(instance as cty::c_uint, rx_buffer_size as cty::c_uint, dma) };
        assert!(!raw.is_null(), "lnserial_rx_create returned NULL");
        Self { raw }
    }

    pub fn raw(&self) -> *mut ln_serial_rx_c {
        self.raw
    }
}

impl Drop for UartRxWrapped {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                lnserial_rx_delete(self.raw);
            }
        }
    }
}

impl UartRxTrait for UartRxWrapped {
    fn configure(&mut self, config: &UartConfig) -> bool {
        self.set_speed(config.baudrate)
    }

    fn set_speed(&mut self, baudrate: u32) -> bool {
        unsafe {
            lnserial_rx_set_speed(self.raw, baudrate as cty::c_uint)
        }
    }

    fn enable_rx(&mut self, enabled: bool) -> bool {
        unsafe {
            lnserial_rx_enable_rx(self.raw, enabled)
        }
    }

    fn set_rx_handler(&mut self, _handler: Option<&'static dyn UartRxHandler>) {}
}
