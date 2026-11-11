#![no_std]

pub mod constants;
pub mod registers;

pub use constants::*;
pub use registers::*;

use core::ffi::c_void;
use portable_atomic::{AtomicBool, AtomicU16, Ordering};
use rs_esprit::BinarySemaphore;

// External C interrupt functions provided by the RP2040 platform SDK / runtime.
unsafe extern "C" {
    fn irq_set_exclusive_handler(num: u32, handler: unsafe extern "C" fn());
    fn irq_set_enabled(num: u32, enabled: bool);
}

/// RP2040 IRQ index for DMA IRQ 1 (DMA_IRQ_0 is 11, DMA_IRQ_1 is 12).
const NVIC_DMA_IRQ_1: u32 = 12;

/// Atomic bitmask tracking claimed channels (bits 0..11).
static DMA_CHANNELS_IN_USE: AtomicU16 = AtomicU16::new(0);

/// Tracks whether the DMA IRQ 1 handler has been installed.
static IRQ_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Per-channel callback table for DMA completion.
struct ChannelCallback {
    cb: Option<unsafe extern "C" fn(cookie: *mut c_void)>,
    cookie: *mut c_void,
}

static mut CHANNEL_CALLBACKS: [ChannelCallback; DMA_MAX_CHANNELS] = [
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
    ChannelCallback { cb: None, cookie: core::ptr::null_mut() },
];

/// Unreset the DMA peripheral if held in reset.
pub fn unreset_dma() {
    unsafe {
        let reset_reg = RESETS_RESET_ADDR as *mut u32;
        let done_reg = RESETS_RESET_DONE_ADDR as *const u32;
        // Atomic bitmask clear: writing bit to RESET clears the reset
        let current = core::ptr::read_volatile(reset_reg);
        if (current & RESET_BIT_DMA) != 0 {
            core::ptr::write_volatile(reset_reg, current & !RESET_BIT_DMA);
            while (core::ptr::read_volatile(done_reg) & RESET_BIT_DMA) == 0 {
                core::hint::spin_loop();
            }
        }
    }
}

/// Global DMA interrupt handler for DMA_IRQ_1.
unsafe extern "C" fn dma_irq1_handler() {
    let dma = DMA_BASE as *mut DmaRegisters;
    let ints1 = unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*dma).ints1)) };
    // Clear pending interrupts by writing back the set flags
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*dma).ints1), ints1);
    }

    for i in 0..DMA_MAX_CHANNELS {
        if (ints1 & (1 << i)) != 0 {
            unsafe {
                let entry = &CHANNEL_CALLBACKS[i];
                if let Some(cb) = entry.cb {
                    cb(entry.cookie);
                }
            }
        }
    }
}

/// Trampoline to give a BinarySemaphore from ISR.
unsafe extern "C" fn sem_give_trampoline(cookie: *mut c_void) {
    if !cookie.is_null() {
        let sem = unsafe { &*(cookie as *const BinarySemaphore) };
        sem.give_from_isr();
    }
}

/// Ensure the DMA controller is unreset and DMA IRQ 1 is installed.
fn ensure_dma_initialized() {
    unreset_dma();
    if !IRQ_INSTALLED.swap(true, Ordering::SeqCst) {
        unsafe {
            irq_set_exclusive_handler(NVIC_DMA_IRQ_1, dma_irq1_handler);
            irq_set_enabled(NVIC_DMA_IRQ_1, true);
        }
    }
}

/// An owned hardware DMA channel on the RP2040.
pub struct DmaChannel {
    channel_idx: usize,
}

impl DmaChannel {
    /// Claim any available DMA channel (0..11).
    pub fn claim() -> Option<Self> {
        ensure_dma_initialized();
        for i in 0..DMA_MAX_CHANNELS {
            let mask = 1u16 << i;
            if (DMA_CHANNELS_IN_USE.fetch_or(mask, Ordering::SeqCst) & mask) == 0 {
                let mut ch = Self { channel_idx: i };
                ch.abort();
                ch.disable_interrupt();
                return Some(ch);
            }
        }
        None
    }

