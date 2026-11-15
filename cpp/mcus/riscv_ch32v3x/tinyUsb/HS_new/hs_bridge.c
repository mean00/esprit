// MEANX
/*
 */
#include "dcd_usbhs_platform.h"
/* USBOTG_FS Registers */
#include "ch32v30x_hs.h"
#define USBHS_BASE 0x40023400UL
USBHSD_TypeDef *USBHSD = ((USBHSD_TypeDef *)USBHS_BASE);
