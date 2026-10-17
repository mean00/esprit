#![allow(dead_code)]

use alloc::boxed::Box;
use crate::c_cdc;
use core::ffi::c_void;
use core::fmt;

/// CDC‑ACM (virtual serial port) events.
///
/// Must match the C++ `lnUsbCDC::lnUsbCDCEvents` enum in `lnUsbCDC.h`:
///
/// ```cpp
/// enum lnUsbCDCEvents {
///     CDC_DATA_AVAILABLE,    // 0
///     CDC_WRITE_AVAILABLE,   // 1
///     CDC_SESSION_START,     // 2
///     CDC_SESSION_END,       // 3
///     CDC_SET_SPEED          // 4
/// };
/// ```
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum CdcEvent {
    DataAvailable = 0,
    WriteAvailable = 1,
    SessionStart = 2,
    SessionEnd = 3,
    SetSpeed = 4,
}

impl CdcEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            CdcEvent::DataAvailable => "DATA_AVAILABLE",
            CdcEvent::WriteAvailable => "WRITE_AVAILABLE",
            CdcEvent::SessionStart => "SESSION_START",
            CdcEvent::SessionEnd => "SESSION_END",
            CdcEvent::SetSpeed => "SET_SPEED",
        }
    }
}

impl From<u32> for CdcEvent {
    fn from(val: u32) -> Self {
        match val {
            0 => CdcEvent::DataAvailable,
            1 => CdcEvent::WriteAvailable,
            2 => CdcEvent::SessionStart,
            3 => CdcEvent::SessionEnd,
            4 => CdcEvent::SetSpeed,
            _ => panic!("invalid CdcEvent: {}", val),
        }
    }
}

/// Types that can receive CDC‑ACM events.
pub trait CdcEventHandler {
    fn handle(&self, interface: i32, event: CdcEvent, payload: u32);
}

/// USB / CDC error conditions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UsbError {
    NotConnected,
    BufferFull,
    InvalidParam,
}

/// CDC line state flags (DTR / RTS).
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct LineState {
    pub dtr: bool,
    pub rts: bool,
}

impl LineState {
    pub fn from_bits(bits: u8) -> Self {
        Self {
            dtr: (bits & 1) != 0,
            rts: (bits & 2) != 0,
        }
    }

    #[inline]
    pub fn dtr(&self) -> bool {
        self.dtr
    }

    #[inline]
    pub fn rts(&self) -> bool {
        self.rts
    }
}

/// Zero-sized compile-time port handle for TinyUSB CDC interfaces.
///
/// Bypasses C++ object allocation and talks directly to TinyUSB `tud_cdc_n_*`.
#[derive(Debug, Copy, Clone, Default)]
pub struct CdcPort<const ITF: u8>;

impl<const ITF: u8> CdcPort<ITF> {
    pub const fn new() -> Self {
        Self
    }

    #[inline]
    pub fn is_connected(&self) -> bool {
        unsafe { c_cdc::lncdc_port_is_connected(ITF as u32) }
    }

    #[inline]
    pub fn read_available(&self) -> usize {
        unsafe { c_cdc::lncdc_port_read_available(ITF as u32).max(0) as usize }
    }

    #[inline]
    pub fn write_available(&self) -> usize {
        unsafe { c_cdc::lncdc_port_write_available(ITF as u32).max(0) as usize }
    }

    #[inline]
    pub fn read(&mut self, buffer: &mut [u8]) -> usize {
        unsafe { c_cdc::lncdc_port_read(ITF as u32, buffer.as_mut_ptr(), buffer.len() as i32).max(0) as usize }
    }

    #[inline]
    pub fn write(&mut self, data: &[u8]) -> usize {
        unsafe { c_cdc::lncdc_port_write(ITF as u32, data.as_ptr(), data.len() as i32).max(0) as usize }
    }

    #[inline]
    pub fn write_no_block(&mut self, data: &[u8]) -> usize {
        unsafe { c_cdc::lncdc_port_write_no_block(ITF as u32, data.as_ptr(), data.len() as i32).max(0) as usize }
    }

    #[inline]
    pub fn flush(&mut self) {
        unsafe { c_cdc::lncdc_port_flush(ITF as u32); }
    }

