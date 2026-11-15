
#include "esprit.h"
//
#include "device/dcd.h"
#include "lnUsbDFU.h"
#include "lnUsbStack.h"
#include "tusb.h"

static lnDFUCb *_dfuCb = NULL;
extern void lnSoftSystemReset();

void lnUsbDFU::addDFURTCb(lnDFUCb *cb)
{
    xAssert(!_dfuCb); // do only once
    _dfuCb = cb;
};

extern "C" void tud_dfu_download_cb(uint8_t alt, uint16_t block_num, uint8_t const *data, uint16_t length)
{
    if (!_dfuCb)
    {
        xAssert(0);
    }
    _dfuCb(block_num, data, length);
}
/*
 */
extern "C" void tud_dfu_detach_cb(void)
{
    lnSoftSystemReset();
}
/**
 */
extern "C" void tud_dfu_manifest_cb(uint8_t alt)
{
    tud_dfu_detach_cb();
}
/**
 * @brief
 *
 */
extern "C" uint32_t tud_dfu_get_timeout_cb(uint8_t alt, uint8_t state)
{
    return 10; // ??
}
// EOF
