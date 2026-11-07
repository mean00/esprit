//! SPI master driver for the Bluepill family (GD32F303 / CH32V307).
//!
//! This is the Rust counterpart of `lnSPI_bp` (`lnSPI_bp.cpp` + `lnSPI_bp_dma.cpp`):
//!
//! * [`Spi::begin`] / [`Spi::end`] bracket a session (frame size, CS on/off),
//! * [`Spi::write8`], [`Spi::write16`] and [`Spi::transfer`] poke the data
//!   register and poll the status register,
//! * [`Spi::block_write8`] / [`Spi::async_write8`] (and the 16 bit variants)
//!   move the data with the instance's TX DMA channel and use its **transfer
//!   complete interrupt** as the completion signal.
//!
//! No SPI interrupt is ever enabled: the `SPI*_IRQHandler` vectors are
//! `VEC(unsupported)` on GD32/CH32 and the C++ driver does not arm an SPI
//! interrupt source either (`_irq` in `lnSPI_bp` is only stored, never used).
//! Everything asynchronous is driven by the DMA channel.

#![no_std]
// The names below are the C ABI names (`spiBitOrder`, `spiDataMode_SPI_MODE0`,
// `lnSpiCallback`, `lnspi_create(instance, pinCs)`, …) and are kept as they are
// on purpose, so the driver can be read side by side with `lnSPI_c.h`.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub mod registers;

use core::ffi::c_void;
use core::ptr::{read_volatile, write_volatile};

use registers::*;
use rs_dma_bluepill::{DmaChannel, DmaEngine};
use rs_esprit::{BinarySemaphore, SpiBitOrder, SpiConfig, SpiMode, SpiRxHandler, SpiTxHandler};
use rs_gpio_bluepill::{Mode, Pin};
use rs_rcu_bluepill::{enable, get_clock, Peripheral};

pub mod shim;
pub use shim::*;

pub type spiBitOrder = u32;
pub const spiBitOrder_SPI_LSBFIRST: spiBitOrder = 0;
pub const spiBitOrder_SPI_MSBFIRST: spiBitOrder = 1;

pub type spiDataMode = u32;
pub const spiDataMode_SPI_MODE0: spiDataMode = 0;
pub const spiDataMode_SPI_MODE1: spiDataMode = 1;
pub const spiDataMode_SPI_MODE2: spiDataMode = 2;
pub const spiDataMode_SPI_MODE3: spiDataMode = 3;

/// Timeout of a blocking DMA write, in milliseconds (`LN_SPI_TIMEOUT`).
const DMA_TIMEOUT_MS: u32 = 100;
/// Timeout of [`Spi::wait_async`], in milliseconds (`LN_SPI_LONG_TIMEOUT`).
const ASYNC_TIMEOUT_MS: u32 = 1_000;
/// Speed of the SPI pins, in MHz (the alternate function is push-pull).
const PIN_SPEED_MHZ: u32 = 50;
/// Bit rate used until [`Spi::set_speed`] is called, as in `lnSPISettings`.
const DEFAULT_SPEED_HZ: u32 = 1_000_000;
/// Highest GPIO number (`PG15`); used to validate a chip-select pin number.
const MAX_PIN_NUMBER: u32 = Pin::PG15 as u32;
/// Upper bound on a status-register poll, so a disabled or mis-clocked
/// peripheral cannot spin here forever.  The DMA paths time out through the
/// completion semaphore instead.
const POLL_LIMIT: u32 = 1_000_000;

/// Half duplex / full duplex selection (the `lnDuplex` / `lnTxOnly` half of
/// `lnSpiDirection` in lnSPI_priv.h; the receive-only mode is only used by the
/// C++ `read1wire()`, which this driver does not expose).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    /// MOSI and MISO both active.
    Duplex,
    /// Output only: the receive path is disconnected, which is also what keeps
    /// a DMA-fed transfer from raising overrun errors.
    TxOnly,
}

/// What one SPI instance needs: its clock, its DMA channel and its pins.
/// Mirrors `spiDescriptor[]` in `lnSPI_bp.cpp`.
struct SpiDescriptor {
    periph: Peripheral,
    dma_engine: DmaEngine,
    dma_tx_channel: usize,
    mosi: Pin,
    miso: Pin,
    clk: Pin,
}

