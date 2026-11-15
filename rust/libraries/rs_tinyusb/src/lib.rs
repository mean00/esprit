#![no_std]
#![allow(non_camel_case_types, deprecated)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// The `lncdc_*` / `lnusb_*` C ABI (bindgen). Only for the transition: see `usb.md`.
#[cfg(feature = "c_abi")]
pub mod c_cdc;
#[cfg(feature = "c_abi")]
pub mod c_usb;

pub mod cdc;
/// The retired wrapper over the C++ `lnUsbStack` (`c_usb`). Replaced by
/// `rs_tinyusb::stack` + `rs_usb` (usb.md P2/P4); kept only for the transition ABI.
#[cfg(feature = "c_abi")]
pub mod usb;

/// The device stack itself: task, descriptors, `tud_*` callbacks and the board ABI
/// (`usb.md` P2). Off unless the C++ `lnUsbStack` left the link - two sets of `tud_*`
/// would collide.
#[cfg(feature = "usb_stack")]
pub mod stack;

pub use cdc::{Cdc, CdcAcm, CdcEvent, CdcEventHandler, CdcPort, LineState, UsbError};
#[cfg(feature = "c_abi")]
pub use usb::{Usb, UsbBus, UsbEvent, UsbEventHandler};

/// The raw C types behind the transition ABI - see [`c_cdc`] / [`c_usb`].
#[cfg(feature = "c_abi")]
pub mod raw {
    pub use crate::c_cdc::lncdc_c;
    pub use crate::c_usb::{lnUsbStackEventHandler, lnusb_c};
}
