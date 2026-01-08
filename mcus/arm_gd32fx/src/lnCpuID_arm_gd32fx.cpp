#include "esprit.h"
#include "lnCpuID.h"
#include "lnFreeRTOS_pp.h"
#include "lnSCB_arm_priv.h"

static lnCpuID::LN_MCU _mcu;
static lnCpuID::LN_VENDOR _vendor;
static uint32_t _cpuId = 0;
static int _flashSize = 0;
static int _ramSize = 0;
extern LN_SCB_Registers *aSCB;

enum MCU_IDENTIFICATION
{
    MCU_NONE,
    MCU_STM32_64K,
    MCU_STM32_128K,
    MCU_GD32_64K,
    MCU_GD32_128K,
    MCU_GD32F3_256K,
    MCU_CH32F1_64K
};

static MCU_IDENTIFICATION _chipId;
uint32_t MVFR0;
uint32_t MVFR1;

extern bool FMC_hasFastUnlock();

/**
 *
 */
void lnCpuID::identify()
{

    if (_flashSize)
        return; // already done

    _cpuId = aSCB->CPUID;
    //   GD32: 41 2 f c23  1
    // Implementer 41 : ARM
    // VARIANT 2
    // Architecture C -> ARM V6
    // Partno :  0xC23 => Cortex M3
    // Rev=1

    // GD32F3
    // 0x41 0 f c24 1
    // ARM
    // variant 0
    // C24 cortex m4
    // Rev 1

    // STM32F1 : 0x41 1 f c23 1
    // Variant 1
    // Architecture C -> ARM V6
    // Partno : C23  => Cortex M3
    // Rev=1

    // CH32F103 : 0x41 2 f c23 1
    // Same as STM32F1
    //
    //

    // cortex M4, do we have FPU ?
    if (((_cpuId >> 4) & 0xfff) == 0xc24)
    {
        //  https://developer.arm.com/documentation/100166/0001/Floating-Point-Unit/FPU-programmers-model/Enabling-the-FPU?lang=en
        uint32_t *fp = (uint32_t *)0xE000EF40;
        MVFR0 = fp[0];
        MVFR1 = fp[1];
        if (((MVFR0 >> 16) & 0xff) == 0x11) // hw multiply & divide
        {
            // Yes, activate FPU
            volatile uint32_t *cpacr = (uint32_t *)0xE000ED88;
            uint32_t c = cpacr[0];
            c |= (3 << 20) + (3 << 22); // enable priviledged & unpriviledged
                                        // https://interrupt.memfault.com/blog/cortex-m-rtos-context-switching
            cpacr[0] = c;
        }
    }

    // 2nd part ram & flash size
    uint32_t density = *(uint32_t *)0x1FFFF7E0;
    int ramSize = density >> 16;
    _flashSize = density & 0xffff;
    // If we have the ram size, it is a gd32 chip. Do something better...
    if (ramSize != 0xffff) // GD
    {
        _vendor = LN_VENDOR_GD;
        switch (_flashSize)
        {
        case 128:
            _chipId = MCU_GD32_128K;
            break;
        case 64:
            _chipId = MCU_GD32_64K;
            break;
        case 256:
            _chipId = MCU_GD32F3_256K;
            break;
        default:
            xAssert(0);
            break;
        }
        _ramSize = ramSize;
    }
    else // STM32, we dont have the ram size
    {
        // do we have valid serial number ?
        // can use fast unlock ?
        if (FMC_hasFastUnlock())
        {
            _vendor = LN_VENDOR_WCH;
            _chipId = MCU_CH32F1_64K;
        }
        else
        {
            _vendor = LN_VENDOR_STM;
            if (_flashSize == 128)
                _chipId = MCU_STM32_128K;
            else if (_flashSize == 64)
                _chipId = MCU_STM32_64K;
            else
                xAssert(0);
            _ramSize = 20;
        }
    }
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
    case LN_MCU_ARM_F3:
        return "ARMF3";
        break;
    case LN_MCU_ARM_F4:
        return "ARMF4";
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
    case MCU_STM32_64K:
        return "STM32F1/64kB";
        break;
    case MCU_STM32_128K:
        return "STM32F1/128kB";
        break;
    case MCU_GD32_64K:
        return "GD32F1/64kB";
        break;
    case MCU_GD32_128K:
        return "GD32F1/128kB";
        break;
    case MCU_GD32F3_256K:
        return "GD32F3/256kB";
        break;
    case MCU_CH32F1_64K:
        return "CH32F1/64kB";
        break;
    default:
        break;
    }
    xAssert(0);
    return "???";
}
/**
 *
 * @return
 */
int lnCpuID::clockSpeed()
{
    return SystemCoreClock;
}
