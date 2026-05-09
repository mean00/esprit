#![allow(dead_code)]

//use alloc::boxed::Box;
//use crate::rust_esprit as rn;
use crate::rn_fast_event_c::{
    lnfast_event_group_c, lnfast_event_group_create, lnfast_event_group_delete,
    lnfast_event_group_read_events, lnfast_event_group_set_events,
    lnfast_event_group_takeOwnership, lnfast_event_group_wait_events,
};

pub struct rnFastEventGroup {
    ln: *const lnfast_event_group_c,
}
impl Default for rnFastEventGroup {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for rnFastEventGroup {
    fn drop(&mut self) {
        unsafe {
            lnfast_event_group_delete(self.ln);
        }
    }
}
impl rnFastEventGroup {
    pub fn new() -> Self {
        unsafe {
            let t: rnFastEventGroup = rnFastEventGroup {
                ln: lnfast_event_group_create(),
            };
            t
        }
    }

    //
    //
    //
    pub fn take_ownership(&mut self) {
        unsafe {
            lnfast_event_group_takeOwnership(self.ln);
        }
    }
    //
    //
    //
    pub fn set_events(&self, event: u32) {
        unsafe {
            lnfast_event_group_set_events(self.ln, event);
        }
    }
    //
    //
    //
    pub fn wait_events(&self, maskint: u32, timeout: i32) -> u32 {
        unsafe { lnfast_event_group_wait_events(self.ln, maskint, timeout) }
    }
    //
    //
    //
    pub fn read_events(&self, msk: u32) -> u32 {
        unsafe { lnfast_event_group_read_events(self.ln, msk) }
    }
}

//--
