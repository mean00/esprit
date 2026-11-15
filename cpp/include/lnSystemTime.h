/**

 */
#pragma once
#include "stdint.h"
uint32_t lnGetUs();
uint64_t lnGetUs64();
#ifdef __cplusplus
extern "C" {
#endif
void lnDelayUs(uint32_t wait); // us
#ifdef __cplusplus
}
#endif
void lnDelay(uint32_t wait);   // ms
#define lnDelayMs lnDelay
uint32_t lnGetMs();
