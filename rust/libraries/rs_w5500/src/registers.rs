//! W5500 register address definitions, block selection, and command/status codes.

// ---------------------------------------------------------------------------
// Block select bits (BSB[4:0] in control phase byte 2)
// ---------------------------------------------------------------------------

pub const CREG_BLOCK: u8 = 0x00;

#[inline(always)]
pub const fn sreg_block(sn: u8) -> u8 {
    1 + 4 * sn
}

#[inline(always)]
pub const fn txbuf_block(sn: u8) -> u8 {
    2 + 4 * sn
}

#[inline(always)]
pub const fn rxbuf_block(sn: u8) -> u8 {
    3 + 4 * sn
}

// ---------------------------------------------------------------------------
// Common registers (CREG_BLOCK, 16-bit offset)
// ---------------------------------------------------------------------------

pub const MR: u16 = 0x0000;       // Mode Register (1 byte)
pub const GAR: u16 = 0x0001;      // Gateway IP Address (4 bytes)
pub const SUBR: u16 = 0x0005;     // Subnet Mask Address (4 bytes)
pub const SHAR: u16 = 0x0009;     // Source Hardware (MAC) Address (6 bytes)
pub const SIPR: u16 = 0x000F;     // Source IP Address (4 bytes)
pub const INTLEVEL: u16 = 0x0013; // Interrupt Low-level Timer (2 bytes)
pub const IR: u16 = 0x0015;       // Interrupt Register (1 byte)
pub const IMR: u16 = 0x0016;      // Interrupt Mask (1 byte)
pub const SIR: u16 = 0x0017;      // Socket Interrupt Register (1 byte)
pub const SIMR: u16 = 0x0018;     // Socket Interrupt Mask (1 byte)
pub const RTR: u16 = 0x0019;      // Retransmission Timeout (2 bytes, 100us units)
pub const RCR: u16 = 0x001B;      // Retransmission Count (1 byte)
pub const PHYCFGR: u16 = 0x002E;  // PHY Configuration Register (1 byte)
pub const VERSIONR: u16 = 0x0039; // Chip Version Register (1 byte, reads 0x04)

// Mode Register (MR) bits
pub const MR_RST: u8 = 1 << 7;    // Software Reset
pub const MR_WOL: u8 = 1 << 5;    // Wake on LAN
pub const MR_PB: u8 = 1 << 4;     // Ping Block
pub const MR_PPPOE: u8 = 1 << 3;  // PPPoE mode
pub const MR_FARP: u8 = 1 << 1;   // Force ARP

// PHY Configuration Register (PHYCFGR) bits
pub const PHYCFGR_RST: u8 = 0 << 7;          // Inverted: 0 resets PHY
pub const PHYCFGR_NORMAL: u8 = 1 << 7;       // 1: Normal operation
pub const PHYCFGR_OPMD: u8 = 1 << 6;         // Configure PHY by S/W (1) or H/W (0)
pub const PHYCFGR_OPMDC_ALLA: u8 = 7 << 3;   // Auto-negotiation all capable
pub const PHYCFGR_OPMDC_PDOWN: u8 = 6 << 3;  // Power Down mode
pub const PHYCFGR_OPMDC_NA: u8 = 5 << 3;     // Not Used
pub const PHYCFGR_OPMDC_100FA: u8 = 4 << 3;  // 100M Full Auto-negotiation
pub const PHYCFGR_OPMDC_100F: u8 = 3 << 3;   // 100M Full duplex
pub const PHYCFGR_OPMDC_100H: u8 = 2 << 3;   // 100M Half duplex
pub const PHYCFGR_OPMDC_10F: u8 = 1 << 3;    // 10M Full duplex
pub const PHYCFGR_OPMDC_10H: u8 = 0 << 3;    // 10M Half duplex
pub const PHYCFGR_DPX_FULL: u8 = 1 << 2;     // Duplex status: 1 = Full, 0 = Half
pub const PHYCFGR_SPD_100: u8 = 1 << 1;      // Speed status: 1 = 100M, 0 = 10M
pub const PHYCFGR_LNK_ON: u8 = 1 << 0;       // Link status: 1 = Up, 0 = Down

// ---------------------------------------------------------------------------
// Socket registers (sreg_block(sn), 16-bit offset)
// ---------------------------------------------------------------------------

