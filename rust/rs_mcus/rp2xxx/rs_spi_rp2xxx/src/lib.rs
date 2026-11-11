//! RP2040 / RP2350 SPI driver (ARM PrimeCell PL022) with DMA.
//!
//! This is the `rp2040` implementation of [`rs_mcu_traits::spi::Spi`].
//!
//! The crate used to also carry `shim.rs`, the `lnspi_*` C ABI, behind a `c_abi`
//! feature. Nothing references those symbols on this platform — a Rust build does
//! not compile `rust_esprit/c_interface/lnSPI_c.cpp`, no C/C++ code includes
//! `lnSPI_c.h`, and `rust_esprit`'s SPI wrapper is gone (M7b) — so the shim and
//! its feature were deleted rather than left as a dormant trap. The swindle
//! images carry no `lnspi_*` symbol at all.

#![no_std]

pub mod constants;
pub mod registers;

pub use constants::*;
pub use registers::*;

use core::ffi::c_void;
use rs_dma_rp2xxx::{
    DmaChannel, DmaGatherBlock, DmaScatterBlock, DmaTransferConfig, DmaTransferWidth,
    DREQ_SPI0_RX, DREQ_SPI0_TX, DREQ_SPI1_RX, DREQ_SPI1_TX,
};
use rs_mcu_traits::gpio::{Gpio, PinMode};
use rs_rtos::BinarySemaphore;
use rs_mcu_traits::handler::HandlerTable;
use rs_mcu_traits::spi;
use rs_gpio_rp2xxx::Pin;
use rs_rcu_rp2xxx::{Peripheral, Rcu};

const MAX_CHUNKS: usize = 32;

fn spi_periph(instance: u32) -> Peripheral {
    if instance == 0 {
        Peripheral::Spi0
    } else {
        Peripheral::Spi1
    }
}

fn cs_pin(pin_cs: i32) -> Option<Pin> {
    // A negative CS number (no chip select) or one past the last pad is "no pin";
    // `Pin::from_number` owns the range, so there is no literal here.
    Pin::from_number(u32::try_from(pin_cs).ok()?)
}

fn init_cs_pin(pin: i32) {
    if let Some(mut p) = cs_pin(pin) {
        let _ = p.set_mode(PinMode::Output);
        p.write(true); // Idle high (CS is active low)
    }
}

fn set_cs(pin: i32, high: bool) {
    if let Some(mut p) = cs_pin(pin) {
        p.write(high);
    }
}

/// Reject LSB-first: the PL022 shifts **MSB first only** (`SSPCR0` has no
/// bit-order field, so there is nothing to program).
///
/// This is an unsupported configuration, so it panics with a message instead of
/// silently sending the caller's bytes out backwards.
fn check_bit_order(order: BitOrder) {
    assert!(
        order == BitOrder::MsbFirst,
        "SPI bit order LSB-first is not supported: the PL022 shifts MSB first only"
    );
}

/// Reject frame sizes other than 8 and 16 bits — the two this driver moves
/// (`SSPCR0.DSS` could encode 4..16 bit frames, but nothing here packs them).
fn check_data_size(data_size: u32) {
    assert!(
        data_size == SPI_DATA_SIZE_8 || data_size == SPI_DATA_SIZE_16,
        "SPI frame size must be 8 or 16 bits: this driver moves bytes / half-words only"
    );
}

/// Spin until `ready()` says yes, giving up after [`POLL_LIMIT`] iterations.
///
/// Every polling loop in this driver goes through here, so the timeout policy —
/// the budget, the `spin_loop` hint, and "gave up" meaning `false` — lives in one
/// place. A bus that never answers (wrong pins, unpowered target) makes the
/// caller return instead of hanging the probe.
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

/// Idiomatic Rust hardware SPI driver for the RP2040 (ARM PrimeCell PL022).
pub struct Spi {
    instance: u32,
    pin_cs: i32,
    speed_hz: u32,
    data_size: u32,
    mode: SpiMode,
    dma_enabled: bool,

    tx_dma: Option<DmaChannel>,
    rx_dma: Option<DmaChannel>,
    ctrl_dma: Option<DmaChannel>,

    done_sem: BinarySemaphore,
    async_cb: Option<unsafe extern "C" fn(*mut c_void)>,
    async_cookie: *mut c_void,
}

impl Spi {
    /// Create a new SPI driver for the given instance (0 or 1) and CS pin.
    pub fn new(instance: u32, pin_cs: i32) -> Self {
        assert!(instance < SPI_MAX_INSTANCES as u32, "Invalid SPI instance");
        Rcu::enable(spi_periph(instance));
        init_cs_pin(pin_cs);

        let mut spi = Self {
            instance,
            pin_cs,
            speed_hz: DEFAULT_SPEED_HZ,
            data_size: SPI_DATA_SIZE_8,
            mode: SpiMode::Mode0,
            dma_enabled: true,
            tx_dma: None,
            rx_dma: None,
            ctrl_dma: None,
            done_sem: BinarySemaphore::new(),
            async_cb: None,
            async_cookie: core::ptr::null_mut(),
        };

        spi.claim_dma_channels();
        spi.begin(SPI_DATA_SIZE_8);
        spi
    }

