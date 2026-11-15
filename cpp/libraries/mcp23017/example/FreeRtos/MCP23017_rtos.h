/**
 */
#pragma once
#include "MapleFreeRTOS1000_pp.h"
#include "Wire.h"
#include "mcp23017.h"
/**
 *
 * @param i2cAddress
 * @param pinInterrupt
 * @param tex
 * @param w
 */
class myMcp23017_rtos
{
  public:
    myMcp23017_rtos(int pinInterrupt, xMutex *tex, int i2cAddress = 0, WireBase *w = NULL);
    void start();
    void runTask();

    myMcp23017 *mcp()
    {
        return _mcp;
    }
    void process()
    {
        _mcp->process();
    }

  protected:
    myMcp23017 *_mcp;
    xMutex *_i2cMutex;
    int _pinInterrupt;
    xBinarySemaphore *_sem;
};