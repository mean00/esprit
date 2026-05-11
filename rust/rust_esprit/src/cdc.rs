#![allow(dead_code)]

use crate::prelude::*;
use crate::rn_cdc_c;
use core::ffi::c_void;
use core::fmt;

pub use rn_cdc_c::lncdc_c;

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

/// Handle to a CDC‑ACM (virtual COM port) instance.
///
/// Created via `CdcAcm::new(instance, handler)`.
///
/// # Lifetime
///
/// The underlying C object is **not** destroyed on drop — it is owned by the
/// USB stack (registered in the global `_cdc_instances[]` table) and must
/// remain alive for as long as TinyUSB may deliver events.  Drop this handle
/// only after the USB connection has been torn down, or leak it intentionally
/// (e.g. store it in a `static`).
pub struct CdcAcm {
    raw: *mut rn_cdc_c::lncdc_c,
}


impl CdcAcm {
    /// Create a new CDC instance on `instance` and attach `handler`.
    pub fn new(instance: u32, handler: Box<dyn CdcEventHandler>) -> Self {
        let raw = unsafe { rn_cdc_c::lncdc_create(instance) };
        assert!(!raw.is_null(), "lncdc_create returned NULL");

        // Double-box so the cookie is a thin pointer (Box<Box<dyn Trait>>).
        let cookie = Box::into_raw(Box::new(handler)) as *mut c_void;
        unsafe {
            rn_cdc_c::lncdc_set_event_handler(raw, Some(Self::trampoline), cookie);
        }

        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut rn_cdc_c::lncdc_c {
        self.raw
    }

    /// Blocking read.  Returns the number of bytes read (0 if nothing available).
    pub fn read(&mut self, buffer: &mut [u8]) -> i32 {
        unsafe { rn_cdc_c::lncdc_read(self.raw, buffer.as_mut_ptr(), buffer.len() as i32) }
    }

    /// Blocking write.  Returns the number of bytes sent.
    pub fn write(&mut self, data: &[u8]) -> i32 {
        unsafe { rn_cdc_c::lncdc_write(self.raw, data.as_ptr(), data.len() as i32) }
    }

    /// Flush output buffers.
    pub fn flush(&mut self) {
        unsafe { rn_cdc_c::lncdc_flush(self.raw); }
    }

    /// Clear input buffers.
    pub fn clear_input(&mut self) {
        unsafe { rn_cdc_c::lncdc_clear_input_buffers(self.raw); }
    }

    // ---- trampoline ----

    extern "C" fn trampoline(cookie: *mut c_void, interface: i32, event: i32, payload: u32) {
        unsafe {
            // The cookie is a double-box: Box<Box<dyn CdcEventHandler>>.
            // Recover the outer Box (thin pointer), call the handler, then
            // re-box to keep ownership with the C driver.
            let outer: Box<Box<dyn CdcEventHandler>> =
                Box::from_raw(cookie as *mut Box<dyn CdcEventHandler>);
            outer.handle(interface, CdcEvent::from(event as u32), payload);
            let _ = Box::into_raw(outer);
        }
    }
}

// NOTE: no Drop — the underlying C++ `lnUsbCDC` object is registered in the
// global `_cdc_instances[]` table and is owned by the USB stack.  Destroying
// it while TinyUSB may still deliver events (e.g. `tud_cdc_line_coding_cb`)
// would cause a use-after-free.  The C++ object lives until the USB stack is
// fully deinitialised.

/// Convenience `write_fmt` using `write`.

impl fmt::Write for CdcAcm {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let _ = self.write(bytes);
        Ok(())
    }
}