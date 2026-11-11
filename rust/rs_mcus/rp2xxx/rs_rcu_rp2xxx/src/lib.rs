#![no_std]

//! Reset and Clock Unit (RCU) driver for RP2040 and RP2350.
//!
//! Implements the `rs_esprit::rcu::Rcu` super trait, handling peripheral reset
//! and clock management via the RP2xxx RESETS and CLOCKS blocks.

pub mod registers;
pub mod shim;

pub use registers::*;
pub use shim::*;

pub use rs_esprit::rcu::{Peripheral, Rcu as RcuTrait};

unsafe extern "C" {
    fn clock_get_hz(clk_index: u32) -> u32;
}

/// Detect if running on RP2040 (PART == 2) or RP2350.
#[inline(always)]
pub fn is_rp2040() -> bool {
    #[cfg(feature = "rp2040")]
    {
        true
    }
    #[cfg(feature = "rp2350")]
    {
        false
    }
    #[cfg(not(any(feature = "rp2040", feature = "rp2350")))]
    {
        let chip_id = unsafe {
            core::ptr::read_volatile(
                (rs_rp2xxx::common::SYSINFO_BASE + SYSINFO_CHIP_ID_OFFSET) as *const u32,
            )
        };
        let part = (chip_id >> SYSINFO_CHIP_ID_PART_LSB) & SYSINFO_CHIP_ID_PART_MASK;
        part == SYSINFO_CHIP_ID_PART_RP2040
    }
}

/// Dynamic or cfg-based RESETS peripheral base address.
#[inline(always)]
pub fn resets_base() -> u32 {
    if is_rp2040() {
        rs_rp2xxx::rp2040::RESETS_BASE
    } else {
        rs_rp2xxx::rp2350::RESETS_BASE
    }
}

/// Dynamic or cfg-based CLOCKS peripheral base address.
#[inline(always)]
pub fn clocks_base() -> u32 {
    if is_rp2040() {
        rs_rp2xxx::rp2040::CLOCKS_BASE
    } else {
        rs_rp2xxx::rp2350::CLOCKS_BASE
    }
}

/// Map a high-level Peripheral identifier to its bitmask in the RESETS register.
pub fn get_reset_bit(periph: Peripheral) -> Option<u32> {
    if is_rp2040() {
        match periph {
            Peripheral::Spi0 => Some(rp2040_resets::SPI0),
            Peripheral::Spi1 => Some(rp2040_resets::SPI1),
            Peripheral::Dma0 | Peripheral::Dma1 => Some(rp2040_resets::DMA),
            Peripheral::Adc0 | Peripheral::Adc1 => Some(rp2040_resets::ADC),
            Peripheral::I2c0 => Some(rp2040_resets::I2C0),
            Peripheral::I2c1 => Some(rp2040_resets::I2C1),
            Peripheral::Uart0 => Some(rp2040_resets::UART0),
            Peripheral::Uart1 => Some(rp2040_resets::UART1),
            Peripheral::Timer0 | Peripheral::Timer1 => Some(rp2040_resets::TIMER),
            Peripheral::Usb => Some(rp2040_resets::USBCTRL),
            Peripheral::GpioA
            | Peripheral::GpioB
            | Peripheral::GpioC
            | Peripheral::GpioD
            | Peripheral::GpioE => Some(rp2040_resets::IO_BANK0),
            _ => None,
        }
    } else {
        match periph {
            Peripheral::Spi0 => Some(rp2350_resets::SPI0),
            Peripheral::Spi1 => Some(rp2350_resets::SPI1),
            Peripheral::Dma0 | Peripheral::Dma1 => Some(rp2350_resets::DMA),
            Peripheral::Adc0 | Peripheral::Adc1 => Some(rp2350_resets::ADC),
            Peripheral::I2c0 => Some(rp2350_resets::I2C0),
            Peripheral::I2c1 => Some(rp2350_resets::I2C1),
            Peripheral::Uart0 => Some(rp2350_resets::UART0),
            Peripheral::Uart1 => Some(rp2350_resets::UART1),
            Peripheral::Timer0 => Some(rp2350_resets::TIMER0),
            Peripheral::Timer1 => Some(rp2350_resets::TIMER1),
            Peripheral::Usb => Some(rp2350_resets::USBCTRL),
            Peripheral::GpioA
            | Peripheral::GpioB
            | Peripheral::GpioC
            | Peripheral::GpioD
            | Peripheral::GpioE => Some(rp2350_resets::IO_BANK0),
            _ => None,
        }
    }
}

/// Take a peripheral out of reset and enable its clocks if applicable.
pub fn enable(periph: Peripheral) {
    // If it's a peripheral requiring clk_peri, ensure CLK_PERI is enabled
    match periph {
        Peripheral::Spi0 | Peripheral::Spi1 | Peripheral::Uart0 | Peripheral::Uart1 => {
            let clk_peri_ctrl = (clocks_base() + CLOCKS_CLK_PERI_CTRL_OFFSET) as *mut u32;
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
            let reset_clr = (resets_base() + ATOMIC_CLR_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            let done_reg = (resets_base() + RESETS_RESET_DONE_OFFSET) as *const u32;
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
            let reset_set = (resets_base() + ATOMIC_SET_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            core::ptr::write_volatile(reset_set, mask);
        }
    }
}

/// Reset the peripheral: assert reset, then deassert and wait for RESET_DONE.
pub fn reset(periph: Peripheral) {
    if let Some(mask) = get_reset_bit(periph) {
        unsafe {
            let reset_set = (resets_base() + ATOMIC_SET_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            let reset_clr = (resets_base() + ATOMIC_CLR_OFFSET + RESETS_RESET_OFFSET) as *mut u32;
            let done_reg = (resets_base() + RESETS_RESET_DONE_OFFSET) as *const u32;
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
    let rp2040 = is_rp2040();
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
            let idx = if rp2040 {
                CLK_USB_INDEX_RP2040
            } else {
                CLK_USB_INDEX_RP2350
            };
            let hz = unsafe { clock_get_hz(idx) };
            if hz == 0 {
                DEFAULT_USB_CLOCK_HZ
            } else {
                hz
            }
        }
        Peripheral::Adc0 | Peripheral::Adc1 => {
            let idx = if rp2040 {
                CLK_ADC_INDEX_RP2040
            } else {
                CLK_ADC_INDEX_RP2350
            };
            let hz = unsafe { clock_get_hz(idx) };
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

/// Struct implementing the `rs_esprit::rcu::Rcu` super trait.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Rcu;

impl rs_esprit::rcu::Rcu for Rcu {
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
    fn get_clock(periph: Peripheral) -> u32 {
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
