#pragma once
#include "esprit.h"

#define LN_FREERTOS_MAGIC 0x1FEEBAEUL

#if !defined(configUSE_TRACE_FACILITY)
#error configUSE_TRACE_FACILITY must be defined to 1
#endif

#if configUSE_TRACE_FACILITY != 1
#error configUSE_TRACE_FACILITY must be defined to 1
#endif

/* Layout type values for the LAYOUT_TYPE field.
 * Debuggers read this to know the stack frame format. */
#define LAYOUT_ARM_NOFPU 0  /* ARM Cortex-M0/M3 (no hardware FPU) */
#define LAYOUT_ARM_FPU 1    /* ARM Cortex-M4F/M7 (with hardware FPU) */
#define LAYOUT_RV_STD 2     /* Standard FreeRTOS RISC-V port, no FPU */
#define LAYOUT_RV_STD_FPU 3 /* Standard FreeRTOS RISC-V port, with FPU */
#define LAYOUT_CH32 4       /* CH32V2xx/V3xx (Bumblebee core), no FPU */
#define LAYOUT_CH32_FPU 5   /* CH32V2xx/V3xx (Bumblebee core), with FPU */

struct lnFreeRTOSDebug
{
    int MAGIC;
    int LIST_SIZE;
    int OFFSET_LIST_ITEM_NEXT;
    int OFFSET_LIST_ITEM_OWNER;

    int OFFSET_LIST_NUMBER_OF_ITEM;
    int OFFSET_LIST_INDEX;

    int NB_OF_PRIORITIES;
    int LAYOUT_TYPE; // Formerly MPU_ENABLED.  See LAYOUT_* constants above.
    int MAX_TASK_NAME_LEN;
    int OFFSET_TASK_NAME;
    int OFFSET_TASK_NUM;
};
