/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "lnIRQ.h"
#include "lnPeripheral_priv.h"
struct LN_ECLIC_irqx
{
    uint8_t ip;
    uint8_t ie;
    uint8_t attr;
    uint8_t control;
};

typedef volatile LN_ECLIC_irqx LN_ECLIC_irq;

uint32_t *eclicCfg = (uint32_t *)(LN_ECLIC_ADR);
uint32_t *eclicInfo = (uint32_t *)(LN_ECLIC_ADR + 4); // Read only
uint8_t *eclicMth = (uint8_t *)(LN_ECLIC_ADR + 11);

// List of interrupts
LN_ECLIC_irq *eclicIrqs = (LN_ECLIC_irq *)(LN_ECLIC_ADR + 0x1000);

// Bit description
struct _irqDesc
{
    LnIRQ interrpt;
    int irqNb;
};

static const _irqDesc _irqs[] = {
    {LN_IRQ_NONE, 18},
    {LN_IRQ_WWDGT, 19},          /*!< window watchDog timer interrupt                          */
    {LN_IRQ_LVD, 20},            /*!< LVD through EXTI line detect interrupt                   */
    {LN_IRQ_TAMPER, 21},         /*!< tamper through EXTI line detect                          */
    {LN_IRQ_RTC, 22},            /*!< RTC alarm interrupt                                      */
    {LN_IRQ_FMC, 23},            /*!< FMC interrupt                                            */
    {LN_IRQ_RCU_CTC, 24},        /*!< RCU and CTC interrupt                                    */
    {LN_IRQ_EXTI0, 25},          /*!< EXTI line 0 interrupts                                   */
    {LN_IRQ_EXTI1, 26},          /*!< EXTI line 1 interrupts                                   */
    {LN_IRQ_EXTI2, 27},          /*!< EXTI line 2 interrupts                                   */
    {LN_IRQ_EXTI3, 28},          /*!< EXTI line 3 interrupts                                   */
    {LN_IRQ_EXTI4, 29},          /*!< EXTI line 4 interrupts                                   */
    {LN_IRQ_DMA0_Channel0, 30},  /*!< DMA0 channel0 interrupt                                  */
    {LN_IRQ_DMA0_Channel1, 31},  /*!< DMA0 channel1 interrupt                                  */
    {LN_IRQ_DMA0_Channel2, 32},  /*!< DMA0 channel2 interrupt                                  */
    {LN_IRQ_DMA0_Channel3, 33},  /*!< DMA0 channel3 interrupt                                  */
    {LN_IRQ_DMA0_Channel4, 34},  /*!< DMA0 channel4 interrupt                                  */
    {LN_IRQ_DMA0_Channel5, 35},  /*!< DMA0 channel5 interrupt                                  */
    {LN_IRQ_DMA0_Channel6, 36},  /*!< DMA0 channel6 interrupt                                  */
    {LN_IRQ_ADC0_1, 37},         /*!< ADC0 and ADC1 interrupt                                  */
    {LN_IRQ_CAN0_TX, 38},        /*!< CAN0 TX interrupts                                       */
    {LN_IRQ_CAN0_RX0, 39},       /*!< CAN0 RX0 interrupts                                      */
    {LN_IRQ_CAN0_RX1, 40},       /*!< CAN0 RX1 interrupts                                      */
    {LN_IRQ_CAN0_EWMC, 41},      /*!< CAN0 EWMC interrupts                                     */
    {LN_IRQ_EXTI5_9, 42},        /*!< EXTI[9:5] interrupts                                     */
    {LN_IRQ_TIMER0_BRK, 43},     /*!< TIMER0 break interrupts                                  */
    {LN_IRQ_TIMER0_UP, 44},      /*!< TIMER0 update interrupts                                 */
    {LN_IRQ_TIMER0_TRG_CMT, 45}, /*!< TIMER0 trigger and commutation interrupts                */
    {LN_IRQ_TIMER0_Channel, 46}, /*!< TIMER0 channel capture compare interrupts                */
    {LN_IRQ_TIMER1, 47},         /*!< TIMER1 interrupt                                         */
    {LN_IRQ_TIMER2, 48},         /*!< TIMER2 interrupt                                         */
    {LN_IRQ_TIMER3, 49},         /*!< TIMER3 interrupts                                        */
    {LN_IRQ_I2C0_EV, 50},        /*!< I2C0 event interrupt                                     */
    {LN_IRQ_I2C0_ER, 51},        /*!< I2C0 error interrupt                                     */
    {LN_IRQ_I2C1_EV, 52},        /*!< I2C1 event interrupt                                     */
    {LN_IRQ_I2C1_ER, 53},        /*!< I2C1 error interrupt                                     */
    {LN_IRQ_SPI0, 54},           /*!< SPI0 interrupt                                           */
    {LN_IRQ_SPI1, 55},           /*!< SPI1 interrupt                                           */
    {LN_IRQ_USART0, 56},         /*!< USART0 interrupt                                         */
    {LN_IRQ_USART1, 57},         /*!< USART1 interrupt                                         */
    {LN_IRQ_USART2, 58},         /*!< USART2 interrupt                                         */
    {LN_IRQ_EXTI10_15, 59},      /*!< EXTI[15:10] interrupts                                   */
    {LN_IRQ_RTC_ALARM, 60},      /*!< RTC alarm interrupt EXTI                                 */
    {LN_IRQ_USBFS_WKUP, 61},     /*!< USBFS wakeup interrupt                                   */
    {LN_IRQ_EXMC, 67},           /*!< EXMC global interrupt                                    */
    {LN_IRQ_TIMER4, 69},         /*!< TIMER4 global interrupt                                  */
    {LN_IRQ_SPI2, 70},           /*!< SPI2 global interrupt                                    */
    {LN_IRQ_UART3, 71},          /*!< UART3 global interrupt                                   */
    {LN_IRQ_UART4, 72},          /*!< UART4 global interrupt                                   */
    {LN_IRQ_TIMER5, 73},         /*!< TIMER5 global interrupt                                  */
    {LN_IRQ_TIMER6, 74},         /*!< TIMER6 global interrupt                                  */
    {LN_IRQ_DMA1_Channel0, 75},  /*!< DMA1 channel0 global interrupt                           */
    {LN_IRQ_DMA1_Channel1, 76},  /*!< DMA1 channel1 global interrupt                           */
    {LN_IRQ_DMA1_Channel2, 77},  /*!< DMA1 channel2 global interrupt                           */
    {LN_IRQ_DMA1_Channel3, 78},  /*!< DMA1 channel3 global interrupt                           */
    {LN_IRQ_DMA1_Channel4, 79},  /*!< DMA1 channel3 global interrupt                           */
    {LN_IRQ_CAN1_TX, 82},        /*!< CAN1 TX interrupt                                        */
    {LN_IRQ_CAN1_RX0, 83},       /*!< CAN1 RX0 interrupt                                       */
    {LN_IRQ_CAN1_RX1, 84},       /*!< CAN1 RX1 interrupt                                       */
    {LN_IRQ_CAN1_EWMC, 85},      /*!< CAN1 EWMC interrupt                                      */
    {LN_IRQ_USBFS, 86},          /*!< USBFS global interrupt                                   */
};

extern "C" void lnWriteMthDirect(int val);
extern "C" int lnReadMthDirect();