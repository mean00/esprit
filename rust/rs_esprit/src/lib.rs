#![no_std]

//! High-level peripheral abstraction traits for Esprit HAL.
//!
//! Platform-specific driver crates (`rs_*_bluepill`, `rs_*_rp2xxx`, etc.) implement
//! these traits to provide uniform, event-driven driver capabilities across targets.

pub mod uart {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Parity {
        None,
        Even,
        Odd,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum StopBits {
        One,
        Two,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct UartConfig {
        pub baudrate: u32,
        pub data_bits: u8,
        pub stop_bits: StopBits,
        pub parity: Parity,
    }

    impl Default for UartConfig {
        fn default() -> Self {
            Self {
                baudrate: 115200,
                data_bits: 8,
                stop_bits: StopBits::One,
                parity: Parity::None,
            }
        }
    }

    pub trait UartRxHandler: Sync {
        fn on_rx_byte(&self, byte: u8);
        fn on_rx_buffer(&self, _buffer: &[u8]) {}
        fn on_error(&self) {}
    }

    pub trait UartTxHandler: Sync {
        fn on_tx_complete(&self) {}
    }

    pub trait UartTx {
        fn configure(&mut self, config: &UartConfig) -> bool;
        fn set_speed(&mut self, baudrate: u32) -> bool;
        fn transmit(&mut self, buffer: &[u8]) -> bool;
        fn transmit_no_block(&mut self, buffer: &[u8]) -> i32 {
            if self.transmit(buffer) {
                buffer.len() as i32
            } else {
                -1
            }
        }
        fn set_tx_handler(&mut self, handler: Option<&'static dyn UartTxHandler>);
    }

    pub trait UartRx {
        fn configure(&mut self, config: &UartConfig) -> bool;
        fn set_speed(&mut self, baudrate: u32) -> bool;
        fn enable_rx(&mut self, enabled: bool) -> bool;
        fn set_rx_handler(&mut self, handler: Option<&'static dyn UartRxHandler>);
    }
}

pub mod spi {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SpiMode {
        Mode0,
        Mode1,
        Mode2,
        Mode3,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SpiBitOrder {
        MsbFirst,
        LsbFirst,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SpiConfig {
        pub speed_hz: u32,
        pub mode: SpiMode,
        pub bit_order: SpiBitOrder,
        pub data_size: u8,
    }

    impl Default for SpiConfig {
        fn default() -> Self {
            Self {
                speed_hz: 1_000_000,
                mode: SpiMode::Mode0,
                bit_order: SpiBitOrder::MsbFirst,
                data_size: 8,
            }
        }
    }

    pub trait SpiRxHandler: Sync {
        fn on_rx_byte(&self, byte: u8);
        fn on_rx_buffer(&self, _buffer: &[u8]) {}
    }

    pub trait SpiTxHandler: Sync {
        fn on_tx_complete(&self) {}
    }

    pub trait Spi {
        fn configure(&mut self, config: &SpiConfig) -> bool;
        fn transfer8(&mut self, val: u8) -> u8;
        fn write_async(&mut self, data: &[u8]) -> bool;
        fn set_rx_handler(&mut self, handler: Option<&'static dyn SpiRxHandler>);
        fn set_tx_handler(&mut self, handler: Option<&'static dyn SpiTxHandler>);

        // Synchronous / blocking block operations
        fn write_slice8(&mut self, data: &[u8]) -> bool {
            for &byte in data {
                self.transfer8(byte);
            }
            true
        }
        fn write_slice16(&mut self, data: &[u16]) -> bool {
            for &word in data {
                self.transfer8((word >> 8) as u8);
                self.transfer8(word as u8);
            }
            true
        }
        fn fill8(&mut self, count: u32, val: u8) -> bool {
            for _ in 0..count {
                self.transfer8(val);
            }
            true
        }
        fn fill16(&mut self, count: u32, val: u16) -> bool {
            for _ in 0..count {
                self.transfer8((val >> 8) as u8);
                self.transfer8(val as u8);
            }
            true
        }
        fn transfer_exact(&mut self, tx: &[u8], rx: &mut [u8]) -> bool {
            if tx.len() != rx.len() {
                return false;
            }
            for (t, r) in tx.iter().zip(rx.iter_mut()) {
                *r = self.transfer8(*t);
            }
            true
        }
        fn set_speed(&mut self, speed_hz: u32);
        fn set_dma_mode(&mut self, enable: bool);
        fn set_ssel(&mut self, ssel: i32);
        fn wait_done(&mut self) -> bool {
            true
        }
    }
}

pub mod timer {
    pub trait TimerHandler: Sync {
        fn on_timer_tick(&self);
    }

    pub trait Timer {
        fn set_frequency(&mut self, frequency_hz: u32) -> Result<u32, &'static str>;
        fn enable(&mut self);
        fn disable(&mut self);
        fn set_handler(&mut self, handler: Option<&'static dyn TimerHandler>);
        fn single_shot(&mut self, duration_ms: u32, up: bool);
    }

    pub trait DelayTimer {
        fn arm(&mut self, duration_us: u32);
        fn set_interrupt(
            &mut self,
            handler: Option<unsafe extern "C" fn(cookie: *mut core::ffi::c_void)>,
            cookie: *mut core::ffi::c_void,
        );
        fn enable_interrupt(&mut self);
        fn disable_interrupt(&mut self);
    }
}

pub mod i2c {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct I2cConfig {
        pub speed_hz: u32,
        pub own_address: u16,
    }

    impl Default for I2cConfig {
        fn default() -> Self {
            Self {
                speed_hz: 100_000,
                own_address: 0,
            }
        }
    }

    pub trait I2cHandler: Sync {
        fn on_rx_byte(&self, _byte: u8) {}
        fn on_rx_buffer(&self, _buffer: &[u8]) {}
        fn on_tx_complete(&self) {}
        fn on_error(&self) {}
    }

    pub trait I2c {
        fn configure(&mut self, config: &I2cConfig) -> bool;
        fn write_to(&mut self, target: u16, data: &[u8]) -> bool;
        fn read_from(&mut self, target: u16, buffer: &mut [u8]) -> bool;
        fn set_handler(&mut self, handler: Option<&'static dyn I2cHandler>);

        fn set_speed(&mut self, speed_hz: u32);
        fn set_address(&mut self, address: u32);
        fn set_dma_mode(&mut self, enable: bool);
        fn begin(&mut self, target: u8) -> bool;
        fn write(&mut self, data: &[u8]) -> bool;
        fn read(&mut self, buffer: &mut [u8]) -> bool;
        fn multi_write_to(&mut self, target: u8, chunks: &[&[u8]]) -> bool;
    }
}

pub mod adc {
    pub trait AdcHandler: Sync {
        fn on_conversion_complete(&self, sample: u16);
        fn on_sequence_complete(&self, _samples: &[u16]) {}
    }

    pub trait SimpleAdc {
        fn read(&self) -> i32;
        fn set_handler(&mut self, handler: Option<&'static dyn AdcHandler>);
    }

    pub trait TimingAdc {
        fn set_source(&mut self, timer: u32, channel: u32, fq: u32, pins: &[u32]) -> bool;
        fn multi_read(&mut self, nb_sample_per_channel: u32, output: &mut [u16]) -> bool;
        fn set_handler(&mut self, handler: Option<&'static dyn AdcHandler>);
    }
}

pub mod gpio {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum EdgeTrigger {
        None,
        Rising,
        Falling,
        Both,
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum GpioMode {
        Floating = 0,
        InputFloating = 1,
        InputPullUp = 2,
        InputPullDown = 3,
        Output = 4,
        OutputOpenDrain = 5,
        AlternatePushPull = 6,
        AlternateOpenDrain = 7,
        Pwm = 8,
        Adc = 9,
        Dac = 10,
        Uart = 11,
        Spi = 12,
        UartAlt = 13,
    }

    pub trait GpioInterruptHandler: Sync {
        fn on_edge(&self, pin_state: bool);
    }

    pub trait GpioPin {
        fn write(&mut self, value: bool);
        fn read(&self) -> bool;
        fn toggle(&mut self);
        fn is_high(&self) -> bool {
            self.read()
        }
        fn is_low(&self) -> bool {
            !self.read()
        }
        fn set_mode(&mut self, mode: GpioMode);
        fn set_mode_speed(&mut self, mode: GpioMode, speed_mhz: u32);
        fn open_drain_close(&mut self, close: bool);
        fn enable_interrupt(&mut self, trigger: EdgeTrigger) -> bool;
        fn set_handler(&mut self, handler: Option<&'static dyn GpioInterruptHandler>);
    }

    pub trait Exti {
        fn attach_interrupt(
            &mut self,
            pin: u32,
            edge: EdgeTrigger,
            callback: Option<unsafe extern "C" fn(pin: i32, cookie: *mut core::ffi::c_void)>,
            cookie: *mut core::ffi::c_void,
        );
        fn detach_interrupt(&mut self, pin: u32);
        fn enable_interrupt(&mut self, pin: u32);
        fn disable_interrupt(&mut self, pin: u32);
    }
}

pub use rs_rtos as rtos;
pub use rs_rtos::sync;
pub use rs_rtos::queue;
pub use rs_rtos::event;
pub use rs_rtos::task;

pub use rs_rtos::{
    BinarySemaphore, CountingSemaphore, Mutex, MutexGuard, OnceLock,
    RecursiveMutex, RecursiveMutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
    SemaphoreGuard, Queue, EventGroup,
};

// Convenient top-level re-exports
pub use uart::{UartTx, UartRx, UartTxHandler, UartRxHandler, UartConfig};
pub use spi::{Spi, SpiTxHandler, SpiRxHandler, SpiConfig, SpiMode, SpiBitOrder};
pub use timer::{Timer, TimerHandler, DelayTimer};
pub use i2c::{I2c, I2cHandler, I2cConfig};
pub use adc::{SimpleAdc, TimingAdc, AdcHandler};
pub use gpio::{GpioPin, GpioInterruptHandler, EdgeTrigger, GpioMode, Exti};