    #[inline]
    pub fn clear_input(&mut self) {
        unsafe { c_cdc::lncdc_port_clear_input(ITF as u32); }
    }

    #[inline]
    pub fn line_state(&self) -> LineState {
        LineState::from_bits(unsafe { c_cdc::lncdc_port_get_line_state(ITF as u32) })
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        let mut ch = 0u8;
        if unsafe { c_cdc::lncdc_port_peek(ITF as u32, &mut ch) } {
            Some(ch)
        } else {
            None
        }
    }

    pub fn try_write(&mut self, data: &[u8]) -> Result<usize, UsbError> {
        if !self.is_connected() {
            return Err(UsbError::NotConnected);
        }
        let avail = self.write_available();
        if avail == 0 {
            return Err(UsbError::BufferFull);
        }
        let to_write = data.len().min(avail);
        let written = self.write_no_block(&data[..to_write]);
        Ok(written)
    }

    pub fn try_read(&mut self, buffer: &mut [u8]) -> Result<usize, UsbError> {
        if !self.is_connected() {
            return Err(UsbError::NotConnected);
        }
        Ok(self.read(buffer))
    }
}

impl<const ITF: u8> fmt::Write for CdcPort<ITF> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = self.write(s.as_bytes());
        Ok(())
    }
}

/// Handle to a CDC‑ACM (virtual COM port) instance.
///
/// Created via `Cdc::new(instance, handler)`.
///
/// # Lifetime
///
/// The underlying C object is **not** destroyed on drop — it is owned by the
/// USB stack (registered in the global `_cdc_instances[]` table) and must
/// remain alive for as long as TinyUSB may deliver events.  Drop this handle
/// only after the USB connection has been torn down, or leak it intentionally
/// (e.g. store it in a `static`).
pub struct Cdc {
    raw: *mut c_cdc::lncdc_c,
}

/// Legacy name for [`Cdc`].
#[deprecated(note = "use `Cdc` instead")]
pub type CdcAcm = Cdc;

impl Cdc {
    /// Create a new CDC instance on `instance` and attach `handler`.
    pub fn new(instance: u32, handler: Box<dyn CdcEventHandler>) -> Self {
        let raw = unsafe { c_cdc::lncdc_create(instance) };
        assert!(!raw.is_null(), "lncdc_create returned NULL");

        // Double-box so the cookie is a thin pointer (Box<Box<dyn Trait>>).
        let cookie = Box::into_raw(Box::new(handler)) as *mut c_void;
        unsafe {
            c_cdc::lncdc_set_event_handler(raw, Some(Self::trampoline), cookie);
        }

        Self { raw }
    }

