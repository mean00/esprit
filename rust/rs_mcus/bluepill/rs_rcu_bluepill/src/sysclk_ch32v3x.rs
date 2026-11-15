//! CH32V3x system clock (PLL/HSE/HSI) and bus-clock query (`drop_c_part2.md` Phase C2).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/lnRCU_systemClock_riscv_ch32v3x.cpp`
//! in `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::registers::{
    LN_CLOCK_IRC8, LN_CLOCK_PLL, LN_CLOCK_XTAL, LN_RCU_CFG0_SYSCLOCK_MASK,
    LN_RCU_CFG0_SYSCLOCK_PLL, LN_RCU_CTL_HXTALEN, LN_RCU_CTL_HXTASTB, LN_RCU_CTL_PLLEN,
    LN_RCU_CTL_PLLSTB, RcuRegisters,
};
use crate::registers_chip::{
    CH32_CFG0_AHB_MASK, CH32_CFG0_APB1_MASK, CH32_CFG0_APB2_MASK, CH32_CFG0_PLL_EXTERNAL,
    CH32_CFG0_PLL_MASK,
};
use crate::{parse_u32, Peripheral};

const CLOCK_XTAL_VALUE: u32 = parse_u32(option_env!("LN_XTAL_VALUE"), 8);
const CLOCK_TARGET_SYSCLOCK_MHZ: u32 =
    parse_u32(option_env!("LN_MCU_SPEED"), 144_000_000) / 1_000_000;

// 0 = APB1 is sysclock, 1 = sysclock / 2
const CH32_APB1_DIVIDER: u32 = 0;

static MULTIPLIERS: [u8; 19] = [
    0, 0, 0, 1, // 0 1 2 3
    2, 3, 4, 5, // 4 5 6 7
    6, 7, 8, 9, // 8 9 10 11
    10, 11, 12, 14, // 12 13 14 15
    15, 0, 0, // 16 17 18 (18 -> 0 = x18 = 144 MHz)
];

#[unsafe(no_mangle)]
pub static _rcuClockApb1: AtomicU32 = AtomicU32::new(108_000_000 / 2);
#[unsafe(no_mangle)]
pub static _rcuClockApb2: AtomicU32 = AtomicU32::new(108_000_000);
#[unsafe(no_mangle)]
pub static SystemCoreClock: AtomicU32 = AtomicU32::new(0);

unsafe extern "C" {
    #[link_name = "\u{1}_ZN7lnCpuID8identifyEv"]
    fn ln_cpuid_identify();
}

/// `uint32_t lnPeripherals::getClock(const Peripherals periph)`
#[unsafe(export_name = "_ZN13lnPeripherals8getClockE11Peripherals")]
pub extern "C" fn get_clock_c(periph: Peripheral) -> u32 {
    let apb1 = _rcuClockApb1.load(Ordering::Relaxed);
    let apb2 = _rcuClockApb2.load(Ordering::Relaxed);
    match periph {
        Peripheral::Uart0 | Peripheral::Timer0 | Peripheral::Spi0 | Peripheral::Afio => apb2,
        Peripheral::Apb1 => apb1,
        Peripheral::Apb2 => apb2,
        Peripheral::SysClock => SystemCoreClock.load(Ordering::Relaxed),
        Peripheral::Timer1
        | Peripheral::Timer2
        | Peripheral::Timer3
        | Peripheral::Timer4
        | Peripheral::Timer5
        | Peripheral::Timer6 => {
            if CH32_APB1_DIVIDER == 0 {
                apb1
            } else {
                apb1 * 2
            }
        }
        _ => apb1,
    }
}

#[inline(always)]
fn wait_control_bit(mask: u32) {
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    while unsafe { read_reg!(ctl) } & mask == 0 {
        core::hint::spin_loop();
    }
}

#[inline(always)]
fn wait_cfg0_bit(mask: u32) {
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    while unsafe { read_reg!(cfg0) } & mask == 0 {
        core::hint::spin_loop();
    }
}

