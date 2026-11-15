//! GD32F3x / STM32F1 system clock (PLL/HXTAL/IRC8) and bus-clock query
//! (`drop_c_part3.md` Phase G2).
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/lnRCU_systemClock_arm_gd32fx.cpp` in
//! `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::registers::{
    DBG_CTL0_HALT_TIMERS_I2C, FMC_WS_PREFETCH_BASE, LN_CLOCK_IRC8, LN_CLOCK_PLL, LN_CLOCK_XTAL,
    LN_RCU_CFG0_AHBPSC_MASK, LN_RCU_CFG0_APB1PSC_DIV2, LN_RCU_CFG0_APB1PSC_MASK,
    LN_RCU_CFG0_APB2PSC_MASK, LN_RCU_CFG0_PLLMF_MASK, LN_RCU_CFG0_PLLMF_MSB,
    LN_RCU_CFG0_PLLMF_SHIFT, LN_RCU_CFG0_PLLSEL, LN_RCU_CFG0_SYSCLOCK_MASK,
    LN_RCU_CFG0_SYSCLOCK_PLL, LN_RCU_CTL_HXTALEN, LN_RCU_CTL_HXTASTB, LN_RCU_CTL_PLLEN,
    LN_RCU_CTL_PLLSTB, LN_RCU_RSTCLK_RSTFC, RcuRegisters,
};
use crate::{parse_u32, Peripheral};

const LN_VENDOR_GD: u32 = 2;
const CLOCK_TARGET_PREDIV: u32 = 1;

const CLOCK_XTAL_VALUE_MHZ: u32 = parse_u32(option_env!("LN_XTAL_VALUE"), 8);
const CLOCK_TARGET_SYSCLOCK_MHZ: u32 = parse_u32(option_env!("LN_MCU_SPEED"), 96_000_000) / 1_000_000;

static MULTIPLIERS: [u8; 25] = [
    0, 0, 0, 1, // 0 1 2 3
    2, 3, 4, 5, // 4 5 6 7
    6, 7, 8, 9, // 8 9 10 11
    // the ones below don't work on stm32
    10, 11, 12, 12, // 12 13 14 15
    14, 16, 17, 18, // 16 17 18 19
    // needed for internal clock
    19, 20, 21, 22, 23, // 20 21 22 23 24 (96 MHz)
];

#[unsafe(no_mangle)]
pub static _rcuClockApb1: AtomicU32 = AtomicU32::new(108_000_000 / 2);
#[unsafe(no_mangle)]
pub static _rcuClockApb2: AtomicU32 = AtomicU32::new(108_000_000);
#[unsafe(no_mangle)]
pub static SystemCoreClock: AtomicU32 = AtomicU32::new(0);

#[repr(C)]
struct DbgRegisters {
    id: u32,
    ctl0: u32,
}

const _: () = assert!(core::mem::offset_of!(DbgRegisters, ctl0) == 0x04);

unsafe extern "C" {
    #[link_name = "\u{1}_ZN7lnCpuID8identifyEv"]
    fn ln_cpuid_identify();
    #[link_name = "\u{1}_ZN7lnCpuID6vendorEv"]
    fn ln_cpuid_vendor() -> u32;
}

/// `uint32_t lnPeripherals::getClock(const Peripherals periph)`
#[unsafe(export_name = "_ZN13lnPeripherals8getClockE11Peripherals")]
pub extern "C" fn get_clock_c(periph: Peripheral) -> u32 {
    let apb1 = _rcuClockApb1.load(Ordering::Relaxed);
    let apb2 = _rcuClockApb2.load(Ordering::Relaxed);
    match periph {
        Peripheral::Timer1
        | Peripheral::Timer2
        | Peripheral::Timer3
        | Peripheral::Timer4
        | Peripheral::Timer5
        | Peripheral::Timer6 => apb1 * 2,
        Peripheral::Uart0
        | Peripheral::Timer0
        | Peripheral::Spi0
        | Peripheral::Afio
        | Peripheral::Adc0
        | Peripheral::Adc1 => apb2,
        Peripheral::Apb1 => apb1,
        Peripheral::Apb2 => apb2,
        Peripheral::SysClock => SystemCoreClock.load(Ordering::Relaxed),
        _ => apb1,
    }
}

fn wait_ctl_bit(mask: u32) {
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe {
        while (read_reg!(ctl) & mask) == 0 {
            core::hint::spin_loop();
        }
    }
}

fn enable_disable_clock(clock_bit: u32, enabled: bool) {
    let set_bit = 1u32 << clock_bit;
    let status_bit = 1u32 << (clock_bit + 1);
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block;
    // called during single-threaded early clock init.
    unsafe {
        if enabled {
            modify_reg!(ctl, |v| v | set_bit);
            wait_ctl_bit(status_bit);
        } else {
            modify_reg!(ctl, |v| v & !set_bit);
        }
    }
}

