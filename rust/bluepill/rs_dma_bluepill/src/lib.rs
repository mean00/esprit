#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;
pub mod shim;
pub use shim::*;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use rs_esprit::BinarySemaphore;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU8, Ordering};

/// In-use channel bitmasks for DMA engines:
/// - Index 0: DMA0 (bits 0..6 for channels 0..6)
/// - Index 1: DMA1 (bits 0..4 for channels 0..4)
static DMA_CHANNELS_IN_USE: [AtomicU8; 2] = [AtomicU8::new(0), AtomicU8::new(0)];

pub trait DmaTransferHandler: Send + Sync {
    fn on_transfer_complete(&self);
    fn on_half_complete(&self) {}
}

unsafe extern "C" fn dma_sem_trampoline(half: bool, cookie: *mut core::ffi::c_void) {
    if !half && !cookie.is_null() {
        let sem = unsafe { &*(cookie as *const BinarySemaphore) };
        sem.give_from_isr();
    }
}

unsafe extern "C" fn dma_handler_trampoline<H: DmaTransferHandler>(half: bool, cookie: *mut core::ffi::c_void) {
    if !cookie.is_null() {
        let handler = unsafe { &*(cookie as *const H) };
        if half {
            handler.on_half_complete();
        } else {
            handler.on_transfer_complete();
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaEngine {
    Dma0 = 0, // DMA1 in STM32 terminology
    Dma1 = 1, // DMA2 in STM32 terminology
}

unsafe extern "C" {
    fn lnDmaAttachRawCallback(
        dma: i32,
        channel: i32,
        cb: unsafe extern "C" fn(bool, *mut core::ffi::c_void),
        cookie: *mut core::ffi::c_void,
    );
    fn lnDmaDetachRawCallback(dma: i32, channel: i32);
}

pub struct DmaChannel {
    engine: DmaEngine,
    channel_idx: usize, // 0 to 6
}

impl DmaChannel {
    pub fn new(engine: DmaEngine, channel_idx: usize) -> Self {
        assert!(
            channel_idx < 7,
            "DMA channel index out of range: {} (max 6)",
            channel_idx
        );
        if engine == DmaEngine::Dma1 {
            assert!(
                channel_idx < 5,
                "DMA1 channel index out of range: {} (DMA1 has only 5 channels: 0..4)",
                channel_idx
            );
        }

        let engine_idx = engine as usize;
        let mask = 1u8 << channel_idx;
        let prev = DMA_CHANNELS_IN_USE[engine_idx].fetch_or(mask, Ordering::SeqCst);
        if (prev & mask) != 0 {
            panic!(
                "DMA channel conflict: {:?} Channel {} is already in use by another peripheral!",
                engine, channel_idx
            );
        }

        #[cfg(not(test))]
        match engine {
            DmaEngine::Dma0 => enable(Peripheral::Dma0),
            DmaEngine::Dma1 => enable(Peripheral::Dma1),
        }
        Self { engine, channel_idx }
    }

    /// Check whether a specific DMA channel is currently in use.
    pub fn is_channel_in_use(engine: DmaEngine, channel_idx: usize) -> bool {
        if channel_idx >= 7 || (engine == DmaEngine::Dma1 && channel_idx >= 5) {
            return false;
        }
        let engine_idx = engine as usize;
        let mask = 1u8 << channel_idx;
        (DMA_CHANNELS_IN_USE[engine_idx].load(Ordering::SeqCst) & mask) != 0
    }
    
    fn regs(&self) -> *mut DmaRegisters {
        match self.engine {
            DmaEngine::Dma0 => DmaRegisters::dma0_ptr(),
            DmaEngine::Dma1 => DmaRegisters::dma1_ptr(),
        }
    }

    /// Attach a binary semaphore to be awakened from the DMA interrupt when transfer completes.
    pub fn attach_completion_semaphore(&mut self, sem: &BinarySemaphore) {
        unsafe {
            lnDmaAttachRawCallback(
                self.engine as i32,
                self.channel_idx as i32,
                dma_sem_trampoline,
                sem as *const BinarySemaphore as *mut core::ffi::c_void,
            );
        }
    }

    /// Attach a typed handler trait implementing `DmaTransferHandler`.
    pub fn attach_handler<H: DmaTransferHandler + 'static>(&mut self, handler: &'static H) {
        unsafe {
            lnDmaAttachRawCallback(
                self.engine as i32,
                self.channel_idx as i32,
                dma_handler_trampoline::<H>,
                handler as *const H as *mut core::ffi::c_void,
            );
        }
    }

    /// Attach a raw C-style callback function with cookie.
    pub fn attach_callback(
        &mut self,
        cb: unsafe extern "C" fn(bool, *mut core::ffi::c_void),
        cookie: *mut core::ffi::c_void,
    ) {
        unsafe {
            lnDmaAttachRawCallback(self.engine as i32, self.channel_idx as i32, cb, cookie);
        }
    }

    pub fn detach_callback(&mut self) {
        unsafe {
            lnDmaDetachRawCallback(self.engine as i32, self.channel_idx as i32);
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

    /// Begin a peripheral-to-memory transfer (DIR=0).
    pub fn begin_rx_transfer(
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

            // 5. Configure CCR: Memory increment, DIR=PeriphToMem (0), sizes
            ccr = DMA_CCR_MINC;

            if src_16bit { ccr |= DMA_CCR_PSIZE_16BIT; }
            if dst_16bit { ccr |= DMA_CCR_MSIZE_16BIT; }

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
            
            // 5. Configure CCR: Circular mode, Memory increment, DIR=MemToPeriph, High Priority, TCIE + HTIE
            ccr = DMA_CCR_DIR_MEM2PERIPH | DMA_CCR_CIRC | DMA_CCR_MINC | DMA_CCR_PL_HIGH | DMA_CCR_TCIE | DMA_CCR_HTIE;
            
            if src_16bit { ccr |= DMA_CCR_MSIZE_16BIT; } else { ccr |= DMA_CCR_MSIZE_8BIT; }
            if dst_16bit { ccr |= DMA_CCR_PSIZE_16BIT; } else { ccr |= DMA_CCR_PSIZE_8BIT; }
            
            // Enable channel
            ccr |= DMA_CCR_EN;
            write_volatile(&mut ch_regs.ccr, ccr);
        }
    }

    /// Begin a circular peripheral-to-memory transfer (DIR=0).
    pub fn begin_circular_rx_transfer(
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

            // 5. Configure CCR: Circular mode, Memory increment, DIR=PeriphToMem, High Priority, TCIE + HTIE
            ccr = DMA_CCR_CIRC | DMA_CCR_MINC | DMA_CCR_PL_HIGH | DMA_CCR_TCIE | DMA_CCR_HTIE;

            if src_16bit { ccr |= DMA_CCR_PSIZE_16BIT; } else { ccr |= DMA_CCR_PSIZE_8BIT; }
            if dst_16bit { ccr |= DMA_CCR_MSIZE_16BIT; } else { ccr |= DMA_CCR_MSIZE_8BIT; }

            // Enable channel
            ccr |= DMA_CCR_EN;
            write_volatile(&mut ch_regs.ccr, ccr);
        }
    }
    
    pub fn end_transfer(&mut self) {
        self.detach_callback();
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

    /// Return the number of remaining transfers in CNDTR.
    pub fn remaining_transfers(&self) -> u32 {
        let regs = self.regs();
        unsafe {
            let ch_regs = &(*regs).channels[self.channel_idx];
            read_volatile(&ch_regs.cndtr)
        }
    }
}

impl Drop for DmaChannel {
    fn drop(&mut self) {
        self.detach_callback();
        let engine_idx = self.engine as usize;
        let mask = 1u8 << self.channel_idx;
        DMA_CHANNELS_IN_USE[engine_idx].fetch_and(!mask, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[unsafe(no_mangle)]
    extern "C" fn lnDmaDetachRawCallback(_dma: i32, _channel: i32) {}

    #[test]
    fn test_dma_channel_double_allocation_prevention() {
        assert!(!DmaChannel::is_channel_in_use(DmaEngine::Dma0, 3));
        {
            let _ch = DmaChannel::new(DmaEngine::Dma0, 3);
            assert!(DmaChannel::is_channel_in_use(DmaEngine::Dma0, 3));
        }
        // After drop, channel is released
        assert!(!DmaChannel::is_channel_in_use(DmaEngine::Dma0, 3));
    }

    #[test]
    #[should_panic(expected = "DMA channel conflict")]
    fn test_dma_channel_conflict_panics() {
        let _ch1 = DmaChannel::new(DmaEngine::Dma0, 2);
        let _ch2 = DmaChannel::new(DmaEngine::Dma0, 2);
    }

    #[test]
    #[should_panic(expected = "DMA channel index out of range")]
    fn test_dma_invalid_channel_panics() {
        let _ch = DmaChannel::new(DmaEngine::Dma0, 7);
    }
}