pub const Sn_MR: u16 = 0x0000;         // Socket Mode (1 byte)
pub const Sn_CR: u16 = 0x0001;         // Socket Command (1 byte)
pub const Sn_IR: u16 = 0x0002;         // Socket Interrupt (1 byte)
pub const Sn_SR: u16 = 0x0003;         // Socket Status (1 byte)
pub const Sn_PORT: u16 = 0x0004;       // Source Port (2 bytes)
pub const Sn_DHAR: u16 = 0x0006;       // Destination MAC (6 bytes)
pub const Sn_DIPR: u16 = 0x000C;       // Destination IP (4 bytes)
pub const Sn_DPORT: u16 = 0x0010;      // Destination Port (2 bytes)
pub const Sn_MSSR: u16 = 0x0012;       // Max Segment Size (2 bytes)
pub const Sn_RXBUF_SIZE: u16 = 0x001E; // RX Buffer Size in KB (1 byte)
pub const Sn_TXBUF_SIZE: u16 = 0x001F; // TX Buffer Size in KB (1 byte)
pub const Sn_TX_FSR: u16 = 0x0020;     // TX Free Size (2 bytes)
pub const Sn_TX_RD: u16 = 0x0022;      // TX Read Pointer (2 bytes)
pub const Sn_TX_WR: u16 = 0x0024;      // TX Write Pointer (2 bytes)
pub const Sn_RX_RSR: u16 = 0x0026;     // RX Received Size (2 bytes)
pub const Sn_RX_RD: u16 = 0x0028;      // RX Read Pointer (2 bytes)
pub const Sn_RX_WR: u16 = 0x002A;      // RX Write Pointer (2 bytes)
pub const Sn_IMR: u16 = 0x002C;        // Socket Interrupt Mask (1 byte)
pub const Sn_FRAG: u16 = 0x002D;       // Fragment field in IP header (2 bytes)
pub const Sn_KPALVTR: u16 = 0x002F;    // Keep Alive Timer (1 byte, 5s units)

// Socket Mode Register (Sn_MR) values
pub const Sn_MR_CLOSED: u8 = 0x00;
pub const Sn_MR_TCP: u8 = 0x01;
pub const Sn_MR_UDP: u8 = 0x02;
pub const Sn_MR_MACRAW: u8 = 0x04;

// Socket Command Register (Sn_CR) values
pub const Sn_CR_OPEN: u8 = 0x01;
pub const Sn_CR_LISTEN: u8 = 0x02;
pub const Sn_CR_CONNECT: u8 = 0x04;
pub const Sn_CR_DISCON: u8 = 0x08;
pub const Sn_CR_CLOSE: u8 = 0x10;
pub const Sn_CR_SEND: u8 = 0x20;
pub const Sn_CR_SEND_MAC: u8 = 0x21;
pub const Sn_CR_SEND_KEEP: u8 = 0x22;
pub const Sn_CR_RECV: u8 = 0x40;

// Socket Interrupt Register (Sn_IR) bits
pub const Sn_IR_CON: u8 = 1 << 0;     // Connection established
pub const Sn_IR_DISCON: u8 = 1 << 1;  // Disconnected
pub const Sn_IR_RECV: u8 = 1 << 2;    // Data received
pub const Sn_IR_TIMEOUT: u8 = 1 << 3; // Timeout
pub const Sn_IR_SENDOK: u8 = 1 << 4;  // Send completed

// Socket Status Register (Sn_SR)
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SocketStatus {
    Closed = 0x00,
    Init = 0x13,
    Listen = 0x14,
    SynSent = 0x15,
    SynRecv = 0x16,
    Established = 0x17,
    FinWait = 0x18,
    Closing = 0x1A,
    TimeWait = 0x1B,
    CloseWait = 0x1C,
    LastAck = 0x1D,
    Udp = 0x22,
    MacRaw = 0x42,
}

impl TryFrom<u8> for SocketStatus {
    type Error = u8;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(Self::Closed),
            0x13 => Ok(Self::Init),
            0x14 => Ok(Self::Listen),
            0x15 => Ok(Self::SynSent),
            0x16 => Ok(Self::SynRecv),
            0x17 => Ok(Self::Established),
            0x18 => Ok(Self::FinWait),
            0x1A => Ok(Self::Closing),
            0x1B => Ok(Self::TimeWait),
            0x1C => Ok(Self::CloseWait),
            0x1D => Ok(Self::LastAck),
            0x22 => Ok(Self::Udp),
            0x42 => Ok(Self::MacRaw),
            unknown => Err(unknown),
        }
    }
}

// Legacy constants
pub const SOCK_CLOSED: u8 = SocketStatus::Closed as u8;
pub const SOCK_INIT: u8 = SocketStatus::Init as u8;
pub const SOCK_LISTEN: u8 = SocketStatus::Listen as u8;
pub const SOCK_ESTABLISHED: u8 = SocketStatus::Established as u8;
pub const SOCK_CLOSE_WAIT: u8 = SocketStatus::CloseWait as u8;
pub const SOCK_UDP: u8 = SocketStatus::Udp as u8;
pub const SOCK_MACRAW: u8 = SocketStatus::MacRaw as u8;
