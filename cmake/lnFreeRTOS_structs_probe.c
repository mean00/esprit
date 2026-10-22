/*
 * FreeRTOS object sizes for the current configuration.
 *
 * This file is compiled, never linked: every symbol below is an array whose
 * *size* is the size of one FreeRTOS type (or the value of one config macro),
 * so `nm --print-size` on the object tells us the memory layout of this exact
 * build (architecture, word size, configUSE_*, ...).
 *
 * esprit/cmake/lnFreeRTOS_structs.cmake runs nm on the object and writes
 * ${CMAKE_BINARY_DIR}/freertos_struct.h, which is the size of the storage a
 * caller has to provide to the *Static() API (see lnFreeRTOS_pp.h), and
 * rs_rtos/src/kernel/esprit/generated.rs, which is the same numbers for the Rust
 * side of the crate (regenerated only when a number changes).
 *
 * Add a LN_SIZE()/LN_VALUE() line to measure one more type/setting.
 */
#include "include/FreeRTOS.h"

#include "include/event_groups.h"
#include "include/list.h"
#include "include/queue.h"
#include "include/semphr.h"
#include "include/task.h"
#include "include/timers.h"

#if (configUSE_STREAM_BUFFERS == 1)
#include "include/stream_buffer.h"
#endif

#define LN_SIZE(x) char ln_freertos_size_##x[sizeof(x)]

/* Storage the kernel asks the application to provide (configSUPPORT_STATIC_ALLOCATION). */
#if (configSUPPORT_STATIC_ALLOCATION == 1)
LN_SIZE(StaticTask_t);
LN_SIZE(StaticQueue_t);
LN_SIZE(StaticSemaphore_t);
LN_SIZE(StaticEventGroup_t);
LN_SIZE(StaticTimer_t);
#if (configUSE_STREAM_BUFFERS == 1)
LN_SIZE(StaticStreamBuffer_t);
LN_SIZE(StaticMessageBuffer_t);
#endif
#endif

/* Kernel bookkeeping structures */
LN_SIZE(List_t);
LN_SIZE(ListItem_t);
LN_SIZE(MiniListItem_t);

/* Base types */
LN_SIZE(StackType_t);
LN_SIZE(TickType_t);
LN_SIZE(BaseType_t);
LN_SIZE(UBaseType_t);
LN_SIZE(configSTACK_DEPTH_TYPE);

/* Configuration values, encoded as an array size */
#define LN_VALUE(x) char ln_freertos_value_##x[x]

LN_VALUE(configNUMBER_OF_CORES);
LN_VALUE(configTICK_RATE_HZ);
LN_VALUE(configMAX_PRIORITIES);
LN_VALUE(configMAX_TASK_NAME_LEN);
LN_VALUE(configMINIMAL_STACK_SIZE);
LN_VALUE(configTASK_NOTIFICATION_ARRAY_ENTRIES);
#if (configUSE_TIMERS == 1)
LN_VALUE(configTIMER_QUEUE_LENGTH);
LN_VALUE(configTIMER_TASK_PRIORITY);
LN_VALUE(configTIMER_TASK_STACK_DEPTH);
#endif
#ifdef configTOTAL_HEAP_SIZE
LN_VALUE(configTOTAL_HEAP_SIZE);
#endif
