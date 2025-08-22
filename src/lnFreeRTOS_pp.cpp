/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
    Thin wrapper on top of FreeRTOS to be more C++ friendly
 */

#include "lnFreeRTOS_pp.h"
#include "lnIRQ.h"

#define fos_ms2tick(ms) (((ms) + portTICK_PERIOD_MS - 1) / portTICK_PERIOD_MS)

#define EVENT_INDEX 1

#if configTASK_NOTIFICATION_ARRAY_ENTRIES < 2
#error configTASK_NOTIFICATION_ARRAY_ENTRIES should be at least 2!
#endif

extern "C"
{
    extern void deadEnd(int code);
    void __attribute__((noinline)) __attribute__((noreturn)) do_assert(const char *a)
    {
        deadEnd(0xA000);
        while (1)
        {
            asm("nop");
        }
    }
#define xAssert(a)                                                                                                     \
    if (!(a))                                                                                                          \
    {                                                                                                                  \
        do_assert(#a);                                                                                                 \
    }
    /**
        \fn adjustStackSize
        \brief return size in uint32_t adjusted for the arch
        The input is in bytes
    */
    static int adjustStackSize(int stackSizeInBytes)
    {
        configSTACK_DEPTH_TYPE systemExtraStack = 0;

#if LN_ARCH == LN_ARCH_RISCV
        systemExtraStack += 28; // we have more registers to save... This is probably over kill
#endif
#if ARCH_FPU == 1
        systemExtraStack += 32;
#endif

        int adjustedStackDepth = (stackSizeInBytes / 4) + systemExtraStack;
        return adjustedStackDepth;
    }

    /**
     *
     */
    void vApplicationMallocFailedHook(void)
    {
        do_assert("Malloc");
    }
}
//-- Binary Semaphor --

/**
 *
 */
lnBinarySemaphore::lnBinarySemaphore()
{
    _handle = xSemaphoreCreateBinary();
    xAssert(_handle);
}
/**
 *
 * @return
 */
bool lnBinarySemaphore::take()
{
    return (bool)xSemaphoreTake(_handle, portMAX_DELAY);
}
/**
 *
 * @param timeoutMs
 * @return
 */
bool lnBinarySemaphore::take(int timeoutMs)
{
    int ticks = 1 + (timeoutMs * configTICK_RATE_HZ + 500) / 1000;
    return (bool)xSemaphoreTake(_handle, ticks);
}
/**
 *
 * @param timeoutMs
 * @return
 */
bool lnBinarySemaphore::tryTake()
{
    return (bool)xSemaphoreTake(_handle, 0);
}

/**
 *
 * @return
 */
bool lnBinarySemaphore::give()
{
    if (!underInterrupt)
    {
        xAssert(xSemaphoreGive(_handle)); // should never fail
    }
    else
    {
        BaseType_t awake;
        xSemaphoreGiveFromISR(_handle, &awake); // should never fail...
        portYIELD_FROM_ISR(awake);              // reschedule
    }
    return true;
}
/**
 *
 * @param ms
 */
void lnDelay(uint32_t ms)
{
    const TickType_t dely = 1 + (ms / portTICK_PERIOD_MS);
    vTaskDelay(dely);
}
extern "C" void lnDelay_C(uint32_t ms)
{
    lnDelay(ms);
}

lnMutex::lnMutex()
{
    _handle = xSemaphoreCreateRecursiveMutex();
}
bool lnMutex::lock()
{
    xAssert(xSemaphoreTakeRecursive(_handle, portMAX_DELAY)); // should never fail
    return true;
}
bool lnMutex::unlock()
{
    // xAssert(
    xSemaphoreGiveRecursive(_handle);
    //); // should never fail
    return true;
}

// Task
/**
 * @brief Construct a new x Task::X Task object
 *
 * @param name
 * @param entryPoint
 * @param priority
 * @param taskSize
 */
lnTask::lnTask(const char *name, int priority, int taskSize)
{
    _priority = priority;
    _taskSize = taskSize;
    _name = name;
}
/**
 *
 */
#undef xTaskCreate
void lnTask::start()
{
    BaseType_t er = xTaskCreate(lnTask::Trampoline, _name, adjustStackSize(_taskSize), this, _priority, &_taskHandle);
    xAssert(er == pdPASS);
}
/**
 * @brief Destroy the x Taskx Task object
 *
 */
lnTask::~lnTask()
{
    xAssert(0);
}
/**
 *
 */
lnEventGroup::lnEventGroup()
{
    _handle = xEventGroupCreate();
}
/**
 *
 */
lnEventGroup::~lnEventGroup()
{
    //    xEventGroupDelete(_handle); No delete !
    xAssert(0);
    _handle = 0;
}
/**
 *
 * @param events
 */
void lnEventGroup::setEvents(uint32_t events)
{
    if (!underInterrupt)
    {
        xEventGroupSetBits(_handle, events);
    }
    else
    {
        xAssert(0);
        /*
        BaseType_t wakeUp ;
        xEventGroupSetBitsFromISR(_handle,events,&wakeUp);
        portYIELD_FROM_ISR(wakeUp);
        */
    }
}
/**
 *
 * @param maskint
 * @param timeout
 * @return
 */
uint32_t lnEventGroup::waitEvents(uint32_t maskint, int timeout)
{
    if (timeout == 0)
        timeout = portMAX_DELAY - 1;
    else
        timeout = fos_ms2tick(timeout);
    uint32_t res = xEventGroupWaitBits(_handle, maskint,
                                       pdTRUE,  // auto clear
                                       pdFALSE, // any but
                                       timeout);
    return res;
}
/**
 *
 * @param maskInt
 * @return
 */
uint32_t lnEventGroup::readEvents(uint32_t maskInt) // it is also cleared automatically !
{
    EventBits_t ev = xEventGroupGetBits(_handle);
    ev = ev & maskInt;
    if (ev)
    {
        xEventGroupClearBits(_handle, ev); // Race ?
    }
    return ev;
}

//--------------------------------
/**
 *
 */
#define BEGIN_LOCK() ENTER_CRITICAL()
#define END_LOCK() EXIT_CRITICAL()

#define INVALID_TASK (TaskHandle_t) - 1
lnFastEventGroup::lnFastEventGroup()
{
    _value = 0;
    _mask = 0;
    _waitingTask = INVALID_TASK;
}

/**
 *
 */
lnFastEventGroup::~lnFastEventGroup()
{
}
/**
 *
 */
void lnFastEventGroup::takeOwnership()
{
    _waitingTask = xTaskGetCurrentTaskHandle();
}
/**
 *
 * @param events
 */

#define BEGIN_LOCK() ENTER_CRITICAL()
#define END_LOCK() EXIT_CRITICAL()

void lnFastEventGroup::setEvents(uint32_t events)
{
    if (underInterrupt)
    {
        // #warning : Could we have a race here between different interrupts ? probably
        _value = _value | events;
        bool w = _value & _mask;
        if (!w || _waitingTask == INVALID_TASK) // no need to wake up task
        {
            return;
        }
        BaseType_t awake;
        vTaskNotifyGiveIndexedFromISR(_waitingTask, EVENT_INDEX, &awake);
        portYIELD_FROM_ISR(awake); // reschedule
        return;
    }

    // not under interrupt
    BEGIN_LOCK();
    _value = _value | events;
    bool w = _value & _mask;
    if (!w || _waitingTask == INVALID_TASK) // no need to wake up task
    {
        END_LOCK();
        return;
    }
    END_LOCK();
    xTaskNotifyGiveIndexed(_waitingTask, EVENT_INDEX);
    return;
}
/**
 *
 * @param maskint
 * @param timeout
 * @return
 */

uint32_t lnFastEventGroup::waitEvents(uint32_t maskint, int timeout)
{
    xAssert(_waitingTask != INVALID_TASK);
    xAssert(!underInterrupt);
    while (1)
    {
        BEGIN_LOCK();
        uint32_t set = maskint & _value;
        if (set)
        {
            _value &= ~set;
            _mask = 0;
            END_LOCK();
            xTaskNotifyStateClearIndexed(xTaskGetCurrentTaskHandle(), EVENT_INDEX);
            return set;
        }
        _mask = maskint;
        END_LOCK();
        if (timeout == -1)
            timeout = 0x7fffffff;
        if (pdFALSE == ulTaskNotifyTakeIndexed(EVENT_INDEX, pdTRUE, timeout)) // cleared on exit
        {                                                                     // timeout
            return 0;
        }
        // got it
        BEGIN_LOCK();
        set = maskint & _value;
        // race can cause this to trigger ? xAssert(set);
        _value &= ~set;
        _mask = 0; // no need to clear waitintTask, it must be the same !
        END_LOCK();
        return set;
    }
}

/**
 *
 * @param maskInt
 * @return
 */
uint32_t lnFastEventGroup::readEvents(uint32_t maskInt)
{
    BEGIN_LOCK();
    uint32_t v = _value & maskInt;
    _value &= ~maskInt;
    _mask = 0;
    xTaskNotifyStateClearIndexed(xTaskGetCurrentTaskHandle(), EVENT_INDEX);
    END_LOCK();
    return v;
}
/**

*/

bool lnCreateTask(
    TaskFunction_t pxTaskCode,
    const char *const pcName, /*lint !e971 Unqualified char types are allowed for strings and single characters only. */
    int stackSizeInBytes,     // in bytes  !, // in bytes!
    void *const pvParameters, UBaseType_t uxPriority)
{
    if (pdPASS == xTaskCreate(pxTaskCode, pcName, adjustStackSize(stackSizeInBytes), pvParameters, uxPriority, NULL))
        return true;
    xAssert(0);
}
/**
 * @brief
 *
 */
static void lnTimerCallbackFunction(TimerHandle_t xTimer)
{
    lnPeriodicTimer *t = (lnPeriodicTimer *)pvTimerGetTimerID(xTimer);
    xAssert(t);
    t->timerCallback();
}
/**
 * @brief Construct a new ln Periodic Timer::ln Periodic Timer object
 *
 * @param name
 * @param periodInMs
 */
lnPeriodicTimer::lnPeriodicTimer()
{
}
/**
 * @brief Construct a new ln Periodic Timer::init object
 *
 * @param name
 * @param periodInMs
 */
void lnPeriodicTimer::init(const char *name, int periodInMs)
{
    _timerHandle = xTimerCreate(name, periodInMs, true, this, lnTimerCallbackFunction);
}
/**
 * @brief
 *
 * @return true
 * @return false
 */
bool lnPeriodicTimer::start()
{
    xTimerStart(_timerHandle, 1);
    return true;
}
/**
 * @brief
 *
 * @return true
 * @return false
 */
bool lnPeriodicTimer::restart()
{
    if (underInterrupt)
    {
        BaseType_t wake = pdFALSE;
        xTimerResetFromISR(_timerHandle, &wake);
    }
    else
    {
        xTimerReset(_timerHandle, 1);
    }
    return true;
}
/**
 * @brief
 *
 * @return true
 * @return false
 */
bool lnPeriodicTimer::stop()
{
    xTimerStop(_timerHandle, 1);
    return true;
}
/**
 * @brief Destroy the ln Periodic Timer::ln Periodic Timer object
 *
 */
lnPeriodicTimer::~lnPeriodicTimer()
{
}
// EOF
