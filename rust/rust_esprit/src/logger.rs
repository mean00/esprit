#![allow(dead_code)]

use crate::prelude::*;
use crate::rn_debug_c;
use ufmt::uWrite;
use core::result::Result;
use core::result::Result::Ok;

/// A `ufmt` writer that sends formatted output through the C `Logger_chars`
/// function (which typically routes to UART / RTT / semihosting).
pub struct LoggerWriter;

impl uWrite for LoggerWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
        unsafe {
            rn_debug_c::Logger_chars(s.len() as i32, s.as_ptr() as *const cty::c_char);
        }
        Ok(())
    }
}

/// Print a formatted string via the system logger.
///
/// Internally uses `ufmt::uwrite!`.  Usage:
/// ```ignore
/// logger!("Hello {}", 42);
/// logger!("x={} y={}", x, y);
/// ```
#[macro_export]
macro_rules! logger {

    ($x:expr) => {
        uwrite!(&mut LoggerWriter, "{}", $x).unwrap()
    };

    ($x:expr, $($y:expr),+) => {
        uwrite!(&mut LoggerWriter, $x, $($y),+).unwrap()
    };
}

/// Initialise the C logger subsystem and import the `logger!` macro.
///
/// Call this once at startup:
/// ```ignore
/// logger_init!();
/// ```
#[macro_export]
macro_rules! logger_init {
    () => {
        use rust_esprit::logger::LoggerWriter;
        use ufmt::uwrite;
    };
}

