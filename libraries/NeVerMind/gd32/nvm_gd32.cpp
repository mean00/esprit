/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "esprit.h"
#include "lnFMC.h"

#include "nvm_gd32.h"

extern "C"
{
    extern const uint32_t eeprom_begin, eeprom_size;
}

#define EEPROM_BEGIN ((uint32_t) & eeprom_begin)
#define EEPROM_SIZE ((uint32_t) & eeprom_size)

/**
 *
 * @param nbSectors
 */
lnNvmGd32::lnNvmGd32() : lnNvm(EEPROM_SIZE / LN_NVM_SECTOR_SIZE)
{
    _baseAddress = EEPROM_BEGIN;
}
/**
 *
 */
lnNvmGd32::~lnNvmGd32()
{
}
/**
 *
 * @param sector
 * @return
 */
bool lnNvmGd32::eraseSector(int sector)
{
    xAssert(_baseAddress == EEPROM_BEGIN);
    xAssert(sector < _nbSectors);
    return lnFMC::erase(_baseAddress + sector * 1 * LN_NVM_SECTOR_SIZE, LN_NVM_SECTOR_SIZE / 1024);
}
/**
 *
 * @param sector
 * @param offset
 * @param size
 * @param data
 * @return
 */
bool lnNvmGd32::writeSector(int sector, int offset, int size, uint8_t *data)
{
    xAssert(_baseAddress == EEPROM_BEGIN);
    xAssert(sector < _nbSectors);
    return lnFMC::write(_baseAddress + sector * 1 * LN_NVM_SECTOR_SIZE + offset, data, size);
}
/**
 *
 * @param sector
 * @param offset
 * @param size
 * @param data
 * @return
 */
bool lnNvmGd32::readSector(int sector, int offset, int size, uint8_t *data)
{
    xAssert(_baseAddress == EEPROM_BEGIN);
    xAssert(sector < _nbSectors);
    xAssert(size >= 0);
    xAssert((offset + size) <= LN_NVM_SECTOR_SIZE);
    uint8_t *adr = (uint8_t *)(_baseAddress + sector * 1 * LN_NVM_SECTOR_SIZE + offset);
    memcpy(data, adr, size);
    return true;
}
/**
 *
 * @param sector
 * @return
 */
bool lnNvmGd32::verifyErase(int sector)
{
    xAssert(_baseAddress == EEPROM_BEGIN);
    xAssert(sector < _nbSectors);
    uint8_t *adr = (uint8_t *)(_baseAddress + sector * 1 * LN_NVM_SECTOR_SIZE);
    for (int i = 0; i < LN_NVM_SECTOR_SIZE; i++)
        if (adr[i] != 0xff)
            return false;
    return true;
}

// EOF
