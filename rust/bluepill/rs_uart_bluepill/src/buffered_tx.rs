//! Ring-buffered, DMA-driven UART transmitter.
//!
//! Rust port of the C++ `lnSerialBpTxOnlyBufferedDma`
//! (`esprit/mcus/common_bluepill/lnSerialTxOnlyDmaBuffer.cpp`), i.e. what
//! `createLnSerialTxOnly(instance, /*dma*/ true, /*buffered*/ true)` returns:
//!
//! * a [`RING_SIZE`] byte ring with `lnRingBuffer`'s arithmetic
//!   (`free() = size - count() - 1`: one byte is always left free),
//! * one DMA channel draining it (memory -> USART DATA, byte sized),
//! * [`BufferedTx::write`] blocks **only when the ring is full**; the DMA
//!   completion interrupt frees the room and wakes the writer
//!   (`_txRingSem.give()`),
//! * that same interrupt re-arms the DMA on the next *contiguous* run of the
//!   ring, so no byte is ever copied twice (`txDmaCb2`).
//!
//! Why this exists: the point is that a log line costs a `memcpy` in the
//! caller and the UART keeps draining in the background.  A plain
//! [`UartTx::transmit`](crate::UartTx::transmit) waits for the transmission
//! complete flag on *every* call, which at 115200 Bd is ~87 us per byte.
//!
//! ## Concurrency
//!
//! `head` is only moved by writers (task context, inside [`with_critical`]),
//! `tail` only by the DMA completion interrupt.  Masking interrupts while a
//! writer touches the ring is what makes the writer's view race free, exactly
//! like the C++ original -- there, `ENTER_CRITICAL()` around the ring
//! operations plays the same role.

use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::mem::MaybeUninit;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rs_dma_bluepill::{DmaChannel, DmaEngine};
use rs_esprit::BinarySemaphore;
use rs_gpio_bluepill::{Mode, Pin};
use rs_rcu_bluepill::{enable, Peripheral};
use rs_rtos::with_critical;

use crate::registers::{
    UartRegisters, USART_CTL0_TEN, USART_CTL0_UEN, USART_CTL2_DMAT, USART_STAT_TC, USART_STAT_TBE,
    UART_INSTANCE_1, UART_INSTANCE_2, UART0_DMA_TX_CHANNEL, UART1_DMA_TX_CHANNEL, UART2_DMA_TX_CHANNEL,
};

/// Ring size in bytes: `createLnSerialTxOnly(instance, true, true)` asks the
/// C++ driver for 128 (`lnSerialCore.cpp`).
pub const RING_SIZE: usize = 128;

/// `RING_SIZE - 1`: the ring is addressed modulo its size, which must be a
/// power of two (`lnRingBuffer`'s `_mask`).
const RING_MASK: usize = RING_SIZE - 1;

/// Transmit buffer: a byte ring drained by DMA.
pub struct BufferedTx {
    instance: u32,
    /// Payload. Written under [`with_critical`] by the caller side, read by the
    /// DMA engine (hardware) -- and by `read_ptr`, which only hands out the
    /// part that is no longer writable.
    ring: UnsafeCell<[u8; RING_SIZE]>,
    /// First free byte; moved by writers only.
    head: AtomicUsize,
    /// First byte the DMA has not sent yet; moved by the completion ISR only.
    tail: AtomicUsize,
    /// Bytes handed to the DMA and not completed yet.
    in_flight: AtomicUsize,
    /// A transfer is armed (or about to be): a writer must not start another.
    txing: AtomicBool,
    /// "Room available" notification, given by the completion ISR.
    room: BinarySemaphore,
    /// The DMA channel. `DmaChannel::new` claims the channel for good, so this
    /// is filled once, by [`BufferedTx::init`].  `MaybeUninit` rather than
    /// `Option<DmaChannel>` on purpose: `None` would need a niche value (2 =
    /// "not a `DmaEngine`"), and that non-zero pattern would push the whole
    /// structure into `.data` -- 244 bytes of flash for nothing.
    /// `initialised` below says whether it holds a channel.
    dma: UnsafeCell<MaybeUninit<DmaChannel>>,
    /// [`BufferedTx::init`] has run.
    initialised: AtomicBool,
}

