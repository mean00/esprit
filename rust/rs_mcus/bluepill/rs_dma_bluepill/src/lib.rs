#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

pub mod registers;
pub mod shim;
pub use shim::*;

use registers::*;
use rs_rcu_bluepill::{Peripheral, enable};
use rs_rtos::BinarySemaphore;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicU8, Ordering};
use rs_mcu_traits::dma as hal;

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
        self.begin_tx(peripheral_addr, memory_addr, length, src_16bit, dst_16bit, true);
    }

    pub fn begin_tx_transfer_width(
        &mut self,
        peripheral_addr: u32,
        memory_addr: u32,
        length: u32,
        width: hal::TransferWidth,
    ) {
        let (msize, psize) = match width {
            hal::TransferWidth::Byte => (DMA_CCR_MSIZE_8BIT, DMA_CCR_PSIZE_8BIT),
            hal::TransferWidth::HalfWord => (DMA_CCR_MSIZE_16BIT, DMA_CCR_PSIZE_16BIT),
            hal::TransferWidth::Word => (DMA_CCR_MSIZE_32BIT, DMA_CCR_PSIZE_32BIT),
        };
        self.begin_tx_full(peripheral_addr, memory_addr, length, msize, psize, true);
    }

    /// Begin a memory-to-peripheral transfer that reads the **same** memory word
    /// `length` times (memory increment off).
    ///
    /// This is what `lnDMA::doMemoryToPeripheralTransferNoLock(..., repeat=true)`
    /// does: the peripheral sees `length` copies of `*memory_addr`, so a fill
    /// value does not need a staging buffer.
    pub fn begin_tx_transfer_const(
        &mut self,
        peripheral_addr: u32,
        memory_addr: u32,
        length: u32,
        word_16bit: bool,
    ) {
        self.begin_tx(peripheral_addr, memory_addr, length, word_16bit, word_16bit, false);
    }

    /// Configure and start a memory-to-peripheral transfer.
    /// `memory_increment` selects between marching through the buffer (normal)
    /// and re-reading its first word for the whole transfer (`repeat`).
    fn begin_tx(
        &mut self,
        peripheral_addr: u32,
        memory_addr: u32,
        length: u32,
        src_16bit: bool,
        dst_16bit: bool,
        memory_increment: bool,
    ) {
        let msize = if src_16bit { DMA_CCR_MSIZE_16BIT } else { DMA_CCR_MSIZE_8BIT };
        let psize = if dst_16bit { DMA_CCR_PSIZE_16BIT } else { DMA_CCR_PSIZE_8BIT };
        self.begin_tx_full(peripheral_addr, memory_addr, length, msize, psize, memory_increment);
    }

    fn begin_tx_full(
        &mut self,
        peripheral_addr: u32,
        memory_addr: u32,
        length: u32,
        msize: u32,
        psize: u32,
        memory_increment: bool,
    ) {
        let regs = self.regs();
        unsafe {
            let ch_regs = core::ptr::addr_of_mut!((*regs).channels[self.channel_idx]);

            // 1. Disable channel before configuring
            let mut ccr = rs_mcu_traits::reg_read!(ch_regs, ccr);
            ccr &= !DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);

            // 2. Clear interrupt flags for this channel in IFCR
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            rs_mcu_traits::reg_write!(regs, ifcr, DMA_CHANNEL_CLEAR_ALL << shift);

            // 3. Set Peripheral and Memory addresses
            rs_mcu_traits::reg_write!(ch_regs, cpar, peripheral_addr);
            rs_mcu_traits::reg_write!(ch_regs, cmar, memory_addr);

            // 4. Set Length
            rs_mcu_traits::reg_write!(ch_regs, cndtr, length);

            // 5. Configure CCR: DIR=MemToPeriph, sizes, memory increment
            ccr = DMA_CCR_DIR_MEM2PERIPH | msize | psize | DMA_CCR_TCIE;
            if memory_increment {
                ccr |= DMA_CCR_MINC;
            }

            // Enable channel
            ccr |= DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);
        }
    }

    pub fn begin_rx_transfer_width(
        &mut self,
        peripheral_addr: u32,
        memory_addr: u32,
        length: u32,
        width: hal::TransferWidth,
    ) {
        let (psize, msize) = match width {
            hal::TransferWidth::Byte => (DMA_CCR_PSIZE_8BIT, DMA_CCR_MSIZE_8BIT),
            hal::TransferWidth::HalfWord => (DMA_CCR_PSIZE_16BIT, DMA_CCR_MSIZE_16BIT),
            hal::TransferWidth::Word => (DMA_CCR_PSIZE_32BIT, DMA_CCR_MSIZE_32BIT),
        };
        self.begin_rx_full(peripheral_addr, memory_addr, length, psize, msize);
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
        let psize = if src_16bit { DMA_CCR_PSIZE_16BIT } else { DMA_CCR_PSIZE_8BIT };
        let msize = if dst_16bit { DMA_CCR_MSIZE_16BIT } else { DMA_CCR_MSIZE_8BIT };
        self.begin_rx_full(peripheral_addr, memory_addr, length, psize, msize);
    }

    fn begin_rx_full(
        &mut self,
        peripheral_addr: u32,
        memory_addr: u32,
        length: u32,
        psize: u32,
        msize: u32,
    ) {
        let regs = self.regs();
        unsafe {
            let ch_regs = core::ptr::addr_of_mut!((*regs).channels[self.channel_idx]);

            // 1. Disable channel before configuring
            let mut ccr = rs_mcu_traits::reg_read!(ch_regs, ccr);
            ccr &= !DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);

            // 2. Clear interrupt flags for this channel in IFCR
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            rs_mcu_traits::reg_write!(regs, ifcr, DMA_CHANNEL_CLEAR_ALL << shift);

            // 3. Set Peripheral and Memory addresses
            rs_mcu_traits::reg_write!(ch_regs, cpar, peripheral_addr);
            rs_mcu_traits::reg_write!(ch_regs, cmar, memory_addr);

            // 4. Set Length
            rs_mcu_traits::reg_write!(ch_regs, cndtr, length);

            // 5. Configure CCR: Memory increment, DIR=PeriphToMem (0), sizes
            let ccr = DMA_CCR_MINC | psize | msize | DMA_CCR_TCIE | DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);
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
            let ch_regs = core::ptr::addr_of_mut!((*regs).channels[self.channel_idx]);
            
            // 1. Disable channel before configuring
            let mut ccr = rs_mcu_traits::reg_read!(ch_regs, ccr);
            ccr &= !DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);
            
            // 2. Clear interrupt flags for this channel in IFCR
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            rs_mcu_traits::reg_write!(regs, ifcr, DMA_CHANNEL_CLEAR_ALL << shift);
            
            // 3. Set Peripheral and Memory addresses
            rs_mcu_traits::reg_write!(ch_regs, cpar, peripheral_addr);
            rs_mcu_traits::reg_write!(ch_regs, cmar, memory_addr);
            
            // 4. Set Length
            rs_mcu_traits::reg_write!(ch_regs, cndtr, length);
            
            // 5. Configure CCR: Circular mode, Memory increment, DIR=MemToPeriph, High Priority, TCIE + HTIE
            ccr = DMA_CCR_DIR_MEM2PERIPH | DMA_CCR_CIRC | DMA_CCR_MINC | DMA_CCR_PL_HIGH | DMA_CCR_TCIE | DMA_CCR_HTIE;
            
            if src_16bit { ccr |= DMA_CCR_MSIZE_16BIT; } else { ccr |= DMA_CCR_MSIZE_8BIT; }
            if dst_16bit { ccr |= DMA_CCR_PSIZE_16BIT; } else { ccr |= DMA_CCR_PSIZE_8BIT; }
            
            // Enable channel
            ccr |= DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);
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
            let ch_regs = core::ptr::addr_of_mut!((*regs).channels[self.channel_idx]);

            // 1. Disable channel before configuring
            let mut ccr = rs_mcu_traits::reg_read!(ch_regs, ccr);
            ccr &= !DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);

            // 2. Clear interrupt flags for this channel in IFCR
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            rs_mcu_traits::reg_write!(regs, ifcr, DMA_CHANNEL_CLEAR_ALL << shift);

            // 3. Set Peripheral and Memory addresses
            rs_mcu_traits::reg_write!(ch_regs, cpar, peripheral_addr);
            rs_mcu_traits::reg_write!(ch_regs, cmar, memory_addr);

            // 4. Set Length
            rs_mcu_traits::reg_write!(ch_regs, cndtr, length);

            // 5. Configure CCR: Circular mode, Memory increment, DIR=PeriphToMem, High Priority, TCIE + HTIE
            ccr = DMA_CCR_CIRC | DMA_CCR_MINC | DMA_CCR_PL_HIGH | DMA_CCR_TCIE | DMA_CCR_HTIE;

            if src_16bit { ccr |= DMA_CCR_PSIZE_16BIT; } else { ccr |= DMA_CCR_PSIZE_8BIT; }
            if dst_16bit { ccr |= DMA_CCR_MSIZE_16BIT; } else { ccr |= DMA_CCR_MSIZE_8BIT; }

            // Enable channel
            ccr |= DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);
        }
    }
    
    pub fn end_transfer(&mut self) {
        self.detach_callback();
        let regs = self.regs();
        unsafe {
            let ch_regs = core::ptr::addr_of_mut!((*regs).channels[self.channel_idx]);
            let mut ccr = rs_mcu_traits::reg_read!(ch_regs, ccr);
            ccr &= !DMA_CCR_EN;
            rs_mcu_traits::reg_write!(ch_regs, ccr, ccr);
            
            // Clear interrupt flags
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            rs_mcu_traits::reg_write!(regs, ifcr, DMA_CHANNEL_CLEAR_ALL << shift);
        }
    }
    
    pub fn is_transfer_complete(&self) -> bool {
        let regs = self.regs();
        unsafe {
            let isr = rs_mcu_traits::reg_read!(regs, isr);
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            (isr & (DMA_FLAG_TCIF << shift)) != 0
        }
    }

    pub fn clear_transfer_complete(&mut self) {
        let regs = self.regs();
        unsafe {
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            rs_mcu_traits::reg_write!(regs, ifcr, DMA_FLAG_TCIF << shift);
        }
    }

    pub fn is_half_transfer(&self) -> bool {
        let regs = self.regs();
        unsafe {
            let isr = rs_mcu_traits::reg_read!(regs, isr);
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            (isr & (DMA_FLAG_HTIF << shift)) != 0
        }
    }

    pub fn clear_half_transfer(&mut self) {
        let regs = self.regs();
        unsafe {
            let shift = (self.channel_idx as u32) * DMA_FLAGS_PER_CHANNEL;
            rs_mcu_traits::reg_write!(regs, ifcr, DMA_FLAG_HTIF << shift);
        }
    }

    /// Return the number of remaining transfers in CNDTR.
    pub fn remaining_transfers(&self) -> u32 {
        let regs = self.regs();
        unsafe {

// ---------------------------------------------------------------------------
//  The MCU-agnostic DMA contract
// ---------------------------------------------------------------------------

use rs_mcu_traits::dma as hal;
use rs_mcu_traits::handler::HandlerTable;

use core::ffi::c_void;

/// DMA0 has 7 channels, DMA1 has 5: 12 handles in total, numbered
/// `engine * 7 + channel`.
const DMA0_CHANNELS: usize = 7;
const DMA1_CHANNELS: usize = 5;
const CONTROLLER_CHANNELS: usize = DMA0_CHANNELS + DMA1_CHANNELS;

/// One handler slot per handle.
static HANDLERS: HandlerTable<dyn hal::DmaHandler, CONTROLLER_CHANNELS> = HandlerTable::new();

/// Second step of the completion hand-off: the channel callback is a raw `extern
/// "C"` function, so the typed handler is looked up by handle (the cookie). The
/// `half` flag is the GD32 half-transfer flag, which this contract does not use.
unsafe extern "C" fn completion_trampoline(_half: bool, cookie: *mut c_void) {
    // SAFETY: the handler was installed before the transfer was started, and the
    // interrupt is the only reader.
    if let Some(handler) = unsafe { HANDLERS.isr_get(cookie as usize) } {
        handler.on_complete();
    }
}

/// The GD32 DMA controller: it owns the channels it hands out.
pub struct Controller {
    channels: [Option<DmaChannel>; CONTROLLER_CHANNELS],
    /// Completion semaphores, one per handle.
    done: [BinarySemaphore; CONTROLLER_CHANNELS],
}

impl Controller {
    /// An empty controller, usable in a `static`.
    pub const fn new() -> Self {
        Self {
            channels: [const { None }; CONTROLLER_CHANNELS],
            done: [const { BinarySemaphore::new() }; CONTROLLER_CHANNELS],
        }
    }

    /// The `(engine, index)` of a handle, or `None` when it names no channel.
    fn split(handle: u8) -> Option<(DmaEngine, usize)> {
        let handle = handle as usize;
        let (engine, index) = (handle / DMA0_CHANNELS, handle % DMA0_CHANNELS);
        match engine {
            0 if index < DMA0_CHANNELS => Some((DmaEngine::Dma0, index)),
            1 if index < DMA1_CHANNELS => Some((DmaEngine::Dma1, index)),
            _ => None,
        }
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl hal::Dma for Controller {
    type Channel = u8;

    fn claim(&mut self) -> rs_mcu_traits::Result<u8> {
        for (handle, slot) in self.channels.iter_mut().enumerate() {
            if slot.is_some() {
                continue;
            }
            let Some((engine, index)) = Self::split(handle as u8) else {
                continue;
            };
            if DmaChannel::is_channel_in_use(engine, index) {
                continue;
            }
            *slot = Some(DmaChannel::new(engine, index));
            return Ok(handle as u8);
        }
        Err(rs_mcu_traits::Error::Busy)
    }

    fn release(&mut self, channel: u8) {
        if let Some(slot) = self.channels.get_mut(channel as usize) {
            if let Some(dma) = slot.as_mut() {
                dma.end_transfer();
                dma.detach_callback();
            }
            *slot = None;
        }
    }

    fn start_write(
        &mut self,
        channel: u8,
        buffer: &[u8],
        peripheral_address: u32,
        width: hal::TransferWidth,
        _request: Option<u32>,
    ) -> rs_mcu_traits::Result<()> {
        let count = hal::elements(buffer, width) as u32;
        if count == 0 {
            return Err(rs_mcu_traits::Error::BufferTooSmall);
        }
        let index = channel as usize;
        let (channels, done) = (&mut self.channels, &self.done);
        let Some(dma) = channels.get_mut(index).and_then(Option::as_mut) else {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        };
        done[index].try_take();
        dma.attach_completion_semaphore(&done[index]);
        // The GD32 channel is paced by its peripheral (no DREQ selection): the
        // request number of the trait is a no-op here.
        dma.begin_tx_transfer_width(
            peripheral_address,
            buffer.as_ptr() as u32,
            count,
            width,
        );
        Ok(())
    }

    fn start_read(
        &mut self,
        channel: u8,
        buffer: &mut [u8],
        peripheral_address: u32,
        width: hal::TransferWidth,
        _request: Option<u32>,
    ) -> rs_mcu_traits::Result<()> {
        let count = hal::elements(buffer, width) as u32;
        if count == 0 {
            return Err(rs_mcu_traits::Error::BufferTooSmall);
        }
        let index = channel as usize;
        let (channels, done) = (&mut self.channels, &self.done);
        let Some(dma) = channels.get_mut(index).and_then(Option::as_mut) else {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        };
        done[index].try_take();
        dma.attach_completion_semaphore(&done[index]);
        dma.begin_rx_transfer_width(
            peripheral_address,
            buffer.as_mut_ptr() as u32,
            count,
            width,
        );
        Ok(())
    }

    fn wait(&mut self, channel: u8, timeout_ms: u32) -> rs_mcu_traits::Result<()> {
        match self.done.get(channel as usize) {
            Some(semaphore) if semaphore.take_timeout(timeout_ms) => Ok(()),
            Some(_) => Err(rs_mcu_traits::Error::Timeout),
            None => Err(rs_mcu_traits::Error::InvalidConfig),
        }
    }

    fn set_handler(&mut self, channel: u8, handler: Option<&'static dyn hal::DmaHandler>) {
        let index = channel as usize;
        HANDLERS.set(index, handler);
        let Some(dma) = self.channels.get_mut(index).and_then(Option::as_mut) else {
            return;
        };
        match handler {
            Some(_) => dma.attach_callback(completion_trampoline, index as *mut c_void),
            None => dma.detach_callback(),
        }
    }
}

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

