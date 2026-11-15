//! CDC-ACM (virtual serial port) access, directly over tinyUSB's `tud_cdc_n_*`.
//!
//! This replaces `esprit/libraries/tinyUsb/lnSrc/lnUsbCDC.cpp` (`usb.md`, P3): the C++
//! class was a thin wrapper over the same tinyUSB calls, plus a per-interface event
//! table and the `tud_cdc_*_cb` application callbacks. Both exist here now, so the
//! C++ object and its C ABI (`lnCDC_c.cpp`) are out of the link.
//!
//! Consumers keep the shape they had: `Cdc::new_static(instance, &HANDLER)` plus the
//! blocking `write` / non-blocking `write_no_block` pair.

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::ffi::c_void;
use core::fmt;
use core::sync::atomic::{AtomicPtr, Ordering};

/// Handler slots, one per CDC interface.
///
/// Mirrors the C++ `MAX_CDC` of `lnUsbCDC.cpp`. P4 replaces it with the value the
/// platform's endpoint budget allows (`rs_usb::budget::MAX_CDC`); it is only ever
/// indexed by an interface number tinyUSB reports, so it must be >= `CFG_TUD_CDC`.
pub const MAX_CDC_SLOTS: usize = 4;

// --- tinyUSB CDC class driver -----------------------------------------------------
unsafe extern "C" {
    fn tud_cdc_n_connected(itf: u8) -> bool;
    fn tud_cdc_n_get_line_state(itf: u8) -> u8;
    fn tud_cdc_n_available(itf: u8) -> u32;
    fn tud_cdc_n_read(itf: u8, buffer: *mut c_void, bufsize: u32) -> u32;
    fn tud_cdc_n_read_flush(itf: u8);
    fn tud_cdc_n_peek(itf: u8, ch: *mut u8) -> bool;
    fn tud_cdc_n_write(itf: u8, buffer: *const c_void, bufsize: u32) -> u32;
    fn tud_cdc_n_write_flush(itf: u8) -> u32;
    fn tud_cdc_n_write_available(itf: u8) -> u32;
}

/// `cdc_line_coding_t` of `tinyUsb/src/src/class/cdc/cdc.h`; only `bit_rate` is used.
#[repr(C)]
pub struct CdcLineCoding {
    pub bit_rate: u32,
    pub stop_bits: u8,
    pub parity: u8,
    pub data_bits: u8,
}

/// CDC-ACM (virtual serial port) events - same values as the retired
/// `lnUsbCDC::lnUsbCDCEvents`.
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

impl TryFrom<u32> for CdcEvent {
    type Error = UsbError;

    fn try_from(val: u32) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(CdcEvent::DataAvailable),
            1 => Ok(CdcEvent::WriteAvailable),
            2 => Ok(CdcEvent::SessionStart),
            3 => Ok(CdcEvent::SessionEnd),
            4 => Ok(CdcEvent::SetSpeed),
            _ => Err(UsbError::InvalidParam),
        }
    }
}

/// Types that can receive CDC-ACM events.
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

/// Zero-sized compile-time port handle for a tinyUSB CDC interface.
#[derive(Debug, Copy, Clone, Default)]
pub struct CdcPort<const ITF: u8>;

impl<const ITF: u8> CdcPort<ITF> {
    pub const fn new() -> Self {
        Self
    }

    #[inline]
    pub fn is_connected(&self) -> bool {
        unsafe { tud_cdc_n_connected(ITF) }
    }

    #[inline]
    pub fn read_available(&self) -> usize {
        unsafe { tud_cdc_n_available(ITF) as usize }
    }

    #[inline]
    pub fn write_available(&self) -> usize {
        unsafe { tud_cdc_n_write_available(ITF) as usize }
    }

    #[inline]
    pub fn read(&self, buffer: &mut [u8]) -> usize {
        unsafe {
            tud_cdc_n_read(ITF, buffer.as_mut_ptr() as *mut c_void, buffer.len() as u32) as usize
        }
    }

    #[inline]
    pub fn write(&self, data: &[u8]) -> usize {
        unsafe { tud_cdc_n_write(ITF, data.as_ptr() as *const c_void, data.len() as u32) as usize }
    }

