//! Buffered, DMA-driven UART transmitter for RP2040 / RP2350.
//!
//! Mirrors `rs_uart_bluepill::BufferedTx` and the C++ `lnRpSerialTxOnly`
//! driver: a 128-byte ring buffer in `.bss` drained by a claimed DMA channel
//! paced by `DREQ_UARTx_TX`.

use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::mem::MaybeUninit;
use portable_atomic::{AtomicBool, AtomicUsize, Ordering};
use rs_dma_rp2xxx::constants::{DmaTransferConfig, DmaTransferWidth};
use rs_dma_rp2xxx::{DREQ_UART0_TX, DREQ_UART1_TX, DmaChannel, unreset_dma};
use rs_gpio_rp2xxx::Pin;
use rs_rcu_rp2xxx::Rcu;
use rs_rtos::{BinarySemaphore, with_critical};

use crate::registers::{
    FUNCSEL_UART, UART_CR_TXE, UART_CR_UARTEN, UART_DMACR_TXDMAE, UART_FR_TXFF,
    UART_LCR_H_WLEN_8, UartRegisters, base,
};
use crate::{divisors, peripheral, reg_read, reg_write};

const RING_SIZE: usize = 128;
const RING_MASK: usize = RING_SIZE - 1;
const _: () = assert!(RING_SIZE.is_power_of_two());

macro_rules! reg {
    ($member:ident) => {
        core::mem::offset_of!(UartRegisters, $member) as u32
    };
}

/// Transmit buffer: a 128-byte ring drained by DMA.
pub struct BufferedTx {
    instance: u32,
    ring: UnsafeCell<[u8; RING_SIZE]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    in_flight: AtomicUsize,
    txing: AtomicBool,
    room: BinarySemaphore,
    dma: UnsafeCell<MaybeUninit<DmaChannel>>,
    initialised: AtomicBool,
}

// SAFETY: ring state is mutated only inside `with_critical` (writers) or in
// the DMA completion ISR (`tail`/`in_flight`); `dma` is written once in `init`.
unsafe impl Sync for BufferedTx {}

unsafe extern "C" fn tx_dma_isr(cookie: *mut c_void) {
    if cookie.is_null() {
        return;
    }
    // SAFETY: `cookie` is the `BufferedTx` in `.bss` that armed the channel.
    let me = unsafe { &*(cookie as *const BufferedTx) };
    me.dma_done();
}

impl BufferedTx {
    /// Create a transmitter for UART `instance` (`0` or `1`). No hardware is
    /// touched and nothing is allocated, so the value can live in `.bss`.
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

    /// UART instance this transmitter drives (`0` or `1`).
    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    fn dreq(&self) -> u32 {
        match self.instance {
            1 => DREQ_UART1_TX,
            _ => DREQ_UART0_TX,
        }
    }

    /// One-time hardware setup: peripheral clock, default TX pad, `speed` baud,
    /// PL011 TX DMA enable, and a claimed DMA channel.
    pub fn init(&self, speed: u32) -> bool {
        if self.initialised.swap(true, Ordering::SeqCst) {
            return true;
        }
        let instance = if self.instance == 1 { 1 } else { 0 };
        let periph = peripheral(instance);
        Rcu::enable(periph);
        let mut tx_pin = if instance == 1 { Pin::GPIO4 } else { Pin::GPIO0 };
        tx_pin.set_function(FUNCSEL_UART);

        if let Some((ibrd, fbrd)) = divisors(rs_rcu_rp2xxx::get_clock(periph).max(1), speed) {
            reg_write(instance, reg!(ibrd), ibrd as u32);
            reg_write(instance, reg!(fbrd), fbrd as u32);
        }
        reg_write(instance, reg!(lcr_h), UART_LCR_H_WLEN_8);
        reg_write(instance, reg!(cr), UART_CR_UARTEN | UART_CR_TXE);
        reg_write(instance, reg!(dmacr), UART_DMACR_TXDMAE);

        unreset_dma();
        let Some(chan) = DmaChannel::claim() else {
            self.initialised.store(false, Ordering::SeqCst);
            return false;
        };
        // SAFETY: `dma` is written once here before `initialised` is observed.
        unsafe {
            (*self.dma.get()).write(chan);
        }

        self.room.try_take();
        self.reset_ring();
        true
    }

