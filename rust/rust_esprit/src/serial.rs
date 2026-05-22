#![allow(dead_code)]
use crate::prelude::*;
use crate::rn_serial_c;
use core::ffi::c_void;
use core::ptr;

pub use rn_serial_c::{ln_serial_event_cb, ln_serial_rx_c, ln_serial_tx_c};

//  Serial events (mirrors lnSerialCore::Event)

/// Events that can be delivered by the serial peripheral.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(i32)]
pub enum SerialEvent {
    DataAvailable = 0,
    TxDone = 1,
}

impl SerialEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            SerialEvent::DataAvailable => "dataAvailable",
            SerialEvent::TxDone => "txDone",
        }
    }
}

impl From<i32> for SerialEvent {
    fn from(val: i32) -> Self {
        match val {
            0 => SerialEvent::DataAvailable,
            1 => SerialEvent::TxDone,
            _ => panic!("invalid SerialEvent: {}", val),
        }
    }
}

/// Types that can receive serial events (Rx+Tx variant only).
pub trait SerialEventHandler {
    fn handle(&self, event: SerialEvent);
}

//  SerialTxOnly — transmit‑only serial port

/// Owned handle to a transmit‑only UART.
/// Created via `SerialTxOnly::new(instance, dma, buffered)`.
/// The underlying C++ object is destroyed when `SerialTxOnly` is dropped.
pub struct SerialTxOnly {
    raw: *mut rn_serial_c::ln_serial_tx_c,
}

impl SerialTxOnly {
    /// Open a transmit‑only serial port on hardware UART `instance`.
    /// * `instance` — UART peripheral index (0, 1, 2, …).
    /// * `dma` — use DMA for transmission if `true`.
    /// * `buffered` — use a software Tx ring buffer if `true` (only meaningful
    ///   when `dma` is also `true`).
    pub fn new(instance: u32, dma: bool, buffered: bool) -> Self {
        let raw = unsafe { rn_serial_c::lnserial_tx_create(instance, dma, buffered) };
        assert!(!raw.is_null(), "lnserial_tx_create returned NULL");
        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut rn_serial_c::ln_serial_tx_c {
        self.raw
    }

    /// Initialise the UART hardware.
    pub fn init(&mut self) -> bool {
        unsafe { rn_serial_c::lnserial_tx_init(self.raw) }
    }

    /// Set the baud rate.
    pub fn set_speed(&mut self, speed: u32) -> bool {
        unsafe { rn_serial_c::lnserial_tx_set_speed(self.raw, speed) }
    }

    /// Blocking transmit.  Returns `true` on success.
    pub fn transmit(&mut self, data: &[u8]) -> bool {
        unsafe { rn_serial_c::lnserial_tx_transmit(self.raw, data.len() as u32, data.as_ptr()) }
    }

    /// Low‑level raw write (bypasses any buffering).  Returns `true` on success.
    pub fn raw_write(&mut self, data: &[u8]) -> bool {
        unsafe { rn_serial_c::lnserial_tx_raw_write(self.raw, data.len() as u32, data.as_ptr()) }
    }
}

impl Drop for SerialTxOnly {
    fn drop(&mut self) {
        unsafe { rn_serial_c::lnserial_tx_delete(self.raw); }
    }
}

//  SerialRxTx — bidirectional serial port

/// Owned handle to a bidirectional UART (Rx + Tx).
/// Created via `SerialRxTx::new(instance, rx_buffer_size, dma)`.
/// The underlying C++ object is destroyed when `SerialRxTx` is dropped.
///
/// # Callback safety
/// The callback fires from **interrupt ctx**.  The trampoline recovers
/// the `Box<dyn SerialEventHandler>` from the cookie, calls `handle()`, and
/// re‑boxes it.  Your handler **must not** panic or perform long operations.
pub struct SerialRxTx {
    raw: *mut rn_serial_c::ln_serial_rx_c,
}

impl SerialRxTx {
    /// Open a bidirectional serial port on hardware UART `instance`.
    /// * `instance` — UART peripheral index (0, 1, 2, …).
    /// * `rx_buffer_size` — size of the receive ring buffer in bytes.
    /// * `dma` — use DMA for reception if `true`.
    pub fn new(instance: u32, rx_buffer_size: u32, dma: bool) -> Self {
        let raw = unsafe { rn_serial_c::lnserial_rx_create(instance, rx_buffer_size, dma) };
        assert!(!raw.is_null(), "lnserial_rx_create returned NULL");
        Self { raw }
    }