// SAFETY: the interior mutability follows one rule, enforced by the call sites:
// the ring state changes either inside `with_critical` (writers) or in the DMA
// completion interrupt (tail / in_flight), never both; and `dma` is written
// exactly once, by `init()`, which also publishes `initialised` before any
// writer or interrupt can use the channel.
unsafe impl Sync for BufferedTx {}

/// The DMA completion interrupt (C++ `_dmaCallback2` -> `txDmaCb2`).
unsafe extern "C" fn tx_dma_isr(_half: bool, cookie: *mut c_void) {
    if cookie.is_null() {
        return;
    }
    // SAFETY: the cookie is the `BufferedTx` that armed the channel; it lives
    // in `.bss`, so it outlives every transfer.
    let me = unsafe { &*(cookie as *const BufferedTx) };
    me.dma_done();
}

impl BufferedTx {
    /// Create a transmitter for UART `instance` (0..2).  No hardware is
    /// touched and nothing is allocated, so the value can live in a `static`
    /// (`.bss`, malloc_used.md §0.7).
    pub const fn new(instance: u32) -> Self {
        Self {
            instance,
            ring: UnsafeCell::new([0; RING_SIZE]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            in_flight: AtomicUsize::new(0),
            txing: AtomicBool::new(false),
            room: BinarySemaphore::new(),
            dma: UnsafeCell::new(MaybeUninit::uninit()),
            initialised: AtomicBool::new(false),
        }
    }

    /// UART instance this transmitter drives.
    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    /// One-time hardware setup: peripheral clock, TX pin, `speed` baud, the
    /// transmitter and its DMA request.  Mirrors `lnSerialBpCore::init()` plus
    /// the constructor of the C++ buffered driver.
    ///
    /// Calling it twice is a no-op (the DMA channel is already claimed).
    pub fn init(&self, speed: u32) -> bool {
        if self.initialised.swap(true, Ordering::SeqCst) {
            return true;
        }
        let (peripheral, tx_pin, dma_channel) = match self.instance {
            UART_INSTANCE_1 => (Peripheral::Uart1, Pin::PA2, UART1_DMA_TX_CHANNEL),
            UART_INSTANCE_2 => (Peripheral::Uart2, Pin::PB10, UART2_DMA_TX_CHANNEL),
            // Instance 0 and anything unexpected: the debug UART on PA9.
            _ => (Peripheral::Uart0, Pin::PA9, UART0_DMA_TX_CHANNEL),
        };
        enable(peripheral);
        rs_gpio_bluepill::set_mode(tx_pin, Mode::AlternatePushPull, 50);
        crate::uart_set_baudrate(self.instance, speed);

        // Transmitter on, DMA requests on, and *no* USART interrupt: the C++
        // `_programTx()` does the same (TEN + DMA_TX, USART IRQ masked), the
        // transfer being driven by the DMA.  CTL0 is written, not ORed, to get
        // the 8N1 default: `lnSerialBpCore::init()` clears it the same way
        // ("default is good enough") before setting the speed.
        unsafe {
            let regs = UartRegisters::ptr(self.instance);
            write_volatile(&mut (*regs).ctl0, USART_CTL0_UEN | USART_CTL0_TEN);
            let ctl2 = read_volatile(&mut (*regs).ctl2);
            write_volatile(&mut (*regs).ctl2, ctl2 | USART_CTL2_DMAT);
        }

        // SAFETY: `dma` is only written here, before `initialised` lets any
        // other accessor through.
        unsafe {
            (*self.dma.get()).write(DmaChannel::new(DmaEngine::Dma0, dma_channel));
        }

        // FreeRTOS creates a semaphore's control block on first *use*, and the
        // completion interrupt (`give_from_isr`) cannot do that: force the
        // creation now, from task context.  `try_take` on a fresh (not given)
        // semaphore fails, which is exactly the state we want.
        self.room.try_take();

        self.reset_ring();
        true
    }

    /// Bytes currently queued.
    #[inline]
    fn count(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head >= tail { head - tail } else { RING_SIZE - tail + head }
    }

    /// Bytes a writer may still push (`lnRingBuffer::free()`).
    #[inline]
    fn free(&self) -> usize {
        RING_SIZE - self.count() - 1
    }

    /// Drop whatever is queued and forget the DMA state.
    fn reset_ring(&self) {
        with_critical(|| {
            self.head.store(0, Ordering::Release);
            self.tail.store(0, Ordering::Release);
            self.in_flight.store(0, Ordering::Release);
            self.txing.store(false, Ordering::Release);
        });
    }

    /// Copy `data` into the ring (never more than [`Self::free`]) and move
    /// `head`.  The caller holds the critical section.
    fn put(&self, data: &[u8]) -> usize {
        // SAFETY: the caller holds the critical section, so neither the ISR
        // nor another writer can be looking at the ring.
        let ring = unsafe { &mut *self.ring.get() };
        let mut head = self.head.load(Ordering::Relaxed);
        let mut done = 0;
        while done < data.len() {
            let chunk = (data.len() - done).min(RING_SIZE - head);
            ring[head..head + chunk].copy_from_slice(&data[done..done + chunk]);
            head = (head + chunk) & RING_MASK;
            done += chunk;
        }
        self.head.store(head, Ordering::Release);
        done
    }

    /// What the DMA may send next: `tail..head`, or `tail..RING_SIZE` when the
    /// queue wraps (`lnRingBuffer::getReadPointer` -- no bounce buffer).
    fn read_ptr(&self) -> (*const u8, usize) {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let n = if head < tail { RING_SIZE - tail } else { head - tail };
        let base = unsafe { (*self.ring.get()).as_ptr() };
        // SAFETY: `tail < RING_SIZE` by construction (masked on every update).
        (unsafe { base.add(tail) }, n)
    }

    /// Queue `data`, blocking while the ring is full.  Parity with the C++
    /// `lnSerialTxOnly::transmit()`: it waits for room, it never drops bytes.
    pub fn write(&self, mut data: &[u8]) {
        if !self.initialised.load(Ordering::Acquire) {
            return;
        }
        while !data.is_empty() {
            let must_wait = with_critical(|| {
                let free = self.free();
                if free == 0 {
                    // Drop a stale notification so that we sleep until the
                    // *next* completion frees room (C++ `_txRingSem.tryTake()`).
                    self.room.try_take();
                    return true;
                }
                let n = free.min(data.len());
                self.put(&data[..n]);
                data = &data[n..];
                if !self.txing.load(Ordering::Acquire) {
                    self.ignite();
                }
                false
            });
            if must_wait {
                // Parity with the C++ `_txRingSem.take()`: sleep until the DMA
                // completion interrupt frees room.  Interrupts are enabled here,
                // that is the whole point of releasing the critical section.
                self.room.take();
            }
        }
    }

    /// Queue as much of `data` as fits right now and return the byte count.
    /// Parity with the C++ `transmitNoBlock()`.
    pub fn try_write(&self, data: &[u8]) -> usize {
        if !self.initialised.load(Ordering::Acquire) {
            return 0;
        }
        with_critical(|| {
            let n = self.free().min(data.len());
            if n == 0 {
                return 0;
            }
            self.put(&data[..n]);
            if !self.txing.load(Ordering::Acquire) {
                self.ignite();
            }
            n
        })
    }

    /// Polled write, for the crash path.  Parity with the C++
    /// `rawWrite()` (`ln_serial_rawWrite()`), with one deliberate difference:
    /// the DMA is stopped first and the queued bytes are dropped, so the crash
    /// text cannot be interleaved with what the ring still holds (the C++
    /// version mixes both).  The transmitter keeps working afterwards: the next
    /// [`Self::write`] re-arms the channel.
    pub fn raw_write(&self, data: &[u8]) {
        if !self.initialised.load(Ordering::Acquire) {
            return;
        }
        // SAFETY: `init()` ran, so the channel is initialised and only this
        // module uses it.
        unsafe {
            (*self.dma.get()).assume_init_mut().end_transfer(); // detaches the callback, disables the channel
            let regs = UartRegisters::ptr(self.instance);
            let ctl2 = read_volatile(&mut (*regs).ctl2);
            write_volatile(&mut (*regs).ctl2, ctl2 & !USART_CTL2_DMAT);
        }
        self.reset_ring();

        let regs = UartRegisters::ptr(self.instance);
        for &byte in data {
            unsafe {
                while (read_volatile(&mut (*regs).stat) & USART_STAT_TBE) == 0 {}
                write_volatile(&mut (*regs).data, byte as u32);
            }
        }
    }

    /// Start a transfer for what the ring holds.  Called with the critical
    /// section held from [`Self::write`]/[`Self::try_write`], and from the
    /// completion interrupt itself (C++ `igniteTx()` + `txDmaCb2`).
    fn ignite(&self) {
        let (ptr, len) = self.read_ptr();
        if len == 0 {
            self.txing.store(false, Ordering::Release);
            return;
        }
        self.in_flight.store(len, Ordering::Relaxed);
        self.txing.store(true, Ordering::Release);

        // C++ `_programTx()`: enable the transmitter and its DMA request, and
        // clear TC (`rc_w0`: writing 0 clears it, every other bit is written
        // back with the value just read).
        let regs = UartRegisters::ptr(self.instance);
        unsafe {
            let ctl0 = read_volatile(&mut (*regs).ctl0);
            write_volatile(&mut (*regs).ctl0, ctl0 | USART_CTL0_TEN);
            let ctl2 = read_volatile(&mut (*regs).ctl2);
            write_volatile(&mut (*regs).ctl2, ctl2 | USART_CTL2_DMAT);
            let stat = read_volatile(&mut (*regs).stat);
            write_volatile(&mut (*regs).stat, stat & !USART_STAT_TC);
        }
        self.start_dma(ptr, len);
    }

    /// Hand `len` bytes at `ptr` to the DMA engine (memory -> USART DATA, byte
    /// sized) and arm the completion interrupt.
    fn start_dma(&self, ptr: *const u8, len: usize) {
        let regs = UartRegisters::ptr(self.instance);
        // SAFETY: `init()` ran -- every caller is either behind the
        // `initialised` check or is the ISR of a transfer that `init()` set up.
        let dma = unsafe { (*self.dma.get()).assume_init_mut() };
        // `dmaIrqHandler()` masks the NVIC line before it runs the raw
        // callback (`lnDma.cpp`), so each transfer has to re-arm both -- which
        // is what the C++ side gets from the `lnEnableInterrupt()` at the end
        // of `doMemoryToPeripheralTransferNoLock()`.
        dma.attach_callback(tx_dma_isr, self as *const Self as *mut c_void);
        dma.begin_tx_transfer(
            // SAFETY: the USART registers are MMIO, valid for the whole run.
            unsafe { &(*regs).data } as *const u32 as u32,
            ptr as u32,
            len as u32,
            false,
            false,
        );
    }

    /// Interrupt context (C++ `txDmaCb2`): free the bytes the DMA just sent,
    /// wake a blocked writer and re-arm for the next run.  Must not block and
    /// must not take any lock.
    fn dma_done(&self) {
        let sent = self.in_flight.load(Ordering::Relaxed);
        let tail = (self.tail.load(Ordering::Relaxed) + sent) & RING_MASK;
        self.tail.store(tail, Ordering::Release);
        self.in_flight.store(0, Ordering::Relaxed);
        self.room.give_from_isr(); // room freed, wake up the writer

        let (ptr, len) = self.read_ptr();
        if len == 0 {
            self.txing.store(false, Ordering::Release);
            return;
        }
        self.in_flight.store(len, Ordering::Relaxed);
        self.start_dma(ptr, len);
    }
}

// EOF

