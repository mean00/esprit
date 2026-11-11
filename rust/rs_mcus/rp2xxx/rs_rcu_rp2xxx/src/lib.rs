#![no_std]

//! Reset and Clock Unit (RCU) driver for RP2040 and RP2350.
//!
//! Implements the `rs_mcu_traits::rcu::Rcu` super trait, handling peripheral reset
//! and clock management via the RP2xxx RESETS and CLOCKS blocks.

pub mod registers;

// The chip-specific half of the map: reset-bit table, USB/ADC clock indices and
// the `Peripheral` → reset-bit mapping. This is the **one** place the chip is
// chosen — `rp2350` wins when both features are set, because `rust_esprit`'s
// `rp2350` implies its `rp2040`. The `#[path]`s keep the files flat in `src/`.
#[cfg(feature = "rp2350")]
#[path = "registers_rp2350.rs"]
mod registers_chip;

#[cfg(not(feature = "rp2350"))]
#[path = "registers_rp2040.rs"]
mod registers_chip;

pub use registers::*;
pub use registers_chip::*;

pub use rs_mcu_traits::rcu::Rcu as RcuTrait;

unsafe extern "C" {
    fn clock_get_hz(clk_index: u32) -> u32;
}

/// Take a peripheral out of reset and enable its clocks if applicable.
pub fn enable(periph: Peripheral) {
    // If it's a peripheral requiring clk_peri, ensure CLK_PERI is enabled
    match periph {
        Peripheral::Spi0 | Peripheral::Spi1 | Peripheral::Uart0 | Peripheral::Uart1 => {
            let clk_peri_ctrl = (CLOCKS_BASE + CLOCKS_CLK_PERI_CTRL_OFFSET) as *mut u32;
            unsafe {
                let val = core::ptr::read_volatile(clk_peri_ctrl);
                if (val & CLOCKS_CLK_PERI_CTRL_ENABLE_MASK) == 0 {
                    core::ptr::write_volatile(
                        clk_peri_ctrl,
                        val | CLOCKS_CLK_PERI_CTRL_ENABLE_MASK,
                    );
                }
            }
        }
        _ => {}
    }

    if let Some(mask) = get_reset_bit(periph) {
        unsafe {
            // Atomic clear on RESET register: write mask to CLR alias
            let reset_clr = (RESETS_BASE + ATOMIC_CLR_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            let done_reg = (RESETS_BASE + RESETS_RESET_DONE_OFFSET) as *const u32;
            core::ptr::write_volatile(reset_clr, mask);
            while (core::ptr::read_volatile(done_reg) & mask) != mask {
                core::hint::spin_loop();
            }
        }
    }
}

/// Assert reset on the peripheral (placing it into reset/disabled state).
pub fn disable(periph: Peripheral) {
    if let Some(mask) = get_reset_bit(periph) {
        unsafe {
            // Atomic set on RESET register: write mask to SET alias
            let reset_set = (RESETS_BASE + ATOMIC_SET_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            core::ptr::write_volatile(reset_set, mask);
        }
    }
}

/// Reset the peripheral: assert reset, then deassert and wait for RESET_DONE.
pub fn reset(periph: Peripheral) {
    if let Some(mask) = get_reset_bit(periph) {
        unsafe {
            let reset_set = (RESETS_BASE + ATOMIC_SET_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            let reset_clr = (RESETS_BASE + ATOMIC_CLR_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            let done_reg = (RESETS_BASE + RESETS_RESET_DONE_OFFSET) as *const u32;
            core::ptr::write_volatile(reset_set, mask);
            core::ptr::write_volatile(reset_clr, mask);
            while (core::ptr::read_volatile(done_reg) & mask) != mask {
                core::hint::spin_loop();
            }
        }
    }
}

/// Query the peripheral clock frequency in Hz.
pub fn get_clock(periph: Peripheral) -> u32 {
    match periph {
        Peripheral::Spi0 | Peripheral::Spi1 | Peripheral::Uart0 | Peripheral::Uart1 => {
            let hz = unsafe { clock_get_hz(CLK_PERI_INDEX) };
            if hz == 0 {
                DEFAULT_PERI_CLOCK_HZ
            } else {
                hz
            }
        }
        Peripheral::SysClock
        | Peripheral::Apb1
        | Peripheral::Apb2
        | Peripheral::Dma0
        | Peripheral::Dma1
        | Peripheral::I2c0
        | Peripheral::I2c1 => {
            let hz = unsafe { clock_get_hz(CLK_SYS_INDEX) };
            if hz == 0 {
                DEFAULT_SYS_CLOCK_HZ
            } else {
                hz
            }
        }
        Peripheral::Usb => {
            let hz = unsafe { clock_get_hz(CLK_USB_INDEX) };
            if hz == 0 {
                DEFAULT_USB_CLOCK_HZ
            } else {
                hz
            }
        }
        Peripheral::Adc0 | Peripheral::Adc1 => {
            let hz = unsafe { clock_get_hz(CLK_ADC_INDEX) };
            if hz == 0 {
                DEFAULT_ADC_CLOCK_HZ
            } else {
                hz
            }
        }
        Peripheral::Timer0 | Peripheral::Timer1 => DEFAULT_TIMER_CLOCK_HZ,
        _ => {
            let hz = unsafe { clock_get_hz(CLK_SYS_INDEX) };
            if hz == 0 {
                DEFAULT_SYS_CLOCK_HZ
            } else {
                hz
            }
        }
    }
}

/// Struct implementing the `rs_mcu_traits::rcu::Rcu` super trait.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rcu;

/// The peripherals of the RP2xxx family that have a reset gate and/or a clock of
/// their own.
///
/// Family-owned on purpose: the contract (`rs_mcu_traits::rcu::Rcu`) only knows
/// `Self::Peripheral`, so nothing GD32-shaped (`UsbHsCh32v3x`, `Can0`, …) leaks
/// into the RP2040/RP2350 backend, and vice versa (supertraits.md §2, rule R2).
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Peripheral {
    None = 0,
    /// `ADC` (one block, shared by both "instances" of the old enum).
    Adc0,
    Adc1,
    /// Both DMA engines drive the single `DMA` block.
    Dma0,
    Dma1,
    I2c0,
    I2c1,
    /// `IO_BANK0` / `PADS_BANK0`: the GPIO pads of every port.
    GpioA,
    GpioB,
    GpioC,
    GpioD,
    GpioE,
    Spi0,
    Spi1,
    /// `TIMER0`/`TIMER1` on the RP2350, the single `TIMER` on the RP2040.
    Timer0,
    Timer1,
    Uart0,
    Uart1,
    Usb,

    /// Clock-only entries: they have a frequency but no reset bit.
    Apb1 = 100,
    Apb2,
    SysClock,
}

impl rs_mcu_traits::rcu::Rcu for Rcu {
    type Peripheral = Peripheral;

    #[inline(always)]
    fn enable(periph: Peripheral) {
        enable(periph);
    }

    #[inline(always)]
    fn disable(periph: Peripheral) {
        disable(periph);
    }

    #[inline(always)]
    fn reset(periph: Peripheral) {
        reset(periph);
    }

    #[inline(always)]
    fn clock_hz(periph: Peripheral) -> u32 {
        get_clock(periph)
    }
}

impl Rcu {
    #[inline(always)]
    pub fn enable(periph: Peripheral) {
        enable(periph);
    }

    #[inline(always)]
    pub fn disable(periph: Peripheral) {
        disable(periph);
    }

    #[inline(always)]
    pub fn reset(periph: Peripheral) {
        reset(periph);
    }

    #[inline(always)]
    pub fn get_clock(periph: Peripheral) -> u32 {
        get_clock(periph)
    }
}
