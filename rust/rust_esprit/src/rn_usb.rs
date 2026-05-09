#![allow(dead_code)]
use crate::prelude::*;

use crate::rn_usb_c::lnusb_c as usb_c;
use crate::rn_usb_c::{
    lnusb_create, lnusb_delete, lnusb_setConfiguration, lnusb_setEventHandler, lnusb_start,
    lnusb_stop,
};

use cty::{c_int, c_void};
//
//
pub enum usb_events {
    CONNECT = 0,
    DISCONNECT = 1,
    SUSPEND = 2,
    RESUME = 3,
}
//
impl usb_events {
    pub fn to_str(&self) -> &'static str {
        return match self {
            usb_events::CONNECT => "CONNECT",
            usb_events::DISCONNECT => "DISCONNECT",
            usb_events::SUSPEND => "SUSPEND",
            usb_events::RESUME => "RESUME",
        };
    }
}
//
pub trait usb_event_handler {
    fn handler(&self, _event: usb_events) {
        panic!("oops");
    }
}

//--
pub struct rnUSB<'a> {
    usb: *mut usb_c,
    handler: Option<&'a dyn usb_event_handler>,
}
/*
 */
impl usb_events {
    pub fn from_u32(ix: u32) -> Self {
        return match ix {
           0 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_DATA_AVAILABLE*/ => usb_events::CONNECT,
           1 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_SESSION_START*/  => usb_events::DISCONNECT,
           2 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_SESSION_END*/    => usb_events::SUSPEND,
           3 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_SET_SPEED*/      => usb_events::RESUME,
            _ => panic!("EVENTCDC"),
        };
    }
}

/**
 *
 *
 */

impl<'a> Drop for rnUSB<'a> {
    fn drop(&mut self) {
        unsafe {
            lnusb_delete(self.usb);
        }
    }
}
/*
 */
impl<'a> rnUSB<'a> {
    // ctor
    pub fn new(instance: u32, handler: &'a dyn usb_event_handler) -> Box<rnUSB<'a>> {
        unsafe {
            let r = Box::new(rnUSB {
                usb: lnusb_create(instance),
                handler: Some(handler),
            });
            let r_ptr = &*r as *const rnUSB;
            lnusb_setEventHandler(r.usb, r_ptr as *const c_void, Some(Self::bounceBack));
            r
        }
    }
    //
    pub fn set_configuration(&mut self) {
        //,  const tusb_desc_device_t *desc, const tusb_desc_device_qualifier_t *qual);
        unsafe {
            lnusb_setConfiguration(self.usb);
        }
    }

    //
    pub fn start(&mut self) {
        unsafe {
            lnusb_start(self.usb);
        }
    }

    //
    pub fn stop(&mut self) {
        unsafe {
            lnusb_stop(self.usb);
        }
    }

    //
    fn invoke_callback(&mut self, event: usb_events) {
        match &mut self.handler {
            Some(x) => x.handler(event),
            None => panic!("no cdc handler"),
        }
    }
    /*
     *
     */
    extern "C" fn bounceBack(ptr: *mut cty::c_void, event: c_int) -> () {
        type cookie_monster<'a> = rnUSB<'a>;
        unsafe {
            let e: *mut cookie_monster = ptr as *mut cookie_monster;
            let a = &mut *e;
            a.invoke_callback(usb_events::from_u32(event as u32));
        }
    }
}
