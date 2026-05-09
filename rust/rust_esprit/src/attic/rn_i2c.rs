#![allow(dead_code)]
//use crate::rn_gpio::rnPin;
use crate::rn_i2c_c::*;

type cint = cty::c_int;
type uint = cty::c_uint;

pub struct rnI2C {
    ln: *mut ln_i2c_c,
}
/*
 *
 *
 */
impl rnI2C {
    pub fn new(instance: u32, speed: u32) -> rnI2C {
        unsafe {
            let t: rnI2C = rnI2C {
                ln: lni2c_create(instance as cint, speed as cint),
            };
            t
        }
    }

    pub fn set_speed(&mut self, speed: u32) {
        unsafe {
            lni2c_setSpeed(self.ln, speed as cint);
        }
    }
    pub fn set_address(&mut self, address: u8) {
        unsafe {
            lni2c_setAddress(self.ln, address as cint);
        }
    }
    pub fn begin(&mut self, target: u8) {
        unsafe {
            lni2c_begin(self.ln, target as cint);
        }
    }

    pub fn write(&mut self, data: &[u8]) -> bool {
        unsafe { lni2c_write(self.ln, data.len() as uint, data.as_ptr()) }
    }
    pub fn write_to(&mut self, target: u8, data: &[u8]) -> bool {
        unsafe { lni2c_write_to(self.ln, target as cint, data.len() as uint, data.as_ptr()) }
    }
    pub fn read(&mut self, data: &mut [u8]) -> bool {
        unsafe { lni2c_read(self.ln, data.len() as uint, data.as_ptr() as *mut u8) }
    }
    pub fn read_from(&mut self, target: u8, data: &mut [u8]) -> bool {
        unsafe {
            lni2c_read_from(
                self.ln,
                target as cint,
                data.len() as uint,
                data.as_ptr() as *mut u8,
            )
        }
    }

    pub fn multi_write_to(&mut self, tgt: u8, lengths: &[cty::c_uint], data: &[&[u8]]) -> bool {
        let nb = lengths.len();
        if nb != data.len() {
            panic!("Invalid multiwrite : length & data mismatch\n");
        }
        if nb == 0 {
            panic!("I2C  Zero multiwrite \n");
        }

        if nb > 3 {
            panic!("Oops");
        }
        let mut seqs: [u32; 3] = [0, 0, 0];
        for i in 0..nb {
            seqs[i] = data[i].as_ptr() as u32;
        }
        let sequence_data: *mut *const u8 = seqs.as_ptr() as *mut *const u8;
        let sequence_lengh: *const cty::c_uint = lengths.as_ptr() as *const cty::c_uint;
        unsafe {
            lni2c_multi_write_to(
                self.ln,
                tgt as cty::c_int,
                nb as cty::c_uint,
                sequence_lengh,
                sequence_data,
            )
        }
    }
}

// eof