    #[inline]
    pub fn write_no_block(&self, data: &[u8]) -> usize {
        self.write(data)
    }

    #[inline]
    pub fn flush(&self) {
        while unsafe { tud_cdc_n_write_flush(ITF) } != 0 {}
    }

    /// Kick the TX FIFO once, without waiting for the host - see [`Cdc::flush_once`].
    #[inline]
    pub fn flush_once(&self) {
        unsafe { tud_cdc_n_write_flush(ITF) };
    }

    #[inline]
    pub fn clear_input(&self) {
        unsafe { tud_cdc_n_read_flush(ITF) }
    }

    #[inline]
    pub fn line_state(&self) -> LineState {
        LineState::from_bits(unsafe { tud_cdc_n_get_line_state(ITF) })
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        let mut ch = 0u8;
        if unsafe { tud_cdc_n_peek(ITF, &mut ch) } {
            Some(ch)
        } else {
            None
        }
    }

    pub fn try_write(&self, data: &[u8]) -> Result<usize, UsbError> {
        if !self.is_connected() {
            return Err(UsbError::NotConnected);
        }
        let avail = self.write_available();
        if avail == 0 {
            return Err(UsbError::BufferFull);
        }
        Ok(self.write_no_block(&data[..data.len().min(avail)]))
    }

