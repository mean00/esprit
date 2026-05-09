#![allow(dead_code)]
use crate::prelude::*;

use crate::rn_cdc_c::lncdc_c as cdc_c;
use crate::rn_cdc_c::{
    lncdc_clear_input_buffers, lncdc_create, lncdc_delete, lncdc_flush, lncdc_read,
    lncdc_set_event_handler, lncdc_write,
};
//use crate::rn_os_helper::log;
use cty::{c_int, c_void};
//
pub trait cdc_event_handler {
    fn handler(&self, _interface: usize, _event: cdc_events, _payload: u32) {
        panic!("oops");
    }
}

//--
pub struct rnCDC<'a> {
    cdc: *mut cdc_c,
    handler: Option<&'a dyn cdc_event_handler>,
}
//
pub enum cdc_events {
    DATA_AVAILABLE = 0,
    SESSION_START = 1,
    SESSION_END = 2,
    SET_SPEED = 3,
}
impl cdc_events {
    pub fn from_u32(ix: u32) -> Self {
        return match ix {
           0 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_DATA_AVAILABLE*/ => cdc_events::DATA_AVAILABLE,
           1 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_SESSION_START*/  => cdc_events::SESSION_START,
           2 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_SESSION_END*/    => cdc_events::SESSION_END,
           3 /*rn::lnUsbCDC_lnUsbCDCEvents::CDC_SET_SPEED*/      => cdc_events::SET_SPEED,
            _ => panic!("EVENTCDC"),
        };
    }
    pub fn to_str(&self) -> &'static str {
        return match self {
            cdc_events::DATA_AVAILABLE => "DATA_AVL",
            cdc_events::SESSION_END => "END",
            cdc_events::SESSION_START => "START",
            cdc_events::SET_SPEED => "SPEED",
        };
    }
}

/**
 *
 *
 */
impl<'a> Drop for rnCDC<'a> {
    fn drop(&mut self) {
        unsafe {
            lncdc_delete(self.cdc);
        }
    }
}
impl<'a> rnCDC<'a> {
    // ctor
    pub fn new(instance: u32, handler: &'a dyn cdc_event_handler) -> Box<rnCDC<'a>> {
        unsafe {
            let r = Box::new(rnCDC {
                cdc: lncdc_create(instance),
                handler: Some(handler),
            });
            let r_ptr = &*r as *const rnCDC;
            lncdc_set_event_handler(r.cdc, Some(Self::bounceBack), r_ptr as *mut c_void);
            r
        }
    }
    // returns byte received, can be zero, this is non blocking
    pub fn read(&mut self, data: &mut [u8]) -> i32 {
        unsafe {
            return lncdc_read(self.cdc, data.as_ptr() as *mut u8, data.len() as cty::c_int) as i32;
        }
    }
    // returns byte written
    pub fn write(&mut self, data: &mut [u8]) -> i32 {
        unsafe {
            return lncdc_write(
                self.cdc,
                data.as_ptr() as *const u8,
                data.len() as cty::c_int,
            ) as i32;
        }
    }

    // flush pending writes
    pub fn flush(&mut self) {
        unsafe {
            lncdc_flush(self.cdc);
        }
    }
    // flush input buffers
    pub fn clear_input_buffers(&mut self) {
        unsafe {
            lncdc_clear_input_buffers(self.cdc);
        }
    }
    fn invoke_callback(&mut self, instance: usize, event: cdc_events, payload: u32) {
        match &mut self.handler {
            Some(x) => x.handler(instance, event, payload),
            None => panic!("no cdc handler"),
        }
    }
    /*
     *
     */
    extern "C" fn bounceBack(
        ptr: *mut cty::c_void,
        instance: cty::c_int,
        event: c_int,
        payload: cty::c_uint,
    ) -> () {
        //type cookie_monster <'a>=  Box<&'a rnCDC> ;
        type cookie_monster<'a> = rnCDC<'a>;
        unsafe {
            let e: *mut cookie_monster = ptr as *mut cookie_monster;
            //let  rf :& mut rnCDC =  &mut **e ;

            let a = &mut *e;
            //let r: () = a;
            a.invoke_callback(
                instance as usize,
                cdc_events::from_u32(event as u32),
                payload as u32,
            );
            /*
            //(*e).invoke_callback( instance as usize,  cdc_events::from_u32(event as u32),payload as u32);
            match rf
            {
                Some(mut x) => match x.handler
                            {
                                Some(mut y) => y.handler(instance as usize, cdc_events::from_u32(event as u32) , payload as u32),
                                None    => panic!("no cdc2"),
                            },
                None    => panic!("no CDC handler"),
            }*/
        }
    }
}
