#![allow(dead_code)]
use rs_bluepill::{DMA0_BASE, DMA1_BASE};

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
