//! Shared types, enums, traits, and RAII guards for SocketRunner.

use core::ops::Deref;
use crate::constants::*;

// ---------------------------------------------------------------------------
//  RunnerEvent Bitfield
// ---------------------------------------------------------------------------

/// Strongly-typed event bitfield for SocketRunner event group multiplexing.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RunnerEvent(pub u32);

impl RunnerEvent {
    pub const NONE: Self          = Self(0);
    pub const CONNECTED: Self     = Self(1 << CONNECTED_BIT);
    pub const DISCONNECTED: Self  = Self(1 << DISCONNECTED_BIT);
    pub const DATA_AVAILABLE: Self= Self(1 << DATA_AVAILABLE_BIT);
    pub const CAN_WRITE: Self     = Self(1 << CAN_WRITE_BIT);
    pub const ERROR: Self         = Self(1 << ERROR_BIT);
    pub const CUSTOM_EVENT: Self  = Self(1 << CUSTOM_EVENT_BIT);
    pub const UP: Self            = Self(1 << LINK_UP_BIT);
    pub const DOWN: Self          = Self(1 << LINK_DOWN_BIT);
    pub const MASK: u32           = SLOT_EVENT_MASK;

    /// Checks if this event bitfield contains all bits of `other`.
    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0 && other.0 != 0
    }

    /// Checks if this event bitfield intersects any bit of `other`.
    #[inline(always)]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Applies slot bit-shifting if the event is a local slot event.
    /// Global link events (`UP` and `DOWN`) are not shifted.
    #[inline(always)]
    pub const fn shifted(self, shift: u32) -> u32 {
        if self.0 < Self::UP.0 {
            self.0 << shift
        } else {
            self.0
        }
    }

    /// Extracts the local slot events given a slot shift.
    #[inline(always)]
    pub const fn extract_local(events: u32, shift: u32) -> Self {
        Self((events >> shift) & SLOT_EVENT_MASK)
    }

    /// Extracts global link events (UP / DOWN).
    #[inline(always)]
    pub const fn extract_global(events: u32) -> Self {
        Self(events & GLOBAL_EVENT_MASK)
    }
}

impl core::ops::BitOr for RunnerEvent {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd for RunnerEvent {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

// ---------------------------------------------------------------------------
//  Socket Events & Status Enums
// ---------------------------------------------------------------------------

/// C-ABI compatible socket event enumeration matching `lnSocketEvent`.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SocketEvent {
    Ignore = 0,
    Connected = 1,
    Disconnect = 2,
    DataAvailable = 3,
    WriteAvailable = 4,
    Error = 5,
    Custom = 6,
}

impl SocketEvent {
    /// Converts a raw C integer value to `SocketEvent`.
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(Self::Ignore),
            1 => Some(Self::Connected),
            2 => Some(Self::Disconnect),
            3 => Some(Self::DataAvailable),
            4 => Some(Self::WriteAvailable),
            5 => Some(Self::Error),
            6 => Some(Self::Custom),
            _ => None,
        }
    }

    /// Maps the socket event to corresponding `RunnerEvent`.
    pub fn to_runner_event(self) -> RunnerEvent {
        match self {
            Self::Connected => RunnerEvent::CONNECTED,
            Self::Disconnect => RunnerEvent::DISCONNECTED,
            Self::DataAvailable => RunnerEvent::DATA_AVAILABLE,
            Self::WriteAvailable => RunnerEvent::CAN_WRITE,
            Self::Error => RunnerEvent::ERROR,
            Self::Custom => RunnerEvent::CUSTOM_EVENT,
            Self::Ignore => RunnerEvent::NONE,
        }
    }
}

/// Socket operation status.
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SocketStatus {
    Ok = 0,
    Error = 1,
}

/// Strongly-typed socket runner errors.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SocketError {
    CreationFailed,
    AsyncModeFailed,
    AcceptFailed,
    ReadFailed,
    WriteFailed,
    NotConnected,
    Disconnected,
    BufferFull,
}

// ---------------------------------------------------------------------------
//  RAII Zero-Copy Read Guard
// ---------------------------------------------------------------------------

/// Trait implemented by socket backends to release read data.
pub trait ReadReleaser {
    fn release_read_data(&mut self);
}

/// RAII guard providing safe zero-copy access to socket receive data.
/// Automatically releases the receive buffer on drop.
pub struct RxPacket<'a, R: ReadReleaser> {
    releaser: &'a mut R,
    data: &'a [u8],
}

impl<'a, R: ReadReleaser> RxPacket<'a, R> {
    pub fn new(releaser: &'a mut R, data: &'a [u8]) -> Self {
        Self { releaser, data }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<'a, R: ReadReleaser> Deref for RxPacket<'a, R> {
    type Target = [u8];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, R: ReadReleaser> Drop for RxPacket<'a, R> {
    fn drop(&mut self) {
        self.releaser.release_read_data();
    }
}

// ---------------------------------------------------------------------------
//  Runner Hook Trait
// ---------------------------------------------------------------------------

/// Trait implemented by applications to receive lifecycle events from SocketRunner.
pub trait SocketRunnerHandler {
    /// Invoked when a client connects to the listening socket.
    fn on_connected(&mut self);

    /// Invoked when the client disconnects.
    fn on_disconnected(&mut self);

    /// Periodic poll hook invoked every event cycle.
    fn on_poll(&mut self);

    /// Invoked when new data has arrived in the socket RX buffer.
    fn on_incoming_data(&mut self);
}