const SPI_DESCRIPTORS: [SpiDescriptor; SPI_MAX_INSTANCES] = [
    // SPI0: MOSI PA7 / MISO PA6 / CLK PA5, DMA0 channel 2.
    SpiDescriptor {
        periph: Peripheral::Spi0,
        dma_engine: DmaEngine::Dma0,
        dma_tx_channel: 2,
        mosi: Pin::PA7,
        miso: Pin::PA6,
        clk: Pin::PA5,
    },
    // SPI1: MOSI PB15 / MISO PB14 / CLK PB13, DMA0 channel 4.
    SpiDescriptor {
        periph: Peripheral::Spi1,
        dma_engine: DmaEngine::Dma0,
        dma_tx_channel: 4,
        mosi: Pin::PB15,
        miso: Pin::PB14,
        clk: Pin::PB13,
    },
    // SPI2: MOSI PB5 / MISO PB4 / CLK PB3, DMA1 channel 1.
    SpiDescriptor {
        periph: Peripheral::Spi2,
        dma_engine: DmaEngine::Dma1,
        dma_tx_channel: 1,
        mosi: Pin::PB5,
        miso: Pin::PB4,
        clk: Pin::PB3,
    },
];

/// The chip-select pin, or `None` when the caller drives CS itself (`pin_cs`
/// is `-1`) or the number is not a GPIO.
fn cs_pin(pin_cs: i32) -> Option<Pin> {
    if pin_cs >= 0 && (pin_cs as u32) <= MAX_PIN_NUMBER {
        Some(Pin::from(pin_cs as u32))
    } else {
        None
    }
}

/// Poll `condition` until it returns `false`, giving up after [`POLL_LIMIT`]
/// rounds.  Returns `false` if the condition never cleared.
fn spin_while(condition: impl Fn() -> bool) -> bool {
    let mut budget = POLL_LIMIT;
    while condition() {
        if budget == 0 {
            return false;
        }
        budget -= 1;
        core::hint::spin_loop();
    }
    true
}

/// `BR` field value for `speed_hz`: the SPI clock is `PCLK / 2^(BR+1)`, so pick
/// the smallest power of two that does not exceed the requested rate
/// (`lnSPI_bp::setup()` rounds the same way).
fn baud_control_bits(pclk: u32, speed_hz: u32) -> u32 {
    let speed = if speed_hz == 0 { DEFAULT_SPEED_HZ } else { speed_hz };
    let divisor =
        ((pclk + speed / 2) / speed).clamp(SPI_BAUDRATE_MIN_DIVISOR, SPI_BAUDRATE_MAX_DIVISOR);
    let mut bits = 0;
    let mut current = SPI_BAUDRATE_MIN_DIVISOR;
    while divisor > current && bits < SPI_BAUDRATE_MAX_BITS {
        current *= 2;
        bits += 1;
    }
    bits
}

/// DMA completion of an `async_write`: runs in interrupt context.
unsafe extern "C" fn spi_dma_async_done(_is_half: bool, cookie: *mut c_void) {
    // SAFETY: the cookie is the `Spi` that armed the channel (see `async_write`).
    let spi = unsafe { &mut *(cookie as *mut Spi) };
    spi.async_done();
}

/// An SPI master on one of the three instances.
pub struct Spi {
    pub(crate) instance: u32,
    /// Chip-select GPIO, `-1` when the caller drives CS itself.
    pub(crate) pin_cs: i32,
    /// TX channel, claimed for the whole life of the object.  Its interrupt is
    /// the completion signal of the block / async transfers.
    dma: DmaChannel,
    /// Given by the DMA interrupt, taken by `block_write*` / `wait_async`.
    done: BinarySemaphore,
    // The settings `setup()` programs.  Kept here (and not only in the
    // registers) so `set_speed()` / `set_data_mode()` can be used between
    // transfers, like `_internalSettings` in the C++ driver.
    speed_hz: u32,
    data_mode: u32,
    bit_order: u32,
    data_size: u32,
    /// `false` makes the block writes poll instead of using the DMA.
    use_dma: bool,
    /// Completion of an `async_write`, called from the DMA interrupt.
    callback: lnSpiCallback,
    callback_cookie: *mut c_void,
    /// `rs_esprit::Spi` handlers.
    rx_handler: Option<&'static dyn SpiRxHandler>,
    tx_handler: Option<&'static dyn SpiTxHandler>,
}

