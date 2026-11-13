#![no_std]

pub mod constants;
pub mod registers;

// The chip-specific half of the register map: channel count, global block,
// `CTRL_TRIG` fields and DREQ numbers. This is the **one** place the chip is
// chosen — `rp2350` wins when both features are set, because `rust_esprit`'s
// `rp2350` implies its `rp2040`. The `#[path]`s keep the files flat in `src/`.
#[cfg(feature = "rp2350")]
#[path = "registers_rp2350.rs"]
mod registers_chip;

#[cfg(not(feature = "rp2350"))]
#[path = "registers_rp2040.rs"]
mod registers_chip;

pub use constants::*;
pub use registers::*;
pub use registers_chip::*;

use core::ffi::c_void;
use portable_atomic::{AtomicBool, AtomicPtr, AtomicU16, AtomicUsize, Ordering};
use rs_rtos::BinarySemaphore;

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

/// Completion callback of each channel, split into its two words so the DMA ISR
/// can read it without a lock: `CALLBACKS[i]` is the function's address (`0` =
/// no callback) and `COOKIES[i]` is the argument it is called with.
///
/// Written only through [`publish`] / [`unpublish`] and read only through
/// [`claimed`], which is what keeps the pair consistent across the GDB task and
/// the DMA interrupt: the cookie is stored first (`Relaxed`) and the address
/// that publishes it second (`Release`), the ISR reads the address with
/// `Acquire` before the cookie. A `static mut` of a `{ cb, cookie }` struct
/// could not do that — it is two words, so an interrupt taken between the two
/// stores would hand the new callback the old cookie.
static CALLBACKS: [AtomicUsize; DMA_MAX_CHANNELS] =
    [const { AtomicUsize::new(0) }; DMA_MAX_CHANNELS];
static COOKIES: [AtomicPtr<c_void>; DMA_MAX_CHANNELS] =
    [const { AtomicPtr::new(core::ptr::null_mut()) }; DMA_MAX_CHANNELS];

/// Arm `channel`'s completion callback and the value it is called with.
fn publish(channel: usize, cb: unsafe extern "C" fn(cookie: *mut c_void), cookie: *mut c_void) {
    COOKIES[channel].store(cookie, Ordering::Relaxed);
    // Release: everything above is visible to whoever sees this address.
    CALLBACKS[channel].store(cb as usize, Ordering::Release);
}

/// Disarm `channel`'s callback. The address goes first, so an interrupt that
/// lands mid-way already sees "no callback" instead of a stale cookie.
fn unpublish(channel: usize) {
    CALLBACKS[channel].store(0, Ordering::Release);
    COOKIES[channel].store(core::ptr::null_mut(), Ordering::Relaxed);
}

/// The callback armed on `channel`, with its cookie, or `None`.
fn claimed(channel: usize) -> Option<(unsafe extern "C" fn(cookie: *mut c_void), *mut c_void)> {
    // Acquire pairs with the `Release` in `publish`: the cookie stored before the
    // address is the one this load returns.
    let address = CALLBACKS[channel].load(Ordering::Acquire);
    if address == 0 {
        return None;
    }
    // SAFETY: the only writer of this slot is `publish`, which is given a
    // function pointer of exactly this type (and the cookie that goes with it).
    let cb = unsafe { core::mem::transmute::<usize, unsafe extern "C" fn(*mut c_void)>(address) };
    Some((cb, COOKIES[channel].load(Ordering::Relaxed)))
}

/// Spin until `ready()` says yes, giving up after [`POLL_LIMIT`] iterations.
///
/// Every wait in this driver goes through here, so the timeout policy — the
/// budget, the `spin_loop` hint, and "gave up" meaning `false` — lives in one
/// place. A channel that never reports done makes the caller return instead of
/// hanging the probe.
#[inline]
fn wait_until(ready: impl Fn() -> bool) -> bool {
    let mut budget = POLL_LIMIT;
    while !ready() {
        if budget == 0 {
            return false;
        }
        budget -= 1;
        core::hint::spin_loop();
    }
    true
}

/// Unreset the DMA peripheral if held in reset.
pub fn unreset_dma() {
    unsafe {
        let reset_reg = RESETS_RESET_ADDR as *mut u32;
        let done_reg = RESETS_RESET_DONE_ADDR as *const u32;
        // Atomic bitmask clear: writing bit to RESET clears the reset
        let current = core::ptr::read_volatile(reset_reg);
        if (current & RESET_BIT_DMA) != 0 {
            core::ptr::write_volatile(reset_reg, current & !RESET_BIT_DMA);
            // Deliberately unbounded no more: a DMA that never leaves reset shows
            // up as a bounded failure in the first channel operation, which beats
            // hanging the probe here.
            let _ = wait_until(|| (core::ptr::read_volatile(done_reg) & RESET_BIT_DMA) != 0);
        }
    }
}

