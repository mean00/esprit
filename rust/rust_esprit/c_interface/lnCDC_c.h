#pragma once
#include "stdint.h"

typedef struct
{
    void *dummy;
} lncdc_c;

typedef void cdc_event_handler(void *cookie, int interface, int event, uint32_t payload);

lncdc_c *lncdc_create(uint32_t instance);
void lncdc_delete(lncdc_c *);
void lncdc_delete(lncdc_c *);
void lncdc_delete(lncdc_c *);
int lncdc_read(lncdc_c *, uint8_t *buffer, int maxSize);
int lncdc_write(lncdc_c *, const uint8_t *buffer, int maxSize);
int lncdc_write_noblock(lncdc_c *, const uint8_t *buffer, int maxSize);
void lncdc_flush(lncdc_c *);
void lncdc_clear_input_buffers(lncdc_c *);
void lncdc_set_event_handler(lncdc_c *, cdc_event_handler *h, void *cookie);

// EOF