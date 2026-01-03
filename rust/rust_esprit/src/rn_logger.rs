#![allow(dead_code)]

use crate::rn_freertos_c::Logger_chars;

use core::convert::Infallible;
use ufmt::uWrite;

pub struct W;

impl uWrite for W {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
        unsafe {
            Logger_chars(s.len() as i32, s.as_ptr() as *const cty::c_char);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! lnLogger {

    ($x:expr) => {
        uwrite!(&mut W, "{}", $x).unwrap()
    };

    ($x:expr, $($y:expr),+) => {
        uwrite!(&mut W, $x, $($y),+).unwrap()
    };
}

#[macro_export]
macro_rules! lnLogger_init {
    () => {
        use rust_esprit::rn_logger::W;
        use ufmt::uwrite;
    };
}
