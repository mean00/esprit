//! Shared types, enums and traits for SocketRunner.

use core::ffi::c_void;
use rs_rtos::event::EventGroup;
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

/// Socket event enumeration.
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
    /// Converts an integer value to `SocketEvent`.
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

/// Asynchronous notification listener for socket lifecycle events.
#[derive(Clone)]
pub enum SocketListener {
    /// Dispatches events to a FreeRTOS `EventGroup` at the given slot bit shift.
    EventGroup {
        event_group: EventGroup,
        shift: u32,
    },
    /// Dispatches events to a custom callback function.
    Callback {
        callback: fn(SocketEvent, usize),
        ctx: usize,
    },
    /// Dispatches events to a legacy C ABI callback function.
    CCallback {
        callback: extern "C" fn(u32, *mut c_void),
        arg: *mut c_void,
    },
    None,
}

// SAFETY: the `CCallback` cookie is an opaque pointer that is only handed back to
// the C callback; nothing here dereferences it, so a listener can move between
// tasks. (The `fn` pointers are `Send` already.)
unsafe impl Send for SocketListener {}

impl SocketListener {
    pub fn from_event_group(event_group: EventGroup, shift: u32) -> Self {
        Self::EventGroup { event_group, shift }
    }

    pub fn from_callback(callback: fn(SocketEvent, usize), ctx: usize) -> Self {
        Self::Callback { callback, ctx }
    }

    pub fn from_c_callback(callback: extern "C" fn(u32, *mut c_void), arg: *mut c_void) -> Self {
        Self::CCallback { callback, arg }
    }

    pub fn none() -> Self {
        Self::None
    }

    #[inline(always)]
    pub fn notify(&self, event: SocketEvent) {
        match self {
            Self::EventGroup { event_group, shift } => {
                let revt = event.to_runner_event();
                if revt != RunnerEvent::NONE {
                    event_group.set_events(revt.shifted(*shift));
                }
            }
            Self::Callback { callback, ctx } => {
                (callback)(event, *ctx);
            }
            Self::CCallback { callback, arg } => {
                (callback)(event as u32, *arg);
            }
            Self::None => {}
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