fn enable_disable_clock(clock_bit: u32, enabled: bool) {
    let set_bit = 1u32 << clock_bit;
    let status_bit = 1u32 << (clock_bit + 1);
    if enabled {
        // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
        unsafe { modify_reg!(ctl, |v| v | set_bit) };
        wait_control_bit(status_bit);
    } else {
        // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
        unsafe { modify_reg!(ctl, |v| v & !set_bit) };
    }
}

fn set_pll(multiplier: usize, external: bool) {
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe { write_reg!(cfg1, 0) };
    assert!(multiplier < MULTIPLIERS.len());
    let mut pll_multiplier = MULTIPLIERS[multiplier] as u32;

    // SAFETY: `CH32_CHIPID_BASE` (`0x1FFF_F704`) is the read-only CH32 device ID register.
    let chipid = unsafe { core::ptr::read_volatile(rs_bluepill::CH32_CHIPID_BASE as *const u32) };
    let family = (chipid >> 20) & 0xFFF;
    if multiplier == 18 {
        if family == 0x305 || family == 0x307 {
            pll_multiplier = 0; // D8C: 0000 = x18 (144 MHz)
        } else {
            pll_multiplier = 15; // D8: 1111 = x18 (144 MHz)
        }
    }

    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe {
        modify_reg!(cfg0, |v| {
            let mut c0 = (v & !CH32_CFG0_PLL_MASK) | (pll_multiplier << 18);
            if external {
                c0 |= CH32_CFG0_PLL_EXTERNAL;
            } else {
                c0 &= !CH32_CFG0_PLL_EXTERNAL;
            }
            c0
        });
        modify_reg!(ctl, |v| v | LN_RCU_CTL_PLLEN);
    }
    wait_control_bit(LN_RCU_CTL_PLLSTB);
}

/// `void lnInitSystemClock()`
#[unsafe(export_name = "_Z17lnInitSystemClockv")]
pub extern "C" fn ln_init_system_clock() {
    // SAFETY: Initializes CPUID metadata once at boot.
    unsafe { ln_cpuid_identify() };

    enable_disable_clock(LN_CLOCK_IRC8, true);
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe { modify_reg!(cfg0, |v| v & !LN_RCU_CFG0_SYSCLOCK_MASK) };
    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    while (unsafe { read_reg!(cfg0) } >> 2) & 3 != 0 {
        core::hint::spin_loop();
    }

    enable_disable_clock(LN_CLOCK_XTAL, false);
    enable_disable_clock(LN_CLOCK_PLL, false);

    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe { modify_reg!(ctl, |v| v | LN_RCU_CTL_HXTALEN) };
    wait_control_bit(LN_RCU_CTL_HXTASTB);
    let input_clock = CLOCK_XTAL_VALUE;
    let predivider = 1u32;

    let multiplier = (predivider * CLOCK_TARGET_SYSCLOCK_MHZ) / input_clock;
    set_pll(multiplier as usize, true);

    let sysclk = ((input_clock * multiplier) / predivider) * 1_000_000;
    SystemCoreClock.store(sysclk, Ordering::Relaxed);
    _rcuClockApb1.store(sysclk >> CH32_APB1_DIVIDER, Ordering::Relaxed);
    _rcuClockApb2.store(sysclk, Ordering::Relaxed);

    // SAFETY: `RcuRegisters::ptr()` points to the memory-mapped RCU block.
    unsafe {
        modify_reg!(cfg0, |v| {
            let mut clks = v & !(CH32_CFG0_AHB_MASK | CH32_CFG0_APB1_MASK | CH32_CFG0_APB2_MASK);
            clks |= (CH32_APB1_DIVIDER * 4) << 8;
            clks &= !LN_RCU_CFG0_SYSCLOCK_MASK;
            clks |= LN_RCU_CFG0_SYSCLOCK_PLL;
            clks
        });
    }
    wait_cfg0_bit(LN_RCU_CFG0_SYSCLOCK_PLL << 2);
}
