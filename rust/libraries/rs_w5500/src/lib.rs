//! `rs_w5500` — High-performance W5500 Ethernet driver in Rust for Esprit.

#![no_std]
#![allow(non_upper_case_globals)]

pub mod bus;
pub mod c_api;
pub mod coordinator;
pub mod device;
pub mod dhcp;
pub mod registers;
pub mod socket;

pub use bus::W5500Bus;
pub use coordinator::{Coordinator, CoordinatorState, w5500_coordinator_task};
pub use device::{W5500Device, W5500Pins};
pub use dhcp::{DhcpClient, DhcpInfo, DhcpState};
pub use registers::SocketStatus;
pub use socket::{Socket, SocketError, SocketEvent, SocketListener, W5500Socket, W5500SocketHandle};
