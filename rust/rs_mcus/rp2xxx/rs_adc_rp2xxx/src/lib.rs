//! RP2040 / RP2350 ADC backend for `rust_esprit` — a pure-Rust driver.
//!
//! Two shapes on the same hardware block (`registers.rs`) and the same DMA
//! driver (`rs_dma_rp2xxx`), both implementing the esprit super traits of
//! [`rs_mcu_traits::adc`]:
//!
//! * [`SimpleAdc`] — one pin, one polled conversion (`rs_mcu_traits::adc::Adc`).
//!   This is what the probe's target-voltage sense uses.
//! * [`TimingAdc`] — N inputs × M samples per input into one buffer, moved by
//!   DMA and paced by the ADC's own data request (`rs_mcu_traits::adc::ScanAdc`).
//!   [`TimingAdc::multi_read`] blocks until the transfer is complete,
//!   [`TimingAdc::start_async`] returns immediately and runs the completion
//!   **in the DMA interrupt** — either the handler installed with `set_handler`
//!   (`AdcHandler::on_scan_complete`) or the raw `callback(cookie)`.
//!
//! ## The sequence (RP2040 datasheet §4.9.2.5, RP2350 §12.4.4.4)
//!
//! 1. sampling rate in `DIV` (period = `1 + INT + FRAC/256` `clk_adc` cycles),
//! 2. inputs: `CS.AINSEL` is the first channel, `CS.RROBIN` the set the mux walks,
//! 3. `FCS.EN` + `FCS.DREQ_EN` with `THRESH = 1`, so every sample raises a DREQ,
//! 4. start the **DMA first** (source = the FIFO, `DREQ_ADC`, 16 bits, walking the
//!    destination buffer), then `CS.START_MANY`,
//! 5. when the last sample is in memory: clear `START_MANY`, wait for `CS.READY`
//!    and drain the FIFO, so the next run starts on a clean FIFO.
//!
//! ## What is chip-specific
//!
//! Only the numbers: `ADC_BASE` (inherited from `rs_rp2xxx`), the widths of
//! `CS.AINSEL` / `CS.RROBIN`, the pin → channel table and `DREQ_ADC`. They live
//! in `registers_rp2040.rs` / `registers_rp2350.rs`, selected once, here.
//! The register block, the driver and the DMA wiring are shared.

#![no_std]

pub mod registers;

// The chip-specific half of the map. This is the **one** place the chip is
// chosen — `rp2350` wins when both features are set, because `rust_esprit`'s
// `rp2350` implies its `rp2040` (see `rs_rp2xxx/src/lib.rs`). The `#[path]`s
// keep the files flat in `src/`.
#[cfg(feature = "rp2350")]
#[path = "registers_rp2350.rs"]
mod registers_chip;

#[cfg(not(feature = "rp2350"))]
#[path = "registers_rp2040.rs"]
mod registers_chip;

pub use registers::*;
pub use registers_chip::*;

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};

use rs_dma_rp2xxx::{DmaChannel, DmaTransferConfig, DmaTransferWidth};
use rs_rtos::BinarySemaphore;
use rs_rtos::sync_cell::SyncCell;
use rs_gpio_rp2xxx::Pin;
use rs_mcu_traits::adc::AdcHandler;
use rs_mcu_traits::gpio::{Gpio, PinMode};
use rs_rcu_rp2xxx::{Peripheral, Rcu};

/// The only ADC instance on both chips (the C++ driver asserted the same).
pub const ADC_INSTANCE: u32 = 0;

/// Spins allowed while waiting for `CS.READY`.
///
/// One conversion is 96 `clk_adc` cycles, i.e. 2 µs at the default 48 MHz, so
/// this budget is three orders of magnitude over what a working ADC needs: it
/// exists so a broken clock tree fails the read instead of hanging the probe.
const READY_POLL_LIMIT: u32 = 200_000;

/// Default `clk_adc`, used when the clock tree cannot report it.
const DEFAULT_ADC_CLOCK_HZ: u32 = 48_000_000;

/// Timeout of the DMA wait in [`TimingAdc::multi_read`].
///
/// A 1000-sample read at 10 kHz already takes 100 ms; the timeout is only here so
/// a dead DREQ cannot block the caller forever.
pub const TRANSFER_TIMEOUT_MS: u32 = 1_000;

