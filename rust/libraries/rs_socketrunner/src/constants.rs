//! Constants for SocketRunner and network event multiplexing.
//!
//! All bit definitions, masks, slot shifts, timeouts, and buffer configurations
//! are gathered here to avoid magic numbers throughout the codebase.

// ---------------------------------------------------------------------------
//  Bit Position Definitions
// ---------------------------------------------------------------------------

/// Per-slot event bit: Client connection established.
pub const CONNECTED_BIT: u32 = 0;

/// Per-slot event bit: Client disconnected.
pub const DISCONNECTED_BIT: u32 = 1;

/// Per-slot event bit: Incoming data available in socket RX buffer.
pub const DATA_AVAILABLE_BIT: u32 = 2;

/// Per-slot event bit: Socket TX buffer has space for writing.
pub const CAN_WRITE_BIT: u32 = 3;

/// Per-slot event bit: Socket encountered an error.
pub const ERROR_BIT: u32 = 4;

/// Per-slot event bit: Custom hardware interrupt notification (e.g. W5500).
pub const CUSTOM_EVENT_BIT: u32 = 5;

/// Global event bit: Network link up (DHCP leased / interface ready).
pub const LINK_UP_BIT: u32 = 30;

/// Global event bit: Network link down (cable unplugged / interface lost).
pub const LINK_DOWN_BIT: u32 = 31;

// ---------------------------------------------------------------------------
//  Slot Multiplexing Configuration
// ---------------------------------------------------------------------------

/// Width in bits allocated per runner slot in the 32-bit EventGroup.
pub const SLOT_WIDTH_BITS: u32 = 6;

/// Bitmask covering all local event bits within a single slot (bits 0..5).
pub const SLOT_EVENT_MASK: u32 = (1 << SLOT_WIDTH_BITS) - 1;

/// Bitmask covering all global link events (bits 30..31).
pub const GLOBAL_EVENT_MASK: u32 = (1 << LINK_UP_BIT) | (1 << LINK_DOWN_BIT);

/// Mask to wait for all events in a slot, including global events.
pub const ALL_EVENTS_MASK: u32 = 0xFFFF_FFFF;

// ---------------------------------------------------------------------------
//  Standard Slot Shift Offsets
// ---------------------------------------------------------------------------

/// Standard slot index and shift offset for GDB server.
pub const GDB_SLOT_INDEX: u32 = 0;
pub const GDB_SLOT_SHIFT: u32 = GDB_SLOT_INDEX * SLOT_WIDTH_BITS; // 0

/// Standard slot index and shift offset for RTT data streaming.
pub const RTT_SLOT_INDEX: u32 = 1;
pub const RTT_SLOT_SHIFT: u32 = RTT_SLOT_INDEX * SLOT_WIDTH_BITS; // 6

/// Standard slot index and shift offset for Virtual Serial bridge.
pub const SERIAL_SLOT_INDEX: u32 = 2;
pub const SERIAL_SLOT_SHIFT: u32 = SERIAL_SLOT_INDEX * SLOT_WIDTH_BITS; // 12

// ---------------------------------------------------------------------------
//  Standard Port Defaults
// ---------------------------------------------------------------------------

/// Default TCP port for GDB remote protocol.
pub const DEFAULT_GDB_PORT: u16 = 2000;

/// Default TCP port for RTT streaming output.
pub const DEFAULT_RTT_PORT: u16 = 2001;

/// Default TCP port for Echo demonstrator server.
pub const DEFAULT_ECHO_PORT: u16 = 3000;

// ---------------------------------------------------------------------------
//  Buffer Sizes and Timeouts
// ---------------------------------------------------------------------------

/// Default staging write buffer size in bytes for socketRunner instances.
pub const DEFAULT_WRITE_BUFFER_SIZE: usize = 64;

/// Timeout in milliseconds when waiting for socket write availability.
pub const WAIT_WRITE_TIMEOUT_MS: i32 = 10;

/// Default event loop poll period in milliseconds.
pub const DEFAULT_EVENT_WAIT_TIMEOUT_MS: i32 = 20;

/// Default long event wait timeout in milliseconds (e.g. echo server demo).
pub const LONG_EVENT_WAIT_TIMEOUT_MS: i32 = 100;