impl Spi {
    /// Create the driver for `instance` (`0..2`) and set the pins up.
    ///
    /// `pin_cs` is the chip-select GPIO (`-1` when the caller drives CS), as in
    /// `lnSPI::create(instance, pinCs)`.  The TX DMA channel is claimed here, so
    /// creating two objects for the same instance, or one on a channel another
    /// peripheral already uses, panics (`DmaChannel::new`).
    pub fn new(instance: u32, pin_cs: i32) -> Self {
        assert!(
            (instance as usize) < SPI_MAX_INSTANCES,
            "SPI instance {} out of range (0..{})",
            instance,
            SPI_MAX_INSTANCES - 1
        );
        let desc = &SPI_DESCRIPTORS[instance as usize];

        // The clock of the peripheral must be on before its registers are used.
        enable(desc.periph);

        // MOSI and CLK push-pull alternate function, MISO floating, like
        // `lnSPI_bp::lnSPI_bp()`.
        desc.mosi.set_mode(Mode::AlternatePushPull, PIN_SPEED_MHZ);
        desc.miso.set_mode(Mode::Floating, PIN_SPEED_MHZ);
        desc.clk.set_mode(Mode::AlternatePushPull, PIN_SPEED_MHZ);

        let mut me = Self {
            instance,
            pin_cs,
            dma: DmaChannel::new(desc.dma_engine, desc.dma_tx_channel),
            done: BinarySemaphore::new(),
            speed_hz: DEFAULT_SPEED_HZ,
            data_mode: spiDataMode_SPI_MODE0,
            bit_order: spiBitOrder_SPI_MSBFIRST,
            data_size: SPI_DATA_SIZE_8,
            use_dma: true,
            callback: None,
            callback_cookie: core::ptr::null_mut(),
            rx_handler: None,
            tx_handler: None,
        };
        me.set_ssel(pin_cs); // output, idle high (CS is active low)
        me.set_disabled();
        me
    }

    #[inline(always)]
    pub fn instance(&self) -> u32 {
        self.instance
    }

    // ---- register level helpers ------------------------------------------

