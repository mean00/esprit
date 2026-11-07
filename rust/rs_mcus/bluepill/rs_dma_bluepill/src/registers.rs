#![allow(dead_code)]
use rs_bluepill::{DMA0_BASE, DMA1_BASE};

// --- DMA Control Register (CCR) Bits ---
pub const DMA_CCR_EN: u32 = 1 << 0;
pub const DMA_CCR_TCIE: u32 = 1 << 1;
pub const DMA_CCR_HTIE: u32 = 1 << 2;
pub const DMA_CCR_TEIE: u32 = 1 << 3;
pub const DMA_CCR_DIR_MEM2PERIPH: u32 = 1 << 4;
pub const DMA_CCR_CIRC: u32 = 1 << 5;
pub const DMA_CCR_PINC: u32 = 1 << 6;
pub const DMA_CCR_MINC: u32 = 1 << 7;
pub const DMA_CCR_PSIZE_8BIT: u32 = 0 << 8;
pub const DMA_CCR_PSIZE_16BIT: u32 = 1 << 8;
pub const DMA_CCR_PSIZE_32BIT: u32 = 2 << 8;
pub const DMA_CCR_MSIZE_8BIT: u32 = 0 << 10;
pub const DMA_CCR_MSIZE_16BIT: u32 = 1 << 10;
pub const DMA_CCR_MSIZE_32BIT: u32 = 2 << 10;
pub const DMA_CCR_PL_HIGH: u32 = 2 << 12;

// --- DMA Interrupt Status / Clear Flags (ISR / IFCR) ---
pub const DMA_FLAG_GIF: u32 = 1 << 0;
pub const DMA_FLAG_TCIF: u32 = 1 << 1;
pub const DMA_FLAG_HTIF: u32 = 1 << 2;
pub const DMA_FLAG_TEIF: u32 = 1 << 3;
pub const DMA_FLAGS_PER_CHANNEL: u32 = 4;
pub const DMA_CHANNEL_CLEAR_ALL: u32 = 0x0F;

#[repr(C)]
pub struct DmaChannelRegisters {
    pub ccr: u32,
    pub cndtr: u32,
    pub cpar: u32,
    pub cmar: u32,
    _reserved: u32,
}

#[repr(C)]
pub struct DmaRegisters {
    pub isr: u32,
    pub ifcr: u32,
    pub channels: [DmaChannelRegisters; 7],
}

impl DmaRegisters {
    #[inline(always)]
    pub fn dma0_ptr() -> *mut Self {
        DMA0_BASE as *mut Self
    }
    
    #[inline(always)]
    pub fn dma1_ptr() -> *mut Self {
        DMA1_BASE as *mut Self
    }
}
