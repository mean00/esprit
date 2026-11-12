//! External interrupts on the RP2040 / RP2350, as the
//! [`rs_mcu_traits::exti::Exti`] super trait.
//!
//! On an RP2xxx a pin interrupt *is* the GPIO interrupt: IO_BANK0 keeps a nibble
//! per pad (level-low, level-high, edge-low, edge-high) in `INTR`, and the enable
//! lives in `PROC0_INTE`. All pads share one NVIC line (`IO_IRQ_BANK0`), so the
//! driver installs a single ISR that walks the masked status words and calls the
//! handler armed for the pad that fired.
//!
//! ```ignore
//! use rs_mcu_traits::exti::{Edge, Exti as ExtiTrait, ExtiHandler};
//!
//! struct Button;
//! impl ExtiHandler for Button {
//!     fn on_exti(&self, pin: u32) { /* pin == 15 */ }
//! }
//!
//! let mut exti = rs_exti_rp2xxx::Exti::new();
//! exti.attach(15, Edge::Falling, Some(&Button)).unwrap(); // enables it
//! exti.disable(15).unwrap();                              // keep the handler
//! exti.detach(15);                                        // forget it
//! ```
//!
//! The pad itself has to be an input before the edge can be seen
//! (`rs_mcu_traits::gpio::Gpio::set_mode(.., PinMode::InputPullUp)` or similar);
//! that is the [`Gpio`](rs_mcu_traits::gpio::Gpio) contract, not this one.
#![no_std]

#[cfg(feature = "rp2350")]
#[path = "registers_rp2350.rs"]
mod registers_chip;
#[cfg(not(feature = "rp2350"))]
#[path = "registers_rp2040.rs"]
mod registers_chip;

mod registers;

pub use registers::*;

use portable_atomic::{AtomicBool, AtomicU8, Ordering};
use rs_mcu_traits::error::{Error, Result};
use rs_mcu_traits::exti::{Edge, Exti as ExtiTrait, ExtiHandler};
use rs_mcu_traits::handler::HandlerTable;

/// Interrupt handlers, one slot per pad (only the armed ones are ever set).
static HANDLERS: HandlerTable<dyn ExtiHandler, GPIO_COUNT> = HandlerTable::new();
/// Armed edges per pad, as an `INTE_*` nibble (`0` = not attached).
static EDGES: [AtomicU8; GPIO_COUNT] = [const { AtomicU8::new(0) }; GPIO_COUNT];
/// Whether the shared ISR is installed on the NVIC line.
static IRQ_INSTALLED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    /// Install the ISR of an NVIC line (provided by the RP2xxx runtime).
    fn irq_set_exclusive_handler(num: u32, handler: unsafe extern "C" fn());
    /// Enable or disable an NVIC line (provided by the RP2xxx runtime).
    fn irq_set_enabled(num: u32, enabled: bool);
}

#[inline(always)]
fn write_intr(word: usize, value: u32) {
    // SAFETY: same block; `INTR` is write-1-to-clear.
    unsafe { core::ptr::write_volatile(core::ptr::addr_of_mut!((*registers()).intr[word]), value) }
}

#[inline(always)]
fn read_inte(word: usize) -> u32 {
    // SAFETY: same block.
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*registers()).proc0_inte[word])) }
}

#[inline(always)]
fn write_inte(word: usize, value: u32) {
    // SAFETY: same block.
    unsafe {
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!((*registers()).proc0_inte[word]),
            value,
        )
    }
}

#[inline(always)]
fn read_ints(word: usize) -> u32 {
    // SAFETY: same block.
    unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*registers()).proc0_ints[word])) }
}

/// The shared `IO_IRQ_BANK0` handler: dispatch every pad whose edge latched.
unsafe extern "C" fn io_irq_bank0_handler() {
    for group in 0..IRQ_WORDS {
        let pending = read_ints(group);
        if pending == 0 {
            continue;
        }
        for nibble in 0..8u32 {
            let bits = (pending >> (nibble * 4)) & INTE_MASK;
            if bits == 0 {
                continue;
            }
            let pin = (group as u32) * 8 + nibble;
            // Acknowledge before dispatching: the handler may re-arm the pad, and
            // the source has to be clear or the line would stay asserted.
            write_intr(group, bits << (nibble * 4));
            // SAFETY: `attach` published the handler (task context) before the
            // enable bit that let this interrupt reach us was set, and the slot is
            // a `&'static dyn ExtiHandler` that the ISR only copies out.
            if let Some(handler) = unsafe { HANDLERS.isr_get(pin as usize) } {
                handler.on_exti(pin);
            }
        }
    }
}

