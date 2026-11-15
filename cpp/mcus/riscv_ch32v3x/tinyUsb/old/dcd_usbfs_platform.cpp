
#include "esprit.h"
/**
 */
extern "C"
{

    void dcd_int_enable(uint8_t rhport)
    {
        (void)rhport;
        lnEnableInterrupt(LN_IRQ_OTG_FS);
    }
    /**
     */
    void dcd_int_disable(uint8_t rhport)
    {
        (void)rhport;
        lnDisableInterrupt(LN_IRQ_OTG_FS);
    }

    /**
     */
    void dcd_int_handler(uint8_t port);

    void OTG_FS_IRQHandler() LN_INTERRUPT_TYPE;
    void OTG_FS_IRQHandler()
    {
        dcd_int_handler(0);
    }
    void dcd_fs_platform_init()
    {
        lnPeripherals::enable(pUSBFS_OTG_CH32v3x);
    }
    void dcd_fs_platform_delay(int ms)
    {
        xDelay(ms);
    }
}
// EOF