fn set_pll(input_clock_mhz: u32, multiplier: u32, predivider: u32, external: bool) {
    let idx = multiplier as usize;
    if idx >= MULTIPLIERS.len() {
        panic!("PLL multiplier out of range");
    }
    let pll_multiplier = MULTIPLIERS[idx] as u32;

    let sysclk = (input_clock_mhz * multiplier * 1_000_000) / predivider;
    SystemCoreClock.store(sysclk, Ordering::Relaxed);
    _rcuClockApb1.store(sysclk / 2, Ordering::Relaxed);
    _rcuClockApb2.store(sysclk, Ordering::Relaxed);

    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block;
    // called during single-threaded early clock init.
    unsafe {
        write_reg!(cfg1, 0);
        modify_reg!(cfg0, |mut c0| {
            c0 &= !LN_RCU_CFG0_PLLMF_MASK;
            c0 |= (pll_multiplier & 0x0F) << LN_RCU_CFG0_PLLMF_SHIFT;
            if (pll_multiplier & 0x10) != 0 {
                c0 |= LN_RCU_CFG0_PLLMF_MSB;
            } else {
                c0 &= !LN_RCU_CFG0_PLLMF_MSB;
            }
            if external {
                c0 |= LN_RCU_CFG0_PLLSEL;
            } else {
                c0 &= !LN_RCU_CFG0_PLLSEL;
            }
            c0
        });
        modify_reg!(ctl, |v| v | LN_RCU_CTL_PLLEN);
    }
    wait_ctl_bit(LN_RCU_CTL_PLLSTB);
}

/// Clear all RCU reset flags (`RSTCLK.RSTFC`).
pub fn clear_reset_flags() {
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block (`0x4002_1000`).
    unsafe {
        write_reg!(rstclk, LN_RCU_RSTCLK_RSTFC);
    }
}

/// `void lnInitSystemClock()`
#[unsafe(export_name = "_Z17lnInitSystemClockv")]
pub extern "C" fn ln_init_system_clock() {
    // SAFETY: `lnCpuID::identify()` initializes static CPU identification once at boot.
    unsafe {
        ln_cpuid_identify();
    }

    // Enable IRC8 and switch SYSCLK to IRC8 before reconfiguring PLL/HXTAL.
    enable_disable_clock(LN_CLOCK_IRC8, true);
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe {
        modify_reg!(cfg0, |v| v & !LN_RCU_CFG0_SYSCLOCK_MASK);
        while (read_reg!(cfg0) & (LN_RCU_CFG0_SYSCLOCK_MASK << 2)) != 0 {
            core::hint::spin_loop();
        }
    }
    enable_disable_clock(LN_CLOCK_XTAL, false);
    enable_disable_clock(LN_CLOCK_PLL, false);

    let (input_clock_mhz, use_external) = if option_env!("LN_USE_INTERNAL_CLOCK").is_some() {
        (8 / 2, false)
    } else {
        // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
        unsafe {
            modify_reg!(ctl, |v| v | LN_RCU_CTL_HXTALEN);
        }
        wait_ctl_bit(LN_RCU_CTL_HXTASTB);
        (CLOCK_XTAL_VALUE_MHZ, true)
    };

    let multiplier = CLOCK_TARGET_SYSCLOCK_MHZ / input_clock_mhz;
    set_pll(input_clock_mhz, multiplier, CLOCK_TARGET_PREDIV, use_external);

    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block;
    // sets AHB = SYSCLK, APB2 = AHB, APB1 = AHB/2.
    unsafe {
        modify_reg!(cfg0, |mut a| {
            a &= !LN_RCU_CFG0_AHBPSC_MASK;
            a &= !LN_RCU_CFG0_APB2PSC_MASK;
            a &= !LN_RCU_CFG0_APB1PSC_MASK;
            a |= LN_RCU_CFG0_APB1PSC_DIV2;
            a
        });
    }

    // SAFETY: `lnCpuID::vendor()` reads static initialized by `identify()`.
    let vendor = unsafe { ln_cpuid_vendor() };
    if vendor != LN_VENDOR_GD {
        let ws = if CLOCK_TARGET_SYSCLOCK_MHZ > 48 {
            2
        } else if CLOCK_TARGET_SYSCLOCK_MHZ > 24 {
            1
        } else {
            0
        };
        let fmc_ws = rs_bluepill::FMC_BASE as *mut u32;
        // SAFETY: `FMC_BASE` points to the Flash controller WS register (`0x4002_2000`).
        unsafe {
            core::ptr::write_volatile(fmc_ws, FMC_WS_PREFETCH_BASE + ws);
        }
    }

    // Switch system clock to PLL.
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe {
        modify_reg!(cfg0, |v| (v & !LN_RCU_CFG0_SYSCLOCK_MASK) | LN_RCU_CFG0_SYSCLOCK_PLL);
    }

    let dbg = rs_bluepill::DBG_BASE as *mut DbgRegisters;
    // SAFETY: `DBG_BASE` points to the Cortex-M DBGMCU block (`0xE004_2000`).
    unsafe {
        let _jtag_id = core::ptr::read_volatile(&(*dbg).id);
        core::ptr::write_volatile(&mut (*dbg).ctl0, DBG_CTL0_HALT_TIMERS_I2C);
    }
}
