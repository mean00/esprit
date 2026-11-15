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
    static lnI2C *create(uint32_t instance, uint32_t speed = 0);
    virtual ~lnI2C() {};
    virtual void setSpeed(uint32_t speed) = 0;
    virtual void setAddress(uint32_t address) = 0;
    virtual bool write(uint32_t n, const uint8_t *data) = 0;
    virtual bool read(uint32_t n, uint8_t *data) = 0;
    virtual bool write(uint32_t target, uint32_t n, const uint8_t *data) = 0;
    virtual bool multiWrite(uint32_t target, uint32_t nbSeqn, const uint32_t *seqLength, const uint8_t **data) = 0;
    virtual bool read(uint32_t target, uint32_t n, uint8_t *data) = 0;
    virtual bool begin(uint32_t target = 0) = 0;
};