    #[inline(always)]
    fn descriptor(&self) -> &'static SpiDescriptor {
        &SPI_DESCRIPTORS[self.instance as usize]
    }

    #[inline(always)]
    fn regs(&self) -> *mut SpiRegisters {
        SpiRegisters::ptr(self.instance)
    }

    #[inline(always)]
    fn cr1(&self) -> u32 {
        unsafe { read_volatile(&mut (*self.regs()).cr1) }
    }

    #[inline(always)]
    fn set_cr1(&self, value: u32) {
        unsafe { write_volatile(&mut (*self.regs()).cr1, value) }
    }

    #[inline(always)]
    fn cr2(&self) -> u32 {
        unsafe { read_volatile(&mut (*self.regs()).cr2) }
    }

    #[inline(always)]
    fn set_cr2(&self, value: u32) {
        unsafe { write_volatile(&mut (*self.regs()).cr2, value) }
    }

    #[inline(always)]
    fn status(&self) -> u32 {
        unsafe { read_volatile(&mut (*self.regs()).sr) }
    }

    #[inline(always)]
    fn set_status(&self, value: u32) {
        unsafe { write_volatile(&mut (*self.regs()).sr, value) }
    }

    #[inline(always)]
    fn write_data(&self, value: u32) {
        unsafe { write_volatile(&mut (*self.regs()).dr, value) }
    }

    #[inline(always)]
    fn read_data(&self) -> u32 {
        unsafe { read_volatile(&mut (*self.regs()).dr) }
    }

    /// Address of the data register, i.e. the peripheral side of the DMA.
    #[inline(always)]
    fn data_register_addr(&self) -> u32 {
        unsafe { core::ptr::addr_of_mut!((*self.regs()).dr) as u32 }
    }

    /// The shifter is busy with the last frame.
    #[inline(always)]
    fn busy(&self) -> bool {
        self.status() & SPI_STAT_TRANS != 0
    }

    /// The transmit buffer still holds a frame that was not shifted out yet.
    #[inline(always)]
    fn tx_busy(&self) -> bool {
        self.status() & SPI_STAT_TBE == 0
    }

    #[inline(always)]
    fn rx_ready(&self) -> bool {
        self.status() & SPI_STAT_RBNE != 0
    }

    /// Mode fault: the configuration is inconsistent (NSS pulled low in master
    /// mode), so any transfer would be lost.
    #[inline(always)]
    fn config_error(&self) -> bool {
        self.status() & SPI_STAT_CONFERR != 0
    }

    fn set_disabled(&mut self) {
        let cr1 = self.cr1();
        self.set_cr1(cr1 & !SPI_CR1_SPE);
    }

    fn set_enabled(&mut self) {
        let cr1 = self.cr1();
        self.set_cr1(cr1 | SPI_CR1_SPE);
    }

    fn cs_on(&mut self) {
        if let Some(pin) = cs_pin(self.pin_cs) {
            pin.write(false); // selected: CS is active low
        }
    }

    fn cs_off(&mut self) {
        if let Some(pin) = cs_pin(self.pin_cs) {
            pin.write(true);
        }
    }

    /// `updateMode()`: full duplex or output only.
    fn update_direction(&mut self, direction: Direction) {
        let mut cr1 = self.cr1() & !(SPI_CR1_BIDIOE | SPI_CR1_BIDIMODE | SPI_CR1_RXONLY);
        match direction {
            Direction::Duplex => {}
            Direction::TxOnly => cr1 |= SPI_CR1_BIDIOE | SPI_CR1_BIDIMODE,
        }
        self.set_cr1(cr1);
    }

    /// `updateDataSize()` for an explicit frame size.
    fn update_data_size_for(&mut self, frame_bits: u32) {
        let mut cr1 = self.cr1();
        match frame_bits {
            SPI_DATA_SIZE_8 => cr1 &= !SPI_CR1_DFF,
            SPI_DATA_SIZE_16 => cr1 |= SPI_CR1_DFF,
            other => panic!("unsupported SPI frame size {}", other),
        }
        self.set_cr1(cr1);
    }

    /// `updateDataSize()` for the configured frame size.
    fn update_data_size(&mut self) {
        self.update_data_size_for(self.data_size);
    }

    /// `updateDmaTX()`: DMA requests on the transmit side.
    fn update_dma_tx(&mut self, on: bool) {
        let mut cr2 = self.cr2();
        if on {
            cr2 |= SPI_CR2_TXDMAEN;
        } else {
            cr2 &= !SPI_CR2_TXDMAEN;
        }
        self.set_cr2(cr2);
    }

    /// Program the peripheral with the current settings (`lnSPI_bp::setup()`).
    /// Leaves it disabled: the caller enables it when a session starts.
    fn setup(&mut self) {
        self.set_disabled();
        // Clear the sticky error flags and everything below the reserved bits:
        // the driver programs those itself.
        self.set_status(self.status() & SPI_STAT_KEEP_MASK);
        let mut cr1 = self.cr1() & SPI_CTL0_KEEP_MASK;
        let cr2 = self.cr2() & SPI_CTL1_KEEP_MASK;

        // Master mode, NSS driven by software and held high: there is no way to
        // disconnect the NSS management (see lnSPI_bp.cpp).
        cr1 |= SPI_CR1_MSTR | SPI_CR1_SSI | SPI_CR1_SSM;

        if self.bit_order == spiBitOrder_SPI_LSBFIRST {
            cr1 |= SPI_CR1_LSBFIRST;
        }
        cr1 &= !SPI_CR1_MODE_MASK;
        match self.data_mode {
            spiDataMode_SPI_MODE0 => {}
            spiDataMode_SPI_MODE1 => cr1 |= SPI_CR1_CPHA,
            spiDataMode_SPI_MODE2 => cr1 |= SPI_CR1_CPOL,
            spiDataMode_SPI_MODE3 => cr1 |= SPI_CR1_CPHA | SPI_CR1_CPOL,
            other => panic!("invalid SPI data mode {}", other),
        }

        let pclk = get_clock(self.descriptor().periph);
        cr1 &= !SPI_CR1_BR_MASK;
        cr1 |= baud_control_bits(pclk, self.speed_hz) << SPI_CR1_BR_SHIFT;

        self.set_cr1(cr1);
        self.set_cr2(cr2);
        self.update_data_size();
        // Output only by default; this also keeps the receive path from raising
        // overrun errors while the DMA feeds the data register.
        self.update_direction(Direction::TxOnly);
    }

    /// Reprogram after a settings change, keeping the peripheral enabled if it
    /// was (`setup()` leaves it disabled, which would be wrong mid-session).
    fn reprogram(&mut self) {
        let was_enabled = self.cr1() & SPI_CR1_SPE != 0;
        self.setup();
        if was_enabled {
            self.set_enabled();
        }
    }

    // ---- public API ------------------------------------------------------

    /// Start a session: program the settings, set the frame size, enable the
    /// peripheral and drive CS low (`lnSPI_bp::begin`).
    pub fn begin(&mut self, data_size: u32) {
        self.data_size = data_size;
        self.setup();
        self.set_enabled();
        self.cs_on();
    }

    /// End a session: release CS and disable the peripheral.
    pub fn end(&mut self) {
        self.cs_off();
        self.set_disabled();
    }

    /// Set the bit rate, in b/s.  Rounded down to the next `PCLK / 2^n`.
    pub fn set_speed(&mut self, speed: u32) {
        self.speed_hz = speed;
        self.reprogram();
    }

    pub fn set_data_mode(&mut self, mode: u32) {
        self.data_mode = mode;
        self.reprogram();
    }

    pub fn set_bit_order(&mut self, order: u32) {
        self.bit_order = order;
        self.reprogram();
    }

    /// Set the chip-select pin (`-1` = the caller drives CS).  The pin is
    /// configured as an output and left idle high, so `set_ssel()` also works
    /// after the object was created without a CS.
    pub fn set_ssel(&mut self, ssel: i32) {
        self.pin_cs = ssel;
        if let Some(pin) = cs_pin(ssel) {
            pin.set_mode(Mode::Output, PIN_SPEED_MHZ);
            pin.write(true); // idle high: CS is active low
        }
    }

    /// Use the DMA for the block writes (the default), or poll instead.
    ///
    /// The polling path does not touch CS: it is meant to be used inside a
    /// `begin` / `end` session, like `lnSPI_bp::writesInternal()`.
    pub fn set_dma_mode(&mut self, enable: bool) {
        self.use_dma = enable;
    }

    /// Wait until the last frame left both the transmit buffer and the shifter,
    /// i.e. the last clock edge has been produced
    /// (`lnSPI_bp::waitForCompletion`).  Returns `false` if it timed out.
    pub fn wait_for_completion(&self) -> bool {
        let buffer_empty = spin_while(|| self.tx_busy());
        let shifter_idle = spin_while(|| self.busy());
        buffer_empty && shifter_idle
    }

    /// Write one frame (8 or 16 bits, as configured) and wait for it to be sent.
    fn write_internal(&mut self, value: u32) -> bool {
        if self.config_error() {
            return false;
        }
        if !spin_while(|| self.busy()) {
            return false;
        }
        self.write_data(value);
        self.wait_for_completion()
    }

    /// Write one byte.  The frame size is the configured one, so this needs
    /// [`Spi::begin`]`(8)` (or an 8 bit `SpiConfig`).
    pub fn write8(&mut self, data: u8) -> bool {
        self.write_internal(data as u32)
    }

    /// Write one 16 bit word.  The frame size is the configured one, so this
    /// needs [`Spi::begin`]`(16)`.
    pub fn write16(&mut self, data: u16) -> bool {
        self.write_internal(data as u32)
    }

    /// Full duplex byte transfer (`lnSPI_bp::transfer`).
    ///
    /// The peripheral is switched to full duplex first: the DMA paths leave it
    /// in output-only mode, where the received bytes are dropped and the read
    /// side would spin forever.  CS is left alone so the caller can keep a
    /// session open around the transfer.
    pub fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> bool {
        assert_eq!(tx.len(), rx.len(), "SPI transfer needs equally sized slices");
        assert_eq!(
            self.data_size, SPI_DATA_SIZE_8,
            "transfer() is byte oriented, use begin(8)"
        );
        self.update_direction(Direction::Duplex);
        for (out, input) in tx.iter().zip(rx.iter_mut()) {
            if !spin_while(|| self.tx_busy()) {
                return false;
            }
            self.write_data(*out as u32);
            if !spin_while(|| !self.rx_ready()) {
                return false;
            }
            *input = (self.read_data() & SPI_DR_DATA_8BIT_MASK) as u8;
        }
        self.wait_for_completion()
    }

    /// Send one byte and return the byte received at the same time.
    pub fn transfer8(&mut self, val: u8) -> u8 {
        let tx = [val];
        let mut rx = [0u8];
        if self.transfer(&tx, &mut rx) {
            rx[0]
        } else {
            0
        }
    }

    // ---- block writes ----------------------------------------------------

    /// Write `data` with the DMA and block until it is sent (`blockWrite8`).
    pub fn block_write8(&mut self, data: &[u8]) -> bool {
        self.block_write(SPI_DATA_SIZE_8, data.len() as u32, data.as_ptr(), false)
    }

    /// Write `data` with the DMA, 16 bit frames (`blockWrite16`).
    pub fn block_write16(&mut self, data: &[u16]) -> bool {
        self.block_write(
            SPI_DATA_SIZE_16,
            data.len() as u32,
            data.as_ptr() as *const u8,
            false,
        )
    }

    /// Write `count` copies of `value` (`blockWrite8Repeat`): a single DMA
    /// transfer that re-reads the same source byte.
    pub fn block_write8_repeat(&mut self, count: u32, value: u8) -> bool {
        self.block_write(SPI_DATA_SIZE_8, count, &value as *const u8, true)
    }

    /// Write `count` copies of `value`, 16 bit frames (`blockWrite16Repeat`).
    pub fn block_write16_repeat(&mut self, count: u32, value: u16) -> bool {
        self.block_write(
            SPI_DATA_SIZE_16,
            count,
            &value as *const u16 as *const u8,
            true,
        )
    }

    fn block_write(&mut self, frame_bits: u32, count: u32, source: *const u8, repeat: bool) -> bool {
        if self.use_dma {
            self.dma_write(frame_bits, count, source, repeat)
        } else {
            self.poll_write(frame_bits, count, source, repeat)
        }
    }

    /// DMA write, completion through the channel interrupt (`dmaWriteInternal`
    /// in `lnSPI_bp_dma.cpp`).
    fn dma_write(&mut self, frame_bits: u32, count: u32, source: *const u8, repeat: bool) -> bool {
        self.set_disabled();
        if count == 0 {
            return true;
        }
        // Clears the error flags a previous transfer may have left behind.
        self.update_direction(Direction::TxOnly);

        // 1- DMA: memory -> data register.
        self.done.try_take();
        self.dma.attach_completion_semaphore(&self.done);
        let wide = frame_bits == SPI_DATA_SIZE_16;
        if repeat {
            self.dma
                .begin_tx_transfer_const(self.data_register_addr(), source as u32, count, wide);
        } else {
            self.dma
                .begin_tx_transfer(self.data_register_addr(), source as u32, count, wide, wide);
        }

        // 2- SPI: DMA requests on the transmit side.
        self.update_data_size_for(frame_bits);
        self.update_dma_tx(true);

        // 3- Go.
        if self.config_error() {
            self.dma.end_transfer();
            self.update_dma_tx(false);
            return false;
        }
        self.cs_on();
        self.set_enabled();
        let completed = self.done.take_timeout(DMA_TIMEOUT_MS);
        if completed {
            self.wait_for_completion();
        }
        self.dma.end_transfer();
        self.cs_off();
        self.update_dma_tx(false);
        // Back to the configured frame size for the next transfer.
        self.update_data_size();
        completed
    }

    /// Polling write, used when the DMA is disabled (`writesInternal` in
    /// `lnSPI_bp.cpp`).
    fn poll_write(&mut self, frame_bits: u32, count: u32, source: *const u8, repeat: bool) -> bool {
        self.update_data_size_for(frame_bits);
        let stride = if frame_bits == SPI_DATA_SIZE_16 { 2 } else { 1 };
        for index in 0..count as usize {
            let offset = if repeat { 0 } else { index * stride };
            // SAFETY: the caller guarantees `count` frames at `source`.
            let value = unsafe {
                if stride == 2 {
                    read_volatile(source.add(offset) as *const u16) as u32
                } else {
                    read_volatile(source.add(offset)) as u32
                }
            };
            if !spin_while(|| self.busy()) {
                return false;
            }
            self.write_data(value);
        }
        let done = self.wait_for_completion();
        self.update_data_size();
        done
    }

    // ---- async writes ----------------------------------------------------

    /// Start a DMA write and return at once; `callback` runs in the DMA
    /// interrupt when the transfer is over (`asyncWrite8`).
    ///
    /// `data` must stay alive and untouched until the callback has run, and the
    /// `Spi` itself must not move until then: the DMA interrupt reaches it
    /// through a pointer to the object.
    pub fn async_write8(
        &mut self,
        data: &[u8],
        callback: lnSpiCallback,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write(
            SPI_DATA_SIZE_8,
            data.len() as u32,
            data.as_ptr(),
            callback,
            cookie,
            repeat,
        )
    }

    /// Start an async DMA write with 16 bit frames (`asyncWrite16`).
    pub fn async_write16(
        &mut self,
        data: &[u16],
        callback: lnSpiCallback,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write(
            SPI_DATA_SIZE_16,
            data.len() as u32,
            data.as_ptr() as *const u8,
            callback,
            cookie,
            repeat,
        )
    }

    /// Queue another async 8 bit write.  The channel is fully re-armed on every
    /// transfer here, so this is the same as [`Spi::async_write8`]: the C++
    /// split exists because `lnDMA::beginTransfer()` does the one-off setup.
    pub fn next_write8(
        &mut self,
        data: &[u8],
        callback: lnSpiCallback,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write8(data, callback, cookie, repeat)
    }

    /// Queue another async 16 bit write.
    pub fn next_write16(
        &mut self,
        data: &[u16],
        callback: lnSpiCallback,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.async_write16(data, callback, cookie, repeat)
    }

    fn async_write(
        &mut self,
        frame_bits: u32,
        count: u32,
        source: *const u8,
        callback: lnSpiCallback,
        cookie: *mut c_void,
        repeat: bool,
    ) -> bool {
        self.done.try_take();
        if count == 0 {
            return true;
        }
        self.update_direction(Direction::TxOnly);

        // The DMA interrupt is the completion signal: it wakes `wait_async()`
        // and calls `callback`.
        self.callback = callback;
        self.callback_cookie = cookie;
        let cookie = self as *mut Spi as *mut c_void;
        self.dma.attach_callback(spi_dma_async_done, cookie);

        let wide = frame_bits == SPI_DATA_SIZE_16;
        if repeat {
            self.dma
                .begin_tx_transfer_const(self.data_register_addr(), source as u32, count, wide);
        } else {
            self.dma
                .begin_tx_transfer(self.data_register_addr(), source as u32, count, wide, wide);
        }
        self.update_data_size_for(frame_bits);
        self.update_dma_tx(true);
        // The C++ `nextWrite()` leaves CS to the caller; here the driver owns it
        // and releases it from `wait_async()`.
        self.cs_on();
        self.set_enabled();
        true
    }

    /// DMA side of the async API: wake [`Spi::wait_async`] and hand control to
    /// the caller's callback (`lnSPI_bp::invokeCallback`).
    fn async_done(&mut self) {
        self.done.give_from_isr();
        let callback = self.callback;
        let cookie = self.callback_cookie;
        self.callback = None;
        self.callback_cookie = core::ptr::null_mut();
        if let Some(callback) = callback {
            unsafe { callback(cookie) };
        }
        if let Some(handler) = self.tx_handler {
            handler.on_tx_complete();
        }
    }

    /// Pretend the pending transfer completed, so a following
    /// [`Spi::wait_async`] returns immediately (`finishAsyncDma`).
    pub fn finish_async_dma(&mut self) -> bool {
        self.done.give();
        true
    }

    /// Wait for the pending async transfer, then release CS and the DMA channel
    /// (`waitForAsync`).  Returns `false` if it did not complete in time.
    pub fn wait_async(&mut self) -> bool {
        let completed = self.done.take_timeout(ASYNC_TIMEOUT_MS);
        if completed {
            self.wait_for_completion();
        }
        self.dma.end_transfer();
        self.cs_off();
        self.set_disabled();
        self.update_dma_tx(false);
        self.update_data_size();
        completed
    }
}

