// This is GD32/ARM specific

#define configPRIO_BITS (4UL)
#define configLIBRARY_LOWEST_INTERRUPT_PRIORITY 0xf
#define configKERNEL_INTERRUPT_PRIORITY (configLIBRARY_LOWEST_INTERRUPT_PRIORITY << (8 - configPRIO_BITS))
#define configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY 5
#define configMAX_SYSCALL_INTERRUPT_PRIORITY (configLIBRARY_MAX_SYSCALL_INTERRUPT_PRIORITY << (8 - configPRIO_BITS))

#define configCPU_CLOCK_HZ ((uint32_t)SystemCoreClock)
#define configRTC_CLOCK_HZ ((uint32_t)TIMER_FREQ)

#define configMINIMAL_STACK_SIZE ((unsigned short)128)
#define configTOTAL_HEAP_SIZE ((size_t)((LN_FREERTOS_HEAP_SIZE * 1024)))
