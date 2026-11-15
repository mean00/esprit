#include "esprit.h"
#include "lnUsbStack.h"
#include "tusb.h"
#include "device/dcd.h"
#include "lnUsbDFUrt.h"


static lnDFUCb *_dfuCb=NULL;
/**
 * 
 */
void lnUsbDFURT::addDFURTCb(lnDFUCb *cb)
{
        _dfuCb=cb;
}
/**
 * 
 */
extern "C" void tud_dfu_runtime_reboot_to_dfu_cb()
{

    if(!_dfuCb)
    {
        Logger("No DFU RT callback\n");
        return;
    }
    _dfuCb();
}
// EOF
