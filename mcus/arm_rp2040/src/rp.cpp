// clang-format off
#include "esprit.h"
#include "lnIRQ.h"
#include "stdio.h"
#include "ln_rp.h" 
#include "hardware/gpio.h"
#include "ln_rp_memory_map.h"
#ifdef ESPRIT_MULTICORE
#include "pico/multicore.h"
#endif
// clang-format on
#ifdef __clang__
FILE *const stdout = NULL;
#endif

extern void setup();
extern void loop();

uint8_t ucHeap[configTOTAL_HEAP_SIZE];
extern void lnRpDmaSysInit();
extern void lnExtiSysInit();
/**
 * @brief
 *
 */
extern "C" void alarm_pool_init_default()
{
// xAssert(0);
#warning this is a stub to avoid pulling the pico-sdk libc malloc
}

#if 1
#define X_SYSTICK SysTick_Handler
#define X_PENDSV PendSV_Handler
#define X_SVHANDLER SVC_Handler
#else
#define X_SYSTICK xPortSysTickHandler
#define X_PENDSV xPortPendSVHandler
#define X_SVHANDLER vPortSVCHandler
#endif

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
 * Called by the FreeRTOS timer/daemon task once the scheduler is running.
 * configTASK_DEFAULT_CORE_AFFINITY pins every task (user tasks, timer task
 * and both idle tasks) to core 0 - so re-pin the passive idle task (IDLE1)
 * back to core 1, otherwise core 1 has no task eligible to run.
 */
#if ( configUSE_DAEMON_TASK_STARTUP_HOOK == 1 )
void vApplicationDaemonTaskStartupHook( void )
{
    vTaskCoreAffinitySet( xTaskGetIdleTaskHandleForCore( 1 ), ( 1U << 1 ) );
}
#endif

#define LN_INITIAL_STACK_SIZE 8 * 1024
#define LN_INITIAL_TASK_PRIORITY 2
uint32_t SystemCoreClock = 125000000;

#ifndef ESPRIT_MULTICORE
extern "C"
{
    void X_SYSTICK(void);
    void X_PENDSV(void);
    void X_SVHANDLER(void);
}
#endif

/**
 * @brief
 *
 * @return int
 */
int main()
{
    lnPinMode(GPIO17, lnUART);
    lnPinMode(GPIO16, lnUART);
    lnRpDmaSysInit();
    lnExtiSysInit();

#ifndef ESPRIT_MULTICORE
    lnSetInterruptHandler(LN_IRQ_SYSTICK, X_SYSTICK);
    lnSetInterruptHandler(LN_IRQ_PENDSV, X_PENDSV);
    lnSetInterruptHandler(LN_IRQ_SVCALL, X_SVHANDLER);
#endif


    lnCreateTask(initTask, "entryTask", LN_INITIAL_STACK_SIZE, NULL, LN_INITIAL_TASK_PRIORITY);
#ifdef ESPRIT_MULTICORE
    multicore_reset_core1();
#endif
    vTaskStartScheduler();
    // lnGetFreeRTOSDebug();
    deadEnd(25);
}

/**
 *
 */
void lnSoftSystemReset(void)
{
    volatile uint32_t *aircr = (volatile uint32_t *)(0xe000ed0cUL); // see 2.4.8 in RP2040 doc
    *aircr = (1 << 2) + (0x5fa << 16);                              // SYSRESETREQ + VECTKEY
}
/**
 *
 */
extern "C" void deadEnd(int code)

{
    __asm__("bkpt #0");
    lnSoftSystemReset();
}

/**
 *
 */
void lnRp2040_reboot_to_usb()
{
    // we hardcode the address for the momment
#warning HARDCODED ROM ADDRESS
    typedef void *cb_usb_reset(uint32_t a, uint32_t b);
    cb_usb_reset *reset = (cb_usb_reset *)0x2591;
    reset(0, 0);
}
/**
 *
 */
void Rp2040ResetToFwUpload()
{
    lnRp2040_reboot_to_usb();
}

// EOF
