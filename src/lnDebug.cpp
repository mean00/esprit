/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "esprit.h"
#include "lnSerial.h"
#include "stdarg.h"
#define LOGGER_USE_DMA 1

static lnLoggerFunction *loggerFunction = NULL;
extern lnSerialTxOnly *serial0;
lnSerialTxOnly *serial0 = NULL;
volatile uint32_t lnScratchRegister;
static lnMutex *loggerMutex;
extern "C" void Logger_crash(const char *st)
{
    serial0->init();
    serial0->setSpeed(115200);
    serial0->rawWrite(strlen(st), (const uint8_t *)st);
}
/**

*/
extern "C" void Logger_chars(int n, const char *data)
{
    if (!n)
        return; // 0 sized dma does not work...
    if (loggerFunction)
        loggerFunction(n, data);
    else
        serial0->transmit(n, (uint8_t *)data);
}

extern "C" int Logger_C(const char *fmt, ...)
{
    static char buffer[128];

    va_list va;
    va_start(va, fmt);
    vsnprintf(buffer, 127, fmt, va);

    buffer[127] = 0;
    va_end(va);
    Logger_chars(strlen(buffer), buffer);
    return 0;
}
/**
 */
extern "C" void setLogger(lnLoggerFunction *f)
{
    loggerFunction = f;
}
/**
 *
 * @param fmt
 */
void Logger(const char *fmt...)
{
    static char buffer[128];

    if (fmt[0] == 0)
        return;

    loggerMutex->lock();
    va_list va;
    va_start(va, fmt);
    vsnprintf(buffer, 127, fmt, va);

    buffer[127] = 0;
    va_end(va);
    Logger_chars(strlen(buffer), buffer);
    loggerMutex->unlock();
}
/**
 *
 */
void LoggerInitMutex()
{
    loggerMutex = new lnMutex;
}
/**
 *
 */
void LoggerInit()
{
    LoggerInitMutex();

    int debugUart = 0;
#ifdef LN_DEBUG_UART
    debugUart = LN_DEBUG_UART;
#endif
    bool dma = false;
#ifdef LOGGER_USE_DMA
    dma = true;
#endif

    bool buffered = true;
    serial0 = createLnSerialTxOnly(debugUart, dma, buffered);
    serial0->init();
    serial0->setSpeed(115200);
}
