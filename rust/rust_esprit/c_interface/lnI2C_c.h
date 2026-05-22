#pragma once
#include "stdbool.h"
#include "stdint.h"

typedef struct // opaque
{
    void *dummy;
} ln_i2c_c;

ln_i2c_c *lni2c_create(uint32_t instance, uint32_t speed);
void lni2c_delete(ln_i2c_c *i2c);
void lni2c_setSpeed(ln_i2c_c *ptr, uint32_t speed);
void lni2c_setAddress(ln_i2c_c *ptr, uint32_t address);
bool lni2c_write(ln_i2c_c *ptr, uint32_t n, const uint8_t *data);
bool lni2c_read(ln_i2c_c *ptr, uint32_t n, uint8_t *data);
bool lni2c_write_to(ln_i2c_c *ptr, uint32_t target, uint32_t n, const uint8_t *data);
bool lni2c_multi_write_to(ln_i2c_c *ptr, uint32_t target, uint32_t nbSeqn, const uint32_t *seqLength,
                          const uint8_t **data);
bool lni2c_read_from(ln_i2c_c *ptr, uint32_t target, uint32_t n, uint8_t *data);
bool lni2c_begin(ln_i2c_c *ptr, uint32_t target);

// EOF