/// Global DMA interrupt handler for DMA_IRQ_1.
unsafe extern "C" fn dma_irq1_handler() {
    let dma = DMA_BASE as *mut DmaRegisters;
    let ints1 = unsafe { rs_mcu_traits::reg_read!(dma, ints1) };
    // Clear pending interrupts by writing back the set flags
    unsafe {
        rs_mcu_traits::reg_write!(dma, ints1, ints1);
    }

    for channel in 0..DMA_MAX_CHANNELS {
        if ints1 & (1 << channel) == 0 {
            continue;
        }
        if let Some((cb, cookie)) = claimed(channel) {
            // SAFETY: `claimed` only returns a pointer that `publish` stored from
            // this very signature, and the cookie that was stored with it.
            unsafe { cb(cookie) }
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
        // `addr_of_mut!` avoids creating a reference, but the place expression
        // still projects through the raw pointer, which rustc requires to be
        // `unsafe`. The pointer is `DMA_BASE` + a claimed channel, so it is live.
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
        wait_until(|| !self.is_busy())
    }

    /// Abort an ongoing transfer on this channel.
    pub fn abort(&mut self) {
        unsafe {
            let dma = DMA_BASE as *mut DmaRegisters;
            let mask = 1u32 << self.channel_idx;
            rs_mcu_traits::reg_write!(dma, chan_abort, mask);
            // Bounded: `wait_busy` right after reports whether it stopped.
            let _ = wait_until(|| {
                (rs_mcu_traits::reg_read!(dma, chan_abort) & mask) == 0
            });
        }
        self.wait_busy();
    }

    /// Enable interrupts for this channel on DMA IRQ 1.
    pub fn enable_interrupt(&mut self) {
        unsafe {
            let dma = DMA_BASE as *mut DmaRegisters;
            let val = rs_mcu_traits::reg_read!(dma, inte1);
            rs_mcu_traits::reg_write!(dma, inte1, val | (1 << self.channel_idx));
        }
    }

    /// Disable interrupts for this channel on DMA IRQ 1.
    pub fn disable_interrupt(&mut self) {
        unsafe {
            let dma = DMA_BASE as *mut DmaRegisters;
            let val = rs_mcu_traits::reg_read!(dma, inte1);
            rs_mcu_traits::reg_write!(dma, inte1, val & !(1 << self.channel_idx));
        }
    }

    /// Attach a completion callback.
    ///
    /// The slot is published before the interrupt is enabled, so an interrupt
    /// that fires early sees the callback (or none), never half of it.
    pub fn attach_callback(&mut self, cb: unsafe extern "C" fn(cookie: *mut c_void), cookie: *mut c_void) {
        publish(self.channel_idx, cb, cookie);
        self.enable_interrupt();
    }

    /// Attach a FreeRTOS BinarySemaphore to be signalled when the transfer completes.
    pub fn attach_completion_semaphore(&mut self, sem: &BinarySemaphore) {
        self.attach_callback(sem_give_trampoline, sem as *const BinarySemaphore as *mut c_void);
    }

    /// Detach any attached callback and disable channel interrupts.
    pub fn detach_callback(&mut self) {
        self.disable_interrupt();
        unpublish(self.channel_idx);
    }

    /// Configure a memory-to-peripheral (TX) transfer.
    pub fn configure_tx(&mut self, src: *const u8, dst_addr: u32, count: u32, cfg: DmaTransferConfig) {
        let mut ctrl: u32 = DMA_CTRL_EN
            | ((cfg.width as u32) << DMA_CTRL_DATA_SIZE_SHIFT)
            | (cfg.treq << DMA_CTRL_TREQ_SHIFT)
            | ((self.channel_idx as u32) << DMA_CTRL_CHAIN_TO_SHIFT); // chain to self = no chaining

        if cfg.increment {
            ctrl |= DMA_CTRL_INCR_READ;
        }

        unsafe {
            let r = self.regs();
            rs_mcu_traits::reg_write!(r, read_addr, src as u32);
            rs_mcu_traits::reg_write!(r, write_addr, dst_addr);
            rs_mcu_traits::reg_write!(r, trans_count, count);
            if cfg.trigger {
                rs_mcu_traits::reg_write!(r, ctrl_trig, ctrl);
            } else {
                rs_mcu_traits::reg_write!(r, al1_ctrl, ctrl);
            }
        }
    }

    /// Configure a peripheral-to-memory (RX) transfer.
    pub fn configure_rx(&mut self, src_addr: u32, dst: *mut u8, count: u32, cfg: DmaTransferConfig) {
        let mut ctrl: u32 = DMA_CTRL_EN
            | ((cfg.width as u32) << DMA_CTRL_DATA_SIZE_SHIFT)
            | (cfg.treq << DMA_CTRL_TREQ_SHIFT)
            | ((self.channel_idx as u32) << DMA_CTRL_CHAIN_TO_SHIFT);

        if cfg.increment {
            ctrl |= DMA_CTRL_INCR_WRITE;
        }

        unsafe {
            let r = self.regs();
            rs_mcu_traits::reg_write!(r, read_addr, src_addr);
            rs_mcu_traits::reg_write!(r, write_addr, dst as u32);
            rs_mcu_traits::reg_write!(r, trans_count, count);
            if cfg.trigger {
                rs_mcu_traits::reg_write!(r, ctrl_trig, ctrl);
            } else {
                rs_mcu_traits::reg_write!(r, al1_ctrl, ctrl);
            }
        }
    }

    /// Start transfer on this channel by setting the enable bit in ctrl_trig.
    pub fn start(&mut self) {
        unsafe {
            let r = self.regs();
            let ctrl = rs_mcu_traits::reg_read!(r, ctrl_trig);
            rs_mcu_traits::reg_write!(r, ctrl_trig, ctrl | DMA_CTRL_EN);
        }
    }

    /// End/stop an active transfer and disable channel interrupt.
    pub fn end_transfer(&mut self) {
        self.disable_interrupt();
        unsafe {
            let r = self.regs();
            let ctrl = rs_mcu_traits::reg_read!(r, ctrl_trig);
            rs_mcu_traits::reg_write!(r, al1_ctrl, ctrl & !DMA_CTRL_EN);
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
            rs_mcu_traits::reg_write!(data_r, write_addr, dst_fifo_addr);
            // Write control configuration without triggering
            rs_mcu_traits::reg_write!(data_r, al1_ctrl, data_ctrl);
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
            rs_mcu_traits::reg_write!(ctrl_r, write_addr, target_addr);
            // Total transfers is 2 32-bit words per block
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*ctrl_r).trans_count),
                (blocks.len() * 2) as u32,
            );
            // Configure control channel without triggering yet
            rs_mcu_traits::reg_write!(ctrl_r, al1_ctrl, ctrl_config);
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
            rs_mcu_traits::reg_write!(data_r, read_addr, src_fifo_addr);
            rs_mcu_traits::reg_write!(data_r, al1_ctrl, data_ctrl);
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
            rs_mcu_traits::reg_write!(ctrl_r, write_addr, target_addr);
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*ctrl_r).trans_count),
                (blocks.len() * 2) as u32,
            );
            rs_mcu_traits::reg_write!(ctrl_r, al1_ctrl, ctrl_config);
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

