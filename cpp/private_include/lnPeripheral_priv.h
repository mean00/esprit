/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once

#if LN_ARCH == LN_ARCH_RISCV
#if LN_MCU == LN_MCU_CH32V3x
#include "lnPeripheral_priv_ch32v3x.h"
#else
#include "lnPeripheral_priv_stm32.h"
#endif
#else // ARM
      //
#include "lnPeripheral_priv_stm32.h"
#endif
