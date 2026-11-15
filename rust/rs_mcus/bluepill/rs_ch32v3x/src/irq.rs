//! WCH PFIC interrupt controller & IRQ wrappers (`drop_c_part2.md` Phase C4).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/lnIRQ_riscv_ch32v3x.cpp` in
//! `LN_ENABLE_RUST` builds.

use core::ffi::c_char;
use core::ptr::{read_volatile, write_volatile};

use crate::registers::{
    CH32_SYSCR_HWSTKEN, CH32_SYSCR_HWSTKOVEN, CH32_SYSCR_INESTEN, CH32_SYSCR_MPTCFG_2NESTED,
    DEFAULT_PRIO, LN_IRQ_SW, LN_IRQ_SYSTICK, LN_RISCV_FPU_MODE_1, MTVEC_VECTORED_MODE,
    PFIC_GISR_NESTSTA_SHIFT, PFIC_SCTLR_SYSRESET, PFIC_SW_IRQ_NUM, PFIC_VTFADDR_ENABLE,
    PficRegs, SW_IRQ_PRIO, SYSTICK_PRIO,
};
use crate::vectors::VEC_TABLE;

const fn pfic_irq(x: i32) -> u8 {
    (x + 16) as u8
}

/// Maps `enum LnIRQ` (0..=79) to PFIC vector index (matches `_irqs[]` in
/// `lnIRQ_riscv_priv_ch32v3x.h`).
static LN_IRQ_TO_PFIC: [u8; 80] = [
    pfic_irq(-1), // 0: LN_IRQ_NONE
    pfic_irq(0),  // 1: LN_IRQ_WWDGT
    pfic_irq(1),  // 2: LN_IRQ_LVD
    pfic_irq(2),  // 3: LN_IRQ_TAMPER
    pfic_irq(3),  // 4: LN_IRQ_RTC
    pfic_irq(4),  // 5: LN_IRQ_FMC
    pfic_irq(5),  // 6: LN_IRQ_RCU_CTC
    pfic_irq(6),  // 7: LN_IRQ_EXTI0
    pfic_irq(7),  // 8: LN_IRQ_EXTI1
    pfic_irq(8),  // 9: LN_IRQ_EXTI2
    pfic_irq(9),  // 10: LN_IRQ_EXTI3
    pfic_irq(10), // 11: LN_IRQ_EXTI4
    pfic_irq(11), // 12: LN_IRQ_DMA0_Channel0
    pfic_irq(12), // 13: LN_IRQ_DMA0_Channel1
    pfic_irq(13), // 14: LN_IRQ_DMA0_Channel2
    pfic_irq(14), // 15: LN_IRQ_DMA0_Channel3
    pfic_irq(15), // 16: LN_IRQ_DMA0_Channel4
    pfic_irq(16), // 17: LN_IRQ_DMA0_Channel5
    pfic_irq(17), // 18: LN_IRQ_DMA0_Channel6
    pfic_irq(18), // 19: LN_IRQ_ADC0_1
    pfic_irq(19), // 20: LN_IRQ_USB_HP_CAN_TX
    pfic_irq(20), // 21: LN_IRQ_USB_LP_CAN_RX0
    pfic_irq(21), // 22: LN_IRQ_CAN0_RX1
    pfic_irq(22), // 23: LN_IRQ_CAN0_EWMC
    pfic_irq(23), // 24: LN_IRQ_EXTI5_9
    pfic_irq(24), // 25: LN_IRQ_TIMER0_BRK
    pfic_irq(25), // 26: LN_IRQ_TIMER0_UP
    pfic_irq(26), // 27: LN_IRQ_TIMER0_TRG_CMT
    pfic_irq(27), // 28: LN_IRQ_TIMER0_Channel
    pfic_irq(28), // 29: LN_IRQ_TIMER1
    pfic_irq(29), // 30: LN_IRQ_TIMER2
    pfic_irq(30), // 31: LN_IRQ_TIMER3
    pfic_irq(31), // 32: LN_IRQ_I2C0_EV
    pfic_irq(32), // 33: LN_IRQ_I2C0_ER
    pfic_irq(33), // 34: LN_IRQ_I2C1_EV
    pfic_irq(34), // 35: LN_IRQ_I2C1_ER
    pfic_irq(35), // 36: LN_IRQ_SPI0
    pfic_irq(36), // 37: LN_IRQ_SPI1
    pfic_irq(37), // 38: LN_IRQ_USART0
    pfic_irq(38), // 39: LN_IRQ_USART1
    pfic_irq(39), // 40: LN_IRQ_USART2
    pfic_irq(40), // 41: LN_IRQ_EXTI10_15
    pfic_irq(41), // 42: LN_IRQ_RTC_ALARM
    pfic_irq(42), // 43: LN_IRQ_USBFS_WKUP
    0,            // 44: LN_IRQ_EXMC
    pfic_irq(50), // 45: LN_IRQ_TIMER4
    pfic_irq(51), // 46: LN_IRQ_SPI2
    pfic_irq(52), // 47: LN_IRQ_UART3
    pfic_irq(53), // 48: LN_IRQ_UART4
    pfic_irq(54), // 49: LN_IRQ_TIMER5
    pfic_irq(55), // 50: LN_IRQ_TIMER6
    pfic_irq(56), // 51: LN_IRQ_DMA1_Channel0
    pfic_irq(57), // 52: LN_IRQ_DMA1_Channel1
    pfic_irq(58), // 53: LN_IRQ_DMA1_Channel2
    pfic_irq(59), // 54: LN_IRQ_DMA1_Channel3
    pfic_irq(60), // 55: LN_IRQ_DMA1_Channel4
    pfic_irq(61), // 56: LN_IRQ_ETH
    pfic_irq(62), // 57: LN_IRQ_ETH_WKUP
    0,            // 58: LN_IRQ_CAN1_TX
    0,            // 59: LN_IRQ_CAN1_RX0
    0,            // 60: LN_IRQ_CAN1_RX1
    0,            // 61: LN_IRQ_CAN1_EWMC
    0,            // 62: LN_IRQ_USBFS
    pfic_irq(69), // 63: LN_IRQ_USBHS
    pfic_irq(68), // 64: LN_IRQ_USBHS_WKUP
    pfic_irq(67), // 65: LN_IRQ_OTG_FS
    0,            // 66: LN_IRQ_USB_HP
    0,            // 67: LN_IRQ_USB_LP
    pfic_irq(47), // 68: LN_IRQ_RNG
    pfic_irq(48), // 69: LN_IRQ_FSMC
    pfic_irq(49), // 70: LN_IRQ_SDIO
    pfic_irq(82), // 71: LN_IRQ_DMA1_Channel5
    pfic_irq(83), // 72: LN_IRQ_DMA1_Channel6
    pfic_irq(84), // 73: LN_IRQ_DMA1_Channel7
    pfic_irq(85), // 74: LN_IRQ_DMA1_Channel8
    pfic_irq(86), // 75: LN_IRQ_DMA1_Channel9
    pfic_irq(87), // 76: LN_IRQ_DMA1_Channel10
    pfic_irq(88), // 77: LN_IRQ_DMA1_Channel11
    12,           // 78: LN_IRQ_SYSTICK
    14,           // 79: LN_IRQ_SW
];

