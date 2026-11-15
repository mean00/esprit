/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "lnPeripherals.h"

/**
 *
 * @param startAddress
 * @param sizeInKBytes
 */
class lnFMC
{
  public:
    static bool erase(const uint32_t startAddress, int sizeInKBytes);
    static bool write(const uint32_t startAddress, const uint8_t *data, int sizeInBytes);

  protected:
    static bool eraseStm(const uint32_t startAddress, int sizeInKBytes);
    static bool eraseCh32(const uint32_t startAddress, int sizeInKBytes);
};
