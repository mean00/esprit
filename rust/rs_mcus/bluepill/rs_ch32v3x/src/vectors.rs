//! CH32V3x vector table and WCH HW-stack `_relay` thunks (`drop_c_part2.md` Phase C4).
//!
//! Replaces `esprit/cpp/mcus/riscv_ch32v3x/src/ch32v3x_interrupt_table.cpp` in
//! `LN_ENABLE_RUST` builds.

use crate::irq::deadEnd;

pub const VEC_TABLE_LEN: usize = 104;

#[repr(C, align(32))]
pub struct AlignedVecTable(pub [unsafe extern "C" fn(); VEC_TABLE_LEN]);

unsafe impl Sync for AlignedVecTable {}

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn unsupported() {
    deadEnd(11);
}

#[unsafe(no_mangle)]
pub extern "C" fn HardFault() {
    deadEnd(12);
}

#[unsafe(no_mangle)]
pub extern "C" fn HardFault_relay() {
    deadEnd(12);
}

macro_rules! unsupported_no_fn {
    ($name:ident, $code:expr) => {
        // Generated trap stubs for all 1..8 slots; some slots may be unreferenced by the table.
        #[allow(dead_code)]
        extern "C" fn $name() {
            deadEnd($code);
        }
    };
}

unsupported_no_fn!(unsupported_1, 1);
unsupported_no_fn!(unsupported_2, 2);
unsupported_no_fn!(unsupported_3, 3);
unsupported_no_fn!(unsupported_4, 4);
unsupported_no_fn!(unsupported_5, 5);
unsupported_no_fn!(unsupported_6, 6);
unsupported_no_fn!(unsupported_7, 7);
unsupported_no_fn!(unsupported_8, 8);

unsafe extern "C" {
    fn SW_Handler();
    fn SysTick_Handler();
    fn USART0_IRQHandler();
    fn USART1_IRQHandler();
    fn USART2_IRQHandler();
    fn OTG_FS_IRQHandler();
}

// Weak fallback for USBHS (OTG_FS is strong in TinyUSB; USBHS is unused on FS builds).
#[unsafe(no_mangle)]
pub extern "C" fn USBHS_IRQHandler() {
    deadEnd(13);
}

macro_rules! relay_func {
    ($relay:ident, $target:ident) => {
        /// # Safety
        ///
        /// Invoked directly by hardware via the WCH PFIC vector table; jumps to the
        /// corresponding C/Rust IRQ handler and returns with `mret`.
        #[unsafe(link_section = ".isr_code")]
        #[unsafe(no_mangle)]
        #[unsafe(naked)]
        pub unsafe extern "C" fn $relay() {
            core::arch::naked_asm!("jal {target}", "mret", target = sym $target);
        }
    };
}

use crate::irq::{
    Break_Point_Handler, DMA0_Channel0_IRQHandler, DMA0_Channel1_IRQHandler,
    DMA0_Channel2_IRQHandler, DMA0_Channel3_IRQHandler, DMA0_Channel4_IRQHandler,
    DMA0_Channel5_IRQHandler, DMA0_Channel6_IRQHandler, DMA1_Channel0_IRQHandler,
    DMA1_Channel1_IRQHandler, DMA1_Channel2_IRQHandler, DMA1_Channel3_IRQHandler,
    DMA1_Channel4_IRQHandler, DMA1_Channel5_IRQHandler, DMA1_Channel6_IRQHandler,
    ETH_IRQHandler, ETH_WKUP_IRQHandler, I2C0_ERR_IRQHandler, I2C0_EV_IRQHandler,
    I2C1_ERR_IRQHandler, I2C1_EV_IRQHandler,
};