#[inline(always)]
fn lookup_pfic(irq: u32) -> usize {
    let idx = irq as usize;
    assert!(idx < LN_IRQ_TO_PFIC.len());
    let num = LN_IRQ_TO_PFIC[idx] as usize;
    assert!(num != 0);
    num
}

#[unsafe(link_section = ".isr_code")]
fn enable_disable_direct(enable: bool, irq_num: usize) {
    let p = PficRegs::ptr();
    let reg = irq_num >> 5;
    let bit = 1u32 << (irq_num & 0x1F);
    // SAFETY: `PficRegs::ptr()` points to PFIC (`0xE000_E000`); `reg` is within `ienr`/`irer` bounds.
    unsafe {
        if enable {
            write_volatile(&mut (*p).ienr[reg], bit);
        } else {
            write_volatile(&mut (*p).irer[reg], bit);
        }
    }
}

#[unsafe(link_section = ".isr_code")]
fn set_priority_direct(irq_num: usize, prio: i32) {
    let p = PficRegs::ptr();
    let s = (irq_num & 3) * 8;
    let r = irq_num >> 2;
    // SAFETY: `PficRegs::ptr()` points to PFIC; `r` is within `iprioir` bounds.
    unsafe {
        let mut b = read_volatile(&(*p).iprioir[r]);
        b &= !(0xFF << s);
        b |= ((prio as u32) << 4) << s;
        write_volatile(&mut (*p).iprioir[r], b);
    }
}

