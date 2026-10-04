#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use core::ptr::{read_volatile, write_volatile};

#[derive(Clone, Copy)]
pub enum DmaEngine {
    Dma0, // DMA1 in STM32 terminology
    Dma1, // DMA2 in STM32 terminology
}

pub struct DmaChannel {
    engine: DmaEngine,
    channel_idx: usize, // 0 to 6
}

impl DmaChannel {
    pub fn new(engine: DmaEngine, channel_idx: usize) -> Self {
        match engine {
            DmaEngine::Dma0 => enable(Peripheral::Dma0),
            DmaEngine::Dma1 => enable(Peripheral::Dma1),
        }
        Self { engine, channel_idx }
    }
    
    fn regs(&self) -> *mut DmaRegisters {
        match self.engine {
            DmaEngine::Dma0 => DmaRegisters::dma0_ptr(),
            DmaEngine::Dma1 => DmaRegisters::dma1_ptr(),
        }
    }

    pub fn begin_tx_transfer(&mut self, peripheral_addr: u32, memory_addr: u32, length: u32, src_16bit: bool, dst_16bit: bool) {
        let regs = self.regs();
        unsafe {
            let ch_regs = &mut (*regs).channels[self.channel_idx];
            
            // 1. Disable channel before configuring
            let mut ccr = read_volatile(&mut ch_regs.ccr);
            ccr &= !DMA_CCR_EN;
            write_volatile(&mut ch_regs.ccr, ccr);
            
            // 2. Clear interrupt flags for this channel in IFCR
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            write_volatile(&mut (*regs).ifcr, DMA_CHANNEL_CLEAR_ALL << shift);
            
            // 3. Set Peripheral and Memory addresses
            write_volatile(&mut ch_regs.cpar, peripheral_addr);
            write_volatile(&mut ch_regs.cmar, memory_addr);
            
            // 4. Set Length
            write_volatile(&mut ch_regs.cndtr, length);
            
            // 5. Configure CCR: Memory increment, DIR=MemToPeriph, sizes
            ccr = DMA_CCR_MINC | DMA_CCR_DIR_MEM2PERIPH;
            
            if src_16bit { ccr |= DMA_CCR_MSIZE_16BIT; }
            if dst_16bit { ccr |= DMA_CCR_PSIZE_16BIT; }
            
            // Enable Transfer Complete Interrupt (TCIE)
            ccr |= DMA_CCR_TCIE;
            
            // Enable channel
            ccr |= DMA_CCR_EN;
            write_volatile(&mut ch_regs.ccr, ccr);
        }
    }

    /// Begin a circular memory-to-peripheral transfer.
    pub fn begin_circular_tx_transfer(
        &mut self,
        peripheral_addr: u32,
        memory_addr: u32,
        length: u32,
        src_16bit: bool,
        dst_16bit: bool,
    ) {
        let regs = self.regs();
        unsafe {
            let ch_regs = &mut (*regs).channels[self.channel_idx];
            
            // 1. Disable channel before configuring
            let mut ccr = read_volatile(&mut ch_regs.ccr);
            ccr &= !DMA_CCR_EN;
            write_volatile(&mut ch_regs.ccr, ccr);
            
            // 2. Clear interrupt flags for this channel in IFCR
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            write_volatile(&mut (*regs).ifcr, DMA_CHANNEL_CLEAR_ALL << shift);
            
            // 3. Set Peripheral and Memory addresses
            write_volatile(&mut ch_regs.cpar, peripheral_addr);
            write_volatile(&mut ch_regs.cmar, memory_addr);
            
            // 4. Set Length
            write_volatile(&mut ch_regs.cndtr, length);
            
            // 5. Configure CCR: Circular mode, Memory increment, DIR=MemToPeriph, High Priority
            ccr = DMA_CCR_DIR_MEM2PERIPH | DMA_CCR_CIRC | DMA_CCR_MINC | DMA_CCR_PL_HIGH;
            
            if src_16bit { ccr |= DMA_CCR_MSIZE_16BIT; } else { ccr |= DMA_CCR_MSIZE_8BIT; }
            if dst_16bit { ccr |= DMA_CCR_PSIZE_16BIT; } else { ccr |= DMA_CCR_PSIZE_8BIT; }
            
            // Enable channel
            ccr |= DMA_CCR_EN;
            write_volatile(&mut ch_regs.ccr, ccr);
        }
    }
    
    pub fn end_transfer(&mut self) {
        let regs = self.regs();
        unsafe {
            let ch_regs = &mut (*regs).channels[self.channel_idx];
            let mut ccr = read_volatile(&mut ch_regs.ccr);
            ccr &= !DMA_CCR_EN;
            write_volatile(&mut ch_regs.ccr, ccr);
            
            // Clear interrupt flags
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            write_volatile(&mut (*regs).ifcr, DMA_CHANNEL_CLEAR_ALL << shift);
        }
    }
    
    pub fn is_transfer_complete(&self) -> bool {
        let regs = self.regs();
        unsafe {
            let isr = read_volatile(&mut (*regs).isr);
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            (isr & (DMA_FLAG_TCIF << shift)) != 0
        }
    }

    pub fn clear_transfer_complete(&mut self) {
        let regs = self.regs();
        unsafe {
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            write_volatile(&mut (*regs).ifcr, DMA_FLAG_TCIF << shift);
        }
    }

    pub fn is_half_transfer(&self) -> bool {
        let regs = self.regs();
        unsafe {
            let isr = read_volatile(&mut (*regs).isr);
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            (isr & (DMA_FLAG_HTIF << shift)) != 0
        }
    }

    pub fn clear_half_transfer(&mut self) {
        let regs = self.regs();
        unsafe {
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            write_volatile(&mut (*regs).ifcr, DMA_FLAG_HTIF << shift);
        }
    }
}
