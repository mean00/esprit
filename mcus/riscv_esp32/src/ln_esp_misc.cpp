
#include "esprit.h"
#include "lnSerial.h"

//
#include "esp_clk_tree.h"
#include "esp_flash.h"
#include "esp_heap_caps.h"
#include "esp_system.h"
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
/*
 *
 */
void lnSoftSystemReset()
{
    esp_restart();
}
/**
 *
 *
 */
extern "C" void _putchar(char c)
{
    xAssert(0);
}
/*
 *
 */
lnSerialTxOnly *createLnSerialTxOnly(int instance, bool dma, bool buffered)
{
    xAssert(0);
    return NULL;
}
//  EOF