#[unsafe(link_section = ".isr_code")]
fn promote_irq_to_fast(irq: usize, slot_1_to_4: usize) {
    assert!((1..=4).contains(&slot_1_to_4));
    let no = slot_1_to_4 - 1;
    let irq_num = lookup_pfic(irq as u32);
    let adr = VEC_TABLE.0[irq_num] as usize as u32;
    let p = PficRegs::ptr();
    // SAFETY: `PficRegs::ptr()` points to PFIC; `no` is in `0..4`.
    unsafe {
        write_volatile(&mut (*p).vtfidr[no], irq_num as u8);
        write_volatile(&mut (*p).vtfaddr[no], adr | PFIC_VTFADDR_ENABLE);
    }
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(export_name = "_Z12lnIrqSysInitv")]
pub extern "C" fn ln_irq_sys_init() {
    let p = PficRegs::ptr();
    // SAFETY: Early single-core PFIC initialization.
    unsafe {
        for i in 0..4 {
            write_volatile(&mut (*p).vtfidr[i], 0);
            write_volatile(&mut (*p).vtfaddr[i], 0);
        }
        let prio32 = (DEFAULT_PRIO << 4)
            | (DEFAULT_PRIO << (4 + 8))
            | (DEFAULT_PRIO << (4 + 16))
            | (DEFAULT_PRIO << (4 + 24));
        for i in 0..64 {
            write_volatile(&mut (*p).iprioir[i], prio32);
        }
    }

    promote_irq_to_fast(LN_IRQ_SYSTICK, 1);
    promote_irq_to_fast(LN_IRQ_SW, 2);
    set_priority_direct(lookup_pfic(LN_IRQ_SYSTICK as u32), SYSTICK_PRIO);
    set_priority_direct(lookup_pfic(LN_IRQ_SW as u32), SW_IRQ_PRIO);

    let syscr = CH32_SYSCR_HWSTKEN
        | CH32_SYSCR_INESTEN
        | CH32_SYSCR_MPTCFG_2NESTED
        | CH32_SYSCR_HWSTKOVEN;
    let mstatus = LN_RISCV_FPU_MODE_1;
    let mtvec = (VEC_TABLE.0.as_ptr() as u32) | MTVEC_VECTORED_MODE;

    // SAFETY: Configures QingKe CSRs (`syscr`, `mstatus`, `mtvec`) during early boot.
    unsafe {
        core::arch::asm!(
            "csrw 0x804, {syscr}",
            "csrw mstatus, {mstatus}",
            "csrw mtvec, {mtvec}",
            syscr = in(reg) syscr,
            mstatus = in(reg) mstatus,
            mtvec = in(reg) mtvec,
        );
    }
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(export_name = "_Z22xPortIsInsideInterruptv")]
pub extern "C" fn x_port_is_inside_interrupt() -> bool {
    // SAFETY: `PficRegs::ptr()` points to PFIC.
    let gisr = unsafe { read_volatile(&(*PficRegs::ptr()).gisr) };
    ((gisr >> PFIC_GISR_NESTSTA_SHIFT) & 1) != 0
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(export_name = "_Z17lnEnableInterruptRK5LnIRQ")]
pub extern "C" fn ln_enable_interrupt(irq: &u32) {
    enable_disable_direct(true, lookup_pfic(*irq));
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(export_name = "_Z18lnDisableInterruptRK5LnIRQ")]
pub extern "C" fn ln_disable_interrupt(irq: &u32) {
    enable_disable_direct(false, lookup_pfic(*irq));
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(export_name = "_Z16lnIrqSetPriorityRK5LnIRQi")]
pub extern "C" fn ln_irq_set_priority(irq: &u32, prio: i32) {
    set_priority_direct(lookup_pfic(*irq), prio);
}

