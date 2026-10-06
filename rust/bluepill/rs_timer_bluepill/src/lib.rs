#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;
pub mod timer;
pub mod dma_timer;
pub mod delay_timer;

pub use timer::{Timer, ChannelMode};
pub use dma_timer::{DmaTimer, DmaTimerError};
pub use delay_timer::{DelayTimer, DelayTimerCallback, delay_timer_interrupt_handler};

pub mod shim;
pub use shim::*;
