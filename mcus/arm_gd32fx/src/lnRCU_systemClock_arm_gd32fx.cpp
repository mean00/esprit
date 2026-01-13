/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "esprit.h"
#include "lnCpuID.h"
#include "lnRCU.h"
#include "lnRCU_priv.h"

extern LN_RCU *arcu;
//

uint32_t _rcuClockApb1 = 108000000 / 2;
uint32_t _rcuClockApb2 = 108000000;
extern "C" uint32_t SystemCoreClock;
uint32_t SystemCoreClock = 0;
uint32_t jtagId = 0;
/**
 *
 * @param periph
 * @return
 */
uint32_t lnPeripherals::getClock(const Peripherals periph)
{
    switch (periph)
    {
    case pTIMER5:
    case pTIMER6:
        return _rcuClockApb1 * 2;
        break;
    case pUART0:
    case pTIMER0:
    case pSPI0:
    case pAF:
    case pADC0:
    case pADC1:
        return _rcuClockApb2;
        break;
    case pAPB1:
        return _rcuClockApb1;
        break;
    case pAPB2:
        return _rcuClockApb2;
        break;
    case pSYSCLOCK:
        return SystemCoreClock;
        break;
    default:
        return _rcuClockApb1;
        break;
    }
}
/**
 * \brief wait for the "stabilized" bit to be set
 * @param bit
 */

static void waitControlBit(int mask)
{
    while (!(arcu->CTL & mask))
    {
        __asm__("nop");
    }
}

static void waitCfg0Bit(int mask)
{

    while (!(arcu->CFG0 & mask))
    {
        __asm__("nop");
    }
}

#define CLOCK_XTAL_VALUE 8                                   // 8mhz quartz
#define CLOCK_TARGET_SYSCLOCK (LN_MCU_SPEED / (1000 * 1000)) // 108 Mhz
#define CLOCK_TARGET_PREDIV 1

//{CTL = 0x38683, CFG0 = 0x400, CIR = 0x0, APB2RSTR = 0x0, APB1RSTR = 0x0, AHBENR = 0x14, APB2ENR = 0x0, APB1ENR = 0x0,
// BDCR = 0x18, CSR = 0x1c000000} {CTL = 0x38683, CFG0 = 0x1d0400, CIR = 0x0, APB2RSTR = 0x0, APB1RSTR = 0x0, AHBENR =
// 0x14, APB2ENR = 0x0, APB1ENR = 0x0, BDCR = 0x18, CSR = 0x1c000000} {CTL = 0x3038683, CFG0 = 0x1d0400, CIR = 0x0,
// APB2RSTR = 0x0, APB1RSTR = 0x0, AHBENR = 0x14, APB2ENR = 0x0, APB1ENR = 0x0, BDCR = 0x18, CSR = 0x1c000000}

/**
 *
 * @param multiplier
 * @param predivider
 */

static const int Multipliers[] = {
    0, 0, 0, 1, // 0 1 2 3
    2, 3, 4, 5, // 4 5 6 7
    6, 7, 8, 9, // 8 9 10 11
    // the ones below dont work on stm32 !
    10, 11, 12, 12, // 12 13 14 15
    14, 16, 17, 18, // 16 17 18 19
    // Needed for internal clock
    19, 20, 21, 22, 23, // 20 21 22 23 24 96Mhz

};
/**
 *
 * @param multiplier
 * @param predivider
 * @param external
 */
void setPll(int inputClock, int multiplier, int predivider, bool external)
{
    volatile uint32_t *control = &(arcu->CTL);
    volatile uint32_t *cfg0 = &(arcu->CFG0);
    volatile uint32_t *cfg1 = &(arcu->CFG1);

    uint32_t c0 = *cfg0;
    *cfg1 = 0;
    if (multiplier > sizeof(Multipliers) / sizeof(int))
    {
        xAssert(0);
    }
    int pllMultiplier = Multipliers[multiplier];

    SystemCoreClock = (inputClock * multiplier * 1000000) / predivider;
    _rcuClockApb1 = SystemCoreClock / 2;
    _rcuClockApb2 = SystemCoreClock;

    // Set PLL multiplier
    c0 &= ~((0xf) << 18);
    c0 |= ((pllMultiplier) & 0x0f) << 18; // PLL Multiplier, ignore MSB
    if (pllMultiplier & 0x10)
        c0 |= 1 << 27;
    else
        c0 &= ~(1 << 27);
    if (external)
        c0 |= LN_RCU_CFG0_PLLSEL;
    else
        c0 &= ~LN_RCU_CFG0_PLLSEL;
    *cfg0 = c0;

    // start PLL...
    *control |= LN_RCU_CTL_PLLEN;
    waitControlBit(LN_RCU_CTL_PLLSTB); // Wait Xtal stable
}

