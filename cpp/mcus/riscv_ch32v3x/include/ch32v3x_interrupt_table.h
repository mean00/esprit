
#pragma once
#include "esprit.h"
#include "stdint.h"
/**
 * @brief
 *
 */
struct CH32V3_INTERRUPTx
{
    uint32_t ISR[4];     // 0x00 Interrupt Enable Status Register
    uint32_t dummy0[4];  // 0x10
    uint32_t IPR[4];     // 0x20 Interrupt Pending Status Register
    uint32_t dummy1[4];  // 0x30
    uint32_t ITHRESHOLD; // 0x40 Interrupt priority Threshold configuration register
    uint32_t dummy2;
    uint32_t CFGR;     // 0x48 Interrupt Configuration Register
    uint32_t GISR;     // 0x4C Global Status Register
    uint8_t VTFIDR[4]; // 0x50 VTF Interrupt ID configuration Register vector to assign to that fast vector
    uint32_t dummy3[3];
    uint32_t VTFADDR[4]; // 0x60 VTD Interrupt Address Register
    uint32_t dummy4[(0x100 - 0x70) / 4];
    uint32_t IENR[4]; // 0x100 Interrupt Enable Set Register
    uint32_t dummy5[(0x180 - 0x110) / 4];
    uint32_t IRER[4]; // 0x180 Interrupt Enable Reset Register
    uint32_t dummy6[(0x200 - 0x190) / 4];
    uint32_t IPSR[4]; // 0x200 Interrupt Pending Set Register
    uint32_t dummy7[(0x280 - 0x210) / 4];
    uint32_t IPRR[4]; // 0x280 Interrupt Pending Reset Register
    uint32_t dummy8[(0x300 - 0x290) / 4];
    uint32_t IACTR[4]; // 0x300 Interrupt Activation Register
    uint32_t dummy9[(0x400 - 0x310) / 4];
    uint32_t IPRIOIR[64]; // 0x400 Priority(0..63)
};
typedef volatile CH32V3_INTERRUPTx CH32V3_INTERRUPT;

/**
 * @brief
 *
 */
