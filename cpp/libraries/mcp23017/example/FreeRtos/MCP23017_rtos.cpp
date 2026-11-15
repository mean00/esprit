/**
 *
 * @param x
 */
#include "MCP23017_rtos.h"

/**
 *
 * @param x
 */
static void _myInterrupt(void *x)
{
    xBinarySemaphore *sem = (xBinarySemaphore *)x;
    sem->giveFromInterrupt();
}
/**
 *
 * @param a
 */
static void TaskMCP(void *a)
{
    myMcp23017_rtos *mcp_rtos = (myMcp23017_rtos *)a;
    mcp_rtos->runTask();
}
/**
 */
void myMcp23017_rtos::runTask()
{
    while (1)
    {
        _sem->take(); // Wait for signal from interrupt
        // Mcp will do i2c request, protect us...
        _i2cMutex->lock();
        _mcp->process();
        _i2cMutex->unlock();
    }
}
/**
 */
myMcp23017_rtos::myMcp23017_rtos(int pinInterrupt, xMutex *tex, int i2cAddress, WireBase *w)
{
    _mcp = myMcp23017::create(i2cAddress, w);
    _sem = new xBinarySemaphore();
    _i2cMutex = tex;
    _pinInterrupt = pinInterrupt;
}
/**
 */
void myMcp23017_rtos::start()
{
    // Start our task
    xTaskCreate(TaskMCP, "TaskMCP", 200, (void *)this, 1, NULL);

    // Enable interrupts generation at chip level
    _mcp->start();

    // Hook our interrupt handler
    lnNoInterrupt();
    pinMode(_pinInterrupt, INPUT_PULLUP);
    attachInterrupt(_pinInterrupt, _myInterrupt, _sem, FALLING);
    interrupts();
}

// EOF