    #[inline]
    fn count(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if head >= tail {
            head - tail
        } else {
            RING_SIZE - tail + head
        }
    }

    #[inline]
    fn free(&self) -> usize {
        RING_SIZE - self.count() - 1
    }

    fn reset_ring(&self) {
        with_critical(|| {
            self.head.store(0, Ordering::Release);
            self.tail.store(0, Ordering::Release);
            self.in_flight.store(0, Ordering::Release);
            self.txing.store(false, Ordering::Release);
        });
    }

    fn put(&self, data: &[u8]) -> usize {
        // SAFETY: caller holds critical section.
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

    fn read_ptr(&self) -> (*const u8, usize) {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let n = if head < tail {
            RING_SIZE - tail
        } else {
            head - tail
        };
        let base_ptr = unsafe { (*self.ring.get()).as_ptr() };
        // SAFETY: `tail < RING_SIZE`.
        (unsafe { base_ptr.add(tail) }, n)
    }

    /// Queue `data`, blocking while the ring is full.
    pub fn write(&self, mut data: &[u8]) {
        if !self.initialised.load(Ordering::Acquire) {
            return;
        }
        while !data.is_empty() {
            let must_wait = with_critical(|| {
                let free = self.free();
                if free == 0 {
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
                self.room.take();
            }
        }
    }

    /// Bytes queued but not sent yet.
    pub fn pending(&self) -> usize {
        self.count()
    }

    /// Block until everything queued has been handed off by DMA.
    pub fn flush(&self) {
        while self.pending() > 0 {
            self.room.take();
        }
    }

    /// Queue as much of `data` as fits right now and return the byte count.
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

    /// Polled write for the crash path (`Logger_crash`).
    pub fn raw_write(&self, data: &[u8]) {
        if !self.initialised.load(Ordering::Acquire) {
            return;
        }
        let instance = if self.instance == 1 { 1 } else { 0 };
        // SAFETY: `init()` ran and claimed the channel.
        unsafe {
            (*self.dma.get()).assume_init_mut().end_transfer();
        }
        let dmacr = reg_read(instance, reg!(dmacr));
        reg_write(instance, reg!(dmacr), dmacr & !UART_DMACR_TXDMAE);
        self.reset_ring();

        for &byte in data {
            while (reg_read(instance, reg!(fr)) & UART_FR_TXFF) != 0 {}
            reg_write(instance, reg!(dr), byte as u32);
        }
    }

    fn ignite(&self) {
        let (ptr, len) = self.read_ptr();
        if len == 0 {
            self.txing.store(false, Ordering::Release);
            return;
        }
        self.in_flight.store(len, Ordering::Relaxed);
        self.txing.store(true, Ordering::Release);

        let instance = if self.instance == 1 { 1 } else { 0 };
        let dmacr = reg_read(instance, reg!(dmacr));
        if (dmacr & UART_DMACR_TXDMAE) == 0 {
            reg_write(instance, reg!(dmacr), dmacr | UART_DMACR_TXDMAE);
        }
        self.start_dma(ptr, len);
    }

    fn start_dma(&self, ptr: *const u8, len: usize) {
        let instance = if self.instance == 1 { 1 } else { 0 };
        let dr_addr = base(instance).expect("instance") + reg!(dr);
        // SAFETY: `init()` ran and only this driver touches `self.dma`.
        let dma = unsafe { (*self.dma.get()).assume_init_mut() };
        dma.attach_callback(tx_dma_isr, self as *const Self as *mut c_void);
        dma.configure_tx(
            ptr,
            dr_addr,
            len as u32,
            DmaTransferConfig::new(DmaTransferWidth::Byte, self.dreq()),
        );
    }

    fn dma_done(&self) {
        let sent = self.in_flight.load(Ordering::Relaxed);
        let tail = (self.tail.load(Ordering::Relaxed) + sent) & RING_MASK;
        self.tail.store(tail, Ordering::Release);
        self.in_flight.store(0, Ordering::Relaxed);
        self.room.give_from_isr();

        let (ptr, len) = self.read_ptr();
        if len == 0 {
            self.txing.store(false, Ordering::Release);
            if let Some(handler) = unsafe { crate::HANDLERS.isr_get(self.instance as usize) } {
                handler.on_tx_complete();
            }
            return;
        }
        self.in_flight.store(len, Ordering::Relaxed);
        self.start_dma(ptr, len);
    }
}
