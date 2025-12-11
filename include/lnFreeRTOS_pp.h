/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "lnAssert.h"
#include "lnFreeRTOS.h"
#include "systemHelper.h"

/**
 */
class lnBinarySemaphore
{
  public:
    lnBinarySemaphore();
    bool take();
    bool tryTake();
    bool take(int timeoutMs);
    bool give();

  protected:
    SemaphoreHandle_t _handle;
};

/**
 * @brief
 *
 */
class lnTask
{
  public:
    lnTask(const char *name, int priority = 3, int taskSize = 100);
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
    int _priority;
    int _taskSize;
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
                        int timeout = 0);  //  the events are cleared upon return from here ! returns  0 if timeout
    uint32_t readEvents(uint32_t maskInt); // it is also cleared automatically !
  protected:
    EventGroupHandle_t _handle;
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
                        int timeout = -1); //  the events are cleared upon return from here ! returns  0 if timeout
    uint32_t readEvents(uint32_t maskInt); // it is also cleared automatically !
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
    void init(const char *name, int periodInMs);
    bool start();
    bool stop();
    bool restart();

  protected:
    TimerHandle_t _timerHandle;
};
// EOF
