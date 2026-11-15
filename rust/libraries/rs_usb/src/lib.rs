//! The USB device framework above `rs_tinyusb` (`usb.md`, P4).
//!
//! `rs_tinyusb` is the thin wrapper over tinyUSB (the task, the `tud_*` entry points,
//! the CDC port). This crate is the *device* on top of it: how many CDC-ACM interfaces
//! it exposes (with the endpoint budget checked at compile time), the descriptor set
//! it hands to the stack, the DFU-runtime hook, and a typed handle the board claims
//! interfaces from.
//!
//! Nothing here knows who uses the interfaces - that is the board's business
//! (`swindle/rs/rs_swindle/src/usb/`), which is why this crate lives in esprit.

#![no_std]

pub mod budget;

pub use rs_tinyusb::cdc::{Cdc, CdcEvent, CdcEventHandler, CdcPort, LineState, UsbError};
#[cfg(feature = "usb_stack")]
pub use rs_tinyusb::stack::{
    rn_usb_is_mounted, rn_usb_is_ready, rn_usb_is_suspended, rn_usb_last_event, rn_usb_stack_init,
    rn_usb_stack_set_event_handler, rn_usb_stack_start, rn_usb_stack_stop, Descriptors, EventHandler,
    USB_CONNECT, USB_DISCONNECT, USB_RESUME, USB_SUSPEND,
};

/// How many CDC-ACM interfaces this build asks for (CMake `LN_USB_NB_CDC`).
#[cfg(all(feature = "cdc2", feature = "cdc3"))]
compile_error!("rs_usb: pick one CDC count - cdc2 or cdc3, not both");
#[cfg(not(any(feature = "cdc2", feature = "cdc3")))]
pub const CDC_COUNT: usize = 0;
#[cfg(all(feature = "cdc2", not(feature = "cdc3")))]
pub const CDC_COUNT: usize = 2;
#[cfg(all(feature = "cdc3", not(feature = "cdc2")))]
pub const CDC_COUNT: usize = 3;

/// A CDC-ACM device with `N` interfaces on a controller with `MAX_EP` endpoint numbers.
///
/// Constructing it is the compile-time check: `N` interfaces that do not fit the
/// controller's budget fail here, not on a probe.
pub struct CdcAcmDevice<const N: usize, const MAX_EP: usize> {
    _private: (),
}

impl<const N: usize, const MAX_EP: usize> CdcAcmDevice<N, MAX_EP> {
    /// Fails the build when `N` CDC interfaces need more endpoints than the MCU has.
    const CHECK: () = budget::check(N, MAX_EP);

    pub const fn new() -> Self {
        // Force the check for every instantiation that is actually used.
        let () = Self::CHECK;
        Self { _private: () }
    }

    /// Number of interfaces this device exposes.
    pub const fn cdc_count(&self) -> usize {
        N
    }

    /// The handle for interface `ITF`.
    ///
    /// # Panics
    /// At compile time if `ITF` is outside the device's interface count.
    pub const fn port<const ITF: u8>(&self) -> CdcPort<ITF> {
        assert!((ITF as usize) < N, "rs_usb: interface above the device's CDC count");
        CdcPort::<ITF>::new()
    }
}

impl<const N: usize, const MAX_EP: usize> Default for CdcAcmDevice<N, MAX_EP> {
    fn default() -> Self {
        Self::new()
    }
}

/// The DFU runtime interface: the host asks us to reboot into the bootloader, and the
/// board says how (`goDfu`). The `tud_dfu_runtime_reboot_to_dfu_cb` export that calls
/// this hook lands with P5.
pub struct DfuRuntime {
    hook: HookCell,
}

type HookCell = core::sync::atomic::AtomicPtr<()>;

impl DfuRuntime {
    pub const fn new() -> Self {
        Self {
            hook: HookCell::new(core::ptr::null_mut()),
        }
    }

    /// Install the reboot action. Called once, before the stack starts.
    pub fn set_reboot_hook(&'static self, hook: fn()) {
        self.hook
            .store(hook as *mut (), core::sync::atomic::Ordering::Release);
    }

    /// Run the installed hook (no-op when the board did not install one).
    pub fn reboot_to_dfu(&self) {
        let p = self.hook.load(core::sync::atomic::Ordering::Acquire);
        if p.is_null() {
            return;
        }
        // SAFETY: only `set_reboot_hook` stores here, always a `fn()`.
        let f: fn() = unsafe { core::mem::transmute(p) };
        f();
    }
}

impl Default for DfuRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// The device's DFU runtime interface. One per device: tinyUSB's
/// `tud_dfu_runtime_reboot_to_dfu_cb` is a single C entry point and has no interface
/// argument, so there is nothing per-interface to keep apart.
pub static DFU: DfuRuntime = DfuRuntime::new();

/// `tud_dfu_runtime_reboot_to_dfu_cb()` - the host asked us to switch to the
/// bootloader (the C++ `lnUsbDFUrt.cpp`, `usb.md` P5).
#[unsafe(no_mangle)]
pub extern "C" fn tud_dfu_runtime_reboot_to_dfu_cb() {
    DFU.reboot_to_dfu();
}

/// Install the board's "reboot into DFU" action - the C++ `lnUsbDFURT::addDFURTCb`.
pub fn rn_usb_set_dfu_hook(hook: Option<fn()>) {
    if let Some(f) = hook {
        DFU.set_reboot_hook(f);
    }
}
