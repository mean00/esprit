
#include "esprit.h"
void dcd_handle_bus_reset()
{
    xAssert(0);
}
void dcd_ep_ctr_tx_handler(int x)
{
    xAssert(0);
}
void dcd_ep_ctr_rx_handler(int x)
{
    xAssert(0);
}

/**
 * The RISC-V side has no register-level `lnUsbDevice` in the stack path: its DCD is
 * `dcd_ch32_usbfs.c`, which raises the `dcd_event_*` itself (`usb.md`, P2). The entry
 * exists so the board code can call it unconditionally.
 */
extern "C" void lnUsbDcdInit(void)
{
}
// EOF
