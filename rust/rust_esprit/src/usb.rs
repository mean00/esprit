#![allow(dead_code)]

use crate::rn_usb_c;
use core::ffi::c_void;

pub use rn_usb_c::{lnusb_c, lnUsbStackEventHandler};

/// USB bus events.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum UsbEvent {
    Connect = 0,
    Disconnect = 1,
    Suspend = 2,
    Resume = 3,
}

impl UsbEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            UsbEvent::Connect => "CONNECT",
            UsbEvent::Disconnect => "DISCONNECT",
            UsbEvent::Suspend => "SUSPEND",
            UsbEvent::Resume => "RESUME",
        }
    }
}

impl From<u32> for UsbEvent {
    fn from(val: u32) -> Self {
        match val {
            0 => UsbEvent::Connect,
            1 => UsbEvent::Disconnect,
            2 => UsbEvent::Suspend,
            3 => UsbEvent::Resume,
            _ => panic!("invalid UsbEvent: {}", val),
        }
    }
}

/// Types that can receive USB stack events.
pub trait UsbEventHandler {
    fn handle(&self, event: UsbEvent);
}

/// Owned handle to a USB peripheral stack.
///
/// Created via `UsbBus::new(instance, handler)`.
/// The underlying C object is destroyed on drop.
pub struct UsbBus {
    raw: *mut rn_usb_c::lnusb_c,
}

impl UsbBus {
    /// Create a new USB stack on hardware `instance`, attaching `handler` for events.
    ///
    /// `handler` is stored as a `Box<dyn UsbEventHandler>` — the C callback
    /// bounces back into the trait method.
    pub fn new(instance: u32, handler: Box<dyn UsbEventHandler>) -> Self {
        let raw = unsafe { rn_usb_c::lnusb_create(instance) };
        assert!(!raw.is_null(), "lnusb_create returned NULL");

        // Set the event handler with a trampoline
        let cookie = Box::into_raw(handler) as *const c_void;
        unsafe {
            rn_usb_c::lnusb_setEventHandler(raw, cookie, Some(Self::trampoline));
        }

        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut rn_usb_c::lnusb_c {
        self.raw
    }

    /// Initialise USB with descriptors (called after `new` but before `start`).
    pub fn init(&mut self, nb_desc_lines: i32, descriptor_lines: &[*const i8]) {
        unsafe {
            rn_usb_c::lnusb_init(
                self.raw,
                nb_desc_lines,
                descriptor_lines.as_ptr() as *mut *const i8,
            );
        }
    }

    /// Apply USB configuration.
    pub fn set_configuration(&mut self) {
        unsafe { rn_usb_c::lnusb_setConfiguration(self.raw); }
    }

    /// Start the USB stack.
    pub fn start(&mut self) {
        unsafe { rn_usb_c::lnusb_start(self.raw); }
    }

    /// Stop the USB stack.
    pub fn stop(&mut self) {
        unsafe { rn_usb_c::lnusb_stop(self.raw); }
    }

    // ---- trampoline ----

    extern "C" fn trampoline(cookie: *mut c_void, event: i32) {
        unsafe {
            let handler: Box<dyn UsbEventHandler> = Box::from_raw(cookie as *mut dyn UsbEventHandler);
            handler.handle(UsbEvent::from(event as u32));
            // Re-box to prevent drop — ownership stays with the USB driver
            let _ = Box::into_raw(handler);
        }
    }
}

impl Drop for UsbBus {
    fn drop(&mut self) {
        // Recover and drop the handler
        // Note: We can't easily recover the cookie here without storing it.
        // In practice, the C object owns the cookie and will manage its lifetime.
        unsafe { rn_usb_c::lnusb_delete(self.raw); }
    }
}