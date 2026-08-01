// clang-format off
#include "esprit.h"
#include "lnIRQ.h"
#include "stdio.h"
#include "ln_rp.h" 
#include "hardware/gpio.h"
#include "ln_rp_memory_map.h"
// clang-format on
#include "pico/runtime_init.h"
//
#ifdef __clang__
FILE *const stdout = NULL;
#endif

extern void setup();
extern void loop();
extern "C" void runtime_run_initializers();
extern "C" int main();
extern "C" void pre_main();
uint8_t ucHeap[configTOTAL_HEAP_SIZE];
extern "C" void runtime_init_install_ram_vector_table();
extern void lnRpDmaSysInit();
/**
 * @brief
 *
 */
extern "C" void _xalarm_pool_init_default()
{
// xAssert(0);
#warning this is a stub to avoid pulling the pico-sdk libc malloc
}

int main2();

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
uint32_t SystemCoreClock = 150 * 1000 * 1000;

#if 1
#define X_SYSTICK SysTick_Handler
#define X_PENDSV PendSV_Handler
#define X_SVHANDLER SVC_Handler
#else
#define X_SYSTICK xPortSysTickHandler
#define X_PENDSV xPortPendSVHandler
#define X_SVHANDLER vPortSVCHandler
#endif
extern "C"
{
    void X_SYSTICK();
    void X_PENDSV();
    void X_SVHANDLER();
}
/**
 * @brief
 *
 * @return int
 */
#define INITIAL_STACK 1024
#define VTOR (*(volatile uint32_t *)0xE000ED08)
int main()
{
    runtime_run_initializers();
    extern void (*__init_array_start)(void);
    extern void (*__init_array_end)(void);
    for (void (**p)(void) = &__init_array_start; p < &__init_array_end; ++p)
    {
        (*p)();
    }
    extern uint32_t ram_vector_table[];
    uint32_t *dummyStack = new uint32_t[INITIAL_STACK];
    uint32_t top = (uint32_t)&(dummyStack[INITIAL_STACK - 1]);
    ram_vector_table[0] = top;
    /* update vtor_s */
    VTOR = (uint32_t)ram_vector_table;
    __asm__("mov sp,%0\n" ::"r"(top) :);
    return main2();
}
/**
 *
 */
int main2()
{
    lnPinMode(GPIO17, lnUART);
    lnPinMode(GPIO18, lnUART_Alt);
    lnRpDmaSysInit();

    lnSetInterruptHandler(LN_IRQ_SYSTICK, X_SYSTICK);
    lnSetInterruptHandler(LN_IRQ_PENDSV, X_PENDSV);
    lnSetInterruptHandler(LN_IRQ_SVCALL, X_SVHANDLER);

    // allocate a fake stack to jump start the entry
    lnCreateTask(initTask, "entryTask", LN_INITIAL_STACK_SIZE, NULL, LN_INITIAL_TASK_PRIORITY);
    vTaskStartScheduler();
    deadEnd(25);
    return 0;
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
