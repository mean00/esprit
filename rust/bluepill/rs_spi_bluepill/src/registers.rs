#![allow(dead_code)]
use rs_bluepill::{SPI1_BASE, SPI2_BASE, SPI3_BASE};

#[repr(C)]
pub struct SpiRegisters {
    pub cr1: u32,
    pub cr2: u32,
    pub sr: u32,
    pub dr: u32,
    pub crcpr: u32,
    pub rxcrcr: u32,
    pub txcrcr: u32,
    pub i2scfgr: u32,
    pub i2spr: u32,
}

pub const SPI_INSTANCE_0: u32 = 0;
pub const SPI_INSTANCE_1: u32 = 1;
pub const SPI_INSTANCE_2: u32 = 2;
pub const SPI_MAX_INSTANCES: usize = 3;

impl SpiRegisters {
    #[inline(always)]
    pub fn ptr(instance: u32) -> *mut Self {
        match instance {
            SPI_INSTANCE_0 => SPI1_BASE as *mut Self,
            SPI_INSTANCE_1 => SPI2_BASE as *mut Self,
            SPI_INSTANCE_2 => SPI3_BASE as *mut Self,
            _ => SPI1_BASE as *mut Self,
        }
    }
}

// --- CR1 / CTL0 Register Bits ---
pub const SPI_CR1_CPHA: u32 = 1 << 0;       // Clock phase
pub const SPI_CR1_CPOL: u32 = 1 << 1;       // Clock polarity
pub const SPI_CR1_MSTR: u32 = 1 << 2;       // Master configuration
pub const SPI_CR1_BR_MASK: u32 = 7 << 3;    // Baud rate control mask
pub const SPI_CR1_BR_SHIFT: u32 = 3;
pub const SPI_CR1_SPE: u32 = 1 << 6;        // SPI enable
pub const SPI_CR1_LSBFIRST: u32 = 1 << 7;   // Frame format: 1: LSB first, 0: MSB first
pub const SPI_CR1_SSI: u32 = 1 << 8;        // Internal slave select
pub const SPI_CR1_SSM: u32 = 1 << 9;        // Software slave management
pub const SPI_CR1_RXONLY: u32 = 1 << 10;    // Receive only
pub const SPI_CR1_DFF: u32 = 1 << 11;       // Data frame format (0: 8-bit, 1: 16-bit)
pub const SPI_CR1_CRCNEXT: u32 = 1 << 12;   // Transmit CRC next
pub const SPI_CR1_CRCEN: u32 = 1 << 13;     // Hardware CRC calculation enable
pub const SPI_CR1_BIDIOE: u32 = 1 << 14;    // Output enable in bidirectional mode
pub const SPI_CR1_BIDIMODE: u32 = 1 << 15;  // Bidirectional data mode enable

// Clock phase & polarity mask (Mode 0..3)
pub const SPI_CR1_MODE_MASK: u32 = SPI_CR1_CPHA | SPI_CR1_CPOL;

// --- CR2 / CTL1 Register Bits ---
pub const SPI_CR2_RXDMAEN: u32 = 1 << 0;    // Rx buffer DMA enable
pub const SPI_CR2_TXDMAEN: u32 = 1 << 1;    // Tx buffer DMA enable
pub const SPI_CR2_SSOE: u32 = 1 << 2;       // SS output enable
pub const SPI_CR2_ERRIE: u32 = 1 << 5;      // Error interrupt enable
pub const SPI_CR2_RXNEIE: u32 = 1 << 6;     // RX buffer not empty interrupt enable
pub const SPI_CR2_TXEIE: u32 = 1 << 7;      // Tx buffer empty interrupt enable

// --- SR / STAT Register Bits ---
pub const SPI_SR_RXNE: u32 = 1 << 0;        // Receive buffer not empty
pub const SPI_SR_TXE: u32 = 1 << 1;         // Transmit buffer empty
pub const SPI_SR_CHSIDE: u32 = 1 << 2;      // Channel side
pub const SPI_SR_UDR: u32 = 1 << 3;         // Underrun flag
pub const SPI_SR_CRCERR: u32 = 1 << 4;      // CRC error flag
pub const SPI_SR_MODF: u32 = 1 << 5;        // Mode fault
pub const SPI_SR_OVR: u32 = 1 << 6;         // Overrun flag
pub const SPI_SR_BSY: u32 = 1 << 7;         // Busy flag
pub const SPI_SR_FRE: u32 = 1 << 8;         // Frame format error

// --- DR Constants ---
pub const SPI_DR_DATA_8BIT_MASK: u32 = 0xFF;
pub const SPI_DR_DATA_16BIT_MASK: u32 = 0xFFFF;

// --- Data Size Constants ---
pub const SPI_DATA_SIZE_8: u32 = 8;
pub const SPI_DATA_SIZE_16: u32 = 16;

// --- GD32 Aliases ---
pub const SPI_CTL0_CKPH: u32 = SPI_CR1_CPHA;
pub const SPI_CTL0_CKPL: u32 = SPI_CR1_CPOL;
pub const SPI_CTL0_MSTMODE: u32 = SPI_CR1_MSTR;
pub const SPI_CTL0_SPIEN: u32 = SPI_CR1_SPE;
pub const SPI_CTL0_LSB: u32 = SPI_CR1_LSBFIRST;
pub const SPI_CTL0_SWNSS: u32 = SPI_CR1_SSI;
pub const SPI_CTL0_SWNSSEN: u32 = SPI_CR1_SSM;
pub const SPI_CTL0_RO: u32 = SPI_CR1_RXONLY;
pub const SPI_CTL0_FF16: u32 = SPI_CR1_DFF;
pub const SPI_CTL0_CRCNT: u32 = SPI_CR1_CRCNEXT;
pub const SPI_CTL0_CRCEN: u32 = SPI_CR1_CRCEN;
pub const SPI_CTL0_BDOEN: u32 = SPI_CR1_BIDIOE;
pub const SPI_CTL0_BDEN: u32 = SPI_CR1_BIDIMODE;

pub const SPI_CTL1_DMAREN: u32 = SPI_CR2_RXDMAEN;
pub const SPI_CTL1_DMATEN: u32 = SPI_CR2_TXDMAEN;
pub const SPI_CTL1_NSSDRV: u32 = SPI_CR2_SSOE;
pub const SPI_CTL1_ERRIE: u32 = SPI_CR2_ERRIE;
pub const SPI_CTL1_RBNEIE: u32 = SPI_CR2_RXNEIE;
pub const SPI_CTL1_TBEIE: u32 = SPI_CR2_TXEIE;

pub const SPI_STAT_RBNE: u32 = SPI_SR_RXNE;
pub const SPI_STAT_TBE: u32 = SPI_SR_TXE;
pub const SPI_STAT_I2SCH: u32 = SPI_SR_CHSIDE;
pub const SPI_STAT_TXURERR: u32 = SPI_SR_UDR;
pub const SPI_STAT_CRCERR: u32 = SPI_SR_CRCERR;
pub const SPI_STAT_CONFERR: u32 = SPI_SR_MODF;
pub const SPI_STAT_RXORERR: u32 = SPI_SR_OVR;
pub const SPI_STAT_TRANS: u32 = SPI_SR_BSY;
pub const SPI_STAT_FERR: u32 = SPI_SR_FRE;
