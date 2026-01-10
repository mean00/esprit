#include "esprit.h"
#include "lnCpuID.h"
#include "lnFreeRTOS_pp.h"
#include "lnSCB_arm_priv.h"

static lnCpuID::LN_MCU _mcu;
static lnCpuID::LN_VENDOR _vendor;
static int _flashSize = 0;
static int _ramSize = 0;
extern LN_SCB_Registers *aSCB;

enum MCU_IDENTIFICATION
{
    MCU_NONE,
    MCU_GD32VF103_128K
};
static MCU_IDENTIFICATION _chipId;
/**
 *
 */
void lnCpuID::identify()
{

    if (_flashSize)
        return; // already done

    _chipId = MCU_GD32VF103_128K;
    _ramSize = 32;
    _flashSize = 64;
    _vendor = lnCpuID::LN_VENDOR_GD32;
}
/**
 *
 * @return
 */
lnCpuID::LN_VENDOR lnCpuID::vendor()
{
    return _vendor;
}
/**
 *
 * @return
 */
lnCpuID::LN_MCU lnCpuID::mcu()
{
    return _mcu;
}
/**
 */
const char *lnCpuID::mcuAsString()
{
    switch (_mcu)
    {
    case LN_MCU_RISCV:
        return "RISCV";
        break;
    default:
        return "????";
    }
}

/**
 *
 * @return
 */
int lnCpuID::flashSize()
{
    return _flashSize;
}
/**
 *
 * @return
 */
int lnCpuID::ramSize()
{
    return _ramSize;
}
/**
 *
 * @return
 */
const char *lnCpuID::idAsString()
{
    switch (_chipId)
    {
    case MCU_GD32VF103_128K:
        return "GD32VF103/128kB";
        break;
    default:
        break;
    }
    xAssert(0);
    return "???";
}
/**
 */
int lnCpuID::clockSpeed()
{
    return SystemCoreClock;
}