    /// Claim a specific DMA channel if free.
    pub fn claim_specific(idx: usize) -> Option<Self> {
        if idx >= DMA_MAX_CHANNELS {
            return None;
        }
        ensure_dma_initialized();
        let mask = 1u16 << idx;
        if (DMA_CHANNELS_IN_USE.fetch_or(mask, Ordering::SeqCst) & mask) == 0 {
            let mut ch = Self { channel_idx: idx };
            ch.abort();
            ch.disable_interrupt();
            Some(ch)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn channel_idx(&self) -> usize {
        self.channel_idx
    }

    #[inline(always)]
    pub fn regs(&self) -> *mut DmaChannelRegisters {
        let dma = DMA_BASE as *mut DmaRegisters;
        unsafe { core::ptr::addr_of_mut!((*dma).channels[self.channel_idx]) }
    }

    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        unsafe {
            (core::ptr::read_volatile(core::ptr::addr_of!((*self.regs()).ctrl_trig)) & DMA_CTRL_BUSY) != 0
        }
    }

    /// Wait until the channel is no longer busy, with timeout budget.
    pub fn wait_busy(&self) -> bool {
        let mut budget = POLL_LIMIT;
        while self.is_busy() {
            if budget == 0 {
                return false;
            }
            budget -= 1;
            core::hint::spin_loop();
        }
        true
    }

    /// Abort an ongoing transfer on this channel.
    pub fn abort(&mut self) {
        unsafe {
            let dma = DMA_BASE as *mut DmaRegisters;
            let mask = 1u32 << self.channel_idx;
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*dma).chan_abort), mask);
            while (core::ptr::read_volatile(core::ptr::addr_of!((*dma).chan_abort)) & mask) != 0 {
                core::hint::spin_loop();
            }
        }
        self.wait_busy();
    }

    /// Enable interrupts for this channel on DMA IRQ 1.
    pub fn enable_interrupt(&mut self) {
        unsafe {
            let dma = DMA_BASE as *mut DmaRegisters;
            let val = core::ptr::read_volatile(core::ptr::addr_of!((*dma).inte1));
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*dma).inte1), val | (1 << self.channel_idx));
        }
    }

    /// Disable interrupts for this channel on DMA IRQ 1.
    pub fn disable_interrupt(&mut self) {
        unsafe {
            let dma = DMA_BASE as *mut DmaRegisters;
            let val = core::ptr::read_volatile(core::ptr::addr_of!((*dma).inte1));
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*dma).inte1), val & !(1 << self.channel_idx));
        }
    }

    /// Attach a completion callback.
    pub fn attach_callback(&mut self, cb: unsafe extern "C" fn(cookie: *mut c_void), cookie: *mut c_void) {
        unsafe {
            CHANNEL_CALLBACKS[self.channel_idx] = ChannelCallback {
                cb: Some(cb),
                cookie,
            };
        }
        self.enable_interrupt();
    }

    /// Attach a FreeRTOS BinarySemaphore to be signalled when the transfer completes.
    pub fn attach_completion_semaphore(&mut self, sem: &BinarySemaphore) {
        self.attach_callback(sem_give_trampoline, sem as *const BinarySemaphore as *mut c_void);
    }

    /// Detach any attached callback and disable channel interrupts.
    pub fn detach_callback(&mut self) {
        self.disable_interrupt();
        unsafe {
            CHANNEL_CALLBACKS[self.channel_idx] = ChannelCallback {
                cb: None,
                cookie: core::ptr::null_mut(),
            };
        }
    }

    /// Configure a memory-to-peripheral (TX) transfer.
    pub fn configure_tx(
        &mut self,
        src: *const u8,
        dst_addr: u32,
        count: u32,
        width: DmaTransferWidth,
        treq: u32,
        incr_read: bool,
        trigger: bool,
    ) {
        let mut ctrl: u32 = DMA_CTRL_EN
            | ((width as u32) << DMA_CTRL_DATA_SIZE_SHIFT)
            | (treq << DMA_CTRL_TREQ_SHIFT)
            | ((self.channel_idx as u32) << DMA_CTRL_CHAIN_TO_SHIFT); // chain to self = no chaining

        if incr_read {
            ctrl |= DMA_CTRL_INCR_READ;
        }

        unsafe {
            let r = self.regs();
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).read_addr), src as u32);
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).write_addr), dst_addr);
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).trans_count), count);
            if trigger {
                core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).ctrl_trig), ctrl);
            } else {
                core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).al1_ctrl), ctrl);
            }
        }
    }

    /// Configure a peripheral-to-memory (RX) transfer.
    pub fn configure_rx(
        &mut self,
        src_addr: u32,
        dst: *mut u8,
        count: u32,
        width: DmaTransferWidth,
        treq: u32,
        incr_write: bool,
        trigger: bool,
    ) {
        let mut ctrl: u32 = DMA_CTRL_EN
            | ((width as u32) << DMA_CTRL_DATA_SIZE_SHIFT)
            | (treq << DMA_CTRL_TREQ_SHIFT)
            | ((self.channel_idx as u32) << DMA_CTRL_CHAIN_TO_SHIFT);

        if incr_write {
            ctrl |= DMA_CTRL_INCR_WRITE;
        }

        unsafe {
            let r = self.regs();
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).read_addr), src_addr);
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).write_addr), dst as u32);
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).trans_count), count);
            if trigger {
                core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).ctrl_trig), ctrl);
            } else {
                core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).al1_ctrl), ctrl);
            }
        }
    }

    /// Start transfer on this channel by setting the enable bit in ctrl_trig.
    pub fn start(&mut self) {
        unsafe {
            let r = self.regs();
            let ctrl = core::ptr::read_volatile(core::ptr::addr_of!((*r).ctrl_trig));
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).ctrl_trig), ctrl | DMA_CTRL_EN);
        }
    }

    /// End/stop an active transfer and disable channel interrupt.
    pub fn end_transfer(&mut self) {
        self.disable_interrupt();
        unsafe {
            let r = self.regs();
            let ctrl = core::ptr::read_volatile(core::ptr::addr_of!((*r).ctrl_trig));
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*r).al1_ctrl), ctrl & !DMA_CTRL_EN);
        }
        self.wait_busy();
    }

    // --- Multi-Chunk (Scatter-Gather) Support ---

    /// Set up a multi-chunk gather operation (transmitting multiple chunks to a destination FIFO).
    ///
    /// `self` is the data channel, and `ctrl` is the control channel.
    /// Each block in `blocks` consists of `(trans_count, read_addr)`.
    /// The control channel writes to Alias 3 `(al3_trans_count, al3_read_addr_trig)`.
    pub fn setup_gather(
        &mut self,
        ctrl: &mut DmaChannel,
        blocks: &[DmaGatherBlock],
        dst_fifo_addr: u32,
        width: DmaTransferWidth,
        treq: u32,
    ) {
        assert!(!blocks.is_empty(), "blocks must not be empty");

        // 1. Configure the data channel:
        // Reads from buffer, writes to peripheral FIFO.
        // Increments read address, does NOT increment write address.
        // Chains back to the control channel upon completion of each chunk.
        let data_ctrl = DMA_CTRL_EN
            | ((width as u32) << DMA_CTRL_DATA_SIZE_SHIFT)
            | (treq << DMA_CTRL_TREQ_SHIFT)
            | DMA_CTRL_INCR_READ
            | ((ctrl.channel_idx() as u32) << DMA_CTRL_CHAIN_TO_SHIFT);

        unsafe {
            let data_r = self.regs();
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*data_r).write_addr), dst_fifo_addr);
            // Write control configuration without triggering
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*data_r).al1_ctrl), data_ctrl);
        }

        // 2. Configure the control channel:
        // Transfers 2 32-bit words per block into `&data_r.al3_trans_count`.
        // Ring wrapping on write pointer: 8-byte boundary (1 << 3 = 8 bytes), so write
        // address alternates between al3_trans_count and al3_read_addr_trig.
        let ring_size_8_bytes = 3u32;
        let ctrl_config = DMA_CTRL_EN
            | (DMA_CTRL_DATA_SIZE_32)
            | (DREQ_PERMANENT << DMA_CTRL_TREQ_SHIFT)
            | DMA_CTRL_INCR_READ
            | DMA_CTRL_INCR_WRITE
            | (ring_size_8_bytes << DMA_CTRL_RING_SIZE_SHIFT)
            | DMA_CTRL_RING_SEL_WRITE
            | ((ctrl.channel_idx() as u32) << DMA_CTRL_CHAIN_TO_SHIFT); // no chaining on ctrl

        unsafe {
            let ctrl_r = ctrl.regs();
            let data_r = self.regs();
            let target_addr = core::ptr::addr_of_mut!((*data_r).al3_trans_count) as u32;

            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*ctrl_r).read_addr),
                blocks.as_ptr() as u32,
            );
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*ctrl_r).write_addr), target_addr);
            // Total transfers is 2 32-bit words per block
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*ctrl_r).trans_count),
                (blocks.len() * 2) as u32,
            );
            // Configure control channel without triggering yet
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*ctrl_r).al1_ctrl), ctrl_config);
        }
    }

    /// Set up a multi-chunk scatter operation (receiving from a source FIFO into multiple chunks).
    ///
    /// `self` is the data channel, and `ctrl` is the control channel.
    /// Each block in `blocks` consists of `(write_addr, trans_count)`.
    /// The control channel writes to Alias 1 `(al1_write_addr, al1_trans_count_trig)`.
    pub fn setup_scatter(
        &mut self,
        ctrl: &mut DmaChannel,
        blocks: &[DmaScatterBlock],
        src_fifo_addr: u32,
        width: DmaTransferWidth,
        treq: u32,
    ) {
        assert!(!blocks.is_empty(), "blocks must not be empty");

        // 1. Configure the data channel:
        // Reads from peripheral FIFO, writes to memory chunk.
        // Does NOT increment read address, increments write address.
        // Chains back to the control channel upon completion of each chunk.
        let data_ctrl = DMA_CTRL_EN
            | ((width as u32) << DMA_CTRL_DATA_SIZE_SHIFT)
            | (treq << DMA_CTRL_TREQ_SHIFT)
            | DMA_CTRL_INCR_WRITE
            | ((ctrl.channel_idx() as u32) << DMA_CTRL_CHAIN_TO_SHIFT);

        unsafe {
            let data_r = self.regs();
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*data_r).read_addr), src_fifo_addr);
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*data_r).al1_ctrl), data_ctrl);
        }

        // 2. Configure the control channel:
        // Transfers 2 32-bit words per block into `&data_r.al1_write_addr`.
        // Ring wrapping on write pointer: 8-byte boundary (1 << 3 = 8 bytes), so write
        // address alternates between al1_write_addr and al1_trans_count_trig.
        let ring_size_8_bytes = 3u32;
        let ctrl_config = DMA_CTRL_EN
            | (DMA_CTRL_DATA_SIZE_32)
            | (DREQ_PERMANENT << DMA_CTRL_TREQ_SHIFT)
            | DMA_CTRL_INCR_READ
            | DMA_CTRL_INCR_WRITE
            | (ring_size_8_bytes << DMA_CTRL_RING_SIZE_SHIFT)
            | DMA_CTRL_RING_SEL_WRITE
            | ((ctrl.channel_idx() as u32) << DMA_CTRL_CHAIN_TO_SHIFT);

        unsafe {
            let ctrl_r = ctrl.regs();
            let data_r = self.regs();
            let target_addr = core::ptr::addr_of_mut!((*data_r).al1_write_addr) as u32;

            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*ctrl_r).read_addr),
                blocks.as_ptr() as u32,
            );
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*ctrl_r).write_addr), target_addr);
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*ctrl_r).trans_count),
                (blocks.len() * 2) as u32,
            );
            core::ptr::write_volatile(core::ptr::addr_of_mut!((*ctrl_r).al1_ctrl), ctrl_config);
        }
    }

    /// Launch a multi-chunk operation configured with `setup_gather` or `setup_scatter`.
    pub fn trigger_multi_chunk(&mut self, ctrl: &mut DmaChannel) {
        ctrl.start();
    }
}

impl Drop for DmaChannel {
    fn drop(&mut self) {
        self.detach_callback();
        self.abort();
        let mask = 1u16 << self.channel_idx;
        DMA_CHANNELS_IN_USE.fetch_and(!mask, Ordering::SeqCst);
    }
}
