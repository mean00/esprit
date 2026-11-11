//! `rs_socketrunner` — Event-driven socket runner engine in Rust.
//!
//! Provides a safe, zero-copy, event-multiplexed TCP server engine
//! for microcontroller networking (GDB server, RTT streaming, echo demonstrators).

#![no_std]

pub mod constants;
pub mod runner;
pub mod socket;
pub mod types;

// Flatten key symbols for convenient consumption
pub use constants::*;
pub use runner::*;
pub use socket::*;
pub use types::*;
pub use rs_rtos::event::EventGroup;
