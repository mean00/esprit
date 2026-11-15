/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "nvmCore.h"

/**
 *
 * @param nbSectors
 */
class lnNvmGd32 : public lnNvm
{
  public:
    lnNvmGd32();
    virtual ~lnNvmGd32();

  protected:
    virtual bool eraseSector(uint32_t sector);
    virtual bool writeSector(uint32_t sector, uint32_t offset, uint32_t size, uint8_t *data);
    virtual bool readSector(uint32_t sector, uint32_t offset, uint32_t size, uint8_t *data);
    virtual bool verifyErase(uint32_t sector);

  protected:
    uint32_t _baseAddress;
};

// EOF