    fn claim_dma_channels(&mut self) {
        if self.tx_dma.is_none() {
            self.tx_dma = DmaChannel::claim();
        }
        if self.rx_dma.is_none() {
            self.rx_dma = DmaChannel::claim();
        }
        if self.ctrl_dma.is_none() {
            self.ctrl_dma = DmaChannel::claim();
        }
    }

    #[inline(always)]
    pub fn regs(&self) -> *mut SpiRegisters {
        SpiRegisters::ptr(self.instance)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn cr0(&self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*self.regs()).sspcr0)) }
    }

    #[inline(always)]
    fn set_cr0(&self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*self.regs()).sspcr0), val) }
    }

    #[inline(always)]
    fn cr1(&self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*self.regs()).sspcr1)) }
    }

    #[inline(always)]
    fn set_cr1(&self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*self.regs()).sspcr1), val) }
    }

    #[inline(always)]
    fn modify_cr1(&self, set: u32, clear: u32) {
        let val = self.cr1();
        self.set_cr1((val | set) & !clear);
    }

    #[inline(always)]
    fn read_dr(&self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*self.regs()).sspdr)) }
    }

    #[inline(always)]
    fn write_dr(&self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*self.regs()).sspdr), val) }
    }

    #[inline(always)]
    fn status(&self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*self.regs()).sspsr)) }
    }

    #[inline(always)]
    fn set_cpsr(&self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*self.regs()).sspcpsr), val) }
    }

    #[inline(always)]
    fn set_imsc(&self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*self.regs()).sspimsc), val) }
    }

    #[inline(always)]
    fn clear_interrupts(&self, mask: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*self.regs()).sspicr), mask) }
    }

    #[inline(always)]
    fn dmacr(&self) -> u32 {
        unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*self.regs()).sspdmacr)) }
    }

    #[inline(always)]
    fn set_dmacr(&self, val: u32) {
        unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*self.regs()).sspdmacr), val) }
    }

    #[inline(always)]
    fn modify_dmacr(&self, set: u32, clear: u32) {
        let val = self.dmacr();
        self.set_dmacr((val | set) & !clear);
    }

    #[inline(always)]
    fn is_busy(&self) -> bool {
        (self.status() & SSPSR_BSY) != 0
    }

    #[inline(always)]
    fn is_tx_not_full(&self) -> bool {
        (self.status() & SSPSR_TNF) != 0
    }

    #[inline(always)]
    fn is_rx_not_empty(&self) -> bool {
        (self.status() & SSPSR_RNE) != 0
    }

    #[inline(always)]
    pub fn data_register_addr(&self) -> u32 {
        if self.instance == 0 {
            SPI0_SSPDR_ADDR
        } else {
            SPI1_SSPDR_ADDR
        }
    }

    #[inline(always)]
    pub fn tx_dreq(&self) -> u32 {
        if self.instance == 0 {
            DREQ_SPI0_TX
        } else {
            DREQ_SPI1_TX
        }
    }

    #[inline(always)]
    pub fn rx_dreq(&self) -> u32 {
        if self.instance == 0 {
            DREQ_SPI0_RX
        } else {
            DREQ_SPI1_RX
        }
    }

    #[inline(always)]
    pub fn cs_on(&self) {
        set_cs(self.pin_cs, false);
    }

    #[inline(always)]
    pub fn cs_off(&self) {
        set_cs(self.pin_cs, true);
    }

    /// Initialise and enable the SPI bus with the given data frame size (8 or 16).
    ///
    /// # Panics
    ///
    /// Any other frame size: `SSPCR0.DSS` encodes 4..16 bit frames, but this
    /// driver moves bytes / half-words only, so 8 and 16 are the supported
    /// configurations. Panics with a message rather than silently using 8 bits.
    pub fn begin(&mut self, data_size: u32) {
        check_data_size(data_size);
        self.data_size = data_size;
        self.apply_configuration();
        self.modify_cr1(SSPCR1_SSE, 0);
    }

    /// Disable the SPI bus.
    pub fn end(&mut self) {
        self.modify_cr1(0, SSPCR1_SSE);
        self.set_dmacr(0);
        self.set_imsc(0);
    }

    pub fn set_speed(&mut self, speed_hz: u32) {
        self.speed_hz = speed_hz;
        self.apply_configuration();
    }

    pub fn set_data_mode(&mut self, mode: SpiMode) {
        self.mode = mode;
        self.apply_configuration();
    }

    /// Ask for a bit order.
    ///
    /// The PL022 shifts **MSB first only** — `SSPCR0` has no bit-order field, so
    /// there is nothing to program. Asking for LSB-first is an unsupported
    /// configuration, not a no-op: it panics with a message instead of sending
    /// every byte backwards.
    ///
    /// # Panics
    ///
    /// [`BitOrder::LsbFirst`].
    pub fn set_bit_order(&mut self, order: BitOrder) {
        check_bit_order(order);
    }

    pub fn set_dma_mode(&mut self, enable: bool) {
        self.dma_enabled = enable;
    }

    pub fn set_ssel(&mut self, ssel: i32) {
        self.pin_cs = ssel;
        init_cs_pin(ssel);
    }

    fn apply_configuration(&mut self) {
        let fq_in = Rcu::get_clock(spi_periph(self.instance));
        let target_speed = if self.speed_hz == 0 {
            DEFAULT_SPEED_HZ
        } else {
            self.speed_hz
        };

        let div = fq_in / target_speed;
        // Split the ratio the way the PL022 wants it: `CPSDVSR` (even, ≤ 254)
        // times `SCR + 1`, aimed at half the `SCR` range so neither half sits at
        // an extreme.
        let prescaler = ((div / (SCR_STEPS / 2)) & !1).clamp(CPSDVSR_MIN, CPSDVSR_MAX);
        let scaler = ((((div + (prescaler >> 1)) / prescaler) as i32) - 1).clamp(SCR_MIN, SCR_MAX);

        let dss = if self.data_size == 16 {
            SSPCR0_DSS_16BIT
        } else {
            SSPCR0_DSS_8BIT
        };

        let (spo, sph) = match self.mode {
            SpiMode::Mode0 => (0, 0),
            SpiMode::Mode1 => (0, 1),
            SpiMode::Mode2 => (1, 0),
            SpiMode::Mode3 => (1, 1),
        };

        let cr0 = dss
            | SSPCR0_FRF_MOTOROLA
            | (spo << SSPCR0_SPO_SHIFT)
            | (sph << SSPCR0_SPH_SHIFT)
            | ((scaler as u32) << SSPCR0_SCR_SHIFT);

        let sse_active = (self.cr1() & SSPCR1_SSE) != 0;
        if sse_active {
            self.modify_cr1(0, SSPCR1_SSE);
        }

        self.set_cpsr(prescaler);
        self.set_cr0(cr0);

        if sse_active {
            self.modify_cr1(SSPCR1_SSE, 0);
        }
    }

    /// Wait until the SPI hardware FIFO is completely drained and bus is idle.
    pub fn wait_for_completion(&self) -> bool {
        wait_until(|| !self.is_busy())
    }

    pub fn clear_stall_rx(&self) {
        while self.is_rx_not_empty() {
            let _ = self.read_dr();
        }
        self.clear_interrupts(SSPICR_RORIC | SSPICR_RTIC);
    }

    // --- Synchronous Polling Writes and Reads ---

    fn write_poll_internal(&mut self, data: u32) -> bool {
        self.cs_on();
        if !wait_until(|| self.is_tx_not_full()) {
            self.cs_off();
            return false;
        }
        self.write_dr(data);
        let ok = self.wait_for_completion();
        self.clear_stall_rx();
        self.cs_off();
        ok
    }

    pub fn write8(&mut self, data: u8) -> bool {
        self.write_poll_internal(data as u32)
    }

    pub fn write16(&mut self, data: u16) -> bool {
        self.write_poll_internal(data as u32)
    }

    fn block_write_poll_internal(&mut self, values: impl Iterator<Item = u32>) -> bool {
        self.cs_on();
        for val in values {
            if !wait_until(|| self.is_tx_not_full()) {
                self.cs_off();
                return false;
            }
            self.write_dr(val);
        }
        let ok = self.wait_for_completion();
        self.clear_stall_rx();
        self.cs_off();
        ok
    }

    pub fn read_poll8(&mut self, rx: &mut [u8]) -> bool {
        self.clear_stall_rx();
        for byte in rx.iter_mut() {
            if !wait_until(|| self.is_tx_not_full()) {
                return false;
            }
            self.write_dr(SPI_DUMMY_BYTE as u32);

            if !wait_until(|| self.is_rx_not_empty()) {
                return false;
            }
            *byte = (self.read_dr() & 0xFF) as u8;
        }
        self.wait_for_completion()
    }

    pub fn transfer_poll8(&mut self, tx: &[u8], rx: &mut [u8]) -> bool {
        assert_eq!(tx.len(), rx.len());
        self.clear_stall_rx();
        for (&t, byte) in tx.iter().zip(rx.iter_mut()) {
            if !wait_until(|| self.is_tx_not_full()) {
                return false;
            }
            self.write_dr(t as u32);

            if !wait_until(|| self.is_rx_not_empty()) {
                return false;
            }
            *byte = (self.read_dr() & 0xFF) as u8;
        }
        self.wait_for_completion()
    }

    pub fn block_write8(&mut self, data: &[u8]) -> bool {
        if self.dma_enabled && data.len() >= MIN_DMA_TRANSFER_SIZE && self.tx_dma.is_some() {
            return self.dma_write8(data);
        }
        self.block_write_poll_internal(data.iter().map(|&b| b as u32))
    }

    pub fn block_write16(&mut self, data: &[u16]) -> bool {
        if self.dma_enabled && data.len() >= MIN_DMA_TRANSFER_SIZE && self.tx_dma.is_some() {
            return self.dma_write16(data);
        }
        self.block_write_poll_internal(data.iter().map(|&w| w as u32))
    }

    /// The DMA path of [`Self::block_write_repeat_internal`].
    ///
    /// `None` when no TX channel was claimed — nothing has been touched yet in
    /// that case, so the caller simply polls.
    fn dma_write_repeat(
        &mut self,
        val_ptr: *const u8,
        count: u32,
        width: DmaTransferWidth,
    ) -> Option<bool> {
        // No channel claimed: bail out before anything is touched, so the caller
        // can simply poll instead.
        self.tx_dma.as_ref()?;
        let dr_addr = self.data_register_addr();
        let dreq = self.tx_dreq();
        self.done_sem.try_take();
        self.cs_on();
        self.enable_dma_tx(true);

        // `?` here is the borrow-shortening form of the check above: it keeps
        // `tx`'s borrow to the two calls that need it.
        let tx = self.tx_dma.as_mut()?;
        tx.attach_completion_semaphore(&self.done_sem);
        // Repeat mode: one address, no increment.
        tx.configure_tx(
            val_ptr,
            dr_addr,
            count,
            DmaTransferConfig::new(width, dreq).increment(false),
        );

        let ok = self.done_sem.take_timeout(DMA_TIMEOUT_MS);
        self.wait_for_completion();
        if let Some(t) = self.tx_dma.as_mut() {
            t.end_transfer();
        }
        self.enable_dma_tx(false);
        self.clear_stall_rx();
        self.cs_off();
        Some(ok)
    }

    fn block_write_repeat_internal(
        &mut self,
        val_ptr: *const u8,
        val_u32: u32,
        count: u32,
        width: DmaTransferWidth,
    ) -> bool {
        if self.dma_enabled
            && count as usize >= MIN_DMA_TRANSFER_SIZE
            && let Some(ok) = self.dma_write_repeat(val_ptr, count, width)
        {
            return ok;
        }

        self.block_write_poll_internal(core::iter::repeat_n(val_u32, count as usize))
    }

    pub fn block_write8_repeat(&mut self, count: u32, val: u8) -> bool {
        self.block_write_repeat_internal(&val, val as u32, count, DmaTransferWidth::Byte)
    }

    pub fn block_write16_repeat(&mut self, count: u32, val: u16) -> bool {
        self.block_write_repeat_internal(
            &val as *const u16 as *const u8,
            val as u32,
            count,
            DmaTransferWidth::HalfWord,
        )
    }

    // --- DMA Helper Methods ---

    fn enable_dma_tx(&self, enable: bool) {
        if enable {
            self.modify_dmacr(SSPDMACR_TXDMAE, 0);
        } else {
            self.modify_dmacr(0, SSPDMACR_TXDMAE);
        }
    }

    fn enable_dma_rx(&self, enable: bool) {
        if enable {
            self.modify_dmacr(SSPDMACR_RXDMAE, 0);
        } else {
            self.modify_dmacr(0, SSPDMACR_RXDMAE);
        }
    }

    // --- Synchronous DMA Transfers (Deduplicated Core) ---

    fn dma_write_internal(&mut self, src: *const u8, count: u32, width: DmaTransferWidth) -> bool {
        if count == 0 {
            return true;
        }
        self.claim_dma_channels();
        if self.tx_dma.is_none() {
            return false;
        }

        let dr_addr = self.data_register_addr();
        let dreq = self.tx_dreq();
        self.done_sem.try_take();
        self.cs_on();
        self.enable_dma_tx(true);

        let tx = self.tx_dma.as_mut().unwrap();
        tx.attach_completion_semaphore(&self.done_sem);
        tx.configure_tx(src, dr_addr, count, DmaTransferConfig::new(width, dreq));

        let ok = self.done_sem.take_timeout(DMA_TIMEOUT_MS);
        self.wait_for_completion();
        if let Some(t) = self.tx_dma.as_mut() {
            t.end_transfer();
        }
        self.enable_dma_tx(false);
        self.clear_stall_rx();
        self.cs_off();
        ok
    }

    /// Synchronous 8-bit DMA write.
    pub fn dma_write8(&mut self, data: &[u8]) -> bool {
        self.dma_write_internal(data.as_ptr(), data.len() as u32, DmaTransferWidth::Byte)
    }

    /// Synchronous 16-bit DMA write.
    pub fn dma_write16(&mut self, data: &[u16]) -> bool {
        self.dma_write_internal(
            data.as_ptr() as *const u8,
            data.len() as u32,
            DmaTransferWidth::HalfWord,
        )
    }

    fn dma_read_internal(
        &mut self,
        dst: *mut u8,
        dummy_ptr: *const u8,
        count: u32,
        width: DmaTransferWidth,
    ) -> bool {
        if count == 0 {
            return true;
        }
        self.claim_dma_channels();
        if self.tx_dma.is_none() || self.rx_dma.is_none() {
            return false;
        }

        let dr_addr = self.data_register_addr();
        let tx_dreq = self.tx_dreq();
        let rx_dreq = self.rx_dreq();
        self.clear_stall_rx();
        self.done_sem.try_take();
        self.enable_dma_rx(true);
        self.enable_dma_tx(true);

        let rx_ch = self.rx_dma.as_mut().unwrap();
        rx_ch.attach_completion_semaphore(&self.done_sem);
        rx_ch.configure_rx(dr_addr, dst, count, DmaTransferConfig::new(width, rx_dreq));

        let tx = self.tx_dma.as_mut().unwrap();
        tx.configure_tx(dummy_ptr, dr_addr, count, DmaTransferConfig::new(width, tx_dreq).increment(false));

        let ok = self.done_sem.take_timeout(DMA_TIMEOUT_MS);
        self.wait_for_completion();
        if let Some(r) = self.rx_dma.as_mut() {
            r.end_transfer();
        }
        if let Some(t) = self.tx_dma.as_mut() {
            t.end_transfer();
        }
        self.enable_dma_rx(false);
        self.enable_dma_tx(false);
        ok
    }

    /// Synchronous 8-bit DMA read (transmits dummy 0xFF bytes to drive SPI clock).
    pub fn dma_read8(&mut self, rx: &mut [u8]) -> bool {
        let dummy = SPI_DUMMY_BYTE;
        self.dma_read_internal(
            rx.as_mut_ptr(),
            &dummy as *const u8,
            rx.len() as u32,
            DmaTransferWidth::Byte,
        )
    }

    /// Synchronous 16-bit DMA read (transmits dummy 0xFFFF words to drive SPI clock).
    pub fn dma_read16(&mut self, rx: &mut [u16]) -> bool {
        let dummy = SPI_DUMMY_WORD;
        self.dma_read_internal(
            rx.as_mut_ptr() as *mut u8,
            &dummy as *const u16 as *const u8,
            rx.len() as u32,
            DmaTransferWidth::HalfWord,
        )
    }

    fn dma_transfer_internal(
        &mut self,
        src: *const u8,
        dst: *mut u8,
        count: u32,
        width: DmaTransferWidth,
    ) -> bool {
        if count == 0 {
            return true;
        }
        self.claim_dma_channels();
        if self.tx_dma.is_none() || self.rx_dma.is_none() {
            return false;
        }

        let dr_addr = self.data_register_addr();
        let tx_dreq = self.tx_dreq();
        let rx_dreq = self.rx_dreq();
        self.clear_stall_rx();
        self.done_sem.try_take();
        self.enable_dma_rx(true);
        self.enable_dma_tx(true);

        let rx_ch = self.rx_dma.as_mut().unwrap();
        rx_ch.attach_completion_semaphore(&self.done_sem);
        rx_ch.configure_rx(dr_addr, dst, count, DmaTransferConfig::new(width, rx_dreq));

        let tx = self.tx_dma.as_mut().unwrap();
        tx.configure_tx(src, dr_addr, count, DmaTransferConfig::new(width, tx_dreq));

        let ok = self.done_sem.take_timeout(DMA_TIMEOUT_MS);
        self.wait_for_completion();
        if let Some(r) = self.rx_dma.as_mut() {
            r.end_transfer();
        }
        if let Some(t) = self.tx_dma.as_mut() {
            t.end_transfer();
        }
        self.enable_dma_rx(false);
        self.enable_dma_tx(false);
        ok
    }

    /// Synchronous 8-bit read (polling for small sizes, DMA for >= 16 bytes).
    pub fn read8(&mut self, rx: &mut [u8]) -> bool {
        if rx.len() < MIN_DMA_TRANSFER_SIZE || !self.dma_enabled {
            return self.read_poll8(rx);
        }
        self.dma_read8(rx)
    }

    /// Synchronous 8-bit full-duplex transfer (polling for small sizes, DMA for >= 16 bytes).
    pub fn transfer8(&mut self, tx_buf: &[u8], rx_buf: &mut [u8]) -> bool {
        assert_eq!(tx_buf.len(), rx_buf.len());
        if tx_buf.len() < MIN_DMA_TRANSFER_SIZE || !self.dma_enabled {
            return self.transfer_poll8(tx_buf, rx_buf);
        }
        self.dma_transfer8(tx_buf, rx_buf)
    }

    /// Synchronous 8-bit full-duplex transfer.
    pub fn dma_transfer8(&mut self, tx_buf: &[u8], rx_buf: &mut [u8]) -> bool {
        assert_eq!(tx_buf.len(), rx_buf.len());
        self.dma_transfer_internal(
            tx_buf.as_ptr(),
            rx_buf.as_mut_ptr(),
            tx_buf.len() as u32,
            DmaTransferWidth::Byte,
        )
    }

    /// Synchronous 16-bit full-duplex transfer.
    pub fn dma_transfer16(&mut self, tx_buf: &[u16], rx_buf: &mut [u16]) -> bool {
        assert_eq!(tx_buf.len(), rx_buf.len());
        self.dma_transfer_internal(
            tx_buf.as_ptr() as *const u8,
            rx_buf.as_mut_ptr() as *mut u8,
            tx_buf.len() as u32,
            DmaTransferWidth::HalfWord,
        )
    }

    // --- Asynchronous DMA Operations ---

    fn async_write_internal(
        &mut self,
        src: *const u8,
        count: u32,
        width: DmaTransferWidth,
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        if count == 0 {
            return true;
        }
        self.claim_dma_channels();
        if self.tx_dma.is_none() {
            return false;
        }

        self.async_cb = cb;
        self.async_cookie = cookie;
        self.done_sem.try_take();
        let dr_addr = self.data_register_addr();
        let dreq = self.tx_dreq();
        self.cs_on();
        self.enable_dma_tx(true);

        let tx = self.tx_dma.as_mut().unwrap();
        if let Some(cb) = cb {
            tx.attach_callback(cb, cookie);
        } else {
            tx.attach_completion_semaphore(&self.done_sem);
        }

        tx.configure_tx(src, dr_addr, count, DmaTransferConfig::new(width, dreq).increment(!repeat));
        true
    }

    pub fn async_write8(
        &mut self,
        data: &[u8],
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write_internal(
            data.as_ptr(),
            data.len() as u32,
            DmaTransferWidth::Byte,
            cb,
            cookie,
            repeat,
        )
    }

    pub fn async_write16(
        &mut self,
        data: &[u16],
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write_internal(
            data.as_ptr() as *const u8,
            data.len() as u32,
            DmaTransferWidth::HalfWord,
            cb,
            cookie,
            repeat,
        )
    }

    pub fn next_write8(
        &mut self,
        data: &[u8],
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write8(data, cb, cookie, repeat)
    }

    pub fn next_write16(
        &mut self,
        data: &[u16],
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write16(data, cb, cookie, repeat)
    }

    pub fn finish_async_dma(&mut self) -> bool {
        self.done_sem.give();
        true
    }

    pub fn wait_for_async(&mut self) -> bool {
        let ok = self.done_sem.take_timeout(ASYNC_TIMEOUT_MS);
        self.wait_for_completion();
        if let Some(tx) = self.tx_dma.as_mut() {
            tx.end_transfer();
        }
        if let Some(rx) = self.rx_dma.as_mut() {
            rx.end_transfer();
        }
        self.enable_dma_tx(false);
        self.enable_dma_rx(false);
        self.clear_stall_rx();
        self.cs_off();
        ok
    }

    // --- Multi-Chunk (Scatter-Gather) Transfers ---

    fn write_multi_chunk_internal(
        &mut self,
        blocks: &[DmaGatherBlock],
        width: DmaTransferWidth,
    ) -> bool {
        if blocks.is_empty() {
            return true;
        }
        self.claim_dma_channels();
        if self.tx_dma.is_none() || self.ctrl_dma.is_none() {
            return false;
        }

        let dr_addr = self.data_register_addr();
        let dreq = self.tx_dreq();
        self.done_sem.try_take();
        self.cs_on();
        self.enable_dma_tx(true);

        let ctrl = self.ctrl_dma.as_mut().unwrap();
        ctrl.attach_completion_semaphore(&self.done_sem);

        let tx = self.tx_dma.as_mut().unwrap();
        tx.setup_gather(ctrl, blocks, dr_addr, width, dreq);
        tx.trigger_multi_chunk(ctrl);

        let ok = self.done_sem.take_timeout(DMA_TIMEOUT_MS);
        self.wait_for_completion();
        if let Some(t) = self.tx_dma.as_mut() {
            t.end_transfer();
        }
        if let Some(c) = self.ctrl_dma.as_mut() {
            c.end_transfer();
        }
        self.enable_dma_tx(false);
        self.clear_stall_rx();
        self.cs_off();
        ok
    }

    /// Synchronous multi-chunk gather (writes multiple buffer slices in sequence via DMA).
    pub fn write8_multi_chunk(&mut self, chunks: &[&[u8]]) -> bool {
        assert!(chunks.len() <= MAX_CHUNKS, "Too many chunks for static block buffer");
        let mut blocks = [DmaGatherBlock {
            trans_count: 0,
            read_addr: core::ptr::null(),
        }; MAX_CHUNKS];

        for (i, chunk) in chunks.iter().enumerate() {
            blocks[i] = DmaGatherBlock {
                trans_count: chunk.len() as u32,
                read_addr: chunk.as_ptr(),
            };
        }
        self.write_multi_chunk_internal(&blocks[..chunks.len()], DmaTransferWidth::Byte)
    }

    /// Synchronous multi-chunk gather for 16-bit words.
    pub fn write16_multi_chunk(&mut self, chunks: &[&[u16]]) -> bool {
        assert!(chunks.len() <= MAX_CHUNKS, "Too many chunks for static block buffer");
        let mut blocks = [DmaGatherBlock {
            trans_count: 0,
            read_addr: core::ptr::null(),
        }; MAX_CHUNKS];

        for (i, chunk) in chunks.iter().enumerate() {
            blocks[i] = DmaGatherBlock {
                trans_count: chunk.len() as u32,
                read_addr: chunk.as_ptr() as *const u8,
            };
        }
        self.write_multi_chunk_internal(&blocks[..chunks.len()], DmaTransferWidth::HalfWord)
    }

    fn read_multi_chunk_internal(
        &mut self,
        blocks: &[DmaScatterBlock],
        total_count: u32,
        dummy_ptr: *const u8,
        width: DmaTransferWidth,
    ) -> bool {
        if blocks.is_empty() {
            return true;
        }
        self.claim_dma_channels();
        if self.tx_dma.is_none() || self.rx_dma.is_none() || self.ctrl_dma.is_none() {
            return false;
        }

        let dr_addr = self.data_register_addr();
        let tx_dreq = self.tx_dreq();
        let rx_dreq = self.rx_dreq();
        self.clear_stall_rx();
        self.done_sem.try_take();
        self.cs_on();
        self.enable_dma_rx(true);
        self.enable_dma_tx(true);

        let ctrl = self.ctrl_dma.as_mut().unwrap();
        ctrl.attach_completion_semaphore(&self.done_sem);

        let rx = self.rx_dma.as_mut().unwrap();
        rx.setup_scatter(ctrl, blocks, dr_addr, width, rx_dreq);

        let tx = self.tx_dma.as_mut().unwrap();
        tx.configure_tx(
            dummy_ptr,
            dr_addr,
            total_count,
            DmaTransferConfig::new(width, tx_dreq).increment(false),
        );

        rx.trigger_multi_chunk(ctrl);

        let ok = self.done_sem.take_timeout(DMA_TIMEOUT_MS);
        self.wait_for_completion();
        if let Some(r) = self.rx_dma.as_mut() {
            r.end_transfer();
        }
        if let Some(t) = self.tx_dma.as_mut() {
            t.end_transfer();
        }
        if let Some(c) = self.ctrl_dma.as_mut() {
            c.end_transfer();
        }
        self.enable_dma_rx(false);
        self.enable_dma_tx(false);
        self.cs_off();
        ok
    }

    /// Synchronous multi-chunk scatter (receives from SPI into multiple buffer slices in sequence).
    pub fn read8_multi_chunk(&mut self, chunks: &mut [&mut [u8]]) -> bool {
        assert!(chunks.len() <= MAX_CHUNKS, "Too many chunks for static block buffer");
        let mut blocks = [DmaScatterBlock {
            write_addr: core::ptr::null_mut(),
            trans_count: 0,
        }; MAX_CHUNKS];

        let mut total_count = 0u32;
        for (i, chunk) in chunks.iter_mut().enumerate() {
            blocks[i] = DmaScatterBlock {
                write_addr: chunk.as_mut_ptr(),
                trans_count: chunk.len() as u32,
            };
            total_count += chunk.len() as u32;
        }

        let dummy = SPI_DUMMY_BYTE;
        self.read_multi_chunk_internal(
            &blocks[..chunks.len()],
            total_count,
            &dummy as *const u8,
            DmaTransferWidth::Byte,
        )
    }

    /// Synchronous multi-chunk scatter for 16-bit words.
    pub fn read16_multi_chunk(&mut self, chunks: &mut [&mut [u16]]) -> bool {
        assert!(chunks.len() <= MAX_CHUNKS, "Too many chunks for static block buffer");
        let mut blocks = [DmaScatterBlock {
            write_addr: core::ptr::null_mut(),
            trans_count: 0,
        }; MAX_CHUNKS];

        let mut total_count = 0u32;
        for (i, chunk) in chunks.iter_mut().enumerate() {
            blocks[i] = DmaScatterBlock {
                write_addr: chunk.as_mut_ptr() as *mut u8,
                trans_count: chunk.len() as u32,
            };
            total_count += chunk.len() as u32;
        }

        let dummy = SPI_DUMMY_WORD;
        self.read_multi_chunk_internal(
            &blocks[..chunks.len()],
            total_count,
            &dummy as *const u16 as *const u8,
            DmaTransferWidth::HalfWord,
        )
    }

    fn async_write_multi_chunk_internal(
        &mut self,
        blocks: &[DmaGatherBlock],
        width: DmaTransferWidth,
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
    ) -> bool {
        if blocks.is_empty() {
            return true;
        }
        self.claim_dma_channels();
        if self.tx_dma.is_none() || self.ctrl_dma.is_none() {
            return false;
        }

        self.async_cb = cb;
        self.async_cookie = cookie;
        self.done_sem.try_take();

        let dr_addr = self.data_register_addr();
        let dreq = self.tx_dreq();
        self.cs_on();
        self.enable_dma_tx(true);

        let ctrl = self.ctrl_dma.as_mut().unwrap();
        if let Some(cb) = cb {
            ctrl.attach_callback(cb, cookie);
        } else {
            ctrl.attach_completion_semaphore(&self.done_sem);
        }

        let tx = self.tx_dma.as_mut().unwrap();
        tx.setup_gather(ctrl, blocks, dr_addr, width, dreq);
        tx.trigger_multi_chunk(ctrl);
        true
    }

    /// Asynchronous multi-chunk gather write.
    pub fn async_write8_multi_chunk(
        &mut self,
        chunks: &[&[u8]],
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
    ) -> bool {
        assert!(chunks.len() <= MAX_CHUNKS, "Too many chunks for static block buffer");
        let mut blocks = [DmaGatherBlock {
            trans_count: 0,
            read_addr: core::ptr::null(),
        }; MAX_CHUNKS];

        for (i, chunk) in chunks.iter().enumerate() {
            blocks[i] = DmaGatherBlock {
                trans_count: chunk.len() as u32,
                read_addr: chunk.as_ptr(),
            };
        }

        self.async_write_multi_chunk_internal(
            &blocks[..chunks.len()],
            DmaTransferWidth::Byte,
            cb,
            cookie,
        )
    }

    /// Asynchronous multi-chunk gather write for 16-bit words.
    pub fn async_write16_multi_chunk(
        &mut self,
        chunks: &[&[u16]],
        cb: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
    ) -> bool {
        assert!(chunks.len() <= MAX_CHUNKS, "Too many chunks for static block buffer");
        let mut blocks = [DmaGatherBlock {
            trans_count: 0,
            read_addr: core::ptr::null(),
        }; MAX_CHUNKS];

        for (i, chunk) in chunks.iter().enumerate() {
            blocks[i] = DmaGatherBlock {
                trans_count: chunk.len() as u32,
                read_addr: chunk.as_ptr() as *const u8,
            };
        }

        self.async_write_multi_chunk_internal(
            &blocks[..chunks.len()],
            DmaTransferWidth::HalfWord,
            cb,
            cookie,
        )
    }
}

