/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "lnAssert.h"
#include "lnFreeRTOS.h"
#include "systemHelper.h"

/*
 * ---------------------------------------------------------------------------
 * Static storage for the FreeRTOS objects wrapped below
 * ---------------------------------------------------------------------------
 * configSUPPORT_STATIC_ALLOCATION is 1 (esprit/freertos_config/FreeRTOSConfig.h),
 * so every wrapper hands a buffer it owns to the *Static() flavour of the
 * creation call: the kernel object (queue/timer/event group/TCB) is built in
 * place instead of being taken from pvPortMalloc().  There is no allocation to
 * fail, no malloc header to pay for, and nothing to fragment.
 *
 * Semaphores, mutexes, event groups and timers have a known size, so their
 * storage is a private attribute of the object itself (one Static*_t per
 * instance, see the generated freertos_struct.h for the sizes).  Objects live
 * in .bss whenever the caller declares them statically - which is the common
 * case - and a firmware can still create as many as it wants: lnDma alone
 * creates 14 lnMutex, so a per-class pool would only add a hard limit.
 *
 * A task is different: its stack is the big part, and its size is a
 * construction parameter, so the task object only owns the TCB (StaticTask_t,
 * one per instance) and takes the stack from the caller:
 *
 *     static StackType_t stack[LN_TASK_STACK_WORDS(8 * 1024)];
 *     static MyTask task("w5500", 5, stack, 8 * 1024);
 *
 * Keeping the stack outside the object is what lets the caller put it in .bss
 * (or reuse a buffer) while the object itself is whatever it needs to be.
 *
 * The storage types come from the kernel headers; the LN_NO_FREERTOS branch of
 * lnFreeRTOS.h aliases them to plain dummies so this header still compiles for
 * the hosted/Qt build, where none of these objects is ever created.
 */
/* StackType_t words a task buffer needs for `bytes` of usable stack: the stack
 * itself plus the register save area xTaskCreateStatic() gets on top (28 words
 * on RISC-V, 32 more with an FPU, never more than 64; adjustStackSize() in
 * lnFreeRTOS_pp.cpp is the one that computes it).  Size your buffer with this
 * and pass the same byte count to the lnTask constructor or to
 * lnCreateTaskStatic(); both give the task the depth adjustStackSize() returns,
 * which is what a dynamic lnCreateTask() of the same size gets. */
#define LN_TASK_STACK_WORDS(bytes) (((bytes) / sizeof(StackType_t)) + 64)

/**
 */
class lnBinarySemaphore
{
  public:
    lnBinarySemaphore();
    bool take();
    bool tryTake();
    bool take(uint32_t timeoutMs);
    bool give();

  protected:
    SemaphoreHandle_t _handle;

  private:
    StaticSemaphore_t _storage; // storage of the queue behind _handle
};

/**
 * @brief
 *
 */
class lnTask
{
  public:
    lnTask(const char *name, uint32_t priority, StackType_t *stack, uint32_t taskSize);
    virtual ~lnTask();
    void start();
    virtual void run() = 0; // Put your code here
    static void Trampoline(void *param)
    {
        lnTask *tsk = (lnTask *)param;
        tsk->run();
    }

  protected:
    TaskHandle_t _taskHandle;
    const char *_name;
    uint32_t _priority;
    uint32_t _taskSize;  // usable stack, in bytes
    StackType_t *_stack; // caller's buffer, LN_TASK_STACK_WORDS(_taskSize) deep

  private:
    StaticTask_t _storage; // TCB of this task, one per instance
};
/**
 *
 */
class lnEventGroup
{
  public:
    lnEventGroup();
    virtual ~lnEventGroup();
    void setEvents(uint32_t events); // /!\ Cannot be used from interrupt !!!!
    uint32_t waitEvents(uint32_t maskint,
                        uint32_t timeout = 0); //  the events are cleared upon return from here ! returns  0 if timeout
    uint32_t readEvents(uint32_t maskInt);     // it is also cleared automatically !
  protected:
    EventGroupHandle_t _handle;

  private:
    StaticEventGroup_t _storage; // storage of the event group behind _handle
};

/**
 *
 */
class lnFastEventGroup
{
  public:
    lnFastEventGroup();
    virtual ~lnFastEventGroup();
    void takeOwnership(); // the task calling this will own the FastEventGroup
    void setEvents(uint32_t events);
    // -1 timeout means wait forever
    uint32_t waitEvents(uint32_t maskint,
                        int32_t timeout = -1); //  the events are cleared upon return from here ! returns  0 if timeout
    uint32_t readEvents(uint32_t maskInt);     // it is also cleared automatically !
  protected:
    volatile uint32_t _value;
    volatile uint32_t _mask;
    TaskHandle_t _waitingTask;
    void *_cookie;
};

/**
 *
 */
class lnMutex
{
  public:
    lnMutex();
    bool lock();
    bool unlock();

  protected:
    SemaphoreHandle_t _handle;

  private:
    StaticSemaphore_t _storage; // storage of the recursive mutex behind _handle
};

/**
 *
 * @param tex
 */
class lnAutoMutex
{
  public:
    lnAutoMutex(lnMutex *tex)
    {
        _tex = tex;
        _tex->lock();
    }
    ~lnAutoMutex()
    {
        _tex->unlock();
        _tex = NULL;
    }

  protected:
    lnMutex *_tex;
};

void xDelay(uint32_t ms);
/**
 * @brief
 *
 */
class lnPeriodicTimer
{
  public:
    lnPeriodicTimer();
    virtual ~lnPeriodicTimer();
    virtual void timerCallback() = 0;
    void init(const char *name, uint32_t periodInMs);
    bool start();
    bool stop();
    bool restart();

  protected:
    TimerHandle_t _timerHandle;

  private:
    StaticTimer_t _storage; // storage of the timer behind _timerHandle
};
// EOF
