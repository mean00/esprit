#![no_std]
#![allow(dead_code)]

pub mod ffi;
pub use ffi::*;

use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::uart;

/// One handler slot per serial instance.
const UART_INSTANCES: usize = 3;

static HANDLERS: HandlerTable<dyn uart::UartHandler, UART_INSTANCES> = HandlerTable::new();

/// The transmit half of the wrapped C ABI.
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

/// The receive half of the wrapped C ABI.
pub struct UartRxWrapped {
    raw: *mut ln_serial_rx_c,
}

impl UartRxWrapped {
    pub fn new(instance: u32, buffer_size: u32, dma: bool) -> Self {
        let raw = unsafe {
            lnserial_rx_create(instance as cty::c_uint, buffer_size as cty::c_uint, dma)
        };
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

/// One wrapped serial port: the C ABI splits TX and RX into two handles, the
/// super trait sees one port, so this type owns both.
pub struct Uart {
    tx: UartTxWrapped,
    rx: UartRxWrapped,
    instance: u32,
}

/// Receive ring size of the wrapped backend (the C driver's default).
const WRAPPED_RX_BUFFER: u32 = 256;

impl Uart {
    /// Open both directions of `instance`.
    pub fn new(instance: u32, config: &uart::UartConfig) -> rs_mcu_traits::Result<Self> {
        let mut uart = Self {
            tx: UartTxWrapped::new(instance, false, true),
            rx: UartRxWrapped::new(instance, 256, false),
            instance,
        };
        uart::Uart::configure(&mut uart, config)?;
        uart::Uart::enable_rx(&mut uart, true)?;
        Ok(uart)
    }

    /// This port's number.
    pub fn instance(&self) -> u32 {
        self.instance
    }

    /// On this platform the C driver already owns a receive ring and a DMA
    /// transmitter, so this is [`Uart::new`] with both enabled: the ring size is
    /// the C driver's default, and its "data available" callback is what
    /// [`uart::Uart::set_handler`] hooks up (`on_rx_available`).
    pub fn with_dma(instance: u32, config: &uart::UartConfig) -> rs_mcu_traits::Result<Self> {
        let mut uart = Self {
            tx: UartTxWrapped::new(instance, true, true),
            rx: UartRxWrapped::new(instance, WRAPPED_RX_BUFFER, true),
            instance,
        };
        uart::Uart::configure(&mut uart, config)?;
        uart::Uart::enable_rx(&mut uart, true)?;
        Ok(uart)
    }
}

impl uart::Uart for Uart {
    fn configure(&mut self, config: &uart::UartConfig) -> rs_mcu_traits::Result<()> {
        unsafe {
            lnserial_tx_set_speed(self.tx.raw, config.baudrate as cty::c_uint);
            lnserial_rx_set_speed(self.rx.raw, config.baudrate as cty::c_uint);
        }
        Ok(())
    }

    fn set_baudrate(&mut self, baudrate: u32) -> rs_mcu_traits::Result<()> {
        unsafe {
            lnserial_tx_set_speed(self.tx.raw, baudrate as cty::c_uint);
            lnserial_rx_set_speed(self.rx.raw, baudrate as cty::c_uint);
        }
        Ok(())
    }

    fn write(&mut self, data: &[u8]) -> rs_mcu_traits::Result<()> {
        let ok = unsafe {
            lnserial_tx_transmit(self.tx.raw, data.len() as cty::c_uint, data.as_ptr())
        };
        if ok {
            if let Some(handler) = HANDLERS.get(self.instance as usize) {
                handler.on_tx_complete();
            }
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    /// The C ABI offers no partial queue: the DMA path is
    /// `lnserial_tx_async_write`.
    fn try_write(&mut self, _data: &[u8]) -> rs_mcu_traits::Result<usize> {
        Err(rs_mcu_traits::Error::Unsupported)
    }

    fn flush(&mut self) -> rs_mcu_traits::Result<()> {
        Ok(())
    }

    fn read(&mut self, buffer: &mut [u8]) -> rs_mcu_traits::Result<usize> {
        let read = unsafe {
            lnserial_rx_read(self.rx.raw, buffer.len() as cty::c_uint, buffer.as_mut_ptr())
        };
        let read = read.max(0) as usize;
        if read > 0 {
            if let Some(handler) = HANDLERS.get(self.instance as usize) {
                handler.on_rx_buffer(&buffer[..read]);
            }
        }
        Ok(read)
    }

    fn read_until_idle(&mut self, buffer: &mut [u8]) -> rs_mcu_traits::Result<usize> {
        self.read(buffer)
    }

    fn enable_rx(&mut self, enabled: bool) -> rs_mcu_traits::Result<()> {
        unsafe {
            lnserial_rx_enable_rx(self.rx.raw, enabled);
        }
        Ok(())
    }

    fn set_handler(&mut self, handler: Option<&'static dyn uart::UartHandler>) {
        HANDLERS.set(self.instance as usize, handler);
    }
}

