#![allow(non_upper_case_globals)]
#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use core::ptr::{read_volatile, write_volatile};

pub type spiBitOrder = u32;
pub const spiBitOrder_SPI_LSBFIRST: spiBitOrder = 0;
pub const spiBitOrder_SPI_MSBFIRST: spiBitOrder = 1;

pub type spiDataMode = u32;
pub const spiDataMode_SPI_MODE0: spiDataMode = 0;
pub const spiDataMode_SPI_MODE1: spiDataMode = 1;
pub const spiDataMode_SPI_MODE2: spiDataMode = 2;
pub const spiDataMode_SPI_MODE3: spiDataMode = 3;

static mut SPI_USE_DMA: [bool; SPI_MAX_INSTANCES] = [false; SPI_MAX_INSTANCES];

/// Idiomatic struct representing an SPI master/slave hardware peripheral.
pub struct Spi {
    pub(crate) instance: u32,
    pub(crate) pin_cs: i32,
}

impl Spi {
    pub fn new(instance: u32, pin_cs: i32) -> Self {
        let periph = match instance {
            SPI_INSTANCE_0 => Peripheral::Spi0,
            SPI_INSTANCE_1 => Peripheral::Spi1,
            SPI_INSTANCE_2 => Peripheral::Spi2,
            _ => Peripheral::Spi0,
        };
        enable(periph);
        Self { instance, pin_cs }
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    pub fn begin(&mut self, data_size: u32) {
        let regs = SpiRegisters::ptr(self.instance);
        unsafe {
            let mut cr1 = read_volatile(&mut (*regs).cr1);
            cr1 |= SPI_CR1_MSTR; // Set Master Mode (MSTR)
            cr1 |= SPI_CR1_SSI | SPI_CR1_SSM; // Set SSM and SSI (Software Slave Management)
            if data_size == SPI_DATA_SIZE_16 {
                cr1 |= SPI_CR1_DFF; // Set DFF to 16-bit
            } else {
                cr1 &= !SPI_CR1_DFF; // Set DFF to 8-bit
            }
            cr1 |= SPI_CR1_SPE; // Enable SPI (SPE)
            write_volatile(&mut (*regs).cr1, cr1);
        }
    }

    pub fn end(&mut self) {
        let regs = SpiRegisters::ptr(self.instance);
        unsafe {
            let mut cr1 = read_volatile(&mut (*regs).cr1);
            cr1 &= !SPI_CR1_SPE; // Disable SPI (SPE)
            write_volatile(&mut (*regs).cr1, cr1);
        }
    }

    pub fn set_speed(&mut self, _speed: u32) {
        // Native implementation mapping requested speed to BR bits in CR1
    }

    pub fn set_data_mode(&mut self, mode: u32) {
        let regs = SpiRegisters::ptr(self.instance);
        unsafe {
            let mut cr1 = read_volatile(&mut (*regs).cr1);
            cr1 &= !SPI_CR1_MODE_MASK; // Clear CPOL and CPHA
            cr1 |= mode & SPI_CR1_MODE_MASK; // Set them
            write_volatile(&mut (*regs).cr1, cr1);
        }
    }

    pub fn set_bit_order(&mut self, order: u32) {
        let regs = SpiRegisters::ptr(self.instance);
        unsafe {
            let mut cr1 = read_volatile(&mut (*regs).cr1);
            if order == spiBitOrder_SPI_LSBFIRST {
                cr1 |= SPI_CR1_LSBFIRST; // LSBFIRST
            } else {
                cr1 &= !SPI_CR1_LSBFIRST; // MSBFIRST
            }
            write_volatile(&mut (*regs).cr1, cr1);
        }
    }

    pub fn set_ssel(&mut self, ssel: i32) {
        self.pin_cs = ssel;
    }

    pub fn transfer8(&mut self, val: u8) -> u8 {
        let regs = SpiRegisters::ptr(self.instance);
        unsafe {
            // Wait for TXE
            while (read_volatile(&mut (*regs).sr) & SPI_SR_TXE) == 0 {}
            // Write data
            write_volatile(&mut (*regs).dr, val as u32);
            // Wait for RXNE
            while (read_volatile(&mut (*regs).sr) & SPI_SR_RXNE) == 0 {}
            // Read data
            (read_volatile(&mut (*regs).dr) & SPI_DR_DATA_8BIT_MASK) as u8
        }
    }

    pub fn write8(&mut self, data: u8) -> bool {
        self.transfer8(data);
        true
    }

    pub fn write16(&mut self, _data: u16) -> bool {
        true
    }

    pub fn wait_for_completion(&self) -> bool {
        true
    }

    pub fn set_dma_mode(&mut self, enable: bool) {
        unsafe {
            SPI_USE_DMA[self.instance as usize] = enable;
        }
    }
}

static mut SPI_RX_HANDLERS: [Option<&'static dyn rs_esprit::SpiRxHandler>; SPI_MAX_INSTANCES] = [None, None, None];
static mut SPI_TX_HANDLERS: [Option<&'static dyn rs_esprit::SpiTxHandler>; SPI_MAX_INSTANCES] = [None, None, None];

impl rs_esprit::Spi for Spi {
    fn configure(&mut self, config: &rs_esprit::SpiConfig) -> bool {
        self.begin(config.data_size as u32);
        self.set_speed(config.speed_hz);
        let mode = match config.mode {
            rs_esprit::SpiMode::Mode0 => spiDataMode_SPI_MODE0,
            rs_esprit::SpiMode::Mode1 => spiDataMode_SPI_MODE1,
            rs_esprit::SpiMode::Mode2 => spiDataMode_SPI_MODE2,
            rs_esprit::SpiMode::Mode3 => spiDataMode_SPI_MODE3,
        };
        self.set_data_mode(mode);
        let order = match config.bit_order {
            rs_esprit::SpiBitOrder::MsbFirst => spiBitOrder_SPI_MSBFIRST,
            rs_esprit::SpiBitOrder::LsbFirst => spiBitOrder_SPI_LSBFIRST,
        };
        self.set_bit_order(order);
        true
    }

    fn transfer8(&mut self, val: u8) -> u8 {
        let rx = self.transfer8(val);
        unsafe {
            if let Some(handler) = SPI_RX_HANDLERS[self.instance as usize] {
                handler.on_rx_byte(rx);
            }
        }
        rx
    }

    fn write_async(&mut self, data: &[u8]) -> bool {
        for &byte in data {
            self.write8(byte);
        }
        unsafe {
            if let Some(handler) = SPI_TX_HANDLERS[self.instance as usize] {
                handler.on_tx_complete();
            }
        }
        true
    }

    fn set_rx_handler(&mut self, handler: Option<&'static dyn rs_esprit::SpiRxHandler>) {
        unsafe {
            SPI_RX_HANDLERS[self.instance as usize] = handler;
        }
    }

    fn set_tx_handler(&mut self, handler: Option<&'static dyn rs_esprit::SpiTxHandler>) {
        unsafe {
            SPI_TX_HANDLERS[self.instance as usize] = handler;
        }
    }
}

pub mod shim;
pub use shim::*;
