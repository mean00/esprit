#![allow(unused_variables, unused_imports, unused_parens)]
#![allow(non_camel_case_types, non_snake_case, dead_code)]

#![no_std]

mod registers;
use registers::GpioRegisters;

use rs_bluepill::{
    GPIOA_BASE, GPIOB_BASE, GPIOC_BASE, GPIOD_BASE, GPIOE_BASE, GPIOF_BASE,
    GPIOG_BASE,
};
use core::ptr::{read_volatile, write_volatile};

// --- Idiomatic Rust Core ---

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Mode {
    Floating = 0,
    InputFloating = 1,
    InputPullup = 2,
    InputPulldown = 3,
    Output = 4,
    OutputOpenDrain = 5,
    AlternatePushPull = 6,
    AlternateOpenDrain = 7,
    Pwm = 8,
    AdcMode = 9,
    DacMode = 10,
    Uart = 11,
    SpiMode = 12,
    UartAlt = 13,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Pin {
    PA0 = 0, PA1 = 1, PA2 = 2, PA3 = 3, PA4 = 4, PA5 = 5, PA6 = 6, PA7 = 7,
    PA8 = 8, PA9 = 9, PA10 = 10, PA11 = 11, PA12 = 12, PA13 = 13, PA14 = 14, PA15 = 15,
    PB0 = 16, PB1 = 17, PB2 = 18, PB3 = 19, PB4 = 20, PB5 = 21, PB6 = 22, PB7 = 23,
    PB8 = 24, PB9 = 25, PB10 = 26, PB11 = 27, PB12 = 28, PB13 = 29, PB14 = 30, PB15 = 31,
    PC0 = 32, PC1 = 33, PC2 = 34, PC3 = 35, PC4 = 36, PC5 = 37, PC6 = 38, PC7 = 39,
    PC8 = 40, PC9 = 41, PC10 = 42, PC11 = 43, PC12 = 44, PC13 = 45, PC14 = 46, PC15 = 47,
    PD0 = 48, PD1 = 49, PD2 = 50, PD3 = 51, PD4 = 52, PD5 = 53, PD6 = 54, PD7 = 55,
    PD8 = 56, PD9 = 57, PD10 = 58, PD11 = 59, PD12 = 60, PD13 = 61, PD14 = 62, PD15 = 63,
    PE0 = 64, PE1 = 65, PE2 = 66, PE3 = 67, PE4 = 68, PE5 = 69, PE6 = 70, PE7 = 71,
    PE8 = 72, PE9 = 73, PE10 = 74, PE11 = 75, PE12 = 76, PE13 = 77, PE14 = 78, PE15 = 79,
    PF0 = 80, PF1 = 81, PF2 = 82, PF3 = 83, PF4 = 84, PF5 = 85, PF6 = 86, PF7 = 87,
    PF8 = 88, PF9 = 89, PF10 = 90, PF11 = 91, PF12 = 92, PF13 = 93, PF14 = 94, PF15 = 95,
    PG0 = 96, PG1 = 97, PG2 = 98, PG3 = 99, PG4 = 100, PG5 = 101, PG6 = 102, PG7 = 103,
    PG8 = 104, PG9 = 105, PG10 = 106, PG11 = 107, PG12 = 108, PG13 = 109, PG14 = 110, PG15 = 111,
}

impl From<u32> for Pin {
    #[inline]
    fn from(val: u32) -> Self {
        unsafe { core::mem::transmute(val) }
    }
}

const PORTS: [u32; 7] = [GPIOA_BASE, GPIOB_BASE, GPIOC_BASE, GPIOD_BASE, GPIOE_BASE, GPIOF_BASE, GPIOG_BASE];

#[inline]
pub(crate) fn get_port_ptr(port: u32) -> *mut GpioRegisters {
    PORTS[port as usize] as *mut GpioRegisters
}

#[inline]
fn get_ptr(pin: Pin) -> *mut GpioRegisters {
    get_port_ptr((pin as u32) >> 4)
}

#[inline]
fn get_bit(pin: Pin) -> u32 {
    (pin as u32) & 0xF
}

impl Pin {
    #[inline(always)]
    pub fn write(self, value: bool) {
        write(self, value);
    }

    #[inline(always)]
    pub fn read(self) -> bool {
        read(self)
    }

    #[inline(always)]
    pub fn toggle(self) {
        toggle(self);
    }

    #[inline(always)]
    pub fn set_mode(self, mode: Mode, speed_in_mhz: u32) {
        set_mode(self, mode, speed_in_mhz);
    }

    #[inline(always)]
    pub fn open_drain_close(self, close: bool) {
        open_drain_close(self, close);
    }
}

/// Idiomatic struct interface for GPIO peripheral operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Gpio;

impl Gpio {
    #[inline(always)]
    pub fn write(pin: Pin, value: bool) {
        write(pin, value);
    }

    #[inline(always)]
    pub fn read(pin: Pin) -> bool {
        read(pin)
    }

    #[inline(always)]
    pub fn toggle(pin: Pin) {
        toggle(pin);
    }

    #[inline(always)]
    pub fn read_port(port: u32) -> u32 {
        read_port(port)
    }

    #[inline(always)]
    pub fn set_mode(pin: Pin, mode: Mode, speed_in_mhz: u32) {
        set_mode(pin, mode, speed_in_mhz);
    }
}

#[inline]
pub fn write(pin: Pin, value: bool) {
    let port = get_ptr(pin);
    let bit = get_bit(pin);
    unsafe {
        if value {
            write_volatile(&mut (*port).bop, 1 << bit);
        } else {
            write_volatile(&mut (*port).bc, 1 << bit);
        }
    }
}

#[inline]
pub fn read(pin: Pin) -> bool {
    let port = get_ptr(pin);
    let bit = get_bit(pin);
    unsafe {
        (read_volatile(&mut (*port).istat) & (1 << bit)) != 0
    }
}

#[inline]
pub fn toggle(pin: Pin) {
    let port = get_ptr(pin);
    let bit = get_bit(pin);
    unsafe {
        let mut val = read_volatile(&mut (*port).octl);
        val ^= 1 << bit;
        write_volatile(&mut (*port).octl, val);
    }
}

#[inline]
pub fn open_drain_close(pin: Pin, close: bool) {
    write(pin, !close);
}

#[inline]
pub fn read_port(port: u32) -> u32 {
    unsafe {
        read_volatile(&mut (*get_port_ptr(port)).istat)
    }
}

pub fn set_mode(pin: Pin, mode: Mode, speed_in_mhz: u32) {
    use registers::*;
    let port = get_ptr(pin);
    let mut bit = get_bit(pin);

    let speed = if speed_in_mhz == 0 || speed_in_mhz >= 50 {
        SPEED_50MHZ
    } else if speed_in_mhz >= 10 {
        SPEED_10MHZ
    } else {
        SPEED_2MHZ
    };

    let val = match mode {
        Mode::AdcMode | Mode::DacMode => gpio_set(CTL_MD_INPUT, CTL_INPUT_ANALOG),
        Mode::Floating | Mode::InputFloating => gpio_set(CTL_MD_INPUT, CTL_INPUT_FLOATING),
        Mode::InputPullup => {
            unsafe { write_volatile(&mut (*port).bop, 1 << bit) };
            gpio_set(CTL_MD_INPUT, CTL_INPUT_PULLUP_PULLDOWN)
        }
        Mode::InputPulldown => {
            unsafe { write_volatile(&mut (*port).bc, 1 << bit) };
            gpio_set(CTL_MD_INPUT, CTL_INPUT_PULLUP_PULLDOWN)
        }
        Mode::Output => gpio_set(speed, CTL_OUTPUT_PP),
        Mode::OutputOpenDrain => gpio_set(speed, CTL_OUTPUT_OD),
        Mode::AlternatePushPull | Mode::Pwm => gpio_set(speed, CTL_OUTPUT_ALTERNATE_PP),
        Mode::AlternateOpenDrain => gpio_set(speed, CTL_OUTPUT_ALTERNATE_OD),
        _ => gpio_set(CTL_MD_INPUT, CTL_INPUT_FLOATING),
    };

    unsafe {
        if bit > 7 {
            bit &= 7;
            let mut ref_val = read_volatile(&mut (*port).ctl1);
            ref_val &= !(0xF << (bit * 4));
            ref_val |= val << (bit * 4);
            write_volatile(&mut (*port).ctl1, ref_val);
        } else {
            let mut ref_val = read_volatile(&mut (*port).ctl0);
            ref_val &= !(0xF << (bit * 4));
            ref_val |= val << (bit * 4);
            write_volatile(&mut (*port).ctl0, ref_val);
        }
    }
}

pub mod shim;
pub use shim::*;
