#![no_std]
#![allow(non_camel_case_types, deprecated)]

extern crate alloc;

pub mod c_cdc;
pub mod c_usb;
pub mod cdc;
pub mod usb;

pub use cdc::{Cdc, CdcAcm, CdcEvent, CdcEventHandler, CdcPort, LineState, UsbError};
pub use usb::{Usb, UsbBus, UsbEvent, UsbEventHandler};

pub mod raw {
    pub use crate::c_cdc::lncdc_c;
    pub use crate::c_usb::{lnUsbStackEventHandler, lnusb_c};
}