/// Stop the DMA channel and the peripheral when the object goes away, so a
/// dropped bus cannot keep clocking bytes out of a stale buffer.
impl Drop for Spi {
    fn drop(&mut self) {
        self.dma.end_transfer();
        self.cs_off();
        self.set_disabled();
    }
}

impl rs_esprit::Spi for Spi {
    fn configure(&mut self, config: &SpiConfig) -> bool {
        self.speed_hz = config.speed_hz;
        self.data_mode = match config.mode {
            SpiMode::Mode0 => spiDataMode_SPI_MODE0,
            SpiMode::Mode1 => spiDataMode_SPI_MODE1,
            SpiMode::Mode2 => spiDataMode_SPI_MODE2,
            SpiMode::Mode3 => spiDataMode_SPI_MODE3,
        };
        self.bit_order = match config.bit_order {
            SpiBitOrder::MsbFirst => spiBitOrder_SPI_MSBFIRST,
            SpiBitOrder::LsbFirst => spiBitOrder_SPI_LSBFIRST,
        };
        self.begin(config.data_size as u32);
        true
    }

    fn transfer8(&mut self, val: u8) -> u8 {
        let rx = Spi::transfer8(self, val);
        if let Some(handler) = self.rx_handler {
            handler.on_rx_byte(rx);
        }
        rx
    }