impl Drop for Spi {
    fn drop(&mut self) {
        self.end();
        self.cs_off();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// MSB-first is the only order the PL022 can do, and it is accepted silently.
    #[test]
    fn msb_first_is_accepted() {
        check_bit_order(BitOrder::MsbFirst);
    }

    /// LSB-first is an unsupported configuration: it must fail loudly instead of
    /// sending the caller's bytes out backwards.
    #[test]
    #[should_panic(expected = "LSB-first is not supported")]
    fn lsb_first_panics() {
        check_bit_order(BitOrder::LsbFirst);
    }

    /// The two frame sizes the driver moves (`DSS` 7 and 15).
    #[test]
    fn eight_and_sixteen_bit_frames_are_accepted() {
        check_data_size(SPI_DATA_SIZE_8);
        check_data_size(SPI_DATA_SIZE_16);
    }

    /// Anything else is an unsupported configuration and must not be coerced to 8.
    #[test]
    #[should_panic(expected = "must be 8 or 16 bits")]
    fn another_frame_size_panics() {
        check_data_size(7);
    }
}


// ---------------------------------------------------------------------------
//  The MCU-agnostic SPI contract
// ---------------------------------------------------------------------------

/// One handler slot per SPI instance.
static HANDLERS: HandlerTable<dyn spi::SpiHandler, SPI_MAX_INSTANCES> = HandlerTable::new();

impl spi::Spi for Spi {
    fn configure(&mut self, config: &spi::SpiConfig) -> rs_mcu_traits::Result<()> {
        self.set_speed(config.speed_hz);
        self.set_data_mode(match config.mode {
            spi::SpiMode::Mode0 => SpiMode::Mode0,
            spi::SpiMode::Mode1 => SpiMode::Mode1,
            spi::SpiMode::Mode2 => SpiMode::Mode2,
            spi::SpiMode::Mode3 => SpiMode::Mode3,
        });
        self.set_bit_order(match config.bit_order {
            spi::BitOrder::MsbFirst => BitOrder::MsbFirst,
            spi::BitOrder::LsbFirst => BitOrder::LsbFirst,
        });
        self.begin(config.data_size as u32);
        Ok(())
    }