#define LIST_OF_INTERRUPTS                                                                                             \
    INTERRUPT_DESC(unsupported),                  /* 0 .word   RESET                        */                         \
        UNSUPPORTED_NO(1),                        /* 1 .word   0                            */                         \
        UNSUPPORTED_NO(2),                        /* 2 .word   NMI_Handler                  */                         \
        INTERRUPT_DESC(HardFault),                /* 3 .word   HardFault_Handler            */                         \
        UNSUPPORTED_NO(4),                        /* 4 .word   0                            */                         \
        UNSUPPORTED_NO(5),                        /* 5 .word   Ecall_M_Mode_Handler         */                         \
        UNSUPPORTED_NO(6),                        /* 6 .word   0                            */                         \
        UNSUPPORTED_NO(7),                        /* 7 .word   0                            */                         \
        UNSUPPORTED_NO(8),                        /* 8 .word   Ecall_U_Mode_Handler         */                         \
        INTERRUPT_DESC(Break_Point_Handler),      /* 9 .word   Break_Point_Handler          */                         \
        INTERRUPT_DESC(unsupported),              /* 10 .word   0                           */                         \
        INTERRUPT_DESC(unsupported),              /* 11 .word   0                           */                         \
        INTERRUPT_DESC(SysTick_Handler),          /* 12 .word   SysTick_Handler             */                         \
        INTERRUPT_DESC(unsupported),              /* 13 .word   0                           */                         \
        INTERRUPT_DESC_RAW(SW_Handler),           /* 14 .word   SW_Handler                  */                         \
        INTERRUPT_DESC(unsupported),              /* 15 .word   0                           */                         \
        INTERRUPT_DESC(unsupported),              /*.word   WWDG_IRQHandler                 */                         \
        INTERRUPT_DESC(unsupported),              /*.word   PVD_IRQHandler                  */                         \
        INTERRUPT_DESC(unsupported),              /*.word   TAMPER_IRQHandler               */                         \
        INTERRUPT_DESC(unsupported),              /*.word   RTC_IRQHandler                  */                         \
        INTERRUPT_DESC(unsupported),              /* 20 .word   FLASH_IRQHandler            */                         \
        INTERRUPT_DESC(unsupported),              /*.word   RCC_IRQHandler                  */                         \
        INTERRUPT_DESC(unsupported),              /*.word   EXTI0_IRQHandler                */                         \
        INTERRUPT_DESC(unsupported),              /*.word   EXTI1_IRQHandler                */                         \
        INTERRUPT_DESC(unsupported),              /*.word   EXTI2_IRQHandler                */                         \
        INTERRUPT_DESC(unsupported),              /*.word   EXTI3_IRQHandler                */                         \
        INTERRUPT_DESC(unsupported),              /*.word   EXTI4_IRQHandler                */                         \
        INTERRUPT_DESC(DMA0_Channel0_IRQHandler), /*.word   DMA1_Channel1_IRQHandler        */                         \
        INTERRUPT_DESC(DMA0_Channel1_IRQHandler), /*.word   DMA1_Channel2_IRQHandler        */                         \
        INTERRUPT_DESC(DMA0_Channel2_IRQHandler), /*.word   DMA1_Channel3_IRQHandler        */                         \
        INTERRUPT_DESC(DMA0_Channel3_IRQHandler), /* 30 .word   DMA1_Channel4_IRQHandler    */                         \
        INTERRUPT_DESC(DMA0_Channel4_IRQHandler), /*.word   DMA1_Channel5_IRQHandler        */                         \
        INTERRUPT_DESC(DMA0_Channel5_IRQHandler), /*.word   DMA1_Channel6_IRQHandler        */                         \
        INTERRUPT_DESC(DMA0_Channel6_IRQHandler), /*.word   DMA1_Channel7_IRQHandler        */                         \
        INTERRUPT_DESC(unsupported),              /*.word   ADC1_2_IRQHandler               */                         \
        INTERRUPT_DESC(unsupported),              /*.word   USB_TX_IRQHandler USB_HP_CAN1_TX_IRQHandler  */            \
        INTERRUPT_DESC(unsupported),              /*.word   USB_RX_IRQHandler USB_LP_CAN1_RX0_IRQHandler */            \
        INTERRUPT_DESC(unsupported),              /*.word   CAN1_RX1_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   CAN1_SCE_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   EXTI9_5_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /* 40 .word   TIM1_BRK_IRQHandler     */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM1_UP_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM1_TRG_COM_IRQHandler     */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM1_CC_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM2_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM3_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM4_IRQHandler             */                             \
        INTERRUPT_DESC(I2C0_EV_IRQHandler),       /*.word   I2C1_EV_IRQHandler          */                             \
        INTERRUPT_DESC(I2C0_ERR_IRQHandler),      /*.word   I2C1_ER_IRQHandler          */                             \
        INTERRUPT_DESC(I2C1_EV_IRQHandler),       /*.word   I2C2_EV_IRQHandler          */                             \
        INTERRUPT_DESC(I2C1_ERR_IRQHandler),      /*.word   I2C2_ER_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   SPI1_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   SPI2_IRQHandler             */                             \
        INTERRUPT_DESC(USART0_IRQHandler),        /*.word   USART1_IRQHandler           */                             \
        INTERRUPT_DESC(USART1_IRQHandler),        /*.word   USART2_IRQHandler           */                             \
        INTERRUPT_DESC(USART2_IRQHandler),        /*.word   USART3_IRQHandler           */                             \
        INTERRUPT_DESC(unsupported),              /*.word   EXTI15_10_IRQHandler        */                             \
        INTERRUPT_DESC(unsupported),              /*.word   RTCAlarm_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   USBWakeUp_IRQHandler        */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM8_BRK_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM8_UP_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM8_TRG_COM_IRQHandler     */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM8_CC_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   RNG_IRQHandler              */                             \
        INTERRUPT_DESC(unsupported),              /*.word   FSMC_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   SDIO_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM5_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   SPI3_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   UART4_IRQHandler            */                             \
        INTERRUPT_DESC(unsupported),              /*.word   UART5_IRQHandler            */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM6_IRQHandler             */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM7_IRQHandler             */                             \
        INTERRUPT_DESC(DMA1_Channel0_IRQHandler), /*.word   DMA2_Channel1_IRQHandler    */                             \
        INTERRUPT_DESC(DMA1_Channel1_IRQHandler), /*.word   DMA2_Channel2_IRQHandler    */                             \
        INTERRUPT_DESC(DMA1_Channel2_IRQHandler), /*.word   DMA2_Channel3_IRQHandler    */                             \
        INTERRUPT_DESC(DMA1_Channel3_IRQHandler), /*.word   DMA2_Channel4_IRQHandler    */                             \
        INTERRUPT_DESC(DMA1_Channel4_IRQHandler), /*.word   DMA2_Channel5_IRQHandler    */                             \
        INTERRUPT_DESC(ETH_IRQHandler),           /*61 .word   ETH_IRQHandler           */                             \
        INTERRUPT_DESC(ETH_WKUP_IRQHandler),      /*62 .word   ETH_IRQHandler           */                             \
        INTERRUPT_DESC(unsupported),              /*.word   CAN2_TX_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   CAN2_RX0_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   CAN2_RX1_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   CAN2_SCE_IRQHandler         */                             \
        INTERRUPT_DESC(OTG_FS_IRQHandler),        /* 67 .word   OTG_FS_IRQHandler       */                             \
        INTERRUPT_DESC(unsupported),              /*.word   USBHSWakeup_IRQHandler      */                             \
        INTERRUPT_DESC(USBHS_IRQHandler),         /*.word   USBHS_IRQHandler            */                             \
        INTERRUPT_DESC(unsupported),              /*.word   DVP_IRQHandler              */                             \
        INTERRUPT_DESC(unsupported),              /*.word   UART6_IRQHandler            */                             \
        INTERRUPT_DESC(unsupported),              /*.word   UART7_IRQHandler            */                             \
        INTERRUPT_DESC(unsupported),              /*.word   UART8_IRQHandler            */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM9_BRK_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM9_UP_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM9_TRG_COM_IRQHandler     */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM9_CC_IRQHandler          */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM10_BRK_IRQHandler        */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM10_UP_IRQHandler         */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM10_TRG_COM_IRQHandler    */                             \
        INTERRUPT_DESC(unsupported),              /*.word   TIM10_CC_IRQHandler         */                             \
        INTERRUPT_DESC(DMA1_Channel5_IRQHandler), /*.word   DMA2_Channel6_IRQHandler    */                             \
        INTERRUPT_DESC(DMA1_Channel6_IRQHandler), /*.word   DMA2_Channel7_IRQHandler    */                             \
        INTERRUPT_DESC(unsupported),              /*.word   DMA2_Channel8_IRQHandler    */                             \
        INTERRUPT_DESC(unsupported),              /*.word   DMA2_Channel9_IRQHandler    */                             \
        INTERRUPT_DESC(unsupported),              /*.word   DMA2_Channel10_IRQHandler   */                             \
        INTERRUPT_DESC(unsupported),              /*.word   DMA2_Channel11_IRQHandler   */

