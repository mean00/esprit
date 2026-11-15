//! ARM Cortex-M NVIC, vector table, fault trampolines, and IRQ dispatch
//! (`drop_c_part3.md` Phase G4).
//!
//! Replaces `esprit/cpp/mcus/arm_gd32fx/src/lnIRQ_arm_gd32fx.cpp` in
//! `LN_ENABLE_RUST` builds while leaving the C++ file untouched for
//! `LN_ENABLE_RUST=OFF`.

use core::arch::{asm, naked_asm};
use core::ffi::c_char;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicI32, AtomicPtr, AtomicU32, Ordering};

use crate::registers::{
    CONFIG_LIBRARY_LOWEST_INTERRUPT_PRIORITY, CONFIG_LIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY,
    CONFIG_MAX_SYSCALL_INTERRUPT_PRIORITY, CONFIG_PRIO_BITS, LN_IRQ_ADC0_1, LN_IRQ_ARM_LAST,
    LN_IRQ_WWDG, NvicRegisters, SCB_AIRCR_ENDIANNESS_MASK, SCB_AIRCR_KEY,
    SCB_AIRCR_PRIGROUP_4BITS, SCB_AIRCR_SYSRESETREQ, SCB_AIRCR_VECTRESET, SCB_CCR_DIV_0_TRP,
    SCB_CCR_STKALIGN, SCB_SHCSR_FAULTS_ENA, ScbRegisters,
};

const LN_VECTOR_OFFSET: i32 = 16;
const LN_MSP_SIZE_UINT32: usize = 128;
const DEADEND_CRASH_CODE: i32 = 0x90;
const DEADEND_UNSUPPORTED_IRQ: i32 = 0x91;
const NVIC_ICER_INIT_WORDS: usize = 4;
const SCB_SHP_SYSTEM_HANDLERS: usize = 4;

#[repr(C, align(8))]
struct MspStack([AtomicU32; LN_MSP_SIZE_UINT32]);

// Must live in RAM (`.bss`), not `.rodata` in flash: interior mutability via `AtomicU32`
// guarantees the linker places `MSP_STACK` in `.bss`.
static MSP_STACK: MspStack = MspStack([const { AtomicU32::new(0) }; LN_MSP_SIZE_UINT32]);

static CUR_INTERRUPT: AtomicU32 = AtomicU32::new(0);
static CUR_LN_INTERRUPT: AtomicI32 = AtomicI32::new(0);
static LAST_ERROR_CODE: AtomicI32 = AtomicI32::new(0);
static ADC_IRQ_HANDLER: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
static CRASH_STRUCTURE: AtomicPtr<RegisterStack> = AtomicPtr::new(core::ptr::null_mut());

#[repr(C)]
pub struct RegisterStack {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,
    pub pc: u32,
    pub xpsr: u32,
}

unsafe extern "C" {
    static mut lnScratchRegister: u32;
    fn Logger_crash(st: *const c_char);
    fn dmaIrqHandler(dma: i32, channel: i32);

    fn __exc_reset();
    fn vPortSVCHandler();
    fn xPortPendSVHandler();
    fn xPortSysTickHandler();

    fn USART0_IRQHandler();
    fn USART1_IRQHandler();
    fn USART2_IRQHandler();
    fn TIMER1_IRQHandler();
    fn TIMER2_IRQHandler();
    fn TIMER3_IRQHandler();
    fn TIMER5_IRQHandler();
    fn TIMER6_IRQHandler();
    fn EXTI0_IRQHandler();
    fn EXTI1_IRQHandler();
    fn EXTI2_IRQHandler();
    fn EXTI3_IRQHandler();
    fn EXTI4_IRQHandler();
    fn EXTI5_9_IRQHandler();
    fn EXTI10_15_IRQHandler();
    #[link_name = "\u{1}_Z17USB_TX_IRQHandlerv"]
    fn USB_TX_IRQHandler();
    #[link_name = "\u{1}_Z17USB_RX_IRQHandlerv"]
    fn USB_RX_IRQHandler();
    #[link_name = "\u{1}_Z21USB_WAKEUP_IRQHandlerv"]
    fn USB_WAKEUP_IRQHandler();
}

