/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "esprit.h"
#include "lnPeripherals.h"

#if LN_MCU == LN_MCU_CH32V3x
#ifdef USE_CH32v3x_HW_IRQ_STACK
#define LN_INTERRUPT_TYPE __attribute__((used))
//  __attribute__((naked)) // done manually in assembly
//  __attribute__((interrupt("WCH-Interrupt-fast"))
#else
#define LN_INTERRUPT_TYPE __attribute__((used)) __attribute__((interrupt))
#endif
#else
#define LN_INTERRUPT_TYPE __attribute__((used)) __attribute__((interrupt("IRQ")));
#endif

enum LnIRQ : int
{
    LN_IRQ_NONE = 0,
    LN_IRQ_WWDGT,          /*!< window watchDog timer interrupt                          */
    LN_IRQ_LVD,            /*!< LVD through EXTI line detect interrupt                   */
    LN_IRQ_TAMPER,         /*!< tamper through EXTI line detect                          */
    LN_IRQ_RTC,            /*!< RTC alarm interrupt                                      */
    LN_IRQ_FMC,            /*!< FMC interrupt                                            */
    LN_IRQ_RCU_CTC,        /*!< RCU and CTC interrupt                                    */
    LN_IRQ_EXTI0,          /*!< EXTI line 0 interrupts                                   */
    LN_IRQ_EXTI1,          /*!< 20 EXTI line 1 interrupts                                   */
    LN_IRQ_EXTI2,          /*!< EXTI line 2 interrupts                                   */
    LN_IRQ_EXTI3,          /*!< EXTI line 3 interrupts                                   */
    LN_IRQ_EXTI4,          /*!< EXTI line 4 interrupts                                   */
    LN_IRQ_DMA0_Channel0,  /*!< DMA0 channel0 interrupt                                  */
    LN_IRQ_DMA0_Channel1,  /*!< DMA0 channel1 interrupt                                  */
    LN_IRQ_DMA0_Channel2,  /*!< DMA0 channel2 interrupt                                  */
    LN_IRQ_DMA0_Channel3,  /*!< DMA0 channel3 interrupt                                  */
    LN_IRQ_DMA0_Channel4,  /*!< DMA0 channel4 interrupt                                  */
    LN_IRQ_DMA0_Channel5,  /*!< DMA0 channel5 interrupt                                  */
    LN_IRQ_DMA0_Channel6,  /*!< 30 DMA0 channel6 interrupt                                  */
    LN_IRQ_ADC0_1,         /*!< 31 ADC0 and ADC1 interrupt                                  */
    LN_IRQ_USB_HP_CAN_TX,  /*!< CAN0 TX interrupts                                       */
    LN_IRQ_USB_LP_CAN_RX0, /*!< CAN0 RX0 interrupts                                      */
    LN_IRQ_CAN0_RX1,       /*!< CAN0 RX1 interrupts                                      */
    LN_IRQ_CAN0_EWMC,      /*!< CAN0 EWMC interrupts                                     */
    LN_IRQ_EXTI5_9,        /*!< EXTI[9:5] interrupts                                     */
    LN_IRQ_TIMER0_BRK,     /*!< TIMER0 break interrupts                                  */
    LN_IRQ_TIMER0_UP,      /*!< TIMER0 update interrupts                                 */
    LN_IRQ_TIMER0_TRG_CMT, /*!< TIMER0 trigger and commutation interrupts                */
    LN_IRQ_TIMER0_Channel, /*!< 40 TIMER0 channel capture compare interrupts                */
    LN_IRQ_TIMER1,         /*!< TIMER1 interrupt                                         */
    LN_IRQ_TIMER2,         /*!< TIMER2 interrupt                                         */
    LN_IRQ_TIMER3,         /*!< TIMER3 interrupts                                        */
    LN_IRQ_I2C0_EV,        /*!< I2C0 event interrupt                                     */
    LN_IRQ_I2C0_ER,        /*!< I2C0 error interrupt                                     */
    LN_IRQ_I2C1_EV,        /*!< I2C1 event interrupt                                     */
    LN_IRQ_I2C1_ER,        /*!< I2C1 error interrupt                                     */
    LN_IRQ_SPI0,           /*!< SPI0 interrupt                                           */
    LN_IRQ_SPI1,           /*!< SPI1 interrupt                                           */
    LN_IRQ_USART0,         /*!< 50 USART0 interrupt                                         */
    LN_IRQ_USART1,         /*!< USART1 interrupt                                         */
    LN_IRQ_USART2,         /*!< USART2 interrupt                                         */
    LN_IRQ_EXTI10_15,      /*!< EXTI[15:10] interrupts                                   */
    LN_IRQ_RTC_ALARM,      /*!< RTC alarm interrupt EXTI                                 */
    LN_IRQ_USBFS_WKUP,     /*!< USBFS wakeup interrupt                                   */
    LN_IRQ_EXMC,           /*!< EXMC global interrupt                                    */
    LN_IRQ_TIMER4,         /*!< 57 TIMER4 global interrupt                                  */
    LN_IRQ_SPI2,           /*!< SPI2 global interrupt                                    */
    LN_IRQ_UART3,          /*!< UART3 global interrupt                                   */
    LN_IRQ_UART4,          /*!< 60 UART4 global interrupt                                   */
    LN_IRQ_TIMER5,         /*!< TIMER5 global interrupt                                  */
    LN_IRQ_TIMER6,         /*!< TIMER6 global interrupt                                  */
    LN_IRQ_DMA1_Channel0,  /*!< DMA1 channel0 global interrupt                           */
    LN_IRQ_DMA1_Channel1,  /*!< DMA1 channel1 global interrupt                           */
    LN_IRQ_DMA1_Channel2,  /*!< DMA1 channel2 global interrupt                           */
    LN_IRQ_DMA1_Channel3,  /*!< DMA1 channel3 global interrupt                           */
    LN_IRQ_DMA1_Channel4,  /*!< DMA1 channel3 global interrupt                           */
    LN_IRQ_ETH,            /*!< 61 ETH global interrupt                                    */
    LN_IRQ_ETH_WKUP,       /*!< 62 ETH WAKEUP global interrupt                              */
    LN_IRQ_CAN1_TX,        /*!< 63 CAN1 TX interrupt                                        */
    LN_IRQ_CAN1_RX0,       /*!< CAN1 RX0 interrupt                                       */
    LN_IRQ_CAN1_RX1,       /*!< CAN1 RX1 interrupt                                       */
    LN_IRQ_CAN1_EWMC,      /*!< CAN1 EWMC interrupt                                      */
    LN_IRQ_USBFS,          /*!< USBFS global interrupt                                   */

