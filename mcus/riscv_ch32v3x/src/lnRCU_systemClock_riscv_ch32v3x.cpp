/**
 * @file lnRCU_systemClock_riscv_ch32v3x.cpp
 * @author MEAN00 fixounet@free.fr
 * @brief  This deals with the clock tree (except usb)
 * @version 0.1
 *
 * @copyright Copyright (c) 2021-2024
 *
 */
#include "esprit.h"
#include "lnCpuID.h"
#include "lnRCU.h"
#include "lnRCU_priv.h"

extern LN_RCU *arcu;

//
#define CH32_APB1_DIVIDER 0 // If set to 1 APB1 is sysclock /2, if set to 0 APB1 is sysclock

//
#ifndef CLOCK_XTAL_VALUE   // 8mhz quartz
#define CLOCK_XTAL_VALUE 8 // 8mhz quartz
#endif
#define CLOCK_TARGET_SYSCLOCK (LN_MCU_SPEED / (1000 * 1000)) // 108 Mhz
//
#define LN_CLOCK_IRC8 0
#define LN_CLOCK_XTAL 16
#define LN_CLOCK_PLL 24
//
#define CH32_CFG0_PLL_MASK (0xf << 18)
#define CH32_CFG0_PLL_VALUE(x) ((x) << 18)
#define CH32_CFG0_PLL_EXTERNAL (1 << 16)

#define CH32_CFG0_AHB_MASK (0xf << 4)
#define CH32_CFG0_AHB_DIVIDER(x) ((x) << 4)

#define CH32_CFG0_APB1_MASK (7 << 8)
#define CH32_CFG0_APB1_DIVIDER(x) ((x) << 8)

#define CH32_CFG0_APB2_MASK (7 << 11)
#define CH32_CFG0_APB2_DIVIDER(x) ((x) << 11)

#define CH32_CFG0_SYSCLOCK_IN_USE(x) ((x >> 2) & 3)

//
uint32_t _rcuClockApb1 = 108000000 / 2;
uint32_t _rcuClockApb2 = 108000000;
extern "C" uint32_t SystemCoreClock;
uint32_t SystemCoreClock = 0;
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
    case pTIMER1:
    case pTIMER2:
    case pTIMER3:
        return _rcuClockApb1;
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
/**
 * @brief
 *
 * @param mask
 */
static void waitCfg0Bit(int mask)
{

    while (!(arcu->CFG0 & mask))
    {
        __asm__("nop");
    }
}

/**
 *
 * @param multiplier
 * @param predivider
 * \briefs this translates the wanted value into value to put in
 *  i.e. if you want x10, you use multipliers[10]
 */

static const uint8_t Multipliers[] = {
    0, 0, 0, 1, // 0 1 2 3
    2, 3, 4, 5, // 4 5 6 7
    6, 7, 8, 9, // 8 9 10 11
    // the ones below dont work on stm32 !
    10, 11, 12, 13, // 12 13 14 15
    14, 14, 15      // 16 17 18 19
};
/**
 * \brief setup the pll to achieve the wanted frequency
 * @param multiplier of the input clock
 * @param external using crystal (external=true) or internal oscillator (false)
 */
void setPll(int multiplier, bool external)
{
    arcu->CFG1 = 0;
    xAssert(multiplier < sizeof(Multipliers));
    int pllMultiplier = Multipliers[multiplier];

    // Set PLL multiplier
    uint32_t c0 = arcu->CFG0;
    c0 &= CH32_CFG0_PLL_MASK;
    c0 |= CH32_CFG0_PLL_VALUE(pllMultiplier); // PLL Multiplier, ignore MSB

    if (external)
        c0 |= CH32_CFG0_PLL_EXTERNAL;
    else
        c0 &= ~CH32_CFG0_PLL_EXTERNAL;
    arcu->CFG0 = c0;

    // start PLL...
    arcu->CTL |= LN_RCU_CTL_PLLEN;
    waitControlBit(LN_RCU_CTL_PLLSTB); // Wait Xtal stable
}