/// `extern "C" void deadEnd(int code)`
#[unsafe(no_mangle)]
pub extern "C" fn deadEnd(code: i32) -> ! {
    // SAFETY: Disables all maskable interrupts before halting on fatal error.
    unsafe {
        asm!("cpsid if", options(nomem, nostack));
    }
    LAST_ERROR_CODE.store(code, Ordering::Relaxed);
    let msg = b"**** CRASH ***\n\0";
    // SAFETY: `Logger_crash` accepts a NUL-terminated static C string.
    unsafe {
        Logger_crash(msg.as_ptr().cast());
        Logger_crash(msg.as_ptr().cast());
        Logger_crash(msg.as_ptr().cast());
        asm!("bkpt #1", options(nomem, nostack));
    }
    loop {
        core::hint::spin_loop();
    }
}

extern "C" fn unsupported_interrupt() {
    let scb = ScbRegisters::ptr();
    // SAFETY: `ScbRegisters::ptr()` points to the Cortex-M System Control Block.
    let icsr = unsafe { read_volatile(&(*scb).icsr) } & 0xFF;
    CUR_INTERRUPT.store(icsr, Ordering::Relaxed);
    CUR_LN_INTERRUPT.store((icsr as i32) - LN_VECTOR_OFFSET, Ordering::Relaxed);
    // SAFETY: Halts in debugger before entering `deadEnd` on unexpected IRQ.
    unsafe {
        asm!("bkpt #1", options(nomem, nostack));
    }
    deadEnd(DEADEND_UNSUPPORTED_IRQ);
}

#[unsafe(no_mangle)]
extern "C" fn crash_handler2(sp: *mut RegisterStack) -> ! {
    CRASH_STRUCTURE.store(sp, Ordering::Relaxed);
    // SAFETY: Disables interrupts before entering `deadEnd`.
    unsafe {
        asm!("cpsid if", options(nomem, nostack));
    }
    deadEnd(DEADEND_CRASH_CODE);
}

#[unsafe(naked)]
unsafe extern "C" fn fault_trampoline() {
    naked_asm!(
        "mrs r1, ipsr",
        "tst lr, #4",
        "ite eq",
        "mrseq r0, msp",
        "mrsne r0, psp",
        "bkpt #1",
        "b {handler}",
        handler = sym crash_handler2,
    );
}

/// `void lnSoftSystemReset()`
#[unsafe(export_name = "_Z17lnSoftSystemResetv")]
pub extern "C" fn ln_soft_system_reset() -> ! {
    let scb = ScbRegisters::ptr();
    // SAFETY: `ScbRegisters::ptr()` points to SCB; writes VTOR=0 and triggers VECTRESET.
    unsafe {
        write_volatile(&raw mut lnScratchRegister, read_volatile(&(*scb).aircr));
        write_volatile(&mut (*scb).vtor, 0);
        write_volatile(&mut (*scb).aircr, SCB_AIRCR_KEY | SCB_AIRCR_VECTRESET);
    }
    loop {
        core::hint::spin_loop();
    }
}

/// `void lnHardSystemReset()`
#[unsafe(export_name = "_Z17lnHardSystemResetv")]
pub extern "C" fn ln_hard_system_reset() -> ! {
    let scb = ScbRegisters::ptr();
    // SAFETY: `ScbRegisters::ptr()` points to SCB; writes VTOR=0 and triggers SYSRESETREQ.
    unsafe {
        write_volatile(&raw mut lnScratchRegister, read_volatile(&(*scb).aircr));
        write_volatile(&mut (*scb).vtor, 0);
        write_volatile(&mut (*scb).aircr, SCB_AIRCR_KEY | SCB_AIRCR_SYSRESETREQ);
    }
    loop {
        core::hint::spin_loop();
    }
}

/// `void lnIrqSetPriority(const LnIRQ &irq, int prio)`
#[unsafe(export_name = "_Z16lnIrqSetPriorityRK5LnIRQi")]
pub unsafe extern "C" fn ln_irq_set_priority(irq_ptr: *const i32, prio: i32) {
    // SAFETY: Caller passes a valid C++ `const LnIRQ &` reference.
    let irq = unsafe { *irq_ptr };
    if irq < LN_IRQ_WWDG {
        return;
    }
    let clamped = (CONFIG_LIBRARY_LOWEST_INTERRUPT_PRIORITY - prio).clamp(
        CONFIG_LIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY,
        CONFIG_LIBRARY_LOWEST_INTERRUPT_PRIORITY,
    );
    let nvic = NvicRegisters::ptr();
    // SAFETY: `irq >= 0` and within NVIC IP table bounds.
    unsafe {
        write_volatile(
            &mut (*nvic).ip[irq as usize],
            (clamped << (8 - CONFIG_PRIO_BITS)) as u8,
        );
    }
}