    /// Open a bidirectional serial port and attach an event handler.
    /// The handler is stored in a `Box` and passed as the callback cookie.
    pub fn with_handler(
        instance: u32,
        rx_buffer_size: u32,
        dma: bool,
        handler: Box<dyn SerialEventHandler>,
    ) -> Self {
        let raw = unsafe { rn_serial_c::lnserial_rx_create(instance, rx_buffer_size, dma) };
        assert!(!raw.is_null(), "lnserial_rx_create returned NULL");

        // Double‑box so the cookie is a thin pointer.
        let cookie = Box::into_raw(Box::new(handler)) as *mut c_void;
        unsafe {
            rn_serial_c::lnserial_rx_set_callback(raw, Some(Self::trampoline), cookie);
        }
        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    pub fn raw(&self) -> *mut rn_serial_c::ln_serial_rx_c {
        self.raw
    }

    /// Initialise the UART hardware.
    pub fn init(&mut self) -> bool {
        unsafe { rn_serial_c::lnserial_rx_init(self.raw) }
    }

    /// Set the baud rate.
    pub fn set_speed(&mut self, speed: u32) -> bool {
        unsafe { rn_serial_c::lnserial_rx_set_speed(self.raw, speed) }
    }

    /// Blocking transmit.  Returns `true` on success.
    pub fn transmit(&mut self, data: &[u8]) -> bool {
        unsafe { rn_serial_c::lnserial_rx_transmit(self.raw, data.len() as u32, data.as_ptr()) }
    }

    /// Non‑blocking transmit.  Returns the num of bytes accepted (may be
    /// less than `data.len()`).
    pub fn transmit_no_block(&mut self, data: &[u8]) -> i32 {
        unsafe {
            rn_serial_c::lnserial_rx_transmit_no_block(self.raw, data.len() as u32, data.as_ptr())
        }
    }

    /// Enable or disable the receiver.
    pub fn enable_rx(&mut self, enabled: bool) -> bool {
        unsafe { rn_serial_c::lnserial_rx_enable_rx(self.raw, enabled) }
    }

    /// Discard any buffered received data.
    pub fn purge_rx(&mut self) {
        unsafe { rn_serial_c::lnserial_rx_purge_rx(self.raw); }
    }

    /// Read up to `buf.len()` bytes into `buf`.  Returns the num of bytes
    /// actually read.
    pub fn read(&mut self, buf: &mut [u8]) -> i32 {
        unsafe { rn_serial_c::lnserial_rx_read(self.raw, buf.len() as u32, buf.as_mut_ptr()) }
    }

    /// Attach (or replace) an event handler.
    /// The prev handler (if any) is **leaked** — the C driver owns the
    /// cookie and there is no way to retrieve it without a full re‑design.
    /// In practice you call this once during initialisation.
    pub fn set_event_handler(&mut self, handler: Box<dyn SerialEventHandler>) {
        let cookie = Box::into_raw(Box::new(handler)) as *mut c_void;
        unsafe {
            rn_serial_c::lnserial_rx_set_callback(self.raw, Some(Self::trampoline), cookie);
        }
    }

    /// Low‑level: get a pointer to the receive ring‑buffer head.
    /// Returns the num of contiguous bytes available, and sets `*to` to
    /// point into the ring buffer.  Call `consume()` afterwards to advance
    /// the read pointer.
    pub fn get_read_pointer(&mut self) -> (i32, *mut u8) {
        let mut ptr: *mut u8 = ptr::null_mut();
        let n = unsafe { rn_serial_c::lnserial_rx_get_read_pointer(self.raw, &mut ptr) };
        (n, ptr)
    }

    /// Advance the read pointer by `n` bytes (after `get_read_pointer`).
    pub fn consume(&mut self, n: u32) {
        unsafe { rn_serial_c::lnserial_rx_consume(self.raw, n); }
    }

    // ---- trampoline ----
    extern "C" fn trampoline(cookie: *mut c_void, event: i32) {
        unsafe {
            // The cookie is a double‑box: Box<Box<dyn SerialEventHandler>>.
            let outer: Box<Box<dyn SerialEventHandler>> =
                Box::from_raw(cookie as *mut Box<dyn SerialEventHandler>);
            outer.handle(SerialEvent::from(event));
            let _ = Box::into_raw(outer);
        }
    }
}

impl Drop for SerialRxTx {
    fn drop(&mut self) {
        unsafe { rn_serial_c::lnserial_rx_delete(self.raw); }
    }
}

//  Convenience impl: write_fmt via core::fmt::Write

impl core::fmt::Write for SerialTxOnly {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.transmit(s.as_bytes());
        Ok(())
    }
}

impl core::fmt::Write for SerialRxTx {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.transmit(s.as_bytes());
        Ok(())
    }
}