    /// Create a new CDC instance with a static handler (zero heap allocation).
    pub fn new_static<T: CdcEventHandler + 'static>(instance: u32, handler: &'static T) -> Self {
        let raw = unsafe { c_cdc::lncdc_create(instance) };
        assert!(!raw.is_null(), "lncdc_create returned NULL");

        let cookie = (handler as *const T) as *mut c_void;
        unsafe {
            c_cdc::lncdc_set_event_handler(raw, Some(Self::static_trampoline::<T>), cookie);
        }

        Self { raw }
    }

    /// Create a new CDC instance with a function pointer handler (zero heap allocation).
    pub fn new_fn(instance: u32, handler: fn(i32, CdcEvent, u32)) -> Self {
        let raw = unsafe { c_cdc::lncdc_create(instance) };
        assert!(!raw.is_null(), "lncdc_create returned NULL");

        let cookie = handler as usize as *mut c_void;
        unsafe {
            c_cdc::lncdc_set_event_handler(raw, Some(Self::fn_trampoline), cookie);
        }

        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut c_cdc::lncdc_c {
        self.raw
    }

    /// Check if terminal is connected (DTR active).
    #[inline]
    pub fn is_connected(&self) -> bool {
        unsafe { c_cdc::lncdc_is_connected(self.raw) }
    }

    /// Get current line control state (DTR / RTS).
    #[inline]
    pub fn line_state(&self) -> LineState {
        LineState::from_bits(unsafe { c_cdc::lncdc_get_line_state(self.raw) })
    }

    /// Number of bytes available to read.
    #[inline]
    pub fn read_available(&self) -> usize {
        unsafe { c_cdc::lncdc_read_available(self.raw).max(0) as usize }
    }

    /// Number of bytes available in transmit FIFO.
    #[inline]
    pub fn write_available(&self) -> usize {
        unsafe { c_cdc::lncdc_write_available(self.raw).max(0) as usize }
    }

    /// Peek the next byte in input FIFO without removing it.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        let mut ch = 0u8;
        if unsafe { c_cdc::lncdc_peek(self.raw, &mut ch) } {
            Some(ch)
        } else {
            None
        }
    }

    /// Non-blocking write. Writes as much as fits into the endpoint buffer.
    #[inline]
    pub fn write_no_block(&mut self, data: &[u8]) -> usize {
        unsafe { c_cdc::lncdc_write_no_block(self.raw, data.as_ptr(), data.len() as i32).max(0) as usize }
    }

    /// Non-blocking try write. Returns error if not connected or buffer full.
    pub fn try_write(&mut self, data: &[u8]) -> Result<usize, UsbError> {
        if !self.is_connected() {
            return Err(UsbError::NotConnected);
        }
        let avail = self.write_available();
        if avail == 0 {
            return Err(UsbError::BufferFull);
        }
        let to_write = data.len().min(avail);
        let written = self.write_no_block(&data[..to_write]);
        Ok(written)
    }

    /// Try read. Returns error if not connected.
    pub fn try_read(&mut self, buffer: &mut [u8]) -> Result<usize, UsbError> {
        if !self.is_connected() {
            return Err(UsbError::NotConnected);
        }
        let n = self.read(buffer);
        Ok(n.max(0) as usize)
    }

    /// Blocking read.  Returns the number of bytes read (0 if nothing available).
    pub fn read(&mut self, buffer: &mut [u8]) -> i32 {
        unsafe { c_cdc::lncdc_read(self.raw, buffer.as_mut_ptr(), buffer.len() as i32) }
    }

    /// Blocking write.  Returns the number of bytes sent.
    pub fn write(&mut self, data: &[u8]) -> i32 {
        unsafe { c_cdc::lncdc_write(self.raw, data.as_ptr(), data.len() as i32) }
    }

    /// Flush output buffers.
    pub fn flush(&mut self) {
        unsafe { c_cdc::lncdc_flush(self.raw); }
    }

    /// Clear input buffers.
    pub fn clear_input(&mut self) {
        unsafe { c_cdc::lncdc_clear_input_buffers(self.raw); }
    }

    // ---- trampolines ----

    extern "C" fn trampoline(cookie: *mut c_void, interface: i32, event: i32, payload: u32) {
        unsafe {
            let outer: Box<Box<dyn CdcEventHandler>> =
                Box::from_raw(cookie as *mut Box<dyn CdcEventHandler>);
            outer.handle(interface, CdcEvent::from(event as u32), payload);
            let _ = Box::into_raw(outer);
        }
    }

    extern "C" fn static_trampoline<T: CdcEventHandler>(
        cookie: *mut c_void,
        interface: i32,
        event: i32,
        payload: u32,
    ) {
        if cookie.is_null() {
            return;
        }
        let handler = unsafe { &*(cookie as *const T) };
        handler.handle(interface, CdcEvent::from(event as u32), payload);
    }

    extern "C" fn fn_trampoline(cookie: *mut c_void, interface: i32, event: i32, payload: u32) {
        if cookie.is_null() {
            return;
        }
        let func: fn(i32, CdcEvent, u32) = unsafe { core::mem::transmute(cookie) };
        func(interface, CdcEvent::from(event as u32), payload);
    }
}

// NOTE: no Drop — the underlying C++ `lnUsbCDC` object is registered in the
// global `_cdc_instances[]` table and is owned by the USB stack.  Destroying
// it while TinyUSB may still deliver events (e.g. `tud_cdc_line_coding_cb`)
// would cause a use-after-free.  The C++ object lives until the USB stack is
// fully deinitialised.

/// Convenience `write_fmt` using `write`.

impl fmt::Write for Cdc {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let _ = self.write(bytes);
        Ok(())
    }
}
