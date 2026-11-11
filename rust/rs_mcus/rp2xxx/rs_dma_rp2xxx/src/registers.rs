//! Hardware register layout and bit definitions for the RP2040 DMA controller.
//!
//! Defined according to Section 2.5.7 of the RP2040 Datasheet.

#![allow(dead_code)]

/// Macro to calculate the byte offset of a struct field.
#[macro_export]
macro_rules! dma_reg_offset {
    ($type:ty, $field:ident) => {
        core::mem::offset_of!($type, $field) as u32
    };
}

/// Hardware register block for a single DMA channel (0x40 bytes).
///
/// Contains the main registers (Alias 0) and trigger aliases 1, 2, and 3.
#[repr(C)]
pub struct DmaChannelRegisters {
    // Alias 0 (Standard access, trigger on ctrl_trig)
    pub read_addr: u32,
    pub write_addr: u32,
    pub trans_count: u32,
    pub ctrl_trig: u32,

    // Alias 1 (Trigger on trans_count_trig)
    pub al1_ctrl: u32,
    pub al1_read_addr: u32,
    pub al1_write_addr: u32,
    pub al1_trans_count_trig: u32,

    // Alias 2 (Trigger on write_addr_trig)
    pub al2_ctrl: u32,
    pub al2_trans_count: u32,
    pub al2_read_addr: u32,
    pub al2_write_addr_trig: u32,

    // Alias 3 (Trigger on read_addr_trig)
    pub al3_ctrl: u32,
    pub al3_write_addr: u32,
    pub al3_trans_count: u32,
    pub al3_read_addr_trig: u32,
}

impl DmaChannelRegisters {
    #[inline(always)]
    pub unsafe fn read_ctrl_trig(base: *mut Self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*base).ctrl_trig)) }
    }

    #[inline(always)]
    pub unsafe fn write_ctrl_trig(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).ctrl_trig), val) }
    }

    #[inline(always)]
    pub unsafe fn write_read_addr(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).read_addr), val) }
    }

    #[inline(always)]
    pub unsafe fn write_write_addr(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).write_addr), val) }
    }

    #[inline(always)]
    pub unsafe fn write_trans_count(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).trans_count), val) }
    }

    #[inline(always)]
    pub unsafe fn write_al1_ctrl(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).al1_ctrl), val) }
    }
}

/// Full DMA peripheral memory-mapped register block.
#[repr(C)]
pub struct DmaRegisters {
    pub channels: [DmaChannelRegisters; 12],
    _reserved_padding: [u32; 64], // 0x300 to 0x400
    pub intr: u32,               // 0x400 Interrupt Status (raw)
    pub inte0: u32,              // 0x404 Interrupt Enables for IRQ 0
    pub intf0: u32,              // 0x408 Force Interrupts for IRQ 0
    pub ints0: u32,              // 0x40C Interrupt Status for IRQ 0
    _reserved_irq: u32,          // 0x410
    pub inte1: u32,              // 0x414 Interrupt Enables for IRQ 1
    pub intf1: u32,              // 0x418 Force Interrupts for IRQ 1
    pub ints1: u32,              // 0x41C Interrupt Status for IRQ 1
    pub timer0: u32,             // 0x420 Pacing Timer 0
    pub timer1: u32,             // 0x424 Pacing Timer 1
    pub timer2: u32,             // 0x428 Pacing Timer 2
    pub timer3: u32,             // 0x42C Pacing Timer 3
    pub multi_chan_trigger: u32, // 0x430 Multi-channel trigger
    pub sniff_ctrl: u32,         // 0x434 Sniffer Control
    pub sniff_data: u32,         // 0x438 Sniffer Data accumulator
    _reserved_sniff: u32,        // 0x43C
    pub fifo_levels: u32,        // 0x440 FIFO Levels
    pub chan_abort: u32,         // 0x444 Channel Abort
}

impl DmaRegisters {
    #[inline(always)]
    pub unsafe fn read_inte1(base: *mut Self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*base).inte1)) }
    }

    #[inline(always)]
    pub unsafe fn write_inte1(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).inte1), val) }
    }

    #[inline(always)]
    pub unsafe fn modify_inte1(base: *mut Self, set: u32, clear: u32) {
        unsafe {
            let val = Self::read_inte1(base);
            Self::write_inte1(base, (val | set) & !clear);
        }
    }

    #[inline(always)]
    pub unsafe fn read_ints1(base: *mut Self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*base).ints1)) }
    }

    #[inline(always)]
    pub unsafe fn write_ints1(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).ints1), val) }
    }

    #[inline(always)]
    pub unsafe fn read_chan_abort(base: *mut Self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*base).chan_abort)) }
    }

    #[inline(always)]
    pub unsafe fn write_chan_abort(base: *mut Self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*base).chan_abort), val) }
    }
}

// --- Base Addresses re-exported from rs_rp2xxx ---
pub use rs_rp2xxx::common::DMA_BASE;
pub use rs_rp2xxx::rp2040::RESETS_BASE;

// --- Subsystem Reset Registers ---
pub const RESETS_RESET_OFFSET: u32 = 0x00;
pub const RESETS_RESET_DONE_OFFSET: u32 = 0x08;
pub const RESETS_RESET_ADDR: u32 = RESETS_BASE + RESETS_RESET_OFFSET;
pub const RESETS_RESET_DONE_ADDR: u32 = RESETS_BASE + RESETS_RESET_DONE_OFFSET;

