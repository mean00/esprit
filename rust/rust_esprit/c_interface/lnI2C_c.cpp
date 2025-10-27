// this is not thread safe, but it should not matter much
// the returned object ARE thread sage, it's just the creation/deletion that is not
extern "C"
{
#include "lnI2C_c.h"
}

#include "esprit.h"
#include "lnI2C.h"

// manual RC
class rcI2C
{
  public:
    rcI2C()
    {
        i2c = NULL;
        count = 0;
    }
    lnI2C *i2c;
    int count;
};
static rcI2C *_i2cs[3] = {NULL, NULL, NULL};

#define WRAPME(x) ((lnI2C *)x)
/**
 *
 *
 */
ln_i2c_c *lni2c_create(int instance, int speed)
{
    if (!_i2cs[instance]) // create on the fly if needed
    {
        _i2cs[instance] = new rcI2C();
    }

    rcI2C &inst = *(_i2cs[instance]);

    if (inst.count)
    {
        inst.count++;
        return (ln_i2c_c *)inst.i2c;
    }
    inst.i2c = new lnI2C(instance, speed);
    inst.count = 1;
    inst.i2c->begin();
    return (ln_i2c_c *)inst.i2c;
}
/**
 *
 *
 */
void lni2c_delete(ln_i2c_c *i2c)
{
#warning TODO!
}
/**
 *
 */
void lni2c_setSpeed(ln_i2c_c *ptr, int speed)
{
    WRAPME(ptr)->setSpeed(speed);
}
/**
 *
 */
void lni2c_setAddress(ln_i2c_c *ptr, int address)
{
    WRAPME(ptr)->setAddress(address);
}
/**
 *
 */
bool lni2c_write(ln_i2c_c *ptr, uint32_t n, const uint8_t *data)
{
    return WRAPME(ptr)->write(n, data);
}
/**
 *
 */
bool lni2c_read(ln_i2c_c *ptr, uint32_t n, uint8_t *data)
{
    return WRAPME(ptr)->read(n, data);
}
/**
 *
 */
bool lni2c_write_to(ln_i2c_c *ptr, int target, uint32_t n, const uint8_t *data)
{
    return WRAPME(ptr)->write(target, n, data);
}
/**
 *
 */
bool lni2c_multi_write_to(ln_i2c_c *ptr, int target, uint32_t nbSeqn, const uint32_t *seqLength, const uint8_t **data)
{
    return WRAPME(ptr)->multiWrite(target, nbSeqn, seqLength, data);
}
/**
 *
 */
bool lni2c_read_from(ln_i2c_c *ptr, int target, uint32_t n, uint8_t *data)
{
    return WRAPME(ptr)->read(target, n, data);
}
/**
 *
 */
bool lni2c_begin(ln_i2c_c *ptr, int target)
{
    return WRAPME(ptr)->begin(target);
}
// EOF
