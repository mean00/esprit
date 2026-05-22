// this is not thread safe, but it should not matter much
// the returned object ARE thread sage, it's just the creation/deletion that is not
#include "lnSPI_c.h"
#include "esprit.h"
#include "lnSPI.h"
#include "stdbool.h"
#include "stdint.h"

// manual RC
class rcSPI
{
  public:
    rcSPI()
    {
        spi = NULL;
        count = 0;
    }
    lnSPI *spi;
    int count;
};
static rcSPI *_spis[3] = {NULL, NULL, NULL};

#define WRAPME(x) ((lnSPI *)x)
/**
 *
 *
 */
ln_spi_c *lnspi_create(uint32_t instance, int pin)
{
    if (!_spis[instance]) // create on the fly if needed
    {
        _spis[instance] = new rcSPI();
    }

    rcSPI &inst = *(_spis[instance]);

    if (inst.count)
    {
        inst.count++;
        return (ln_spi_c *)inst.spi;
    }
    inst.spi = lnSPI::create(instance, pin);
    inst.count = 1;
    return (ln_spi_c *)inst.spi;
}
/**
 *
 *
 */
void lnspi_delete(ln_spi_c *i2c)
{
//#warning TODO!
}
/**
 *
 */
void lnspi_set_speed(ln_spi_c *ptr, uint32_t speed)
{
    WRAPME(ptr)->setSpeed(speed);
}
/**
 *
 */
void lnspi_set_ssel(ln_spi_c *ptr, int ssel)
{
    WRAPME(ptr)->setSSEL(ssel);
}
/**
 *
 */
void lnspi_set_data_mode(ln_spi_c *ptr, spiDataMode mode)
{
    WRAPME(ptr)->setDataMode(mode);
}
/**
 *
 */
void lnspi_begin(ln_spi_c *ptr, uint32_t dataSize)
{
    WRAPME(ptr)->begin(dataSize);
}
/**
 *
 */
void lnspi_end(ln_spi_c *ptr)
{
    WRAPME(ptr)->end();
}
/**
 *
 */
void lnspi_set(ln_spi_c *ptr, const lnSPISettings &st)
{
    WRAPME(ptr)->set(st);
}

/**
 *
 */
bool lnspi_write8(ln_spi_c *ptr, uint8_t data)
{
    return WRAPME(ptr)->write8(data);
}
/**
 *
 */
bool lnspi_write16(ln_spi_c *ptr, uint16_t data)
{
    return WRAPME(ptr)->write16(data);
}
/**
 *
 */
bool lnspi_wait_for_completion(ln_spi_c *ptr)
{
    WRAPME(ptr)->waitForCompletion();
    return true;
}

/**
 *
 */
bool lnspi_block_write16_repeat(ln_spi_c *ptr, unsigned int nbWord, const uint16_t data)
{
    return WRAPME(ptr)->blockWrite16Repeat(nbWord, data);
}
/**
 *
 */
bool lnspi_block_write8_repeat(ln_spi_c *ptr, unsigned int nbByte, const uint8_t data)
{
    return WRAPME(ptr)->blockWrite8Repeat(nbByte, data);
}
/**
 *
 */
bool lnspi_block_write16(ln_spi_c *ptr, unsigned int nbWord, const uint16_t *data)
{
    return WRAPME(ptr)->blockWrite16(nbWord, data);
}
/**
 *
 */
bool lnspi_block_write8(ln_spi_c *ptr, unsigned int nbByte, const uint8_t *data)
{
    return WRAPME(ptr)->blockWrite8(nbByte, data);
}
// EOF