#[cfg(test)]
mod tests {
    use super::*;

    const CHANNEL: usize = 5;

    unsafe extern "C" fn handler(cookie: *mut c_void) {
        // Never called from a host test; `publish`/`claimed` only pass the
        // address around.
        let _ = cookie;
    }

    /// A published slot comes back with the callback *and* the cookie that was
    /// published with it.
    #[test]
    fn publish_and_claim_carry_the_cookie_with_the_callback() {
        let cookie = 0x1234_5678 as *mut c_void;
        publish(CHANNEL, handler, cookie);

        let (cb, claimed_cookie) = claimed(CHANNEL).expect("published slot");
        let expected = handler as unsafe extern "C" fn(cookie: *mut c_void) as usize;
        assert_eq!(cb as usize, expected);
        assert_eq!(claimed_cookie, cookie);
    }

    /// An untouched channel has no callback, so the ISR calls nothing.
    #[test]
    fn an_unpublished_channel_claims_nothing() {
        assert!(claimed(CHANNEL + 1).is_none());
    }

    /// Detaching clears the address first: a later claim — the ISR running after
    /// `detach_callback` — sees "none", never a stale cookie.
    #[test]
    fn unpublish_clears_the_slot() {
        publish(CHANNEL, handler, 0x42 as *mut c_void);
        unpublish(CHANNEL);

        assert!(claimed(CHANNEL).is_none());
        assert_eq!(CALLBACKS[CHANNEL].load(Ordering::Acquire), 0);
        assert!(COOKIES[CHANNEL].load(Ordering::Acquire).is_null());
    }
}

