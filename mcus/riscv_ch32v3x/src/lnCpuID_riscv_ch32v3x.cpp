#include "esprit.h"
#include "lnCpuID.h"
#include "lnFreeRTOS_pp.h"

static lnCpuID::LN_MCU _mcu;
static lnCpuID::LN_VENDOR _vendor;
static int _flashSize = 0;
static int _ramSize = 0;

enum MCU_IDENTIFICATION
{
    MCU_NONE,
    MCU_CH32V307_128K,
};
static MCU_IDENTIFICATION _chipId;
/**
 *
 */
void lnCpuID::identify()
{

    if (_flashSize)
        return; // already done

    _chipId = MCU_CH32V307_128K;
    _ramSize = 32;
    _flashSize = 64;
    _vendor = lnCpuID::LN_VENDOR_WCH;
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
    case MCU_CH32V307_128K:
        return "CH32V307/128kB";
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
