// This is RP2350/ARM specific
#define configSUPPORT_PICO_SYNC_INTEROP 1
#define configSUPPORT_PICO_TIME_INTEROP 1
#define configUSE_DYNAMIC_EXCEPTION_HANDLERS 0

#define configRUN_FREERTOS_SECURE_ONLY 1

#ifdef ESPRIT_MULTICORE
    #if __has_include("rp2040_config.h")
        #include "rp2040_config.h"
    #endif
#endif
#define configPRIO_BITS (4UL)

#define configLIBRARY_LOWEST_INTERRUPT_PRIORITY 0xf
#define configKERNEL_INTERRUPT_PRIORITY (configLIBRARY_LOWEST_INTERRUPT_PRIORITY << (8 - configPRIO_BITS))
#define configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY 5
#define configMAX_SYSCALL_INTERRUPT_PRIORITY (configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY << (8 - configPRIO_BITS))

#define configCPU_CLOCK_HZ ((uint32_t)SystemCoreClock)
#define configRTC_CLOCK_HZ ((uint32_t)TIMER_FREQ)

#define configMINIMAL_STACK_SIZE ((unsigned short)200)
#define configTOTAL_HEAP_SIZE ((size_t)((LN_FREERTOS_HEAP_SIZE * 1024)))

#define configENABLE_FPU TRUE
#define configENABLE_MPU FALSE
#define configENABLE_TRUSTZONE FALSE
