#pragma once
#include "lnSPI_defines.h"
#include "stdbool.h"
#include "stdint.h"

typedef struct // opaque
{
    void *dummy;
} ln_spi_c;

ln_spi_c *lnspi_create(uint32_t instance, int pinCs);
void lnspi_delete(ln_spi_c *i2c);
//--
void lnspi_begin(ln_spi_c *instance, uint32_t dataSize);
void lnspi_end(ln_spi_c *instance);
void lnspi_set_bit_order(ln_spi_c *instance, spiBitOrder order);
void lnspi_set_data_mode(ln_spi_c *instance, spiDataMode datamode);
void lnspi_set_speed(ln_spi_c *instance, uint32_t speed);
void lnspi_set_ssel(ln_spi_c *instance, int ssel);
void lnspi_set(ln_spi_c *instance, const lnSPISettings &st);

bool lnspi_write8(ln_spi_c *instance, uint8_t data);
bool lnspi_write16(ln_spi_c *instance, uint16_t data);
bool lnspi_wait_for_completion(ln_spi_c *instance);

bool lnspi_block_write16_repeat(ln_spi_c *instance, uint32_t nbWord, const uint16_t data);
bool lnspi_block_write8_repeat(ln_spi_c *instance, uint32_t nbByte, const uint8_t data);
bool lnspi_block_write16(ln_spi_c *instance, uint32_t nbWord, const uint16_t *data);
bool lnspi_block_write8(ln_spi_c *instance, uint32_t nbByte, const uint8_t *data);

bool lnspi_asyncWrite8(ln_spi_c *instance, uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie,
                       bool repeat);
bool lnspi_nextWrite8(ln_spi_c *instance, uint32_t nbBytes, const uint8_t *data, lnSpiCallback *cb, void *cookie,
                      bool repeat);
bool lnspi_asyncWrite16(ln_spi_c *instance, uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie,
                        bool repeat);
bool lnspi_nextWrite16(ln_spi_c *instance, uint32_t nbWords, const uint16_t *data, lnSpiCallback *cb, void *cookie,
                       bool repeat);
bool lnspi_finishAsyncDma(ln_spi_c *instance);
bool lnspi_waitForAsync(ln_spi_c *instance);
bool lnspi_transfer8(ln_spi_c *instance, uint32_t nb, const uint8_t *tx, uint8_t *rx);

// EOF