// ---------------------------------------------------------------------------
//  Register access
//
//  One address computation and one volatile accessor pair for the whole block:
//  the register is *named* at the call site, and its address is
//  `ADC_BASE + offset_of!(AdcRegisters, …)`, so no offset is ever written twice
//  and the layout stays owned by `registers.rs` (its tripwires check it).
// ---------------------------------------------------------------------------

// One volatile accessor pair for the ADC block, used as
//
//     let cs = read_reg!(cs);
//     write_reg!(fcs, ADC_FCS_EN);
//
// so a register is *named* at the call site while its address stays
// `ADC_BASE + offset_of!(…)`.
macro_rules! read_reg {
    ($member:ident) => {
        read_volatile_at(ADC_BASE + core::mem::offset_of!(AdcRegisters, $member) as u32)
    };
}

macro_rules! write_reg {
    ($member:ident, $value:expr) => {
        write_volatile_at(
            ADC_BASE + core::mem::offset_of!(AdcRegisters, $member) as u32,
            $value,
        )
    };
}

/// Read one register of the ADC block.
#[inline(always)]
fn read_volatile_at(address: u32) -> u32 {
    // SAFETY: the callers are the two macros above, so `address` is the offset of a
    // register of the ADC block, which is memory-mapped at `ADC_BASE`
    // (registers.rs owns the layout and pins it with compile-time tripwires).
    unsafe { core::ptr::read_volatile(address as *const u32) }
}

/// Write one register of the ADC block.
#[inline(always)]
fn write_volatile_at(address: u32, value: u32) {
    // SAFETY: same as `read_volatile_at`, write side.
    unsafe { core::ptr::write_volatile(address as *mut u32, value) }
}

/// Set the bits in `set` and clear the bits in `clear` — CS is plain RW, it has no
/// write-1-to-act alias.
#[inline(always)]
fn modify_cs(set: u32, clear: u32) {
    write_reg!(cs, (read_reg!(cs) & !clear) | set);
}

// ---------------------------------------------------------------------------
//  Common ADC operations
// ---------------------------------------------------------------------------

/// ADC channel of a pin, or `None` when the pad has no analog input.
pub fn channel_of(pin: Pin) -> Option<u32> {
    pin_channel(pin.number())
}