/**
 * @brief
 *
 * @param clock
 * @param enabled
 */
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
 * @brief
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
    arcu->CFG0 = sysClock;                  // set sysclock to IRC8
    while (1)
    {
        if (CH32_CFG0_SYSCLOCK_IN_USE(arcu->CFG0) == 0) // 0 is IRC8
            break;
    }
    // Ok now we can disable PLL and Xtal
    enableDisableClock(LN_CLOCK_XTAL, false);
    enableDisableClock(LN_CLOCK_PLL, false);

    //________________________
    //
    //________________________
    int inputClock;
    bool useExternalClock;

#ifdef LN_USE_INTERNAL_CLOCK
    inputClock = 8 / 2; // HSI is divided by 2
    useExternalClock = false;
#else
    // start crystal...
    arcu->CTL |= LN_RCU_CTL_HXTALEN;    // start Xtal
    waitControlBit(LN_RCU_CTL_HXTASTB); // Wait Xtal stable
    inputClock = CLOCK_XTAL_VALUE;
    useExternalClock = true;
#endif
#ifdef USE_CH32V208W
    // assuming we have a 32 Mhz xtal, it will be divided automatically by 4
    // also set USB & prescaller to follow WCH example
    // Magic USB prescaler value of 3
    arcu->CFG0 |= LN_RCU_CFG0_USBPSC(3); // weird CH32V208W stuff, USB divided by 5, PLL source = HSE/2 = 16 Meg
    uint32_t c0 = arcu->CFG0;
    c0 &= ~LN_RCU_CFG0_HPRE_WCH(0xf);
    c0 |= LN_RCU_CFG0_HPRE_WCH(8); // HPRE=8, so  HBCLOCK=SYSCLOCK/2
    arcu->CFG0 = c0;
    const uint32_t predivider = 4; // on the CH32V208w the pll insput is xtal/4 (or /8)
#else
    const uint32_t predivider = 1;

#endif

    int multiplier = (predivider * CLOCK_TARGET_SYSCLOCK) / inputClock;
    setPll(multiplier, useExternalClock); // Program PLL to get the value we want

    SystemCoreClock = (((inputClock * multiplier) / predivider) * 1000000); // update globals reflecting SysClk...
    _rcuClockApb1 = SystemCoreClock >> CH32_APB1_DIVIDER;
    _rcuClockApb2 = SystemCoreClock;

    // Setup AHB...
    // AHB is sysclk:1
    // APB1=AHB/2
    // APB2=AHB/1

    uint32_t clks = arcu->CFG0;
    clks &= CH32_CFG0_AHB_MASK;
    clks |= CH32_CFG0_AHB_DIVIDER(0); // 0-> 1:1

    clks &= CH32_CFG0_APB1_MASK;
    clks |= CH32_CFG0_APB1_DIVIDER(CH32_APB1_DIVIDER * 4); // 0-> 1:1, 4-> 1:2

    clks &= CH32_CFG0_APB2_MASK;
    clks |= CH32_CFG0_APB2_DIVIDER(0); // 0-> 1:1

    arcu->CFG0 = clks;

    // now switch system clock to pll
    clks &= ~(LN_RCU_CFG0_SYSCLOCK_MASK);
    clks |= LN_RCU_CFG0_SYSCLOCK_PLL;
    arcu->CFG0 = clks;

#warning DO WE NEED WAIT STATE FOR THE FLASH (CH32) ????
#warning DO WE NEED WAIT STATE FOR THE FLASH (CH32) ????
#warning DO WE NEED WAIT STATE FOR THE FLASH (CH32) ????
#warning DO WE NEED WAIT STATE FOR THE FLASH (CH32) ????
#warning DO WE NEED WAIT STATE FOR THE FLASH (CH32) ????

} // EOF
