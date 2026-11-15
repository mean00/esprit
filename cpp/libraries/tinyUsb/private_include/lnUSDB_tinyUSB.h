#pragma once
#warning HARDCODED CH32V3x
//#define CFG_TUSB_MCU  OPT_MCU_STM32F1
#define MAX_EP_COUNT LN_USBD_MAX_ENDPOINT

#include "tusb_option.h"
#include "device/dcd.h"
#include "lnUSBD.h"
#include "lnUSBD_priv.h"

/* Endpoint budget (usb.md P4). A CDC-ACM interface takes two endpoint numbers - one
 * IN-only notification endpoint, one shared by the data IN/OUT pair - so N interfaces
 * plus EP0 need `1 + 2 * N`. Asking for more CDC interfaces than the controller has
 * endpoints must fail here, at build time: it cannot be discovered on a probe, it only
 * shows up as an interface that never enumerates.
 * (After the includes: CFG_TUD_CDC comes from tusb_option.h, LN_USBD_MAX_ENDPOINT from
 * lnUSBD_priv.h.) */
#define LN_USB_EP_PER_CDC 2
static_assert(1 + CFG_TUD_CDC * LN_USB_EP_PER_CDC <= MAX_EP_COUNT,
              "too many CDC interfaces for this MCU's endpoint count");
