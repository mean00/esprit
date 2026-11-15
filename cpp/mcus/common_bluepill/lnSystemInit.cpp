
#include "esprit.h"
#include "lnPeripheral_priv.h"

__attribute__((section(".initial_heap"))) uint8_t ucHeap[configTOTAL_HEAP_SIZE];

extern void setup();
extern void loop();
extern "C" void _init();
void lnIrqSysInit();
void lnExtiSysInit();
#ifndef LN_RUST_DMA
void lnDmaSysInit();
#else
#include "lnIRQ.h"
static const LnIRQ _dmaIrqs[2][7] = {
    {LN_IRQ_DMA0_Channel0, LN_IRQ_DMA0_Channel1, LN_IRQ_DMA0_Channel2, LN_IRQ_DMA0_Channel3, LN_IRQ_DMA0_Channel4,
     LN_IRQ_DMA0_Channel5, LN_IRQ_DMA0_Channel6},
    {LN_IRQ_DMA1_Channel0, LN_IRQ_DMA1_Channel1, LN_IRQ_DMA1_Channel2, LN_IRQ_DMA1_Channel3, LN_IRQ_DMA1_Channel4,
     LN_IRQ_NONE, LN_IRQ_NONE}};
extern "C" void lnDmaSetIrqEnabled(int dma, int channel, bool enable)
{
    if (dma < 0 || dma >= 2 || channel < 0 || channel >= 7)
        return;
    LnIRQ irq = _dmaIrqs[dma][channel];
    if (irq == LN_IRQ_NONE)
        return;
    if (enable)
        lnEnableInterrupt(irq);
    else
        lnDisableInterrupt(irq);
}
#endif
void lnRunTimeInit();
void lnRunTimeInitPostPeripherals();

#ifndef LN_INITIAL_TASK_PRIORITY
#define LN_INITIAL_TASK_PRIORITY 2 //
#endif                             //

#ifndef LN_INITIAL_STACK_SIZE
#define LN_INITIAL_STACK_SIZE 1024 //
#endif                             //

/* entryTask: TCB + stack in .bss instead of the FreeRTOS heap (lnFreeRTOS_pp.h) */
static StaticTask_t entryTaskTcb;
static StackType_t entryTaskStack[LN_TASK_STACK_WORDS(LN_INITIAL_STACK_SIZE)];
/**
 *
 * @param
 */
void initTask(void *)
{
    LoggerInit();
    setup();
    while (1)
    {
        loop();
    }
}
/**
 *
 * @return
 */

void resetMe(const Peripherals periph)
{
    lnPeripherals::reset(periph);
    lnPeripherals::enable(periph);
}
void disabled(const Peripherals periph)
{
    lnPeripherals::disable(periph);
}
// dummy call to prevent the linker from removing it...
extern const uint32_t *lnGetFreeRTOSDebug();

int main()
{
    //
    lnRunTimeInit();
    // Initialize system
    lnInitSystemClock();
    // ECLIC init
    lnIrqSysInit();

    // The LEDs are all on GPIO A
    resetMe(pGPIOA);
    resetMe(pGPIOB);
    resetMe(pGPIOC);

    // We need alternate functions too
    resetMe(pAF);
    resetMe(pDMA0);
    resetMe(pDMA1);

    // I2C0 & 1
    disabled(pI2C0);
    disabled(pI2C1);

    // Enable exti
    lnExtiSysInit();

#ifndef LN_RUST_DMA
    // init DMA
    lnDmaSysInit();
#endif

    // Init SPI0/1/2
    disabled(pSPI0);
    disabled(pSPI1);
    disabled(pSPI2);

    // Init Timer 1..4
    disabled(pTIMER1);
    disabled(pTIMER2);
    disabled(pTIMER3);
    disabled(pTIMER4);

    // Basic timer
    disabled(pTIMER5);
    disabled(pTIMER6);

    // DAC
    disabled(pDAC);

    // ADC
    resetMe(pADC0);
    resetMe(pADC1);

    lnRunTimeInitPostPeripherals();

    // Start freertos

    lnCreateTaskStatic(initTask, "entryTask", LN_INITIAL_STACK_SIZE, NULL, LN_INITIAL_TASK_PRIORITY, entryTaskStack,
                       &entryTaskTcb);
    vTaskStartScheduler();
    deadEnd(25);
}
// EOF
