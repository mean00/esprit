#![allow(dead_code)]

use crate::rn_i2c_c;
use crate::rn_i2c_c::lni2c_multi_write_to;

pub use rn_i2c_c::ln_i2c_c;

/// Owned, non‑copy handle to an I²C peripheral.
///
/// Created via `I2cBus::new(instance, speed_hz)`.
/// The underlying C object is destroyed when `I2cBus` is dropped.
pub struct I2cBus {
    raw: *mut rn_i2c_c::ln_i2c_c,
}

impl I2cBus {
    /// Create a new I²C bus on hardware instance `instance` at `speed_hz` Hz.
    pub fn new(instance: i32, speed_hz: i32) -> Self {
        let raw = unsafe { rn_i2c_c::lni2c_create(instance, speed_hz) };
        assert!(!raw.is_null(), "lni2c_create returned NULL");
        Self { raw }
    }

    /// Returns a pointer to the raw C object.  Advanced use only.
    #[inline]
    pub fn raw(&self) -> *mut rn_i2c_c::ln_i2c_c {
        self.raw
    }

    /// Set bus speed in Hz.
    pub fn set_speed(&mut self, speed_hz: i32) {
        unsafe {
            rn_i2c_c::lni2c_setSpeed(self.raw, speed_hz);
        }
    }

    /// Set the default 7‑bit address for subsequent `begin` / `write` / `read` calls.
    pub fn set_address(&mut self, addr: u8) {
        unsafe {
            rn_i2c_c::lni2c_setAddress(self.raw, addr as i32);
        }
    }

    /// Initiate a START condition and send the target address (the one set by
    /// `set_address`). Returns `true` if the device acknowledged.
    pub fn begin(&mut self, addr: u8) -> bool {
        unsafe { rn_i2c_c::lni2c_begin(self.raw, addr as i32) }
    }

    /// Write `data` bytes to the currently addressed device.
    pub fn write(&mut self, data: &[u8]) -> bool {
        unsafe { rn_i2c_c::lni2c_write(self.raw, data.len() as u32, data.as_ptr()) }
    }

    /// Read `data.len()` bytes from the currently addressed device into `data`.
    pub fn read(&mut self, data: &mut [u8]) -> bool {
        unsafe { rn_i2c_c::lni2c_read(self.raw, data.len() as u32, data.as_mut_ptr()) }
    }

    /// Write `data` bytes to device at address `addr` (includes START/STOP).
    pub fn write_to(&mut self, addr: u8, data: &[u8]) -> bool {
        unsafe { rn_i2c_c::lni2c_write_to(self.raw, addr as i32, data.len() as u32, data.as_ptr()) }
    }

    /// Read `data.len()` bytes from device at address `addr` (includes START/STOP).
    pub fn read_from(&mut self, addr: u8, data: &mut [u8]) -> bool {
        unsafe {
            rn_i2c_c::lni2c_read_from(self.raw, addr as i32, data.len() as u32, data.as_mut_ptr())
        }
    }

    // ---------- convenience (like Arduino Wire.h) ----------

    /// Begin a transaction to `addr`, then write `data`.
    /// Equivalent to `begin_transmission(addr)` + `write(data)`.
    pub fn begin_transmission(&mut self, addr: u8) -> bool {
        self.begin(addr)
    }

    /// Write bytes (same as `write`) — for use after `begin_transmission`.
    pub fn send_bytes(&mut self, data: &[u8]) -> bool {
        self.write(data)
    }

    /// Request `len` bytes from `addr` and read them into `data`.
    pub fn request_from(&mut self, addr: u8, data: &mut [u8]) -> bool {
        self.read_from(addr, data)
    }

    /// Gather send from multiple buggfers
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
                self.raw,
                tgt as cty::c_int,
                nb as cty::c_uint,
                sequence_lengh,
                sequence_data,
            )
        }
    }
}

impl Drop for I2cBus {
    fn drop(&mut self) {
        unsafe {
            rn_i2c_c::lni2c_delete(self.raw);
        }
    }
}