relay_func!(unsupported_relay, unsupported);
relay_func!(Break_Point_Handler_relay, Break_Point_Handler);
relay_func!(SysTick_Handler_relay, SysTick_Handler);
relay_func!(USART0_IRQHandler_relay, USART0_IRQHandler);
relay_func!(USART1_IRQHandler_relay, USART1_IRQHandler);
relay_func!(USART2_IRQHandler_relay, USART2_IRQHandler);
relay_func!(I2C0_EV_IRQHandler_relay, I2C0_EV_IRQHandler);
relay_func!(I2C0_ERR_IRQHandler_relay, I2C0_ERR_IRQHandler);
relay_func!(I2C1_EV_IRQHandler_relay, I2C1_EV_IRQHandler);
relay_func!(I2C1_ERR_IRQHandler_relay, I2C1_ERR_IRQHandler);
relay_func!(OTG_FS_IRQHandler_relay, OTG_FS_IRQHandler);
relay_func!(USBHS_IRQHandler_relay, USBHS_IRQHandler);
relay_func!(ETH_IRQHandler_relay, ETH_IRQHandler);
relay_func!(ETH_WKUP_IRQHandler_relay, ETH_WKUP_IRQHandler);
relay_func!(DMA0_Channel0_IRQHandler_relay, DMA0_Channel0_IRQHandler);
relay_func!(DMA0_Channel1_IRQHandler_relay, DMA0_Channel1_IRQHandler);
relay_func!(DMA0_Channel2_IRQHandler_relay, DMA0_Channel2_IRQHandler);
relay_func!(DMA0_Channel3_IRQHandler_relay, DMA0_Channel3_IRQHandler);
relay_func!(DMA0_Channel4_IRQHandler_relay, DMA0_Channel4_IRQHandler);
relay_func!(DMA0_Channel5_IRQHandler_relay, DMA0_Channel5_IRQHandler);
relay_func!(DMA0_Channel6_IRQHandler_relay, DMA0_Channel6_IRQHandler);
relay_func!(DMA1_Channel0_IRQHandler_relay, DMA1_Channel0_IRQHandler);
relay_func!(DMA1_Channel1_IRQHandler_relay, DMA1_Channel1_IRQHandler);
relay_func!(DMA1_Channel2_IRQHandler_relay, DMA1_Channel2_IRQHandler);
relay_func!(DMA1_Channel3_IRQHandler_relay, DMA1_Channel3_IRQHandler);
relay_func!(DMA1_Channel4_IRQHandler_relay, DMA1_Channel4_IRQHandler);
relay_func!(DMA1_Channel5_IRQHandler_relay, DMA1_Channel5_IRQHandler);
relay_func!(DMA1_Channel6_IRQHandler_relay, DMA1_Channel6_IRQHandler);