/// Install the shared ISR on the NVIC line, once.
fn ensure_irq_installed() {
    if !IRQ_INSTALLED.swap(true, Ordering::SeqCst) {
        // SAFETY: called from task context before any enable bit is set, and the
        // ISR has the `unsafe extern "C" fn()` shape the runtime expects.
        unsafe {
            irq_set_exclusive_handler(NVIC_IO_IRQ_BANK0, io_irq_bank0_handler);
            irq_set_enabled(NVIC_IO_IRQ_BANK0, true);
        }
    }
}

/// The external-interrupt controller of an RP2xxx.
///
/// A zero-sized handle over IO_BANK0: all state lives in the registers and in the
/// two statics above, so a handler may build its own (`Exti::new()`) when it has
/// to mask or re-arm a pad from interrupt context.
pub struct Exti;

impl Exti {
    /// The controller (no initialisation needed: `attach` does it lazily).
    pub const fn new() -> Self {
        Self
    }
}

impl Default for Exti {
    fn default() -> Self {
        Self::new()
    }
}

/// Nibble code of an `Edge`.
const fn code_of(edge: Edge) -> u32 {
    match edge {
        Edge::Rising => INTE_EDGE_HIGH,
        Edge::Falling => INTE_EDGE_LOW,
        Edge::Both => INTE_EDGE_BOTH,
    }
}

fn check_pin(pin: u32) -> Result<usize> {
    if pin as usize >= GPIO_COUNT {
        return Err(Error::InvalidConfig);
    }
    Ok(pin as usize)
}

/// Mask this pad's line and drop any pending edge.
fn mask_pin(pin: u32) {
    let group = word(pin);
    let mut inte = read_inte(group);
    inte &= !mask(pin);
    write_inte(group, inte);
    write_intr(group, INTE_MASK << shift(pin));
}

impl ExtiTrait for Exti {
    fn attach(
        &mut self,
        pin: u32,
        edge: Edge,
        handler: Option<&'static dyn ExtiHandler>,
    ) -> Result<()> {
        let index = check_pin(pin)?;
        let code = code_of(edge);
        // Publish the handler *before* the enable bit: the ISR reads the slot as
        // soon as the line can fire.
        HANDLERS.set(index, handler);
        EDGES[index].store(code as u8, Ordering::Relaxed);
        mask_pin(pin);
        if handler.is_some() {
            let group = word(pin);
            write_inte(group, read_inte(group) | (code << shift(pin)));
            ensure_irq_installed();
        }
        Ok(())
    }

    fn enable(&mut self, pin: u32) -> Result<()> {
        let index = check_pin(pin)?;
        let code = EDGES[index].load(Ordering::Relaxed) as u32;
        if code == 0 {
            // Nothing armed on that pad: enabling it would set no source.
            return Err(Error::InvalidConfig);
        }
        let group = word(pin);
        write_intr(group, INTE_MASK << shift(pin));
        write_inte(group, read_inte(group) | (code << shift(pin)));
        ensure_irq_installed();
        Ok(())
    }

    fn disable(&mut self, pin: u32) -> Result<()> {
        check_pin(pin)?;
        mask_pin(pin);
        Ok(())
    }

    fn detach(&mut self, pin: u32) {
        if check_pin(pin).is_err() {
            return;
        }
        mask_pin(pin);
        EDGES[pin as usize].store(0, Ordering::Relaxed);
        HANDLERS.set(pin as usize, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_and_shift_split_the_pads_in_nibbles() {
        assert_eq!((word(0), shift(0), mask(0)), (0, 0, 0x0f));
        assert_eq!((word(7), shift(7), mask(7)), (0, 28, 0xf000_0000));
        assert_eq!((word(8), shift(8), mask(8)), (1, 0, 0x0f));
    }

    #[test]
    fn the_last_pad_fits_in_the_last_word() {
        assert!(word(GPIO_COUNT as u32 - 1) < IRQ_WORDS);
    }

    #[test]
    fn edges_map_to_the_databook_bits() {
        assert_eq!(code_of(Edge::Falling), 0x4);
        assert_eq!(code_of(Edge::Rising), 0x8);
        assert_eq!(code_of(Edge::Both), 0xc);
    }

    #[test]
    fn out_of_range_pads_are_rejected() {
        assert!(check_pin(GPIO_COUNT as u32 - 1).is_ok());
        assert_eq!(check_pin(GPIO_COUNT as u32), Err(Error::InvalidConfig));
    }
}

// EOF
