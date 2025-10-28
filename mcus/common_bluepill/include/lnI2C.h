/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "esprit.h"

/**
 *
 * @param instance
 * @param speed
 */
class lnI2C
{
  public:
    static lnI2C *create(int instance, int speed = 0);
    virtual ~lnI2C() {};
    virtual void setSpeed(int speed) = 0;
    virtual void setAddress(int address) = 0;
    virtual bool write(uint32_t n, const uint8_t *data) = 0;
    virtual bool read(uint32_t n, uint8_t *data) = 0;
    virtual bool write(int target, uint32_t n, const uint8_t *data) = 0;
    virtual bool multiWrite(int target, uint32_t nbSeqn, const uint32_t *seqLength, const uint8_t **data) = 0;
    virtual bool read(int target, uint32_t n, uint8_t *data) = 0;
    virtual bool begin(int target = 0) = 0;
};
