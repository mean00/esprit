
#include "esprit.h"
#include "lnSerial.h"
/*
 *
 */
extern "C" void deadEnd(int code)
{
#ifdef __xtensa__
    asm("break 1, 15");
#else
#ifdef __riscv
    __asm__("ebreak");
#endif
#endif
    //    lnSoftSystemReset();
}
/**
 *
 *
 */
extern "C" void _putchar(char c)
{
    xAssert(0);
}
lnSerialTxOnly *createLnSerialTxOnly(int instance, bool dma, bool buffered)
{
    xAssert(0);
    return NULL;
}
//  EOF
