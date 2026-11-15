
#include "esprit.h"
#include "lnPinMapping.h"
#include "lnUSBD.h"
#include "lnUSBD_priv.h"

// extern "C" {
void dcd_handle_bus_reset()
{
}
void dcd_ep_ctr_rx_handler(int ep)
{
}
void dcd_ep_ctr_tx_handler(int ep)
{
}

//}
/**
 * RP2040/RP2350: nothing to bring up here. The device controller is tinyUSB's own
 * `portable/raspberrypi/rp2040/dcd_rp2040.c`, which raises the `dcd_event_*` itself, so
 * unlike the GD32 there is no register-level `lnUsbDevice` to create and no event
 * bridge to install (`usb.md`, P7). The entry exists so the board code can call it
 * unconditionally.
 */
extern "C" void lnUsbDcdInit(void)
{
}
