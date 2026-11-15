#pragma once
#pragma once
#include "lnIRQ.h"
#include "lnPeripheral_priv.h"

// Each timer has channel 0,1,2,3

/**
 */

struct LN_DAC_DATA
{
    uint32_t DAC_R12DH; // 8
    uint32_t DAC_L12DH; // C
    uint32_t DAC_R8DH;  // 10
};

struct LN_DAC_Registersx
{
    uint32_t CTL; // 0
    uint32_t SWT; // 4

    LN_DAC_DATA DATAS[2];   // 8 c 10 14 18 1c
    LN_DAC_DATA BOTH_DATAS; // 20 24 28
    uint32_t DAC0_DO;       // 2c
    uint32_t DAC1_DO;       // 30
};
/**
 */
typedef volatile LN_DAC_Registersx LN_DAC_Registers;
/**
 */

//-------------- CTL ---------

#define LN_DAC_CTL_DTSEL_SOURCE_SW (7)
#define LN_DAC_CTL_DTSEL_SOURCE_T5 (0)
#define LN_DAC_CTL_DTSEL_SOURCE_T2 (1)
#define LN_DAC_CTL_DTSEL_SOURCE_T6 (2)
#define LN_DAC_CTL_DTSEL_SOURCE_T4 (3)
#define LN_DAC_CTL_DTSEL_SOURCE_T1 (4)
#define LN_DAC_CTL_DTSEL_SOURCE_T3 (5)
#define LN_DAC_CTL_DTSEL_SOURCE_EXTI9 (6)

#define LN_DAC_CTL_ENABLE(dac) (1 << (0 + 16 * dac))             // dac enable
#define LN_DAC_CTL_DBOFF(dac) (1 << (1 + 16 * dac))              // output buffer disable
#define LN_DAC_CTL_DTEN(dac) (1 << (2 + 16 * dac))               // Trigger enable
#define LN_DAC_CTL_DDMAEN0(dac) (1 << (12 + 16 * dac))           // dact enable
#define LN_DAC_CTL_DTSEL(dac, source) (source << (3 + 16 * dac)) // dact enable
#define LN_DAC_CTL_DTSEL_MASK(dac) (~(7 << (3 + 16 * dac)))      // dact enable

//-------------- SWT ---------

#define LN_DAC_SWT_ENABLE(dac) (1 << (dac)) // software trigger

// EOF