fn enable_disable(enable: bool, irq: i32) {
    assert!(irq >= 0, "invalid IRQ number");
    let offset = (irq as usize) / 32;
    let bit = 1u32 << ((irq as u32) & 31);
    let nvic = NvicRegisters::ptr();
    // SAFETY: `NvicRegisters::ptr()` points to the Cortex-M NVIC block (`0xE000_E100`).
    unsafe {
        if enable {
            write_volatile(&mut (*nvic).iser.data[offset], bit);
        } else {
            write_volatile(&mut (*nvic).icer.data[offset], bit);
        }
    }
}

/// `void lnEnableInterrupt(const LnIRQ &irq)`
#[unsafe(export_name = "_Z17lnEnableInterruptRK5LnIRQ")]
pub unsafe extern "C" fn ln_enable_interrupt(irq_ptr: *const i32) {
    // SAFETY: Caller passes a valid C++ `const LnIRQ &` reference.
    enable_disable(true, unsafe { *irq_ptr });
}

/// `void lnDisableInterrupt(const LnIRQ &irq)`
#[unsafe(export_name = "_Z18lnDisableInterruptRK5LnIRQ")]
pub unsafe extern "C" fn ln_disable_interrupt(irq_ptr: *const i32) {
    // SAFETY: Caller passes a valid C++ `const LnIRQ &` reference.
    enable_disable(false, unsafe { *irq_ptr });
}

/// `void lnSetInterruptHandler(const LnIRQ &irq, lnInterruptHandler *handler)`
#[unsafe(export_name = "_Z21lnSetInterruptHandlerRK5LnIRQPFvvE")]
pub unsafe extern "C" fn ln_set_interrupt_handler(
    irq_ptr: *const i32,
    handler: Option<unsafe extern "C" fn()>,
) {
    // SAFETY: Caller passes a valid C++ `const LnIRQ &` reference.
    let irq = unsafe { *irq_ptr };
    assert!(irq == LN_IRQ_ADC0_1, "unsupported lnSetInterruptHandler IRQ");
    let ptr = handler.map_or(core::ptr::null_mut(), |f| f as *mut ());
    ADC_IRQ_HANDLER.store(ptr, Ordering::Release);
}

extern "C" fn adc01_irq_handler() {
    let ptr = ADC_IRQ_HANDLER.load(Ordering::Acquire);
    assert!(!ptr.is_null(), "ADC01 IRQ with null handler");
    // SAFETY: `ptr` was stored from an `unsafe extern "C" fn()`.
    let handler: unsafe extern "C" fn() = unsafe { core::mem::transmute(ptr) };
    // SAFETY: Invokes registered ADC interrupt handler.
    unsafe { handler() };
}