#[unsafe(no_mangle)]
pub extern "C" fn NVIC_trigger_sw_irq() {
    let sw_irq = PFIC_SW_IRQ_NUM;
    // SAFETY: `PficRegs::ptr()` points to PFIC; triggers software IRQ 14.
    unsafe {
        write_volatile(
            &mut (*PficRegs::ptr()).ipsr[sw_irq >> 5],
            1u32 << (sw_irq & 0x1F),
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn NVIC_EnableIRQ(irq_num: i32) {
    enable_disable_direct(true, irq_num as usize);
}

unsafe extern "C" {
    fn Logger_crash(st: *const c_char);
    fn dmaIrqHandler(dma: i32, channel: i32);
    fn i2cIrqHandler(instance: i32, error: bool);
}

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn deadEnd(_code: i32) -> ! {
    unsafe {
        core::arch::asm!("ebreak", "csrci mstatus, 0x8");
        Logger_crash(c"**** CRASH *****".as_ptr());
        Logger_crash(c"**** CRASH *****".as_ptr());
        Logger_crash(c"**** CRASH *****".as_ptr());
        core::arch::asm!("ebreak");
        loop {
            core::arch::asm!("nop");
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Break_Point_Handler() {
    deadEnd(5);
}

#[unsafe(export_name = "_Z17lnSoftSystemResetv")]
pub extern "C" fn ln_soft_system_reset() -> ! {
    // SAFETY: `PficRegs::ptr()` points to PFIC; writes SYSRESET bit in SCTLR.
    unsafe {
        write_volatile(&mut (*PficRegs::ptr()).sctlr, PFIC_SCTLR_SYSRESET);
        loop {
            core::arch::asm!("nop");
        }
    }
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(no_mangle)]
pub extern "C" fn ETH_IRQHandler() {
    deadEnd(1);
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(no_mangle)]
pub extern "C" fn ETH_WKUP_IRQHandler() {
    deadEnd(1);
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(no_mangle)]
pub extern "C" fn I2C0_EV_IRQHandler() {
    unsafe { i2cIrqHandler(0, false) };
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(no_mangle)]
pub extern "C" fn I2C0_ERR_IRQHandler() {
    unsafe { i2cIrqHandler(0, true) };
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(no_mangle)]
pub extern "C" fn I2C1_EV_IRQHandler() {
    unsafe { i2cIrqHandler(1, false) };
}

#[unsafe(link_section = ".isr_code")]
#[unsafe(no_mangle)]
pub extern "C" fn I2C1_ERR_IRQHandler() {
    unsafe { i2cIrqHandler(1, true) };
}

macro_rules! dma_irq {
    ($name:ident, $d:expr, $c:expr) => {
        #[unsafe(link_section = ".isr_code")]
        #[unsafe(no_mangle)]
        pub extern "C" fn $name() {
            unsafe { dmaIrqHandler($d, $c) };
        }
    };
}

dma_irq!(DMA0_Channel0_IRQHandler, 0, 0);
dma_irq!(DMA0_Channel1_IRQHandler, 0, 1);
dma_irq!(DMA0_Channel2_IRQHandler, 0, 2);
dma_irq!(DMA0_Channel3_IRQHandler, 0, 3);
dma_irq!(DMA0_Channel4_IRQHandler, 0, 4);
dma_irq!(DMA0_Channel5_IRQHandler, 0, 5);
dma_irq!(DMA0_Channel6_IRQHandler, 0, 6);
dma_irq!(DMA1_Channel0_IRQHandler, 1, 0);
dma_irq!(DMA1_Channel1_IRQHandler, 1, 1);
dma_irq!(DMA1_Channel2_IRQHandler, 1, 2);
dma_irq!(DMA1_Channel3_IRQHandler, 1, 3);
dma_irq!(DMA1_Channel4_IRQHandler, 1, 4);
dma_irq!(DMA1_Channel5_IRQHandler, 1, 5);
dma_irq!(DMA1_Channel6_IRQHandler, 1, 6);

const DMA_IRQS: [[i32; 7]; 2] = [
    [12, 13, 14, 15, 16, 17, 18],
    [51, 52, 53, 54, 55, -1, -1],
];

#[unsafe(no_mangle)]
pub extern "C" fn lnDmaSetIrqEnabled(dma: core::ffi::c_int, channel: core::ffi::c_int, enable: bool) {
    if !(0..2).contains(&dma) || !(0..7).contains(&channel) {
        return;
    }
    let irq = DMA_IRQS[dma as usize][channel as usize];
    if irq < 0 {
        return;
    }
    let irq_u32 = irq as u32;
    if enable {
        ln_enable_interrupt(&irq_u32);
    } else {
        ln_disable_interrupt(&irq_u32);
    }
}

#[inline(always)]
pub fn in_isr() -> bool {
    x_port_is_inside_interrupt()
}

#[inline(always)]
pub fn enter_critical_from_isr() -> u32 {
    let old_mstatus: u32;
    // SAFETY: Disables `mstatus.MIE` and returns previous `mstatus`.
    unsafe {
        core::arch::asm!(
            "csrrci {old}, mstatus, 8",
            old = out(reg) old_mstatus,
            options(nomem, nostack, preserves_flags)
        );
    }
    old_mstatus
}

#[inline(always)]
pub fn exit_critical_from_isr(old_state: u32) {
    // SAFETY: Restores `mstatus` saved by `enter_critical_from_isr`.
    unsafe {
        core::arch::asm!(
            "csrw mstatus, {old}",
            old = in(reg) old_state,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub fn yield_from_isr() {
    NVIC_trigger_sw_irq();
}
