/*
 * Copyright (c) 2020 Raspberry Pi (Trading) Ltd.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

#include "pico.h"
#include <stdarg.h>
#include <stdio.h>
#include <sys/time.h>
#include <sys/times.h>
#include <unistd.h>

#include "hardware/regs/m0plus.h"
#include "hardware/regs/resets.h"
#include "hardware/structs/mpu.h"
#include "hardware/structs/padsbank0.h"
#include "hardware/structs/scb.h"

#include "hardware/clocks.h"
#include "hardware/irq.h"
#include "hardware/resets.h"

#define LN_IRQ_VECTOR_NUMBER 48
uint32_t __attribute__((section(".ram_vector_table"))) ram_vector_table[LN_IRQ_VECTOR_NUMBER];

#if RP_SDK_VERSION == 2
#define padsbank0_hw_t pads_bank0_hw_t
extern void runtime_init_clocks();
#define clocks_init runtime_init_clocks
#endif

void runtime_init(void)
{
    // Reset all peripherals to put system into a known state,
    // - except for QSPI pads and the XIP IO bank, as this is fatal if running from flash
    // - and the PLLs, as this is fatal if clock muxing has not been reset on this boot
    // - and USB, syscfg, as this disturbs USB-to-SWD on core 1
    reset_block(~(RESETS_RESET_IO_QSPI_BITS | RESETS_RESET_PADS_QSPI_BITS | RESETS_RESET_PLL_USB_BITS |
                  RESETS_RESET_USBCTRL_BITS | RESETS_RESET_SYSCFG_BITS | RESETS_RESET_PLL_SYS_BITS));

    // Remove reset from peripherals which are clocked only by clk_sys and
    // clk_ref. Other peripherals stay in reset until we've configured clocks.
    unreset_block_wait(RESETS_RESET_BITS & ~(RESETS_RESET_ADC_BITS | RESETS_RESET_RTC_BITS | RESETS_RESET_SPI0_BITS |
                                             RESETS_RESET_SPI1_BITS | RESETS_RESET_UART0_BITS |
                                             RESETS_RESET_UART1_BITS | RESETS_RESET_USBCTRL_BITS));

    // After calling preinit we have enough runtime to do the exciting maths
    // in clocks_init
    clocks_init();

    // Peripheral clocks should now all be running
    unreset_block_wait(RESETS_RESET_BITS);

#if !PICO_IE_26_29_UNCHANGED_ON_RESET
    // after resetting BANK0 we should disable IE on 26-29
    padsbank0_hw_t *padsbank0_hw_clear = (padsbank0_hw_t *)hw_clear_alias_untyped(padsbank0_hw);
    padsbank0_hw_clear->io[26] = padsbank0_hw_clear->io[27] = padsbank0_hw_clear->io[28] = padsbank0_hw_clear->io[29] =
        PADS_BANK0_GPIO0_IE_BITS;
#endif

    __builtin_memset(ram_vector_table, 0, LN_IRQ_VECTOR_NUMBER * 4);
    __builtin_memcpy(ram_vector_table, (uint32_t *)scb_hw->vtor, 4 * 4); // just copy reset & msp
    scb_hw->vtor = (uintptr_t)ram_vector_table;

    // MEANX spin_locks_reset();
    irq_init_priorities();
    // MEANX alarm_pool_init_default();

    // Start and end points of the constructor list,
    // defined by the linker script.
    extern void (*__init_array_start)(void);
    extern void (*__init_array_end)(void);

    // Call each function in the list.
    // We have to take the address of the symbols, as __init_array_start *is*
    // the first function pointer, not the address of it.
    for (void (**p)(void) = &__init_array_start; p < &__init_array_end; ++p)
    {
        (*p)();
    }
}
void runtime_run_per_core_initializers(void) {}
