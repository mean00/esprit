/*
 *  (C) 2024 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "esprit.h"
#include "lnExti.h"
#include "lnIRQ.h"
#include "ln_rp_exti_priv.h"

#pragma clang diagnostic ignored "-Wextra"

// IO_IRQ_BANK0 is NVIC IRQ 13 on RP2040
#define LN_RP_IO_IRQ_BANK0 ((LnIRQ)13)

// GPIO count from pico-sdk platform_defs.h
#ifndef NUM_BANK0_GPIOS
#define NUM_BANK0_GPIOS 30
#endif

static LN_RP_EXTI *aExti = LN_RP_EXTI_BASE;
/**
 */
struct _lnExtiDescriptor
{
    uint32_t value;
    lnExtiCallback *cb;
    void *cookie;
};

static _lnExtiDescriptor _extiDesc[NUM_BANK0_GPIOS];

/**
 * @brief System init: clear all descriptors and registers, install IRQ handler
 *
 */
void lnExtiSysInit()
{
    for (int i = 0; i < NUM_BANK0_GPIOS; i++)
    {
        _lnExtiDescriptor *d = _extiDesc + i;
        d->cb = NULL;
        d->cookie = NULL;
        d->value = 0;
    }
    // Clear all interrupt registers
    // uint32_t c = LN_RP_EXTI_INTE_EDGE_LOW + LN_RP_EXTI_INTE_EDGE_HIGH;
    // uint32_t clr = c | (c << 8) | (c << 16) | (c << 24);
    for (int w = 0; w < 4; w++)
    {
        aExti->INTR[w] = 0xffffffffUL; // clear all
        aExti->CPU0_INTE[w] = 0;
        aExti->CPU0_INTF[w] = 0;
    }
    // Install shared handler for IO_IRQ_BANK0
    extern void rp_exti_irq_handler(void);
    lnSetInterruptHandler(LN_RP_IO_IRQ_BANK0, rp_exti_irq_handler);
    lnEnableInterrupt(LN_RP_IO_IRQ_BANK0);
}

/**
 * @brief Attach an interrupt callback to a GPIO pin
 *
 * @param pin  GPIO number (0..29 on RP2040)
 * @param edge LN_EDGE_RISING, LN_EDGE_FALLING, or LN_EDGE_BOTH
 * @param cb   Callback function
 * @param cookie User-supplied cookie passed to callback
 */
void lnExtiAttachInterrupt(const lnPin pin, const lnEdge edge, lnExtiCallback *cb, void *cookie)
{
    xAssert(pin < NUM_BANK0_GPIOS);
    _lnExtiDescriptor *d = _extiDesc + pin;
    uint32_t prog = 0;
    switch (edge)
    {
    case LN_EDGE_FALLING:
        prog = LN_RP_EXTI_INTE_EDGE_LOW;
        break;
    case LN_EDGE_RISING:
        prog = LN_RP_EXTI_INTE_EDGE_HIGH;
    case LN_EDGE_BOTH:
        prog = LN_RP_EXTI_INTE_EDGE_HIGH | LN_RP_EXTI_INTE_EDGE_LOW;
        break;
    default:
        xAssert(0);
        break;
    }

    d->value = prog;
    d->cb = cb;
    d->cookie = cookie;
}

/**
 * @brief Detach an interrupt from a GPIO pin
 *
 * @param pin GPIO number
 */
void lnExtiDetachInterrupt(const lnPin pin)
{
    xAssert(pin < NUM_BANK0_GPIOS);

    _lnExtiDescriptor *d = _extiDesc + pin;
    d->cb = NULL;
    d->value = 0;

    uint32_t word = LN_RP_EXTI_GPIO_WORD(pin);
    uint32_t val = aExti->CPU0_INTE[word];
    val &= ~LN_RP_EXTI_GPIO_MASK(pin);
    aExti->CPU0_INTE[word] = val;
    uint32_t c = LN_RP_EXTI_INTE_EDGE_LOW + LN_RP_EXTI_INTE_EDGE_HIGH;
    aExti->INTR[word] = c << LN_RP_EXTI_GPIO_SHIFT(pin); // clear pending
}

/**
 * @brief Enable the interrupt for a GPIO pin
 *
 * On RP2040 the edge bits (INTE/INTF) are the enables.
 * This function clears any pending interrupt; the edge bits
 * remain as configured by lnExtiAttachInterrupt.
 *
 * @param pin GPIO number
 */
void lnExtiEnableInterrupt(const lnPin pin)
{
    xAssert(pin < NUM_BANK0_GPIOS);
    int word = LN_RP_EXTI_GPIO_WORD(pin);
    _lnExtiDescriptor *d = _extiDesc + pin;
    aExti->CPU0_INTE[word] |= d->value << LN_RP_EXTI_GPIO_SHIFT(pin);
}

/**
 * @brief Disable the interrupt for a GPIO pin
 *
 * @param pin GPIO number
 */
void lnExtiDisableInterrupt(const lnPin pin)
{
    xAssert(pin < NUM_BANK0_GPIOS);
    int word = LN_RP_EXTI_GPIO_WORD(pin);
    aExti->CPU0_INTE[word] &= ~(0xf << LN_RP_EXTI_GPIO_SHIFT(pin));
}

/**
 * @brief Shared IRQ handler for IO_IRQ_BANK0
 *
 * Scans all GPIOs for pending interrupts and dispatches callbacks.
 */
void rp_exti_irq_handler(void)
{
    for (int reg = 0; reg < 4; reg++)
    {
        lnPin startPin = (lnPin)(reg * 8);
        uint32_t pending = aExti->CPU0_INTS[reg];
        if (!pending)
            continue;
        for (int bit = 0; bit < 8; bit++)
        {
            uint32_t mask = LN_RP_EXTI_GPIO_MASK(startPin);
            uint32_t val = pending & mask;

            if (val)
            {
                // Clear the pending bit
                aExti->INTR[reg] = val;
                // Dispatch callback
                _lnExtiDescriptor *d = _extiDesc + startPin;
                xAssert(d->cb);
                d->cb((lnPin)startPin, d->cookie);
            }
            startPin = (lnPin)(startPin + 1);
        }
    }
}

// EOF