    /// Start a DMA write; `SpiTxHandler::on_tx_complete` runs in the DMA
    /// interrupt when the transfer ends.
    fn write_async(&mut self, data: &[u8]) -> bool {
        self.async_write8(data, None, core::ptr::null_mut(), false)
    }

    fn set_rx_handler(&mut self, handler: Option<&'static dyn SpiRxHandler>) {
        self.rx_handler = handler;
    }

    fn set_tx_handler(&mut self, handler: Option<&'static dyn SpiTxHandler>) {
        self.tx_handler = handler;
    }

    fn set_speed(&mut self, speed_hz: u32) {
        Spi::set_speed(self, speed_hz);
    }

    fn set_dma_mode(&mut self, enable: bool) {
        Spi::set_dma_mode(self, enable);
    }

    fn set_ssel(&mut self, ssel: i32) {
        Spi::set_ssel(self, ssel);
    }

    fn wait_done(&mut self) -> bool {
        self.wait_for_completion()
    }

    fn write_slice8(&mut self, data: &[u8]) -> bool {
        self.block_write8(data)
    }

    fn write_slice16(&mut self, data: &[u16]) -> bool {
        self.block_write16(data)
    }

    fn fill8(&mut self, count: u32, val: u8) -> bool {
        self.block_write8_repeat(count, val)
    }

    fn fill16(&mut self, count: u32, val: u16) -> bool {
        self.block_write16_repeat(count, val)
    }

    fn transfer_exact(&mut self, tx: &[u8], rx: &mut [u8]) -> bool {
        if tx.len() != rx.len() {
            return false;
        }
        self.transfer(tx, rx)
    }
}