/// Wait for `CS.READY`, bounded by [`READY_POLL_LIMIT`].
fn wait_ready() -> bool {
    for _ in 0..READY_POLL_LIMIT {
        if read_reg!(cs) & ADC_CS_READY != 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// Unreset the ADC and power it up (§4.9.2.1: `CS.EN` gates the ADC clock).
fn power_on() -> bool {
    rs_rcu_rp2xxx::enable(Peripheral::Adc0);
    modify_cs(ADC_CS_EN, 0);
    wait_ready()
}

/// Put the ADC back in standby. The analog mux keeps its selection; only the SAR
/// and its clock are switched off, which is what a quiet sense input wants
/// between two voltage reads.
fn power_off() {
    modify_cs(0, ADC_CS_EN);
}

/// `clk_adc` in Hz.
fn adc_clock_hz() -> u32 {
    match Rcu::get_clock(Peripheral::Adc0) {
        0 => DEFAULT_ADC_CLOCK_HZ,
        hz => hz,
    }
}

/// `CS.DIV` for `fq` samples per second on a `clk_adc` of `adc_hz`.
///
/// One conversion every `1 + INT + FRAC/256` cycles (§4.9.2.2), so the divider is
/// `clk_adc / fq` minus the one free cycle, in 1/256 cycle units. `fq == 0` means
/// "as fast as the ADC goes", i.e. back-to-back conversions.
///
/// The integer field is 8 bits wide, so the period cannot exceed 255 + 255/256
/// cycles: the slowest rate is roughly `clk_adc / 256` (187 kHz at the default
/// 48 MHz), and anything slower is clamped there instead of wrapping.
///
/// Pure function: the clock is a parameter so the arithmetic is testable on the
/// host.
fn divider_for_hz(fq: u32, adc_hz: u32) -> u32 {
    if fq == 0 {
        return 0;
    }
    // `period_256 = 256 * (1 + INT) + FRAC`, and the register field is
    // `INT << 8 | FRAC`, i.e. exactly `256 * INT + FRAC`, so the subtraction
    // lands bit-for-bit in the register. 64 bits because `clk_adc * 256` does
    // not fit in 32.
    let period_256 = (adc_hz as u64 * 256) / fq as u64;
    let divider = period_256.saturating_sub(256).min(ADC_DIV_MAX as u64);
    divider as u32
}

/// [`divider_for_hz`] on the clock the ADC actually runs from.
fn divider_for(fq: u32) -> u32 {
    divider_for_hz(fq, adc_clock_hz())
}

/// The inputs a transfer samples, in the order the mux walks them.
///
/// The ADC starts at `CS.AINSEL` and advances to the next channel whose bit is
/// set in `CS.RROBIN` (§4.9.2.3), i.e. it walks the mask **upwards** and wraps.
/// The buffer therefore holds the samples in ascending channel order, which is
/// what makes them attributable to a pin: [`Inputs::from_pins`] rejects an
/// unsorted list instead of silently labelling the data wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inputs {
    /// Channel of the first sample (`CS.AINSEL`).
    pub first: u32,
    /// Channels to walk, one bit each (`CS.RROBIN`).
    pub mask: u32,
    /// Number of channels, i.e. samples per pass.
    pub count: u32,
}

impl Inputs {
    /// One channel, no round robin.
    pub const fn single(channel: u32) -> Self {
        Self {
            first: channel,
            mask: 1 << channel,
            count: 1,
        }
    }

    /// The inputs of `pins`, or `None` when the list is empty, too long, contains
    /// a pad without an analog input, or is not in ascending channel order.
    pub fn from_pins(pins: &[Pin]) -> Option<Self> {
        if pins.is_empty() || pins.len() as u32 > ADC_INPUT_COUNT {
            return None;
        }
        let mut inputs = Self {
            first: 0,
            mask: 0,
            count: 0,
        };
        for pin in pins {
            let channel = channel_of(*pin)?;
            if channel >= ADC_INPUT_COUNT || inputs.mask & (1 << channel) != 0 {
                return None;
            }
            if inputs.count > 0 && channel <= inputs.last() {
                // Not ascending: the hardware order would differ from the caller's.
                return None;
            }
            inputs.mask |= 1 << channel;
            if inputs.count == 0 {
                inputs.first = channel;
            }
            inputs.count += 1;
        }
        Some(inputs)
    }

    /// Channel of the last sample added, i.e. the highest one so far.
    fn last(&self) -> u32 {
        u32::BITS - 1 - self.mask.leading_zeros()
    }
}

/// Program the rate and the inputs, and enable the sample FIFO with its DREQ.
fn configure(inputs: Inputs, divider: u32) {
    // Rate first, and before a run: writing either DIV field resets the divider,
    // so it is never touched while converting (§4.9.4).
    write_reg!(div, divider);
    let ainsel = (inputs.first << ADC_CS_AINSEL_LSB) & ADC_CS_AINSEL_MASK;
    let rrobin = if inputs.count > 1 {
        (inputs.mask << ADC_CS_RROBIN_LSB) & ADC_CS_RROBIN_MASK
    } else {
        0 // one channel: leave the round robin off (§4.9.2.3)
    };
    modify_cs(
        ADC_CS_EN | ainsel | rrobin,
        ADC_CS_AINSEL_MASK | ADC_CS_RROBIN_MASK,
    );
    // 16-bit results (no SHIFT) and one DREQ per sample.
    write_reg!(
        fcs,
        ADC_FCS_EN | ADC_FCS_DREQ_EN | (ADC_FCS_DREQ_THRESHOLD << ADC_FCS_THRESH_LSB)
    );
}

/// Start free-running conversions.
fn start_conversions() {
    modify_cs(ADC_CS_START_MANY, 0);
}

/// Stop converting and leave the FIFO empty and its sticky flags clear.
///
/// §4.9.2.5: after clearing `START_MANY` software polls `CS.READY` (the last
/// conversion is still in flight) and then drains the stray samples, otherwise
/// they would be the first entries of the next transfer.
fn stop_conversions() {
    modify_cs(0, ADC_CS_START_MANY);
    wait_ready();
    for _ in 0..ADC_FIFO_DEPTH {
        if read_reg!(fcs) & ADC_FCS_EMPTY != 0 {
            break;
        }
        // Reading the FIFO is what pops it.
        let _ = read_reg!(fifo);
    }
    // OVER / UNDER are write-1-to-clear; keep the FIFO itself enabled.
    write_reg!(fcs, read_reg!(fcs) | ADC_FCS_OVER | ADC_FCS_UNDER);
}

// ---------------------------------------------------------------------------
//  Completion slot (task <-> DMA interrupt)
// ---------------------------------------------------------------------------

/// What the DMA completion interrupt has to do.
///
/// The task that starts a transfer writes it *before* publishing the channel
/// callback; the interrupt reads it once and dispatches. Both completion shapes
/// of the API live here so one record serves the super-trait handler and the raw
/// callback.
#[derive(Clone, Copy)]
struct Completion {
    /// `AdcHandler::on_scan_complete` receiver, if one is installed.
    handler: Option<&'static dyn AdcHandler>,
    /// Raw callback of [`TimingAdc::start_async`].
    callback: Option<unsafe extern "C" fn(*mut c_void)>,
    /// Argument of that callback.
    cookie: *mut c_void,
    /// Buffer the DMA filled, handed to the handler as a slice.
    buffer: *mut u16,
    /// Number of samples in `buffer`.
    len: usize,
}

impl Completion {
    const NONE: Self = Self {
        handler: None,
        callback: None,
        cookie: core::ptr::null_mut(),
        buffer: core::ptr::null_mut(),
        len: 0,
    };

    /// Run both completion paths: the handler with the samples, then the callback.
    fn dispatch(self) {
        if let Some(handler) = self.handler
            && self.len != 0
            && !self.buffer.is_null()
        {
            // SAFETY: the armer kept this buffer alive and untouched until the
            // completion — that is the contract of `start_async` — and `len` is
            // the length it handed over.
            let samples = unsafe { core::slice::from_raw_parts(self.buffer, self.len) };
            handler.on_scan_complete(samples);
        }
        if let Some(callback) = self.callback {
            // SAFETY: `start_async` only ever publishes a function of this type.
            unsafe { callback(self.cookie) };
        }
    }
}

/// The armed completion record. Written by the arming task (under the [`ARMED`]
/// protocol) and read once by the interrupt that consumes it.
static COMPLETION: SyncCell<Completion> = SyncCell::new(Completion::NONE);

/// `true` while a transfer is armed, i.e. from just before the channel callback is
/// published until the interrupt has dispatched it.
///
/// This is the hand-off flag between the task and the interrupt: the task writes
/// the record and then stores `true` with `Release`, the interrupt loads it with
/// `Acquire` before reading the record and clears it with `Release` when done.
/// Plain loads and stores only — no read-modify-write, so this works on the
/// RP2040's `thumbv6m` — and no critical section, so it is safe inside an ISR.
static ARMED: AtomicBool = AtomicBool::new(false);

/// The DMA completion interrupt: stop the converter, then run the completion the
/// task armed.
///
/// Nothing here blocks. `stop_conversions` polls `CS.READY`, which goes high
/// within 96 `clk_adc` cycles of the last conversion (2 µs at 48 MHz).
unsafe extern "C" fn completion_isr(_cookie: *mut c_void) {
    // Acquire pairs with the armer's `ARMED.store(true, Release)`: everything it
    // wrote into COMPLETION is visible below. `false` means the transfer was
    // already dispatched, so this interrupt is spurious and must do nothing.
    if !ARMED.load(Ordering::Acquire) {
        return;
    }
    stop_conversions();
    // SAFETY: the armer bails out while ARMED is set and an interrupt cannot be
    // re-entered, so this is the only reader of the record right now; the atomic
    // load above is what makes its contents visible.
    let done = unsafe { *COMPLETION.as_ref() };
    ARMED.store(false, Ordering::Release);
    done.dispatch();
}

// ---------------------------------------------------------------------------
//  SimpleAdc — one pin, one conversion
// ---------------------------------------------------------------------------

/// A polled single-channel ADC, i.e. `rs_mcu_traits::adc::Adc`.
///
/// `read()` powers the SAR up, converts, and powers it down again: the analog mux
/// keeps its selection (the pad is in ADC mode from construction), so the divider
/// feeding the pin is only loaded while a conversion runs.
pub struct SimpleAdc {
    pin: Pin,
    channel: u32,
}

impl SimpleAdc {
    /// Create the ADC on `pin` (a pin *number*, `GPIO26` … `GPIO29`).
    ///
    /// `instance` must be [`ADC_INSTANCE`]. Panics when the pad has no analog
    /// input — the C++ driver's `xAssert`, and there is nothing sane to return
    /// instead, because the caller asked for a pin the silicon cannot convert.
    pub fn new(instance: u32, pin: u32) -> Self {
        assert_eq!(instance, ADC_INSTANCE, "the RP2xxx has a single ADC");
        let pad = Pin::from_number(pin).expect("SimpleAdc: pin is not an RP2xxx pad");
        let channel = channel_of(pad).expect("SimpleAdc: pad has no ADC input");
        let mut pad = pad;
        // FUNCSEL = NULL and output disabled: the pad must not be driven (§4.9.1).
        let _ = pad.set_mode(PinMode::Analog);
        rs_rcu_rp2xxx::enable(Peripheral::Adc0);
        Self { pin: pad, channel }
    }

    /// The instance of this ADC (always [`ADC_INSTANCE`]).
    #[inline(always)]
    pub fn instance(&self) -> u32 {
        ADC_INSTANCE
    }

    /// The configured pin, as a pin number.
    #[inline(always)]
    pub fn pin(&self) -> u32 {
        self.pin.number()
    }

    /// The analog channel the pin is wired to.
    #[inline(always)]
    pub fn channel(&self) -> u32 {
        self.channel
    }

    /// No-op: the RP2xxx gives no sample-time knob (the SAR's acquisition window
    /// is fixed at 96 `clk_adc` cycles). Kept so a caller written against the
    /// GD32 backend — the probe's voltage sense sets `SMPT` — still compiles.
    pub fn set_smpt(&self, _smpt: u32) {}

    /// One conversion, `0 … 4095`; `None` when the ADC never became ready.
    pub fn convert(&self) -> Option<u16> {
        if !power_on() {
            return None;
        }
        modify_cs(
            (self.channel << ADC_CS_AINSEL_LSB) & ADC_CS_AINSEL_MASK,
            ADC_CS_AINSEL_MASK | ADC_CS_RROBIN_MASK,
        );
        modify_cs(ADC_CS_START_ONCE, 0);
        let ready = wait_ready();
        let value = (read_reg!(result) & ADC_VALUE_MASK) as u16;
        power_off();
        if ready { Some(value) } else { None }
    }

    /// One conversion, `0 … 4095`; `0` when the ADC never became ready.
    pub fn read(&self) -> i32 {
        self.convert().unwrap_or(0) as i32
    }
}

impl rs_mcu_traits::adc::Adc for SimpleAdc {
    /// The RP2xxx acquisition window is fixed by the silicon (96 `clk_adc`
    /// cycles), so `AdcConfig` can only be accepted, not applied: the conversion
    /// stays valid, it is just not tunable.
    fn configure(&mut self, _config: &rs_mcu_traits::adc::AdcConfig) -> rs_mcu_traits::Result<()> {
        Ok(())
    }

    fn read(&mut self) -> rs_mcu_traits::Result<u16> {
        self.convert().ok_or(rs_mcu_traits::Error::Timeout)
    }

    /// A one-shot polled conversion raises no interrupt, so there is nothing to
    /// notify: the value is the return of `read()`. Accepted for the trait's shape
    /// (the GD32 backend calls it from its end-of-conversion interrupt).
    fn set_handler(&mut self, _handler: Option<&'static dyn rs_mcu_traits::adc::AdcHandler>) {}
}

// ---------------------------------------------------------------------------
//  TimingAdc — N inputs × M samples, moved by DMA
// ---------------------------------------------------------------------------

/// A DMA-driven multi-sample ADC, i.e. `rs_mcu_traits::adc::ScanAdc`.
///
/// The conversion is paced by the ADC itself (`CS.DIV` plus free-running
/// `START_MANY`), so the transfer needs no timer and the CPU only touches the ADC
/// to start and to stop it. [`multi_read`](Self::multi_read) waits on a
/// `BinarySemaphore` given by the DMA interrupt, [`start_async`](Self::start_async)
/// returns at once and dispatches from that same interrupt.
pub struct TimingAdc {
    inputs: Inputs,
    divider: u32,
    /// The channel the transfers use, claimed once so a read never steals one
    /// from another peripheral mid-way.
    dma: Option<DmaChannel>,
    /// Given by the DMA interrupt; only the blocking read waits on it.
    done: BinarySemaphore,
    handler: Option<&'static dyn AdcHandler>,
}

/// 16-bit samples, paced by `DREQ_ADC`, destination walking the buffer.
const ADC_DMA_TRANSFER: DmaTransferConfig =
    DmaTransferConfig::new(DmaTransferWidth::HalfWord, DREQ_ADC);

impl TimingAdc {
    /// Create the driver on `instance` (which must be [`ADC_INSTANCE`]).
    ///
    /// Powers the ADC up and claims a DMA channel; the inputs default to channel
    /// 0 (`GPIO26`) and the rate to back-to-back conversions until
    /// [`set_source`](Self::set_source) says otherwise.
    pub fn new(instance: u32) -> Self {
        assert_eq!(instance, ADC_INSTANCE, "the RP2xxx has a single ADC");
        rs_rcu_rp2xxx::enable(Peripheral::Adc0);
        let adc = Self {
            inputs: Inputs::single(0),
            divider: 0,
            dma: DmaChannel::claim(),
            done: BinarySemaphore::new(),
            handler: None,
        };
        adc.program();
        adc
    }

    /// The inputs being sampled.
    #[inline(always)]
    pub fn inputs(&self) -> Inputs {
        self.inputs
    }

    /// Samples per channel, i.e. how many entries one pass of the round robin
    /// produces.
    #[inline(always)]
    pub fn channel_count(&self) -> u32 {
        self.inputs.count
    }

    /// `true` while a transfer started with [`start_async`](Self::start_async) is
    /// still in flight.
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        ARMED.load(Ordering::Acquire)
    }

    /// Program the inputs and the sampling frequency.
    ///
    /// `pins` must be in ascending channel order — the mux walks `CS.RROBIN`
    /// upwards, so the buffer comes back in that order. `fq` is in samples per
    /// second per channel (`0` = as fast as the ADC allows).
    ///
    /// Returns `false` (and changes nothing) for a pin list that cannot be
    /// sampled: empty, longer than [`ADC_INPUT_COUNT`], a pad without an analog
    /// input, or channels out of order.
    pub fn set_source(&mut self, fq: u32, pins: &[Pin]) -> bool {
        let Some(inputs) = Inputs::from_pins(pins) else {
            return false;
        };
        self.inputs = inputs;
        self.divider = divider_for(fq);
        // Put the pads in ADC mode too: from here on the driver owns the analog
        // side of the pins it samples.
        for pin in pins {
            let mut pad = *pin;
            let _ = pad.set_mode(PinMode::Analog);
        }
        self.program();
        true
    }

    /// Install the completion handler called from the DMA interrupt with the
    /// samples of every completed transfer.
    pub fn set_handler(&mut self, handler: Option<&'static dyn AdcHandler>) {
        self.handler = handler;
    }

    /// Blocking multi-sample read of `nb_sample_per_channel` samples per input.
    ///
    /// `output` must hold `nb_sample_per_channel * channel_count()` samples and is
    /// filled in ascending channel order. Returns `false` when the DMA could not
    /// be started, the buffer is too small, another transfer is in flight, or the
    /// transfer did not complete within [`TRANSFER_TIMEOUT_MS`].
    pub fn multi_read(&mut self, nb_sample_per_channel: u32, output: &mut [u16]) -> bool {
        let Some(count) = self.samples_for(nb_sample_per_channel, output.len()) else {
            return false;
        };
        if self.is_busy() {
            return false;
        }
        // Program before borrowing the channel: `program` takes `&self`, and the
        // DMA handle holds a mutable borrow of `self.dma`.
        self.program();
        let Some(dma) = self.dma.as_mut() else {
            return false;
        };

        // Drop a token a previous timeout may have left behind.
        self.done.try_take();
        dma.detach_callback();
        dma.attach_completion_semaphore(&self.done);
        dma.configure_rx(
            fifo_address(),
            output.as_mut_ptr() as *mut u8,
            count,
            ADC_DMA_TRANSFER,
        );
        // DMA first, conversions second (§4.9.2.5).
        start_conversions();
        let complete = self.done.take_timeout(TRANSFER_TIMEOUT_MS);
        stop_conversions();
        dma.end_transfer();
        dma.detach_callback();

        if complete && let Some(handler) = self.handler {
            handler.on_scan_complete(&output[..count as usize]);
        }
        complete
    }

    /// Start an asynchronous multi-sample read.
    ///
    /// Returns as soon as the DMA and the converter are running. When the last
    /// sample has landed, the DMA interrupt stops the converter and runs, in this
    /// order, the handler installed with [`set_handler`](Self::set_handler) — with
    /// `output` as the sample slice — and then `callback(cookie)`.
    ///
    /// # Contract
    ///
    /// `output` belongs to the transfer until the completion runs: it must not be
    /// read, written or dropped before then. The completion itself runs in an
    /// interrupt context, so it must not block (no logger, no semaphore wait).
    // The cookie is only ever forwarded to the completion callback — this
    // function never dereferences it — but clippy cannot see that across the
    // `unsafe` call below.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn start_async(
        &mut self,
        nb_sample_per_channel: u32,
        output: &mut [u16],
        callback: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
    ) -> bool {
        // SAFETY: `output` is a live mutable slice of `output.len()` samples for
        // the whole call, and the completion contract above keeps it alive.
        unsafe {
            self.start_async_raw(
                nb_sample_per_channel,
                output.as_mut_ptr(),
                output.len(),
                callback,
                cookie,
            )
        }
    }

    /// [`start_async`](Self::start_async) over raw buffer parts — the form a
    /// wrapper needs when the buffer lives inside the context handed to the
    /// completion.
    ///
    /// # Safety
    ///
    /// `output` must point to `capacity` writable `u16`s that stay alive and
    /// untouched until the completion has run.
    pub unsafe fn start_async_raw(
        &mut self,
        nb_sample_per_channel: u32,
        output: *mut u16,
        capacity: usize,
        callback: Option<unsafe extern "C" fn(*mut c_void)>,
        cookie: *mut c_void,
    ) -> bool {
        let Some(count) = self.samples_for(nb_sample_per_channel, capacity) else {
            return false;
        };
        if self.is_busy() {
            return false;
        }
        // Program before borrowing the channel: `program` takes `&self`, and the
        // DMA handle holds a mutable borrow of `self.dma`.
        self.program();
        let Some(dma) = self.dma.as_mut() else {
            return false;
        };

        // Arm the record *before* the channel callback is published: the interrupt
        // may fire as soon as it is enabled, and it must never see a half-written
        // record.
        //
        // SAFETY: `ARMED` is false (checked above), so neither the interrupt nor
        // another task is reading the record right now.
        unsafe {
            *COMPLETION.as_mut() = Completion {
                handler: self.handler,
                callback,
                cookie,
                buffer: output,
                len: count as usize,
            }
        }
        ARMED.store(true, Ordering::Release);
        dma.detach_callback();
        dma.attach_callback(completion_isr, core::ptr::null_mut());
        dma.configure_rx(fifo_address(), output as *mut u8, count, ADC_DMA_TRANSFER);
        start_conversions();
        true
    }

    /// Sample count of a transfer of `nb` per channel into a `capacity` buffer.
    ///
    /// `None` when the buffer is too small or the count is zero or overflows.
    fn samples_for(&self, nb_sample_per_channel: u32, capacity: usize) -> Option<u32> {
        let count = nb_sample_per_channel.checked_mul(self.inputs.count)?;
        if count == 0 || count as usize > capacity {
            return None;
        }
        Some(count)
    }

    /// Split a buffer length into whole scan passes: `(samples per channel,
    /// total samples)`. A tail that would not fill a pass is left out.
    fn split(&self, capacity: usize) -> rs_mcu_traits::Result<(u32, usize)> {
        let per_scan = self.inputs.count as usize;
        let count = capacity - capacity % per_scan;
        if count == 0 {
            return Err(rs_mcu_traits::Error::BufferTooSmall);
        }
        Ok(((count / per_scan) as u32, count))
    }

    /// Power the ADC up and program rate, inputs and FIFO.
    fn program(&self) {
        power_on();
        configure(self.inputs, self.divider);
    }
}

