/*
 *  (C) 2024 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once

#include "ln_rp_memory_map.h"

/**
 * @brief IO_BANK0 GPIO interrupt registers
 *
 * Base address: LN_IO_BANK0_BASE_ADR + 0xF0
 *
 * Each GPIO has 4 interrupt bits (INTR, INTE, INTF, INTS) spread across
 * 4 word arrays. Bit N in each word corresponds to GPIO N.
 *
 * INTR      (offset 0x00): Raw interrupt status (pending). Write 1 to clear.
 * CPU0_INTE (offset 0x10): enable.
 * CPU0_INTF (offset 0x20): force.
 * CPU0_INTS (offset 0x30): Masked interrupt status (INTE & INTR).
 *
 * For RP2040: only CPU0 registers are used.
 */
struct LN_RP_EXTIx
{
    uint32_t INTR[4];      // 0x00  Raw interrupt status / clear
    uint32_t CPU0_INTE[4]; // 0x10  ENABLE
    uint32_t CPU0_INTF[4]; // 0x20  FORCE-edge / level-low enable
    uint32_t CPU0_INTS[4]; // 0x30  STATUS Masked interrupt status
};

typedef volatile LN_RP_EXTIx LN_RP_EXTI;

#define LN_RP_EXTI_BASE ((LN_RP_EXTI *)(LN_IO_BANK0_BASE_ADR + 0xF0))

// Bit position helpers
#define LN_RP_EXTI_GPIO_WORD(n) ((n) >> 3)
#define LN_RP_EXTI_GPIO_SHIFT(n) ((((n) & 0x7)) * 4)
#define LN_RP_EXTI_GPIO_MASK(n) (0xf << ((((n) & 0x07)) * 4))
#define LN_RP_EXTI_GPIO_GET_VALUE(n, val) ((val >> LN_RP_EXTI_GPIO_SHIFT(n)) & 0xf)

// Each GPIO is 2 bits with the following meaning (needs to be shifted)
//
#define LN_RP_EXTI_INTE_LEVEL_LOW 0x01
#define LN_RP_EXTI_INTE_LEVEL_HIGH 0x02
#define LN_RP_EXTI_INTE_EDGE_LOW 0x04
#define LN_RP_EXTI_INTE_EDGE_HIGH 0x08
#define LN_RP_EXTI_INTE_MASK 0x0F

// EOF
