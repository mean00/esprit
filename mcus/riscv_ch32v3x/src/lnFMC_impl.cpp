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

#define LN_FMC_STAT_CH32_WR_BUSY (1 << 1)
#define LN_FMC_STAT_CH32_EHMODS (1 << 7)
#define LN_FMC_CTL_CH32_FASTUNLOCK (1 << 15)
#define LN_FMC_CTL_CH32_FASTPROGRAM (1 << 16)
#define LN_FMC_CTL_CH32_FASTERASE (1 << 17)
#define LN_FMC_CTL_CH32_FASTSTART (1 << 21)
#define LN_FMC_CTL_CH32_RSENACT (1 << 22)
#define LN_FMC_CTL_CH32_EHMOD (1 << 24)

bool CH32_fastUnlock();
bool CH32_fastLock();

void CH32V3_flashEnhanceMode(bool enable)
{
    if (enable)
    {
        CH32_fastUnlock();
        aFMC->CTL |= LN_FMC_CTL_CH32_EHMOD;
        volatile uint32_t stat = aFMC->STAT;
        (void)stat;
    }
    else
    {
        CH32_fastUnlock();
        aFMC->CTL &= ~LN_FMC_CTL_CH32_EHMOD;
        aFMC->CTL |= LN_FMC_CTL_CH32_RSENACT;
    }
}

void lnInitFlash_ch32v3x()
{
    CH32V3_flashEnhanceMode(true);
}

/**
 *
 */
class autoNoInterrupt
{
  public:
    autoNoInterrupt()
    {
        lnNoInterrupt();
        CH32V3_flashEnhanceMode(false);
    }
    ~autoNoInterrupt()
    {
        lnInterrupts();
        CH32V3_flashEnhanceMode(true);
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

static void waitNotWrBusy()
{
    // is it webusy  ?
    while (1)
    {
        if (!(aFMC->STAT & LN_FMC_STAT_CH32_WR_BUSY))
            return;
    }
}

/**
  \brief Erase using 256 bytes chunk, CH32 specific operation
  \brief /!\ When successfully erased, the value in flash is 0xe339e339
*/
bool lnFMC::erase(const uint32_t startAddress, int sizeInKBytes)
{
    uint32_t adr = startAddress;
    // Check we target an area within the flash (up to 256 kB)

    xAssert(adr < (256 * 1024));
    xAssert(adr > 8 * 1024);
    // check the start address is 1 kB aligned
    xAssert(!(adr & ((1 << 10) - 1)));
    adr += 0x08000000; // some leftover from older chip it seems
    autoNoInterrupt noInt;
    CH32_fastUnlock();
    // Erase each page
    for (int i = 0; i < sizeInKBytes * 4; i++)
    {
        aFMC->CTL |= LN_FMC_CTL_CH32_FASTERASE;
        aFMC->ADDR = adr;
        aFMC->CTL |= LN_FMC_CTL_START;
        waitNotBusy();
        adr += 256;
        aFMC->STAT |= LN_FMC_STAT_WP_ENDF; // clear end of process bit
        aFMC->CTL &= ~LN_FMC_CTL_CH32_FASTERASE;
        aFMC->CTL &= ~LN_FMC_CTL_START; // clear start
    }
    return true;
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

/*
    Write 256 bytes in one go, the datasheet says 128, it s incorrect
    The
*/
bool lnFMC::write(const uint32_t startAddress, const uint8_t *data, int sizeInBytes)
{
    uint32_t adr = startAddress;
    // Check alignment
    // xAssert(adr>8*1024);
    // xAssert(adr<256*1024);

    // if not aligned at the beginning
    xAssert((adr & 3) == 0);

    adr += 0x08000000; // some leftover from older chip it seems

    autoNoInterrupt noInt;
    CH32_fastUnlock();

    while (sizeInBytes > 0)
    {
        aFMC->CTL |= LN_FMC_CTL_CH32_FASTPROGRAM;
        waitNotBusy();
        waitNotWrBusy();
        // prefill write cache, we write 256 bytes at a time
        for (int i = 0; i < 64; i++)
        {
            uint32_t data32 = (data[0]) + (data[1] << 8) + (data[2] << 16) + (data[3] << 24);
            *(uint32_t *)adr = data32; // write anywhere
            data += 4;
            adr += 4;
            waitNotWrBusy();
        }
        sizeInBytes -= 256;
        // ok cache filled,go!
        aFMC->CTL |= LN_FMC_CTL_CH32_FASTSTART; // and go
        waitNotBusy();
        if (!checkWriting())
        {
            return false;
        }
        aFMC->CTL &= ~LN_FMC_CTL_CH32_FASTPROGRAM;
    }

    //
    return true;
}

// EOF