// ---------------------------------------------------------------------------
//  The MCU-agnostic DMA contract
// ---------------------------------------------------------------------------

use rs_mcu_traits::dma as hal;
use rs_mcu_traits::handler::HandlerTable;

/// One handler slot per channel.
static HANDLERS: HandlerTable<dyn hal::DmaHandler, DMA_MAX_CHANNELS> = HandlerTable::new();

/// Second step of the completion hand-off: the channel callback is a raw `extern
/// "C"` function, so the typed handler is looked up by channel index (the cookie).
unsafe extern "C" fn completion_trampoline(cookie: *mut c_void) {
    // SAFETY: the handler was installed before the transfer was started, and the
    // interrupt is the only reader.
    if let Some(handler) = unsafe { HANDLERS.isr_get(cookie as usize) } {
        handler.on_complete();
    }
}

/// The RP2xxx DMA controller: it owns the channels and hands out indices.
///
/// A driver keeps its index for its whole life, so the channel is claimed once and
/// only released when the driver goes away.
pub struct Controller {
    channels: [Option<DmaChannel>; DMA_MAX_CHANNELS],
    /// Completion semaphores, one per channel: `wait` takes the one of its
    /// channel, the interrupt gives it.
    done: [BinarySemaphore; DMA_MAX_CHANNELS],
}

impl Controller {
    /// An empty controller, usable in a `static`.
    pub const fn new() -> Self {
        Self {
            channels: [const { None }; DMA_MAX_CHANNELS],
            done: [const { BinarySemaphore::new() }; DMA_MAX_CHANNELS],
        }
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

/// `hal` width → the driver's `CTRL.DATA_SIZE`.
const fn dma_width(width: hal::TransferWidth) -> DmaTransferWidth {
    match width {
        hal::TransferWidth::Byte => DmaTransferWidth::Byte,
        hal::TransferWidth::HalfWord => DmaTransferWidth::HalfWord,
        hal::TransferWidth::Word => DmaTransferWidth::Word,
    }
}

impl hal::Dma for Controller {
    type Channel = u8;

    fn claim(&mut self) -> rs_mcu_traits::Result<u8> {
        for (index, slot) in self.channels.iter_mut().enumerate() {
            if slot.is_none()
                && let Some(channel) = DmaChannel::claim()
            {
                *slot = Some(channel);
                return Ok(index as u8);
            }
        }
        Err(rs_mcu_traits::Error::Busy)
    }

    fn release(&mut self, channel: u8) {
        if let Some(slot) = self.channels.get_mut(channel as usize) {
            *slot = None; // `Drop` aborts the transfer and frees the channel.
        }
    }

    fn start_write(
        &mut self,
        channel: u8,
        buffer: &[u8],
        peripheral_address: u32,
        width: hal::TransferWidth,
        request: Option<u32>,
    ) -> rs_mcu_traits::Result<()> {
        let index = channel as usize;
        let (channels, done) = (&mut self.channels, &self.done);
        let Some(dma) = channels.get_mut(index).and_then(Option::as_mut) else {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        };
        let count = hal::elements(buffer, width) as u32;
        if count == 0 {
            return Err(rs_mcu_traits::Error::BufferTooSmall);
        }
        done[index].try_take();
        dma.detach_callback();
        dma.attach_completion_semaphore(&done[index]);
        dma.configure_tx(
            buffer.as_ptr(),
            peripheral_address,
            count,
            DmaTransferConfig::new(dma_width(width), request.unwrap_or(DREQ_PERMANENT)),
        );
        Ok(())
    }

    fn start_read(
        &mut self,
        channel: u8,
        buffer: &mut [u8],
        peripheral_address: u32,
        width: hal::TransferWidth,
        request: Option<u32>,
    ) -> rs_mcu_traits::Result<()> {
        let index = channel as usize;
        let count = hal::elements(buffer, width) as u32;
        if count == 0 {
            return Err(rs_mcu_traits::Error::BufferTooSmall);
        }
        let (channels, done) = (&mut self.channels, &self.done);
        let Some(dma) = channels.get_mut(index).and_then(Option::as_mut) else {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        };
        done[index].try_take();
        dma.detach_callback();
        dma.attach_completion_semaphore(&done[index]);
        dma.configure_rx(
            peripheral_address,
            buffer.as_mut_ptr(),
            count,
            DmaTransferConfig::new(dma_width(width), request.unwrap_or(DREQ_PERMANENT)),
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

    /// The completion runs in the DMA interrupt: the channel's callback is the
    /// trampoline, and the index it is handed is the cookie.
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