pub const RESET_BIT_DMA: u32 = 1 << 2;
pub const RESET_BIT_SPI0: u32 = 1 << 16;
pub const RESET_BIT_SPI1: u32 = 1 << 17;

// --- Offsets within DmaChannelRegisters via Macro ---
pub const DMA_CH_READ_ADDR_OFFSET: u32 = dma_reg_offset!(DmaChannelRegisters, read_addr);
pub const DMA_CH_WRITE_ADDR_OFFSET: u32 = dma_reg_offset!(DmaChannelRegisters, write_addr);
pub const DMA_CH_TRANS_COUNT_OFFSET: u32 = dma_reg_offset!(DmaChannelRegisters, trans_count);
pub const DMA_CH_CTRL_TRIG_OFFSET: u32 = dma_reg_offset!(DmaChannelRegisters, ctrl_trig);

pub const DMA_CH_AL1_WRITE_ADDR_OFFSET: u32 = dma_reg_offset!(DmaChannelRegisters, al1_write_addr);
pub const DMA_CH_AL1_TRANS_COUNT_TRIG_OFFSET: u32 =
    dma_reg_offset!(DmaChannelRegisters, al1_trans_count_trig);

pub const DMA_CH_AL3_TRANS_COUNT_OFFSET: u32 = dma_reg_offset!(DmaChannelRegisters, al3_trans_count);
pub const DMA_CH_AL3_READ_ADDR_TRIG_OFFSET: u32 =
    dma_reg_offset!(DmaChannelRegisters, al3_read_addr_trig);

// --- Offsets within DmaRegisters via Macro ---
pub const DMA_INTR_OFFSET: u32 = dma_reg_offset!(DmaRegisters, intr);
pub const DMA_INTE0_OFFSET: u32 = dma_reg_offset!(DmaRegisters, inte0);
pub const DMA_INTS0_OFFSET: u32 = dma_reg_offset!(DmaRegisters, ints0);
pub const DMA_INTE1_OFFSET: u32 = dma_reg_offset!(DmaRegisters, inte1);
pub const DMA_INTS1_OFFSET: u32 = dma_reg_offset!(DmaRegisters, ints1);
pub const DMA_MULTI_CHAN_TRIGGER_OFFSET: u32 = dma_reg_offset!(DmaRegisters, multi_chan_trigger);
pub const DMA_CHAN_ABORT_OFFSET: u32 = dma_reg_offset!(DmaRegisters, chan_abort);

// --- Global Register Addresses ---
pub const DMA_INTR_ADDR: u32 = DMA_BASE + DMA_INTR_OFFSET;
pub const DMA_INTE0_ADDR: u32 = DMA_BASE + DMA_INTE0_OFFSET;
pub const DMA_INTS0_ADDR: u32 = DMA_BASE + DMA_INTS0_OFFSET;
pub const DMA_INTE1_ADDR: u32 = DMA_BASE + DMA_INTE1_OFFSET;
pub const DMA_INTS1_ADDR: u32 = DMA_BASE + DMA_INTS1_OFFSET;
pub const DMA_MULTI_CHAN_TRIGGER_ADDR: u32 = DMA_BASE + DMA_MULTI_CHAN_TRIGGER_OFFSET;
pub const DMA_CHAN_ABORT_ADDR: u32 = DMA_BASE + DMA_CHAN_ABORT_OFFSET;

// --- DMA Channel CTRL_TRIG Bitfields (Datasheet §2.5.7 Table 124) ---
pub const DMA_CTRL_EN: u32 = 1 << 0;
pub const DMA_CTRL_HIGH_PRIORITY: u32 = 1 << 1;
pub const DMA_CTRL_DATA_SIZE_8: u32 = 0 << 2;
pub const DMA_CTRL_DATA_SIZE_16: u32 = 1 << 2;
pub const DMA_CTRL_DATA_SIZE_32: u32 = 2 << 2;
pub const DMA_CTRL_DATA_SIZE_MASK: u32 = 3 << 2;
pub const DMA_CTRL_DATA_SIZE_SHIFT: u32 = 2;
pub const DMA_CTRL_INCR_READ: u32 = 1 << 4;
pub const DMA_CTRL_INCR_WRITE: u32 = 1 << 5;
pub const DMA_CTRL_RING_SIZE_SHIFT: u32 = 6;
pub const DMA_CTRL_RING_SIZE_MASK: u32 = 0xF << 6;
pub const DMA_CTRL_RING_SEL_WRITE: u32 = 1 << 10;
pub const DMA_CTRL_CHAIN_TO_SHIFT: u32 = 11;
pub const DMA_CTRL_CHAIN_TO_MASK: u32 = 0xF << 11;
pub const DMA_CTRL_TREQ_SHIFT: u32 = 15;
pub const DMA_CTRL_TREQ_MASK: u32 = 0x3F << 15;
pub const DMA_CTRL_IRQ_QUIET: u32 = 1 << 21;
pub const DMA_CTRL_BSWAP: u32 = 1 << 22;
pub const DMA_CTRL_SNIFF_EN: u32 = 1 << 23;
pub const DMA_CTRL_BUSY: u32 = 1 << 24;
pub const DMA_CTRL_WRITE_ERROR: u32 = 1 << 29;
pub const DMA_CTRL_READ_ERROR: u32 = 1 << 30;
pub const DMA_CTRL_AHB_ERROR: u32 = 1 << 31;
