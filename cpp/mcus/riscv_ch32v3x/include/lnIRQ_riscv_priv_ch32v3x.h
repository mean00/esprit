/*
 *  (C) 2022 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "lnIRQ.h"
#include "lnPeripheral_priv.h"

#define CH32_SYSCR_HWSTKEN (1 << 0)        // Hardware stack enabled
#define CH32_SYSCR_INESTEN (1 << 1)        // Interrupt nesting enabled
#define CH32_SYSCR_MPTCFG_2NESTED (1 << 2) //
#define CH32_SYSCR_MPTCFG_8NESTED (3 << 2) //
#define CH32_SYSCR_HWSTKOVEN (1 << 4)      // Continue after hw stack overflow
#define CH32_SYSCR_GIHWSTKNEN (1 << 5)     // Temporarily disable interrupts & hw stack

//
#define LN_RISCV_FPU_MODE(x) (x << 13)
#define LN_RISCV_MPP(x) (x << 11)

// Bit description
struct _irqDesc
{
    LnIRQ interrpt;
    int irqNb;
};
#define XXX(x) (x + 16)
static const _irqDesc _irqs[] = {
    {LN_IRQ_NONE, XXX(-1)},
    {LN_IRQ_WWDGT, XXX(0)},           /*!< window watchDog timer interrupt                          */
    {LN_IRQ_LVD, XXX(1)},             /*!< LVD through EXTI line detect interrupt                   */
    {LN_IRQ_TAMPER, XXX(2)},          /*!< tamper through EXTI line detect                          */
    {LN_IRQ_RTC, XXX(3)},             /*!< RTC alarm interrupt                                      */
    {LN_IRQ_FMC, XXX(4)},             /*!< FMC interrupt                                            */
    {LN_IRQ_RCU_CTC, XXX(5)},         /*!< RCU and CTC interrupt                                    */
    {LN_IRQ_EXTI0, XXX(6)},           /*!< EXTI line 0 interrupts                                   */
    {LN_IRQ_EXTI1, XXX(7)},           /*!< EXTI line 1 interrupts                                   */
    {LN_IRQ_EXTI2, XXX(8)},           /*!< EXTI line 2 interrupts                                   */
    {LN_IRQ_EXTI3, XXX(9)},           /*!< EXTI line 3 interrupts                                   */
    {LN_IRQ_EXTI4, XXX(10)},          /*!< EXTI line 4 interrupts                                   */
    {LN_IRQ_DMA0_Channel0, XXX(11)},  /*!< DMA0 channel0 interrupt                                  */
    {LN_IRQ_DMA0_Channel1, XXX(12)},  /*!< DMA0 channel1 interrupt                                  */
    {LN_IRQ_DMA0_Channel2, XXX(13)},  /*!< DMA0 channel2 interrupt                                  */
    {LN_IRQ_DMA0_Channel3, XXX(14)},  /*!< DMA0 channel3 interrupt                                  */
    {LN_IRQ_DMA0_Channel4, XXX(15)},  /*!< DMA0 channel4 interrupt                                  */
    {LN_IRQ_DMA0_Channel5, XXX(16)},  /*!< DMA0 channel5 interrupt                                  */
    {LN_IRQ_DMA0_Channel6, XXX(17)},  /*!< DMA0 channel6 interrupt                                  */
    {LN_IRQ_ADC0_1, XXX(18)},         /*!< ADC0 and ADC1 interrupt                                  */
    {LN_IRQ_USB_HP_CAN_TX, XXX(19)},  /*!< CAN0 TX interrupts                                       */
    {LN_IRQ_USB_LP_CAN_RX0, XXX(20)}, /*!< CAN0 RX0 interrupts                                      */
    {LN_IRQ_CAN0_RX1, XXX(21)},       /*!< CAN0 RX1 interrupts                                      */
    {LN_IRQ_CAN0_EWMC, XXX(22)},      /*!< CAN0 EWMC interrupts                                     */
    {LN_IRQ_EXTI5_9, XXX(23)},        /*!< EXTI[9:5] interrupts                                     */
    {LN_IRQ_TIMER0_BRK, XXX(24)},     /*!< TIMER0 break interrupts                                  */
    {LN_IRQ_TIMER0_UP, XXX(25)},      /*!< TIMER0 update interrupts                                 */
    {LN_IRQ_TIMER0_TRG_CMT, XXX(26)}, /*!< TIMER0 trigger and commutation interrupts                */
    {LN_IRQ_TIMER0_Channel, XXX(27)}, /*!< TIMER0 channel capture compare interrupts                */
    {LN_IRQ_TIMER1, XXX(28)},         /*!< TIMER1 interrupt                                         */
    {LN_IRQ_TIMER2, XXX(29)},         /*!< TIMER2 interrupt                                         */
    {LN_IRQ_TIMER3, XXX(30)},         /*!< TIMER3 interrupts                                        */
    {LN_IRQ_I2C0_EV, XXX(31)},        /*!< I2C0 event interrupt                                     */
    {LN_IRQ_I2C0_ER, XXX(32)},        /*!< I2C0 error interrupt                                     */
    {LN_IRQ_I2C1_EV, XXX(33)},        /*!< I2C1 event interrupt                                     */
    {LN_IRQ_I2C1_ER, XXX(34)},        /*!< I2C1 error interrupt                                     */
    {LN_IRQ_SPI0, XXX(35)},           /*!< SPI0 interrupt                                           */
    {LN_IRQ_SPI1, XXX(36)},           /*!< SPI1 interrupt                                           */
    {LN_IRQ_USART0, XXX(37)},         /*!< USART0 interrupt                                         */
    {LN_IRQ_USART1, XXX(38)},         /*!< USART1 interrupt                                         */
    {LN_IRQ_USART2, XXX(39)},         /*!< USART2 interrupt                                         */
    {LN_IRQ_EXTI10_15, XXX(40)},      /*!< EXTI[15:10] interrupts                                   */
    {LN_IRQ_RTC_ALARM, XXX(41)},      /*!< RTC alarm interrupt EXTI                                 */
    {LN_IRQ_USBFS_WKUP, XXX(42)},     /*!< RTC alarm interrupt EXTI                                 */
    // tim7 BRK/UP/TRIG/CC
    {LN_IRQ_RNG, XXX(47)},  /*!< USBFS wakeup interrupt                                   */
    {LN_IRQ_FSMC, XXX(48)}, /*!< USBFS wakeup interrupt                                   */
    {LN_IRQ_SDIO, XXX(49)}, /*!< USBFS wakeup interrupt                                   */
    //{LN_IRQ_EXMC                , XXX(50)},     /*!< EXMC global interrupt                                    */
    {LN_IRQ_TIMER4, XXX(50)},        /*!< TIMER4 global interrupt                                  */
    {LN_IRQ_SPI2, XXX(51)},          /*!< SPI2 global interrupt                                    */
    {LN_IRQ_UART3, XXX(52)},         /*!< UART3 global interrupt                                   */
    {LN_IRQ_UART4, XXX(53)},         /*!< UART4 global interrupt                                   */
    {LN_IRQ_TIMER5, XXX(54)},        /*!< TIMER5 global interrupt                                  */
    {LN_IRQ_TIMER6, XXX(55)},        /*!< TIMER6 global interrupt                                  */
    {LN_IRQ_DMA1_Channel0, XXX(56)}, /*!< DMA1 channel0 global interrupt                           */
    {LN_IRQ_DMA1_Channel1, XXX(57)}, /*!< DMA1 channel1 global interrupt                           */
    {LN_IRQ_DMA1_Channel2, XXX(58)}, /*!< DMA1 channel2 global interrupt                           */
    {LN_IRQ_DMA1_Channel3, XXX(59)}, /*!< DMA1 channel3 global interrupt                           */
    {LN_IRQ_DMA1_Channel4, XXX(60)}, /*!< DMA1 channel3 global interrupt                           */
    // {LN_IRQ_ETH CAN1 TX CAN1 RX CAN..                 , XXX(61)},     /*!< DMA1 channel3 global interrupt */
    {LN_IRQ_OTG_FS, XXX(67)},     /*!< USBFS global interrupt                                   */
    {LN_IRQ_USBHS_WKUP, XXX(68)}, /*!< USBFS global interrupt                                   */
    {LN_IRQ_USBHS, XXX(69)},      /*!< USBFS global interrupt                                   */
    // ETH
    {LN_IRQ_ETH, XXX(61)},      /*!< 61 ETH global interrupt                                    */
    {LN_IRQ_ETH_WKUP, XXX(62)}, /*!< 62 ETH WAKEUP global interrupt                              */

    // DVP UART 5 6 7 TIM8 TIM9
    {LN_IRQ_DMA1_Channel5, XXX(82)},  /*!< DMA1 channel0 global interrupt                           */
    {LN_IRQ_DMA1_Channel6, XXX(83)},  /*!< DMA1 channel1 global interrupt                           */
    {LN_IRQ_DMA1_Channel7, XXX(84)},  /*!< DMA1 channel2 global interrupt                           */
    {LN_IRQ_DMA1_Channel8, XXX(85)},  /*!< DMA1 channel3 global interrupt                           */
    {LN_IRQ_DMA1_Channel9, XXX(86)},  /*!< DMA1 channel3 global interrupt                           */
    {LN_IRQ_DMA1_Channel10, XXX(87)}, /*!< DMA1 channel3 global interrupt                           */
    {LN_IRQ_DMA1_Channel11, XXX(88)}, /*!< DMA1 channel3 global interrupt                           */
    {LN_IRQ_SYSTICK, 12},             /*!< Direct PFIC systick interrupt                           */
    {LN_IRQ_SW, 14},                  /*!< Direct PFIC SW interrupt                           */
};
