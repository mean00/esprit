//! PL011 register map of the RP2xxx UARTs.
//!
//! `UART0` and `UART1` are the same PrimeCell PL011, at different bases (RP2040
//! datasheet §4.2, RP2350 §12.2). The bases come from `rs_rp2xxx`'s root
//! re-export; everything inside the block is identical on both chips.
#![allow(dead_code)]

use rs_rp2xxx::{UART0_BASE, UART1_BASE};

#[repr(C)]
pub struct UartRegisters {
    pub dr: u32,       // 0x00 Data
    pub rsr_ecr: u32,  // 0x04 Receive status / error clear
    _reserved_08: u32, // 0x08
    _reserved_0c: u32, // 0x0c
    _reserved_10: u32, // 0x10
    _reserved_14: u32, // 0x14
    pub fr: u32,       // 0x18 Flags
    _reserved_1c: u32, // 0x1c
    pub ilpr: u32,     // 0x20 IrDA low-power counter
    pub ibrd: u32,     // 0x24 Integer baud divisor
    pub fbrd: u32,     // 0x28 Fractional baud divisor
    pub lcr_h: u32,    // 0x2c Line control
    pub cr: u32,       // 0x30 Control
    pub ifls: u32,     // 0x34 Interrupt FIFO level select
    pub imsc: u32,     // 0x38 Interrupt mask set/clear
    pub ris: u32,      // 0x3c Raw interrupt status
    pub mis: u32,      // 0x40 Masked interrupt status
    pub icr: u32,      // 0x44 Interrupt clear
    pub dmacr: u32,    // 0x48 DMA control
}

const _: () = assert!(
    core::mem::offset_of!(UartRegisters, fr) == 0x18
        && core::mem::offset_of!(UartRegisters, ibrd) == 0x24
        && core::mem::offset_of!(UartRegisters, lcr_h) == 0x2c
        && core::mem::offset_of!(UartRegisters, cr) == 0x30
        && core::mem::offset_of!(UartRegisters, mis) == 0x40
        && core::mem::offset_of!(UartRegisters, dmacr) == 0x48,
    "PL011: FR at 0x18, IBRD/FBRD at 0x24/0x28, LCR_H/CR at 0x2c/0x30"
);

/// The two UART blocks of the chip.
#[inline(always)]
pub const fn base(instance: u32) -> Option<u32> {
    match instance {
        0 => Some(UART0_BASE),
        1 => Some(UART1_BASE),
        _ => None,
    }
}

/// Number of instances this family has.
pub const UART_INSTANCES: usize = 2;

// --- FR (0x18) ---------------------------------------------------------------

/// The transmit FIFO cannot take another byte.
pub const UART_FR_TXFF: u32 = 1 << 5;
/// The receive FIFO holds nothing.
pub const UART_FR_RXFE: u32 = 1 << 4;
/// The shifter is still sending.
pub const UART_FR_BUSY: u32 = 1 << 3;

// --- LCR_H (0x2c) ------------------------------------------------------------

/// Enable the FIFOs (without it TX/RX are one byte deep).
pub const UART_LCR_H_FEN: u32 = 1 << 4;
/// Word length, bits 6:5 (`3` = 8 bits).
pub const UART_LCR_H_WLEN_LSB: u32 = 5;
pub const UART_LCR_H_WLEN_MASK: u32 = 0x3 << UART_LCR_H_WLEN_LSB;
pub const UART_LCR_H_WLEN_8: u32 = 3 << UART_LCR_H_WLEN_LSB;
pub const UART_LCR_H_WLEN_7: u32 = 2 << UART_LCR_H_WLEN_LSB;
/// Two stop bits (only valid with a 6/7/8-bit word).
pub const UART_LCR_H_STP: u32 = 1 << 3;
/// Parity enable.
pub const UART_LCR_H_PEN: u32 = 1 << 1;
/// Even parity select.
pub const UART_LCR_H_EPS: u32 = 1 << 2;

// --- CR (0x30) ---------------------------------------------------------------

pub const UART_CR_UARTEN: u32 = 1 << 0;
pub const UART_CR_TXE: u32 = 1 << 8;
pub const UART_CR_RXE: u32 = 1 << 9;

// --- IMSC / ICR (0x38 / 0x44) ------------------------------------------------

pub const UART_INT_RX: u32 = 1 << 4;
pub const UART_INT_TX: u32 = 1 << 5;
pub const UART_INT_RT: u32 = 1 << 6;
pub const UART_INT_OE: u32 = 1 << 10;

/// IO_BANK0 FUNCSEL that muxes a pad to the UART (the pad decides which instance).
pub const FUNCSEL_UART: u32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    /// Both chips have the same two UARTs; a third instance names no block.
    #[test]
    fn only_two_instances_exist() {
        assert!(base(0).is_some());
        assert!(base(1).is_some());
        assert_eq!(base(2), None);
    }
}