impl Drop for TimingAdc {
    fn drop(&mut self) {
        stop_conversions();
        ARMED.store(false, Ordering::Release);
        if let Some(dma) = self.dma.as_mut() {
            dma.detach_callback();
            dma.end_transfer();
        }
        power_off();
    }
}

impl rs_mcu_traits::adc::ScanAdc for TimingAdc {
    /// `ScanConfig::sources` carries pin numbers: this ADC samples its pads
    /// directly, and the mux only has analog inputs.
    fn configure(
        &mut self,
        config: &rs_mcu_traits::adc::ScanConfig<'_>,
    ) -> rs_mcu_traits::Result<()> {
        if config.sources.len() > ADC_INPUT_COUNT as usize {
            return Err(rs_mcu_traits::Error::InvalidConfig);
        }
        let mut pads = [Pin::GPIO26; ADC_INPUT_COUNT as usize];
        for (slot, number) in pads.iter_mut().zip(config.sources) {
            *slot = Pin::from_number(*number).ok_or(rs_mcu_traits::Error::InvalidConfig)?;
        }
        if self.set_source(config.fq_hz, &pads[..config.sources.len()]) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::InvalidConfig)
        }
    }

    fn samples_per_scan(&self) -> usize {
        self.channel_count() as usize
    }

    fn read(&mut self, output: &mut [u16]) -> rs_mcu_traits::Result<usize> {
        let (per_channel, count) = self.split(output.len())?;
        if self.multi_read(per_channel, &mut output[..count]) {
            Ok(count)
        } else {
            Err(rs_mcu_traits::Error::Busy)
        }
    }

    /// The completion is the one armed with [`Self::set_handler`]: `start_async`
    /// only decides how many samples go in.
    fn start_async(&mut self, output: &mut [u16]) -> rs_mcu_traits::Result<()> {
        let (per_channel, count) = self.split(output.len())?;
        if self.start_async(per_channel, &mut output[..count], None, core::ptr::null_mut()) {
            Ok(())
        } else {
            Err(rs_mcu_traits::Error::Busy)
        }
    }

    fn is_busy(&self) -> bool {
        self.is_busy()
    }

    fn set_handler(&mut self, handler: Option<&'static dyn rs_mcu_traits::adc::AdcHandler>) {
        self.set_handler(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `1 + INT + FRAC/256` cycles between two conversions: at the ADC clock
    /// itself the period is one cycle, i.e. no divider at all.
    #[test]
    fn divider_of_the_clock_itself_is_zero() {
        assert_eq!(divider_for_hz(48_000_000, 48_000_000), 0);
    }

    /// 48 MHz / 187.5 cycles: `INT = 186`, `FRAC = 128/256`, i.e. 256 kHz — the
    /// slowest rate the 8-bit integer field can express (`clk_adc / 256`).
    #[test]
    fn divider_encodes_int_and_frac() {
        let div = divider_for_hz(256_000, 48_000_000);
        assert_eq!((div & ADC_DIV_INT_MASK) >> ADC_DIV_INT_LSB, 186);
        assert_eq!(div & ADC_DIV_FRAC_MASK, 0x80);
    }

    /// Asking for "as fast as possible", for more than the clock, and for the
    /// slowest possible rate all land inside the field instead of wrapping.
    #[test]
    fn divider_saturates_instead_of_wrapping() {
        assert_eq!(divider_for_hz(0, 48_000_000), 0);
        assert_eq!(divider_for_hz(u32::MAX, 48_000_000), 0);
        assert_eq!(divider_for_hz(1, 48_000_000), ADC_DIV_MAX);
    }

    /// A single pin is a single channel, with the round robin off.
    #[test]
    fn one_pin_is_one_channel() {
        let inputs = Inputs::from_pins(&[Pin::GPIO27]).expect("GPIO27 is an ADC input");
        assert_eq!(
            inputs,
            Inputs {
                first: 1,
                mask: 1 << 1,
                count: 1
            }
        );
    }

    /// Sorted pins become a mask plus the first channel. The order the caller
    /// gives is the order the samples come back in, so an unsorted list — or a
    /// repeated pin — is refused instead of mislabelling the data.
    #[test]
    fn pins_must_be_in_ascending_channel_order() {
        let inputs = Inputs::from_pins(&[Pin::GPIO26, Pin::GPIO28]).expect("both are inputs");
        assert_eq!(inputs.first, 0);
        assert_eq!(inputs.mask, 0b0101);
        assert_eq!(inputs.count, 2);

        assert!(Inputs::from_pins(&[Pin::GPIO28, Pin::GPIO26]).is_none());
        assert!(Inputs::from_pins(&[Pin::GPIO26, Pin::GPIO26]).is_none());
    }

    /// A pad without an analog input (or an empty list) is a programming error the
    /// driver reports instead of converting garbage.
    #[test]
    fn digital_only_pads_are_refused() {
        assert!(Inputs::from_pins(&[]).is_none());
        assert!(Inputs::from_pins(&[Pin::GPIO0]).is_none());
        assert!(Inputs::from_pins(&[Pin::GPIO25]).is_none());
    }

    /// Every analog channel of the chip is accepted.
    #[test]
    fn all_analog_channels_are_accepted() {
        let mut pins = [Pin::GPIO26; ADC_INPUT_COUNT as usize];
        let mut found = 0;
        for number in 0..48u32 {
            let Some(pin) = Pin::from_number(number) else {
                break;
            };
            if channel_of(pin).is_some() && found < pins.len() {
                pins[found] = pin;
                found += 1;
            }
        }
        assert_eq!(
            found,
            pins.len(),
            "the chip has as many analog inputs as ADC_INPUT_COUNT says"
        );
        assert!(Inputs::from_pins(&pins).is_some());
    }
}
