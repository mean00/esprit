#![allow(dead_code)]

use crate::c_usb;
use crate::cdc::UsbError;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::ffi::c_void;
use cty;

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

impl TryFrom<u32> for UsbEvent {
    type Error = UsbError;

    fn try_from(val: u32) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(UsbEvent::Connect),
            1 => Ok(UsbEvent::Disconnect),
            2 => Ok(UsbEvent::Suspend),
            3 => Ok(UsbEvent::Resume),
            _ => Err(UsbError::InvalidParam),
        }
    }
}

/// Types that can receive USB stack events.
pub trait UsbEventHandler {
    fn handle(&self, event: UsbEvent);
}

/// Owned handle to a USB peripheral stack.
///
/// Created via `Usb::new(instance, handler)`.
/// The underlying C object is destroyed on drop.
pub struct Usb {
    raw: *mut c_usb::lnusb_c,
}

/// Legacy name for [`Usb`].
#[deprecated(note = "use `Usb` instead")]
pub type UsbBus = Usb;

impl Usb {
    /// Create a new USB stack on hardware `instance`, attaching `handler` for events.
    ///
    /// `handler` is stored as a `Box<dyn UsbEventHandler>` — the C callback
    /// bounces back into the trait method.
    #[cfg(feature = "alloc")]
    pub fn new(instance: u32, handler: Box<dyn UsbEventHandler>) -> Self {
        let raw = unsafe { c_usb::lnusb_create(instance) };
        assert!(!raw.is_null(), "lnusb_create returned NULL");

        // Double-box so the cookie is a thin pointer (Box<Box<dyn Trait>>).
        let cookie = Box::into_raw(Box::new(handler)) as *const c_void;
        unsafe {
            c_usb::lnusb_setEventHandler(raw, cookie, Some(Self::trampoline));
        }

        Self { raw }
    }

    /// Create a new USB stack with a static handler (zero heap allocation).
    pub fn new_static<T: UsbEventHandler + 'static>(instance: u32, handler: &'static T) -> Self {
        let raw = unsafe { c_usb::lnusb_create(instance) };
        assert!(!raw.is_null(), "lnusb_create returned NULL");

        let cookie = (handler as *const T) as *const c_void;
        unsafe {
            c_usb::lnusb_setEventHandler(raw, cookie, Some(Self::static_trampoline::<T>));
        }

        Self { raw }
    }

    /// Create a new USB stack with a function pointer handler (zero heap allocation).
    pub fn new_fn(instance: u32, handler: fn(UsbEvent)) -> Self {
        let raw = unsafe { c_usb::lnusb_create(instance) };
        assert!(!raw.is_null(), "lnusb_create returned NULL");

        let cookie = handler as usize as *const c_void;
        unsafe {
            c_usb::lnusb_setEventHandler(raw, cookie, Some(Self::fn_trampoline));
        }

        Self { raw }
    }

    /// Returns true if the device is mounted by the host.
    #[inline]
    pub fn is_mounted() -> bool {
        unsafe { c_usb::lnusb_is_mounted() }
    }

    /// Returns true if the USB bus is suspended.
    #[inline]
    pub fn is_suspended() -> bool {
        unsafe { c_usb::lnusb_is_suspended() }
    }

    /// Returns true if the USB device is mounted and ready for communication.
    #[inline]
    pub fn is_ready() -> bool {
        unsafe { c_usb::lnusb_is_ready() }
    }

    /// Returns true if the device is mounted by the host.
    #[inline]
    pub fn mounted(&self) -> bool {
        Self::is_mounted()
    }

    /// Returns true if the USB bus is suspended.
    #[inline]
    pub fn suspended(&self) -> bool {
        Self::is_suspended()
    }

    /// Returns true if the USB device is mounted and ready for communication.
    #[inline]
    pub fn ready(&self) -> bool {
        Self::is_ready()
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut c_usb::lnusb_c {
        self.raw
    }

    /// Initialise USB with descriptors (called after `new` but before `start`).
    pub fn init(&mut self, nb_desc_lines: i32, descriptor_lines: &[*const cty::c_char]) {
        unsafe {
            c_usb::lnusb_init(
                self.raw,
                nb_desc_lines,
                descriptor_lines.as_ptr() as *mut *const cty::c_char,
            );
        }
    }

    /// Apply USB configuration.
    pub fn set_configuration(&mut self) {
        unsafe {
            c_usb::lnusb_setConfiguration(self.raw);
        }
    }

    /// Start the USB stack.
    pub fn start(&mut self) {
        unsafe {
            c_usb::lnusb_start(self.raw);
        }
    }

    /// Stop the USB stack.
    pub fn stop(&mut self) {
        unsafe {
            c_usb::lnusb_stop(self.raw);
        }
    }

    // ---- trampolines ----

    #[cfg(feature = "alloc")]
    extern "C" fn trampoline(cookie: *mut c_void, event: i32) {
        unsafe {
            let outer: Box<Box<dyn UsbEventHandler>> =
                Box::from_raw(cookie as *mut Box<dyn UsbEventHandler>);
            if let Ok(ev) = UsbEvent::try_from(event as u32) {
                outer.handle(ev);
            }
            let _ = Box::into_raw(outer);
        }
    }

    extern "C" fn static_trampoline<T: UsbEventHandler>(cookie: *mut c_void, event: i32) {
        if cookie.is_null() {
            return;
        }
        if let Ok(ev) = UsbEvent::try_from(event as u32) {
            let handler = unsafe { &*(cookie as *const T) };
            handler.handle(ev);
        }
    }

    extern "C" fn fn_trampoline(cookie: *mut c_void, event: i32) {
        if cookie.is_null() {
            return;
        }
        if let Ok(ev) = UsbEvent::try_from(event as u32) {
            let func: fn(UsbEvent) = unsafe { core::mem::transmute(cookie) };
            func(ev);
        }
    }
}

impl Drop for Usb {
    fn drop(&mut self) {
        unsafe {
            c_usb::lnusb_delete(self.raw);
        }
    }
}