    pub fn try_read(&self, buffer: &mut [u8]) -> Result<usize, UsbError> {
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

// --- the per-interface event table ------------------------------------------------

/// `(cookie, interface, event, payload)` - the shape the C++ event handler had.
type CdcTrampoline = unsafe extern "C" fn(*mut c_void, u8, u32, u32);

static HANDLER_CB: [AtomicPtr<c_void>; MAX_CDC_SLOTS] =
    [const { AtomicPtr::new(core::ptr::null_mut()) }; MAX_CDC_SLOTS];
static HANDLER_COOKIE: [AtomicPtr<c_void>; MAX_CDC_SLOTS] =
    [const { AtomicPtr::new(core::ptr::null_mut()) }; MAX_CDC_SLOTS];

#[inline]
fn slot(interface: u8) -> Option<usize> {
    let itf = interface as usize;
    if itf < MAX_CDC_SLOTS { Some(itf) } else { None }
}

/// Deliver `event` to the handler registered on `interface`, if any.
///
/// Runs in the context tinyUSB calls the `tud_cdc_*_cb` from (the `usbd` task for the
/// data/line callbacks), so handlers must not block.
fn post(interface: u8, event: CdcEvent, payload: u32) {
    let Some(itf) = slot(interface) else {
        return;
    };
    let cb = HANDLER_CB[itf].load(Ordering::Acquire);
    if cb.is_null() {
        return;
    }
    let cookie = HANDLER_COOKIE[itf].load(Ordering::Acquire);
    // SAFETY: only the three `register_*` helpers below store here, all with this type.
    let f: CdcTrampoline = unsafe { core::mem::transmute(cb) };
    unsafe { f(cookie, interface, event as u32, payload) };
}

// --- the `tud_cdc_*_cb` application callbacks --------------------------------------

/// Data arrived from the host (the C++ `tud_cdc_rx_cb`).
#[unsafe(no_mangle)]
pub extern "C" fn tud_cdc_rx_cb(itf: u8) {
    post(itf, CdcEvent::DataAvailable, 0);
}

/// A transfer completed, i.e. the TX FIFO has room again.
#[unsafe(no_mangle)]
pub extern "C" fn tud_cdc_tx_complete_cb(itf: u8) {
    post(itf, CdcEvent::WriteAvailable, 0);
}

/// DTR/RTS changed: DTR on is a session start, DTR off a session end.
#[unsafe(no_mangle)]
pub extern "C" fn tud_cdc_line_state_cb(itf: u8, dtr: bool, _rts: bool) {
    if dtr {
        post(itf, CdcEvent::SessionStart, 0);
    } else {
        post(itf, CdcEvent::SessionEnd, 0);
    }
}

/// The host changed the line coding; the bit rate is the payload.
#[unsafe(no_mangle)]
pub extern "C" fn tud_cdc_line_coding_cb(itf: u8, coding: *const CdcLineCoding) {
    let rate = if coding.is_null() {
        0
    } else {
        unsafe { (*coding).bit_rate }
    };
    post(itf, CdcEvent::SetSpeed, rate);
}

// --- the `Cdc` handle -------------------------------------------------------------

/// Handle to one CDC-ACM interface.
///
/// The interface number is the whole state: tinyUSB owns the object, and the handler
/// table above lives in this crate, so a `Cdc` can sit in a `static` (which is how
/// `rs_swindle`'s gdb / bridge / logger instances use it).
pub struct Cdc {
    instance: u8,
}

/// Legacy name for [`Cdc`].
#[deprecated(note = "use `Cdc` instead")]
pub type CdcAcm = Cdc;

impl Cdc {
    fn register(instance: u32, cb: CdcTrampoline, cookie: *mut c_void) -> Self {
        assert!(
            (instance as usize) < MAX_CDC_SLOTS,
            "Cdc: interface above MAX_CDC_SLOTS"
        );
        let itf = instance as u8;
        HANDLER_COOKIE[itf as usize].store(cookie, Ordering::Release);
        HANDLER_CB[itf as usize].store(cb as *mut c_void, Ordering::Release);
        Self { instance: itf }
    }

    /// Create a CDC handle for `instance` with a boxed handler.
    #[cfg(feature = "alloc")]
    pub fn new(instance: u32, handler: Box<dyn CdcEventHandler>) -> Self {
        // Double-box so the cookie is a thin pointer (Box<Box<dyn Trait>>).
        let cookie = Box::into_raw(Box::new(handler)) as *mut c_void;
        Self::register(instance, Self::trampoline, cookie)
    }

    /// Create a CDC handle for `instance` with a static handler (no heap).
    pub fn new_static<T: CdcEventHandler + 'static>(instance: u32, handler: &'static T) -> Self {
        Self::register(
            instance,
            Self::static_trampoline::<T>,
            (handler as *const T) as *mut c_void,
        )
    }

    /// Create a CDC handle for `instance` with a plain function handler (no heap).
    pub fn new_fn(instance: u32, handler: fn(i32, CdcEvent, u32)) -> Self {
        Self::register(
            instance,
            Self::fn_trampoline,
            handler as usize as *mut c_void,
        )
    }

    /// The tinyUSB interface number this handle serves.
    #[inline]
    pub fn instance(&self) -> u8 {
        self.instance
    }

    #[inline]
    pub fn is_connected(&self) -> bool {
        unsafe { tud_cdc_n_connected(self.instance) }
    }

    #[inline]
    pub fn line_state(&self) -> LineState {
        LineState::from_bits(unsafe { tud_cdc_n_get_line_state(self.instance) })
    }

    #[inline]
    pub fn read_available(&self) -> usize {
        unsafe { tud_cdc_n_available(self.instance) as usize }
    }

    #[inline]
    pub fn write_available(&self) -> usize {
        unsafe { tud_cdc_n_write_available(self.instance) as usize }
    }

    #[inline]
    pub fn peek(&self) -> Option<u8> {
        let mut ch = 0u8;
        if unsafe { tud_cdc_n_peek(self.instance, &mut ch) } {
            Some(ch)
        } else {
            None
        }
    }

    /// Non-blocking write: whatever fits in the TX FIFO goes out.
    #[inline]
    pub fn write_no_block(&self, data: &[u8]) -> usize {
        unsafe {
            tud_cdc_n_write(
                self.instance,
                data.as_ptr() as *const c_void,
                data.len() as u32,
            ) as usize
        }
    }

    /// Write everything, flushing and yielding while the FIFO is full - the loop the
    /// C++ `lnUsbCDC::write` ran (`tud_cdc_n_write` + flush + `lnDelay(1)`).
    pub fn write(&self, data: &[u8]) -> usize {
        let mut sent = 0usize;
        let mut rest = data;
        while !rest.is_empty() {
            let n = self.write_no_block(rest);
            if n == 0 {
                self.flush_once();
                rs_rtos::task::delay_ms(1); // do not busy-loop on a full FIFO
                continue;
            }
            sent += n;
            rest = &rest[n..];
        }
        sent
    }

    /// Bounded write: retries up to `timeout_ms` when the TX FIFO is full, then returns
    /// the number of bytes actually queued so a disconnected/non-reading host cannot
    /// hang the caller indefinitely.
    pub fn write_timeout(&self, data: &[u8], timeout_ms: u32) -> usize {
        let mut sent = 0usize;
        let mut rest = data;
        let mut waited_ms = 0u32;
        while !rest.is_empty() {
            let n = self.write_no_block(rest);
            if n > 0 {
                self.flush_once();
                sent += n;
                rest = &rest[n..];
                waited_ms = 0;
                continue;
            }
            if waited_ms >= timeout_ms {
                break;
            }
            waited_ms += 1;
            rs_rtos::task::delay_ms(1);
        }
        sent
    }

    /// Read up to `buffer.len()` bytes; 0 when the FIFO is empty.
    pub fn read(&self, buffer: &mut [u8]) -> usize {
        unsafe {
            tud_cdc_n_read(
                self.instance,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
            ) as usize
        }
    }

    /// Kick the TX FIFO until `tud_cdc_n_write_flush` returns 0 (FIFO empty or endpoint
    /// busy submitting a packet).
    ///
    /// Note: `tud_cdc_n_write_flush` does not wait for the host to drain the pipe; it
    /// submits at most one USB transfer and returns 0 once the endpoint is busy.
    pub fn flush(&self) {
        while unsafe { tud_cdc_n_write_flush(self.instance) } != 0 {}
    }

    /// Kick the TX FIFO once and return; whatever is left stays queued for the next
    /// IN token.
    #[inline]
    pub fn flush_once(&self) {
        unsafe { tud_cdc_n_write_flush(self.instance) };
    }

    /// Drop whatever is pending in the RX FIFO.
    pub fn clear_input(&self) {
        unsafe { tud_cdc_n_read_flush(self.instance) }
    }

    pub fn try_write(&self, data: &[u8]) -> Result<usize, UsbError> {
        if !self.is_connected() {
            return Err(UsbError::NotConnected);
        }
        let avail = self.write_available();
        if avail == 0 {
            return Err(UsbError::BufferFull);
        }
        Ok(self.write_no_block(&data[..data.len().min(avail)]))
    }

    pub fn try_read(&self, buffer: &mut [u8]) -> Result<usize, UsbError> {
        if !self.is_connected() {
            return Err(UsbError::NotConnected);
        }
        Ok(self.read(buffer))
    }

    // ---- trampolines ----

    #[cfg(feature = "alloc")]
    unsafe extern "C" fn trampoline(cookie: *mut c_void, interface: u8, event: u32, payload: u32) {
        unsafe {
            let outer: Box<Box<dyn CdcEventHandler>> =
                Box::from_raw(cookie as *mut Box<dyn CdcEventHandler>);
            if let Ok(ev) = CdcEvent::try_from(event) {
                outer.handle(interface as i32, ev, payload);
            }
            let _ = Box::into_raw(outer);
        }
    }

    unsafe extern "C" fn static_trampoline<T: CdcEventHandler>(
        cookie: *mut c_void,
        interface: u8,
        event: u32,
        payload: u32,
    ) {
        if cookie.is_null() {
            return;
        }
        if let Ok(ev) = CdcEvent::try_from(event) {
            let handler = unsafe { &*(cookie as *const T) };
            handler.handle(interface as i32, ev, payload);
        }
    }

    unsafe extern "C" fn fn_trampoline(
        cookie: *mut c_void,
        interface: u8,
        event: u32,
        payload: u32,
    ) {
        if cookie.is_null() {
            return;
        }
        if let Ok(ev) = CdcEvent::try_from(event) {
            let func: fn(i32, CdcEvent, u32) = unsafe { core::mem::transmute(cookie) };
            func(interface as i32, ev, payload);
        }
    }
}

impl fmt::Write for Cdc {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let _ = self.write(s.as_bytes());
        Ok(())
    }
}
