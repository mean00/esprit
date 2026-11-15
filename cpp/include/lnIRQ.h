/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "esprit.h"

#if LN_ARCH == LN_ARCH_RISCV
#include "lnIRQ_riscv.h"
#else
#if LN_ARCH == LN_ARCH_ARM
#include "lnIRQ_arm.h"
#else
#error UNSUPPORTED ARCH
#endif
#endif

typedef void(lnInterruptHandler)(void);

void lnEnableInterrupt(const LnIRQ &irq);
void lnDisableInterrupt(const LnIRQ &irq);
void lnSetInterruptHandler(const LnIRQ &irq, lnInterruptHandler *handler);

// prio between 0 and 5
// 0 is less urgent, 5 is more urgent

void lnIrqSetPriority(const LnIRQ &irq, int prio);
