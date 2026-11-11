//! `rs_w5500` — High-performance W5500 Ethernet driver in Rust for Esprit.

#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(unused_imports)]

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
pub use socket::{ReadReleaser, RxPacket, Socket, SocketError, SocketEvent, SocketListener, W5500Socket, W5500SocketHandle};