#define LIST_OF_HANDLERS                                                                                               \
    HANDLER_DESC(Break_Point_Handler)                                                                                  \
    HANDLER_DESC(SysTick_Handler)                                                                                      \
    HANDLER_DESC(Ecall_M_Mode_Handler)                                                                                 \
    HANDLER_DESC(Ecall_U_Mode_Handler)                                                                                 \
    HANDLER_DESC_RAW(SW_Handler)                                                                                       \
    HANDLER_DESC(USART0_IRQHandler)                                                                                    \
    HANDLER_DESC(DMA0_Channel0_IRQHandler)                                                                             \
    HANDLER_DESC(DMA0_Channel1_IRQHandler)                                                                             \
    HANDLER_DESC(DMA0_Channel2_IRQHandler)                                                                             \
    HANDLER_DESC(DMA0_Channel3_IRQHandler)                                                                             \
    HANDLER_DESC(DMA0_Channel4_IRQHandler)                                                                             \
    HANDLER_DESC(DMA0_Channel5_IRQHandler)                                                                             \
    HANDLER_DESC(DMA0_Channel6_IRQHandler)                                                                             \
    HANDLER_DESC(DMA0_Channel7_IRQHandler)                                                                             \
                                                                                                                       \
    HANDLER_DESC(DMA1_Channel0_IRQHandler)                                                                             \
    HANDLER_DESC(DMA1_Channel1_IRQHandler)                                                                             \
    HANDLER_DESC(DMA1_Channel2_IRQHandler)                                                                             \
    HANDLER_DESC(DMA1_Channel3_IRQHandler)                                                                             \
    HANDLER_DESC(DMA1_Channel4_IRQHandler)                                                                             \
    HANDLER_DESC(DMA1_Channel5_IRQHandler)                                                                             \
    HANDLER_DESC(DMA1_Channel6_IRQHandler)                                                                             \
    HANDLER_DESC(DMA1_Channel7_IRQHandler)                                                                             \
    HANDLER_DESC(USBHSWakeup_IRQHandler)                                                                               \
    HANDLER_DESC(USBHS_IRQHandler)                                                                                     \
    HANDLER_DESC(OTG_FS_IRQHandler)                                                                                    \
    HANDLER_DESC(ETH_IRQHandler)                                                                                       \
    HANDLER_DESC(ETH_WKUP_IRQHandler)                                                                                  \
                                                                                                                       \
    HANDLER_DESC(USART1_IRQHandler)                                                                                    \
    HANDLER_DESC(USART2_IRQHandler)                                                                                    \
                                                                                                                       \
    HANDLER_DESC(I2C0_EV_IRQHandler)                                                                                   \
    HANDLER_DESC(I2C0_ERR_IRQHandler)                                                                                  \
    HANDLER_DESC(I2C1_EV_IRQHandler)                                                                                   \
    HANDLER_DESC(I2C1_ERR_IRQHandler)

// EOF
