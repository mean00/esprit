// This is GD32/ARM specific

#define configPRIO_BITS (4UL)
#define configLIBRARY_LOWEST_INTERRUPT_PRIORITY 0xf
#define configKERNEL_INTERRUPT_PRIORITY (configLIBRARY_LOWEST_INTERRUPT_PRIORITY << (8 - configPRIO_BITS))
#define configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY 5
#define configMAX_SYSCALL_INTERRUPT_PRIORITY (configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY << (8 - configPRIO_BITS))

#define configCPU_CLOCK_HZ ((uint32_t)SystemCoreClock)
#define configRTC_CLOCK_HZ ((uint32_t)TIMER_FREQ)

#define configMINIMAL_STACK_SIZE ((unsigned short)200)
#define configTOTAL_HEAP_SIZE ((size_t)((LN_FREERTOS_HEAP_SIZE * 1024)))

#define configENABLE_MPU (0UL)

#ifdef ESPRIT_MULTICORE
#define configSUPPORT_PICO_SYNC_INTEROP 1
#undef INCLUDE_xTimerPendFunctionCall
#define INCLUDE_xTimerPendFunctionCall 1
#if __has_include("rp2040_config.h")
#include "rp2040_config.h"
#endif
#endif