    // CH32V3 specific
    LN_IRQ_USBHS,
    LN_IRQ_USBHS_WKUP,
    LN_IRQ_OTG_FS,
    LN_IRQ_USB_HP,
    LN_IRQ_USB_LP,
    LN_IRQ_RNG,
    LN_IRQ_FSMC,
    LN_IRQ_SDIO,
    LN_IRQ_DMA1_Channel5,  /*!< DMA1 channel0 global interrupt                           */
    LN_IRQ_DMA1_Channel6,  /*!< DMA1 channel1 global interrupt                           */
    LN_IRQ_DMA1_Channel7,  /*!< DMA1 channel2 global interrupt                           */
    LN_IRQ_DMA1_Channel8,  /*!< DMA1 channel3 global interrupt                           */
    LN_IRQ_DMA1_Channel9,  /*!< DMA1 channel3 global interrupt                           */
    LN_IRQ_DMA1_Channel10, /*!< DMA1 channel3 global interrupt                           */
    LN_IRQ_DMA1_Channel11, /*!< DMA1 channel3 global interrupt                           */
    LN_IRQ_SYSTICK,
    LN_IRQ_SW,
    LN_IRQ_USBWAKEUP = LN_IRQ_USBFS_WKUP,
    LN_IRQ_LAST
};
#if LN_MCU == LN_MCU_CH32V3x
extern bool xPortIsInsideInterrupt();
#define underInterrupt xPortIsInsideInterrupt()
#else // GD32
extern uint32_t underInterrupt;
void lnSetInterruptLevelDirect(int intr, int prio, bool vectored);
#endif