//
#define LN_CLOCK_IRC8 0
#define LN_CLOCK_XTAL 16
#define LN_CLOCK_PLL 24
static void enableDisableClock(int clock, bool enabled)
{
    int setBit = 1 << clock;
    int statusBit = 1 << (clock + 1);
    if (enabled)
    {
        arcu->CTL |= setBit;
        waitControlBit(statusBit);
    }
    else
    {
        arcu->CTL &= ~setBit;
    }
}
/**
 *
 */
void lnInitSystemClock()
{
    lnCpuID::identify();

    //________________________
    // Reset everything...
    //________________________
    // Enable IRC8 just in case
    enableDisableClock(LN_CLOCK_IRC8, true);
    // Switch SYS clock to IRC8
    uint32_t sysClock = arcu->CFG0;
    sysClock &= ~LN_RCU_CFG0_SYSCLOCK_MASK; //  0 is IRC8
    arcu->CFG0 = sysClock;
    while (1)
    {
        if (!(arcu->CFG0 & (LN_RCU_CFG0_SYSCLOCK_MASK << 2)))
            break;
    }
    // Ok now we can disable PLL and Xtal
    enableDisableClock(LN_CLOCK_XTAL, false);
    enableDisableClock(LN_CLOCK_PLL, false);

    //________________________
    //
    //________________________

    volatile uint32_t *control = &(arcu->CTL);
    volatile uint32_t *cfg0 = &(arcu->CFG0);

    int inputClock;
    bool useExternalClock;

#ifndef LN_USE_INTERNAL_CLOCK
    {
        // start crystal...
        *control |= LN_RCU_CTL_HXTALEN;
        waitControlBit(LN_RCU_CTL_HXTASTB); // Wait Xtal stable
        inputClock = CLOCK_XTAL_VALUE;
        useExternalClock = true;
    }
#else
    inputClock = 8 / 2; // HSI is divided by 2
    useExternalClock = false;
#endif

    int multiplier = CLOCK_TARGET_SYSCLOCK / inputClock;
    setPll(inputClock, multiplier, CLOCK_TARGET_PREDIV, useExternalClock); // 8*9/1=72 Mhz

    // Setup AHB...
    // AHB is Xtal:1, divider value=0
    // APB2=AHB/1
    uint32_t a = *cfg0;
    a &= ~((0xf) << 4); // AHB PRESCALER CLEAR
    a |= 0 << 4;        // AHB prescaler is CK_SYS
    a &= ~(3 << 11);    //  APB2 Prescaler clear
    a |= 0 << 11;       // APB2 is AHB
    *cfg0 = a;
    // APB1=AHB/2, divider value=4
    a &= ~(7 << 8);
    a |= 4 << 8; // APB1 is AHB/2
    *cfg0 = a;

    // it it is a STM32 chip, increase flash wait state
    // GD has ram, so  0 wait state is fine
    if (lnCpuID::vendor() != lnCpuID::LN_VENDOR_GD)
    {
        int ws = 0;
        if (CLOCK_TARGET_SYSCLOCK > 48)
            ws = 2;
        else if (CLOCK_TARGET_SYSCLOCK > 24)
            ws = 1;

        *(uint32_t *)0x40022000 = 0x30 + ws; // enable prefetch
    }

    // now switch system clock to pll
    a &= ~(LN_RCU_CFG0_SYSCLOCK_MASK);
    a |= LN_RCU_CFG0_SYSCLOCK_PLL;
    *cfg0 = a;

    // In debug mode, halt everthing we can
    volatile uint32_t *debug = (volatile uint32_t *)0xE0042000U;
    // stop timer 0..3 i2c0 i2c1  timer 4..7 timer 8..13
    jtagId = debug[0];
    debug[1] = (0xf << 10) + (1 << 15) + (0x1f << 16) + (0x3f << 25);
}
// EOF
