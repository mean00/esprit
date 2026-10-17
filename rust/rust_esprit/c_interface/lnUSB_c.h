#pragma once
#include "stdint.h"
#include "stdbool.h"

typedef void lnUsbStackEventHandler(void *cookie, int event);

typedef struct
{
    void *dummy;
} lnusb_c;

lnusb_c *lnusb_create(uint32_t instance);
void lnusb_delete(lnusb_c *);

void lnusb_init(lnusb_c *inst, int nbDescriptorLine, const char **deviceDescriptor);
void lnusb_setConfiguration(lnusb_c *inst);

void lnusb_setEventHandler(lnusb_c *inst, const void *cookie, lnUsbStackEventHandler *const ev);
void lnusb_start(lnusb_c *inst);
void lnusb_stop(lnusb_c *inst);

bool lnusb_is_mounted(void);
bool lnusb_is_suspended(void);
bool lnusb_is_ready(void);

// EOF