macro_rules! dma_irq {
    ($name:ident, $dma:expr, $chan:expr) => {
        #[unsafe(no_mangle)]
        extern "C" fn $name() {
            // SAFETY: Calls `dmaIrqHandler(dma, channel)` defined in `lnDma`.
            unsafe { dmaIrqHandler($dma, $chan) };
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

#[unsafe(no_mangle)]
extern "C" fn I2C0_EV_IRQHandler() {
    deadEnd(1);
}

#[unsafe(no_mangle)]
extern "C" fn I2C0_ERR_IRQHandler() {
    deadEnd(1);
}

#[unsafe(no_mangle)]
extern "C" fn I2C1_EV_IRQHandler() {
    deadEnd(1);
}

#[unsafe(no_mangle)]
extern "C" fn I2C1_ERR_IRQHandler() {
    deadEnd(1);
}

/// `void lnIrqSysInit()`
#[unsafe(export_name = "_Z12lnIrqSysInitv")]
pub extern "C" fn ln_irq_sys_init() {
    let nvic = NvicRegisters::ptr();
    let scb = ScbRegisters::ptr();
    // SAFETY: Early single-core NVIC & SCB initialization.
    unsafe {
        for i in 0..NVIC_ICER_INIT_WORDS {
            write_volatile(&mut (*nvic).icer.data[i], u32::MAX);
        }

        let aircr = (read_volatile(&(*scb).aircr) & SCB_AIRCR_ENDIANNESS_MASK)
            | SCB_AIRCR_KEY
            | SCB_AIRCR_PRIGROUP_4BITS;
        write_volatile(&mut (*scb).aircr, aircr);

        for irq in LN_IRQ_WWDG..LN_IRQ_ARM_LAST {
            ln_irq_set_priority(&irq, 0);
        }

        for i in 0..SCB_SHP_SYSTEM_HANDLERS {
            write_volatile(&mut (*scb).shp[i], CONFIG_MAX_SYSCALL_INTERRUPT_PRIORITY);
        }

        write_volatile(&mut (*scb).vtor, (&raw const LN_VECTOR_TABLE) as u32);
        let ccr = read_volatile(&(*scb).ccr) | SCB_CCR_DIV_0_TRP | SCB_CCR_STKALIGN;
        write_volatile(&mut (*scb).ccr, ccr);
        let shcsr = read_volatile(&(*scb).shcsr) | SCB_SHCSR_FAULTS_ENA;
        write_volatile(&mut (*scb).shcsr, shcsr);
        asm!("dsb sy", options(nomem, nostack));
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
union VectorEntry {
    handler: unsafe extern "C" fn(),
    stack_top: *const u32,
}

// SAFETY: `VectorEntry` is read-only in flash (`.LNVECTOR` section).
unsafe impl Sync for VectorEntry {}

#[repr(C, align(512))]
struct VectorTable([VectorEntry; 76]);

const fn vec(handler: unsafe extern "C" fn()) -> VectorEntry {
    VectorEntry { handler }
}

#[unsafe(link_section = "LNVECTOR")]
#[used]
static LN_VECTOR_TABLE: VectorTable = VectorTable([
    VectorEntry {
        stack_top: (&raw const MSP_STACK.0[LN_MSP_SIZE_UINT32 - 1]).cast(),
    },                                      // 00: Initial MSP
    vec(__exc_reset),                       // 01: Reset
    vec(fault_trampoline),                  // 02: NMI
    vec(fault_trampoline),                  // 03: HardFault
    vec(fault_trampoline),                  // 04: MemManage
    vec(fault_trampoline),                  // 05: BusFault
    vec(fault_trampoline),                  // 06: UsageFault
    vec(unsupported_interrupt),             // 07: Reserved
    vec(unsupported_interrupt),             // 08: Reserved
    vec(unsupported_interrupt),             // 09: Reserved
    vec(unsupported_interrupt),             // 10: Reserved
    vec(vPortSVCHandler),                   // 11: SVCall
    vec(fault_trampoline),                  // 12: DebugMon
    vec(unsupported_interrupt),             // 13: Reserved
    vec(xPortPendSVHandler),                // 14: PendSV
    vec(xPortSysTickHandler),               // 15: SysTick
    // External IRQs (0..58)
    vec(fault_trampoline),                  // 16: WWDG
    vec(unsupported_interrupt),             // 17: LVD
    vec(fault_trampoline),                  // 18: TAMPER
    vec(fault_trampoline),                  // 19: RTC
    vec(fault_trampoline),                  // 20: FMC
    vec(fault_trampoline),                  // 21: RCU_CTC
    vec(EXTI0_IRQHandler),                  // 22: EXTI0
    vec(EXTI1_IRQHandler),                  // 23: EXTI1
    vec(EXTI2_IRQHandler),                  // 24: EXTI2
    vec(EXTI3_IRQHandler),                  // 25: EXTI3
    vec(EXTI4_IRQHandler),                  // 26: EXTI4
    vec(DMA0_Channel0_IRQHandler),          // 27: DMA0_Channel0
    vec(DMA0_Channel1_IRQHandler),          // 28: DMA0_Channel1
    vec(DMA0_Channel2_IRQHandler),          // 29: DMA0_Channel2
    vec(DMA0_Channel3_IRQHandler),          // 30: DMA0_Channel3
    vec(DMA0_Channel4_IRQHandler),          // 31: DMA0_Channel4
    vec(DMA0_Channel5_IRQHandler),          // 32: DMA0_Channel5
    vec(DMA0_Channel6_IRQHandler),          // 33: DMA0_Channel6
    vec(adc01_irq_handler),                 // 34: ADC0_1
    vec(USB_TX_IRQHandler),                 // 35: USB_HP_CAN_TX
    vec(USB_RX_IRQHandler),                 // 36: USB_LP_CAN_RX0
    vec(unsupported_interrupt),             // 37: CAN0_RX1
    vec(unsupported_interrupt),             // 38: CAN0_EWMC
    vec(EXTI5_9_IRQHandler),                // 39: EXTI5_9
    vec(unsupported_interrupt),             // 40: TIMER0_BRK
    vec(unsupported_interrupt),             // 41: TIMER0_UP
    vec(unsupported_interrupt),             // 42: TIMER0_TRG_CMT
    vec(unsupported_interrupt),             // 43: TIMER0_Channel
    vec(TIMER1_IRQHandler),                 // 44: TIMER1
    vec(TIMER2_IRQHandler),                 // 45: TIMER2
    vec(TIMER3_IRQHandler),                 // 46: TIMER3
    vec(I2C0_EV_IRQHandler),                // 47: I2C0_EV
    vec(I2C0_ERR_IRQHandler),               // 48: I2C0_ERR
    vec(I2C1_EV_IRQHandler),                // 49: I2C1_EV
    vec(I2C1_ERR_IRQHandler),               // 50: I2C1_ERR
    vec(unsupported_interrupt),             // 51: SPI0
    vec(unsupported_interrupt),             // 52: SPI1
    vec(USART0_IRQHandler),                 // 53: USART0
    vec(USART1_IRQHandler),                 // 54: USART1
    vec(USART2_IRQHandler),                 // 55: USART2
    vec(EXTI10_15_IRQHandler),              // 56: EXTI10_15
    vec(fault_trampoline),                  // 57: RTC_ALARM
    vec(USB_WAKEUP_IRQHandler),             // 58: USBFS_WKUP
    vec(unsupported_interrupt),             // 59: TIMER7_BRK_TIMER12
    vec(unsupported_interrupt),             // 60: TIMER7_UP_TIMER13
    vec(unsupported_interrupt),             // 61: TIMER7_TRG_COM_TIMER14
    vec(unsupported_interrupt),             // 62: TIMER7_CC
    vec(unsupported_interrupt),             // 63: ADC2
    vec(unsupported_interrupt),             // 64: FSMC
    vec(unsupported_interrupt),             // 65: SDIO
    vec(unsupported_interrupt),             // 66: TIMER4
    vec(unsupported_interrupt),             // 67: SPI2
    vec(unsupported_interrupt),             // 68: UART3
    vec(unsupported_interrupt),             // 69: UART4
    vec(TIMER5_IRQHandler),                 // 70: TIMER5
    vec(TIMER6_IRQHandler),                 // 71: TIMER6
    vec(DMA1_Channel0_IRQHandler),          // 72: DMA1_Channel0
    vec(DMA1_Channel1_IRQHandler),          // 73: DMA1_Channel1
    vec(DMA1_Channel2_IRQHandler),          // 74: DMA1_Channel2
    vec(DMA1_Channel3_IRQHandler),          // 75: DMA1_Channel3/4
]);

const DMA_IRQS: [[i32; 7]; 2] = [
    [11, 12, 13, 14, 15, 16, 17],
    [56, 57, 58, 59, 59, -1, -1],
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
    if enable {
        // SAFETY: `irq` is a valid NVIC IRQ number in `DMA_IRQS`.
        unsafe { ln_enable_interrupt(&raw const irq) };
    } else {
        // SAFETY: `irq` is a valid NVIC IRQ number in `DMA_IRQS`.
        unsafe { ln_disable_interrupt(&raw const irq) };
    }
}

const SCB_ICSR: *mut u32 = 0xE000_ED04 as *mut u32;
const PENDSVSET_BIT: u32 = 1 << 28;

#[inline(always)]
pub fn in_isr() -> bool {
    let ipsr: u32;
    // SAFETY: Reading IPSR has no side effects.
    unsafe { core::arch::asm!("mrs {}, ipsr", out(reg) ipsr, options(nomem, nostack, preserves_flags)) };
    ipsr != 0
}

#[inline(always)]
pub fn enter_critical_from_isr() -> u32 {
    let old_basepri: u32;
    let new_basepri: u32 = crate::registers::CONFIG_MAX_SYSCALL_INTERRUPT_PRIORITY as u32;
    // SAFETY: Mirrors FreeRTOS `ulPortRaiseBASEPRI` for ARM_CM4F.
    unsafe {
        core::arch::asm!(
            "mrs {old}, basepri",
            "msr basepri, {new}",
            "dsb",
            "isb",
            old = out(reg) old_basepri,
            new = in(reg) new_basepri,
            options(nomem, nostack, preserves_flags)
        );
    }
    old_basepri
}

#[inline(always)]
pub fn exit_critical_from_isr(old_state: u32) {
    // SAFETY: Restores BASEPRI saved by `enter_critical_from_isr`.
    unsafe {
        core::arch::asm!(
            "msr basepri, {old}",
            old = in(reg) old_state,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub fn yield_from_isr() {
    // SAFETY: `SCB_ICSR` is the Cortex-M4 Interrupt Control and State Register.
    unsafe {
        core::ptr::write_volatile(SCB_ICSR, PENDSVSET_BIT);
        core::arch::asm!("dsb", "isb", options(nomem, nostack, preserves_flags));
    }
}
