#include "esprit.h"
#include "lnRCU_priv.h"
#include "lnSCB_arm_priv.h"

static LN_SCB_Registers *aSCB = (LN_SCB_Registers *)0xE000ED00;
extern void lnExtiSWDOnly();
extern LN_RCU *arcu;
/**
 *
 */
void lnRunTimeInit()
{
    lnExtiSWDOnly(); // release jtag pins early, will be reset later
    __asm__("cpsid if    \n");
    aSCB->VTOR = 0;
    arcu->RSTCLK = 1 << 24; // reset all reset flags
}
/**
 *
 * @return
 */
void lnRunTimeInitPostPeripherals()
{
    lnExtiSWDOnly(); // release jtag pins, after reset, it should stick
    lnRemapTimerPin(1, PartialRemap);
}

// EOF
