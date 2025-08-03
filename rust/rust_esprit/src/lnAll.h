#ifdef __ARM_ARCH
#undef uint32_t
#define _UINT32_T_DECLARED
#define uint32_t unsigned int
#endif
#include "esprit.h"
#include "lnADC.h"
#include "lnBasicTimer.h"
#include "lnCpuID.h"
#include "lnDAC.h"
#include "lnDebug.h"
#include "lnDma.h"
#include "lnExti.h"
#include "lnFMC.h"
// #include "lnFreeRTOS.h"
// #include "lnFreeRTOS_pp.h"
// #include "lnGPIO.h"
//  #include "lnI2C.h"
#include "lnIRQ.h"
// #include "lnIRQ_arm.h"
//  #include "lnIRQ_riscv.h"
//  #include "lnFreeRTOS.h"
//  #include "lnFreeRTOS_pp.h"
#include "lnPeripherals.h"
#include "lnPrintf.h"
#include "lnRCU.h"
// #include "lnSPI.h"
//  RMED #include "lnSerial.h"
#include "lnStopWatch.h"
#include "lnTimer.h"
#include "systemHelper.h"

// #include "../libraries/tinyUsb/include/lnUsbCDC.h"