    fn set_speed(&mut self, speed_hz: u32) -> rs_mcu_traits::Result<()> {
        Spi::set_speed(self, speed_hz);
        Ok(())
    }

    /// One byte exchange, polled. The block and DMA paths of this driver are
    /// reached through its own `transfer_poll8` / `dma_transfer8` methods.
    fn transfer(&mut self, byte: u8) -> u8 {
        let mut rx = [0u8; 1];
        if self.transfer_poll8(&[byte], &mut rx) {
            rx[0]
        } else {
            0
        }
    }

    // ---- blocking blocks: the driver's DMA paths -----------------------------

    fn write_slice(&mut self, data: &[u8]) -> rs_mcu_traits::Result<()> {
        if self.block_write8(data) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    fn write_slice16(&mut self, data: &[u16]) -> rs_mcu_traits::Result<()> {
        if self.block_write16(data) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    fn read_slice(&mut self, buffer: &mut [u8]) -> rs_mcu_traits::Result<()> {
        if self.read_poll8(buffer) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    fn fill(&mut self, count: u32, byte: u8) -> rs_mcu_traits::Result<()> {
        if self.block_write8_repeat(count, byte) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    fn fill16(&mut self, count: u32, word: u16) -> rs_mcu_traits::Result<()> {
        if self.block_write16_repeat(count, word) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    fn transfer_in_place(&mut self, tx: &[u8], rx: &mut [u8]) -> rs_mcu_traits::Result<()> {
        if tx.len() != rx.len() {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        }
        if self.transfer_poll8(tx, rx) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    // ---- asynchronous: DMA, completion in the interrupt ----------------------

    fn start_write(&mut self, data: &[u8]) -> rs_mcu_traits::Result<()> {
        let callback = HANDLERS
            .get(self.instance as usize)
            .map(|_| tx_complete_trampoline as unsafe extern "C" fn(*mut c_void));
        let cookie = self.instance as usize as *mut core::ffi::c_void;
        if self.async_write8(data, callback, cookie, false) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    fn start_write16(&mut self, data: &[u16]) -> rs_mcu_traits::Result<()> {
        let callback = HANDLERS
            .get(self.instance as usize)
            .map(|_| tx_complete_trampoline as unsafe extern "C" fn(*mut c_void));
        let cookie = self.instance as usize as *mut core::ffi::c_void;
        if self.async_write16(data, callback, cookie, false) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Io)
        }
    }

    fn is_busy(&self) -> bool {
        // The TX DMA channel's busy flag, not a poll: `is_busy` must not block.
        self.tx_dma.as_ref().is_some_and(DmaChannel::is_busy)
    }

    fn wait_idle(&mut self) -> rs_mcu_traits::Result<()> {
        if self.wait_for_completion() {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Timeout)
        }
    }

    fn set_handler(&mut self, handler: Option<&'static dyn spi::SpiHandler>) {
        HANDLERS.set(self.instance as usize, handler);
    }
}

/// Second step of the async completion: the DMA callback is a raw `extern "C"`
/// function, so the typed handler is looked up by instance (the cookie).
unsafe extern "C" fn tx_complete_trampoline(cookie: *mut core::ffi::c_void) {
    // SAFETY: the handler was installed before the transfer was started, and the
    // interrupt is the only reader.
    if let Some(handler) = unsafe { HANDLERS.isr_get(cookie as usize) } {
        handler.on_tx_complete();
    }
}

