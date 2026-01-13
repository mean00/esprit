/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#include "esprit.h"
#include "lnCpuID.h"
#include "lnFMC.h"
#include "lnFMC_priv.h"
#include "lnPeripheral_priv.h"

LN_FMC *aFMC = (LN_FMC *)LN_FMC_ADR;

// extra bits for CH32 chips
#define LN_FMC_CTL_CH32_FASTUNLOCK (1 << 15)
#define LN_FMC_CTL_CH32_FASTERASE (1 << 17)

/**
 *
 */
class autoNoInterrupt
{
  public:
    autoNoInterrupt()
    {
        lnNoInterrupt();
    }
    ~autoNoInterrupt()
    {
        lnInterrupts();
    }
};

/**
 * \fn FMCX_hasFastUnlock
 * \brief This tries to do a fast unlock. it is one way to detect CH32F103 chips
 */

/**

*/
bool CH32_fastUnlock()
{
    // send unlock sequence
    aFMC->KEY = 0x45670123;
    aFMC->KEY = 0xCDEF89AB;
    // send fast unlock sequence
    volatile uint32_t *CHF103 = (uint32_t *)LN_FMC_ADR;
    CHF103[9] = 0x45670123;
    CHF103[9] = 0xCDEF89AB;

    uint32_t v = aFMC->CTL;
    return !(v & LN_FMC_CTL_CH32_FASTUNLOCK);
}
//
//
//
bool CH32_fastLock()
{
    aFMC->CTL |= LN_FMC_CTL_LK;
    return true;
}
// this check if the fast unlock bit is settable
// it is called early in the boot so it cannot use high level calls
bool FMC_hasFastUnlock()
{
    // reset fast unlock
    aFMC->CTL |= LN_FMC_CTL_CH32_FASTUNLOCK; // clear fast unlock, the bit should go to '1'
    if (!(aFMC->CTL & LN_FMC_CTL_CH32_FASTUNLOCK))
    {
        return false;
    }
    // send unlock sequence
    aFMC->KEY = 0x45670123;
    aFMC->KEY = 0xCDEF89AB;
    for (int i = 0; i < 50; i++)
        __asm__("nop");
    // send fast unlock sequence
    volatile uint32_t *CHF103 = (uint32_t *)LN_FMC_ADR;
    CHF103[9] = 0x45670123;
    CHF103[9] = 0xCDEF89AB;
    for (int i = 0; i < 50; i++)
        __asm__("nop");

    uint32_t v = aFMC->CTL;
    return !(v & LN_FMC_CTL_CH32_FASTUNLOCK);
}

/**
 * \fn unlock
 * \brief unlock the flash controller so we can write/erase
 */
static void unlock()
{
    // is it busy  ?
    xAssert(!(aFMC->STAT & LN_FMC_STAT_BUSY)); // should NOT be !
    if (!(aFMC->CTL & LN_FMC_CTL_LK))
        return; // already unlocked ? nothing to do
    // send unlock sequence
    aFMC->KEY = 0x45670123;
    aFMC->KEY = 0xCDEF89AB;
    lnDelayUs(100);
    // verify it is unlocked
    xAssert(!(aFMC->CTL & LN_FMC_CTL_LK));
}
/**
 * \fn wait till the busy bit disdappear
 */