#[unsafe(link_section = ".vector_table")]
#[unsafe(no_mangle)]
pub static VEC_TABLE: AlignedVecTable = AlignedVecTable([
    unsupported_relay,               // 0: RESET
    unsupported_1,                   // 1
    unsupported_2,                   // 2: NMI
    HardFault_relay,                 // 3: HardFault
    unsupported_4,                   // 4
    unsupported_5,                   // 5: Ecall_M
    unsupported_6,                   // 6
    unsupported_7,                   // 7
    unsupported_8,                   // 8: Ecall_U
    Break_Point_Handler_relay,       // 9: Break_Point_Handler
    unsupported_relay,               // 10
    unsupported_relay,               // 11
    SysTick_Handler_relay,           // 12: SysTick_Handler
    unsupported_relay,               // 13
    SW_Handler,                      // 14: SW_Handler (raw)
    unsupported_relay,               // 15
    unsupported_relay,               // 16: WWDG
    unsupported_relay,               // 17: PVD
    unsupported_relay,               // 18: TAMPER
    unsupported_relay,               // 19: RTC
    unsupported_relay,               // 20: FLASH
    unsupported_relay,               // 21: RCC
    unsupported_relay,               // 22: EXTI0
    unsupported_relay,               // 23: EXTI1
    unsupported_relay,               // 24: EXTI2
    unsupported_relay,               // 25: EXTI3
    unsupported_relay,               // 26: EXTI4
    DMA0_Channel0_IRQHandler_relay,  // 27
    DMA0_Channel1_IRQHandler_relay,  // 28
    DMA0_Channel2_IRQHandler_relay,  // 29
    DMA0_Channel3_IRQHandler_relay,  // 30
    DMA0_Channel4_IRQHandler_relay,  // 31
    DMA0_Channel5_IRQHandler_relay,  // 32
    DMA0_Channel6_IRQHandler_relay,  // 33
    unsupported_relay,               // 34: ADC1_2
    unsupported_relay,               // 35: USB_TX
    unsupported_relay,               // 36: USB_RX
    unsupported_relay,               // 37: CAN1_RX1
    unsupported_relay,               // 38: CAN1_SCE
    unsupported_relay,               // 39: EXTI9_5
    unsupported_relay,               // 40: TIM1_BRK
    unsupported_relay,               // 41: TIM1_UP
    unsupported_relay,               // 42: TIM1_TRG_COM
    unsupported_relay,               // 43: TIM1_CC
    unsupported_relay,               // 44: TIM2
    unsupported_relay,               // 45: TIM3
    unsupported_relay,               // 46: TIM4
    I2C0_EV_IRQHandler_relay,        // 47
    I2C0_ERR_IRQHandler_relay,       // 48
    I2C1_EV_IRQHandler_relay,        // 49
    I2C1_ERR_IRQHandler_relay,       // 50
    unsupported_relay,               // 51: SPI1
    unsupported_relay,               // 52: SPI2
    USART0_IRQHandler_relay,         // 53
    USART1_IRQHandler_relay,         // 54
    USART2_IRQHandler_relay,         // 55
    unsupported_relay,               // 56: EXTI15_10
    unsupported_relay,               // 57: RTCAlarm
    unsupported_relay,               // 58: USBWakeUp
    unsupported_relay,               // 59: TIM8_BRK
    unsupported_relay,               // 60: TIM8_UP
    unsupported_relay,               // 61: TIM8_TRG_COM
    unsupported_relay,               // 62: TIM8_CC
    unsupported_relay,               // 63: RNG
    unsupported_relay,               // 64: FSMC
    unsupported_relay,               // 65: SDIO
    unsupported_relay,               // 66: TIM5
    unsupported_relay,               // 67: SPI3
    unsupported_relay,               // 68: UART4
    unsupported_relay,               // 69: UART5
    unsupported_relay,               // 70: TIM6
    unsupported_relay,               // 71: TIM7
    DMA1_Channel0_IRQHandler_relay,  // 72
    DMA1_Channel1_IRQHandler_relay,  // 73
    DMA1_Channel2_IRQHandler_relay,  // 74
    DMA1_Channel3_IRQHandler_relay,  // 75
    DMA1_Channel4_IRQHandler_relay,  // 76
    ETH_IRQHandler_relay,            // 77
    ETH_WKUP_IRQHandler_relay,       // 78
    unsupported_relay,               // 79: CAN2_TX
    unsupported_relay,               // 80: CAN2_RX0
    unsupported_relay,               // 81: CAN2_RX1
    unsupported_relay,               // 82: CAN2_SCE
    OTG_FS_IRQHandler_relay,         // 83
    unsupported_relay,               // 84: USBHSWakeup
    USBHS_IRQHandler_relay,          // 85
    unsupported_relay,               // 86: DVP
    unsupported_relay,               // 87: UART6
    unsupported_relay,               // 88: UART7
    unsupported_relay,               // 89: UART8
    unsupported_relay,               // 90: TIM9_BRK
    unsupported_relay,               // 91: TIM9_UP
    unsupported_relay,               // 92: TIM9_TRG_COM
    unsupported_relay,               // 93: TIM9_CC
    unsupported_relay,               // 94: TIM10_BRK
    unsupported_relay,               // 95: TIM10_UP
    unsupported_relay,               // 96: TIM10_TRG_COM
    unsupported_relay,               // 97: TIM10_CC
    DMA1_Channel5_IRQHandler_relay,  // 98
    DMA1_Channel6_IRQHandler_relay,  // 99
    unsupported_relay,               // 100
    unsupported_relay,               // 101
    unsupported_relay,               // 102
    unsupported_relay,               // 103
]);
