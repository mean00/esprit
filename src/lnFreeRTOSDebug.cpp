
/*
 *
 * */
#include "lnFreeRTOSDebug.h"

struct copyOfTcbStruct /* The old naming convention is used to prevent breaking kernel aware debuggers. */
{
    volatile StackType_t *pxTopOfStack; /*< Points to the location of the last item placed on the tasks stack.  THIS
                                           MUST BE THE FIRST MEMBER OF THE TCB STRUCT. */

#if (portUSING_MPU_WRAPPERS == 1)
    xMPU_SETTINGS xMPUSettings; /*< The MPU settings are defined as part of the port layer.  THIS MUST BE THE SECOND
                                   MEMBER OF THE TCB STRUCT. */
#endif

    ListItem_t xStateListItem; /*< The list that the state list item of a task is reference from denotes the state of
                                  that task (Ready, Blocked, Suspended ). */
    ListItem_t xEventListItem; /*< Used to reference a task from an event list. */
    UBaseType_t uxPriority;    /*< The priority of the task.  0 is the lowest priority. */
    StackType_t *pxStack;      /*< Points to the start of the stack. */
    char pcTaskName[configMAX_TASK_NAME_LEN];
    /*< Descriptive name given to the task when created.  Facilitates debugging only. */ /*lint !e971 Unqualified
                                                                                            char types are allowed
                                                                                            for strings and single
                                                                                            characters only. */

#if ((portSTACK_GROWTH > 0) || (configRECORD_STACK_HIGH_ADDRESS == 1))
    StackType_t *pxEndOfStack; /*< Points to the highest valid address for the stack. */
#endif

#if (portCRITICAL_NESTING_IN_TCB == 1)
    UBaseType_t uxCriticalNesting; /*< Holds the critical section nesting depth for ports that do not maintain their own
                                      count in the port layer. */
#endif

#if (configUSE_TRACE_FACILITY == 1)
    UBaseType_t uxTCBNumber;  /*< Stores a number that increments each time a TCB is created.  It allows debuggers to
                                 determine when a task has been deleted and then recreated. */
    UBaseType_t uxTaskNumber; /*< Stores a number specifically for use by third party trace code. */
#endif
};

extern "C" const uint32_t *lnGetFreeRTOSDebug() LN_USED;

// The `used` attribute forces the compiler to emit this symbol even if
// nothing references it. The custom section with KEEP in the linker script
// prevents --gc-sections from discarding it, ensuring it's always present
// in any esprit-based binary so the debugger can find and read it.
extern "C" const lnFreeRTOSDebug freeRTOSDebug LN_USED;
__attribute__((section(".rodata.freertos_debug"))) const lnFreeRTOSDebug freeRTOSDebug = {
    LN_FREERTOS_MAGIC,
    sizeof(List_t),                // LIST_SIZE
    offsetof(xLIST_ITEM, pxNext),  // OFFSET_LIST_ITEM_NEXT;
    offsetof(xLIST_ITEM, pvOwner), // OFFSET_LIST_ITEM_OWNER;

    offsetof(xLIST, uxNumberOfItems), // OFFSET_LIST_NUMBER_OF_ITEM;
    offsetof(xLIST, pxIndex),         // OFFSET_LIST_INDEX;

    configMAX_PRIORITIES, // NB_OF_PRIORITIES;
#ifdef __arm__
#if defined(__thumb2__) && defined(__SOFTFP__) && (__ARM_ARCH == 7 || __ARM_ARCH == 6)
    0, // M0 or M3
#else
    1, // M4
#endif
#else
    0,
#endif
    configMAX_TASK_NAME_LEN,                // MAX_TASK_NAME_LEN
    offsetof(copyOfTcbStruct, pcTaskName),  // OFFSET_TASK_NAME
    offsetof(copyOfTcbStruct, uxTCBNumber), // OFFSET_TASK_NUM
};
/*
  dummy function to prevent the linker from removing it
*/
extern "C" const uint32_t *lnGetFreeRTOSDebug()
{
    return (uint32_t *)&freeRTOSDebug;
}

// EOF