static void waitNotBusy()
{
    // is it busy  ?
    while (1)
    {
        if (!(aFMC->STAT & LN_FMC_STAT_BUSY))
            return;
    }
}
/**

*/
bool lnFMC::eraseStm(const uint32_t startAddress, int sizeInKBytes)
{
    uint32_t adr = startAddress;
    // Check we target an area within the flash (up to 256 kB)
    xAssert(adr >= 0x800 * 0x10000);
    xAssert(adr < (0x800 * 0x10000 + 256 * 1024));
    // check the start address is 1 kB aligned
    xAssert(!(adr & ((1 << 10) - 1)));
    autoNoInterrupt noInt;
    unlock();
    // Erase each page
    for (int i = 0; i < sizeInKBytes; i++)
    {
        aFMC->CTL |= LN_FMC_CTL_PER;
        aFMC->ADDR = adr;
        aFMC->CTL |= LN_FMC_CTL_START;
        waitNotBusy();
        adr += 1024;
        aFMC->CTL &= ~LN_FMC_CTL_PER;
    }

    return true;
}
/**
  \brief Erase using 128 bytes chunk, CH32 specific operation
*/
bool lnFMC::eraseCh32(const uint32_t startAddress, int sizeInKBytes)
{
    uint32_t adr = startAddress;
    // Check we target an area within the flash (up to 256 kB)
    xAssert(adr >= 0x800 * 0x10000);
    xAssert(adr < (0x800 * 0x10000 + 256 * 1024));
    // check the start address is 1 kB aligned
    xAssert(!(adr & ((1 << 10) - 1)));
    autoNoInterrupt noInt;
    CH32_fastUnlock();
    // Erase each page
    for (int i = 0; i < sizeInKBytes * 8; i++)
    {
        aFMC->CTL |= LN_FMC_CTL_CH32_FASTERASE;
        aFMC->ADDR = adr;
        aFMC->CTL |= LN_FMC_CTL_START;
        waitNotBusy();
        adr += 128;
        aFMC->STAT |= LN_FMC_STAT_WP_ENDF; // clear end of process bit
        aFMC->CTL &= ~LN_FMC_CTL_CH32_FASTERASE;
        aFMC->CTL &= ~LN_FMC_CTL_START; // clear start
    }
    return true;
}

/**
 *
 * @param startAddress
 * @param sizeInKBytes
 * @return
 */
bool lnFMC::erase(const uint32_t startAddress, int sizeInKBytes)
{
    if (lnCpuID::vendor() == lnCpuID::LN_VENDOR_WCH)
        return eraseCh32(startAddress, sizeInKBytes);
    return eraseStm(startAddress, sizeInKBytes);
}
/**

*/

static bool checkWriting()
{
    uint32_t stat = aFMC->STAT;

    aFMC->CTL &= ~LN_FMC_CTL_PG;
    if (stat & (LN_FMC_STAT_PG_ERR + LN_FMC_STAT_WP_ERR))
    {
        aFMC->STAT = LN_FMC_STAT_PG_ERR + LN_FMC_STAT_WP_ERR;
        return false;
    }
    aFMC->STAT = LN_FMC_STAT_WP_ENDF;
    if (!(stat & LN_FMC_STAT_WP_ENDF))
        return false;
    return true;
}
/**
 *
 * @param startAddress
 * @param data
 * @param sizeInBytes
 * @return
 */
bool lnFMC::write(const uint32_t startAddress, const uint8_t *data, int sizeInBytes)
{
    uint32_t adr = startAddress;
    // Check alignment
    xAssert(adr >= 0x800 * 0x10000);
    xAssert(adr < (0x800 * 0x10000 + 256 * 1024));
    autoNoInterrupt noInt;
    unlock();

    // if not aligned at the beginning
    if (adr & 1)
    {
        // it is the high byte of the PREVIOUS adr
        uint8_t *previous = (uint8_t *)adr;
        previous--;
        uint16_t data16 = (data[0] << 8) + previous[0];
        // write it
        aFMC->CTL |= LN_FMC_CTL_PG;
        __asm__("nop");
        __asm__("nop");
        *(uint16_t *)previous = (uint32_t)data16;
        waitNotBusy();
        if (!checkWriting())
            return false;
        //
        adr++;
        sizeInBytes--;
        data++;
    }
    // do N 16 bytes aligned write
    volatile uint16_t *adr16 = (volatile uint16_t *)adr;
    int nbWord = sizeInBytes / 2;
    for (int i = 0; i < nbWord; i++)
    {
        uint16_t data16 = (data[0]) + (data[1] << 8);
        aFMC->CTL |= LN_FMC_CTL_PG;
        __asm__("nop");
        __asm__("nop");
        *adr16 = data16;
        waitNotBusy();
        if (!checkWriting())
            return false;
        adr16++;
        data += 2;
    }
    // not aligned at the end ?
    if (sizeInBytes & 1)
    {
        // it is the low byte of the NEXT adr
        uint8_t *next = (uint8_t *)adr16;

        uint16_t data16 = (data[0]) + (0xff << 8); // we let the other byte as is
        // write it
        aFMC->CTL |= LN_FMC_CTL_PG;
        __asm__("nop");
        __asm__("nop");
        *(uint16_t *)next = (uint32_t)data16;
        waitNotBusy();
        if (!checkWriting())
            return false;
        //
    }
    return true;
}
// EOF
