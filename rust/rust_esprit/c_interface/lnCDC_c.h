#pragma once
#include "stdint.h"
#include "stdbool.h"

typedef struct
{
    void *dummy;
} lncdc_c;

typedef void cdc_event_handler(void *cookie, int interface, int event, uint32_t payload);

lncdc_c *lncdc_create(uint32_t instance);
void lncdc_delete(lncdc_c *);
int lncdc_read(lncdc_c *, uint8_t *buffer, int maxSize);
int lncdc_write(lncdc_c *, const uint8_t *buffer, int maxSize);
int lncdc_write_no_block(lncdc_c *, const uint8_t *buffer, int maxSize);
int lncdc_write_available(lncdc_c *);
int lncdc_read_available(lncdc_c *);
bool lncdc_is_connected(lncdc_c *);
bool lncdc_peek(lncdc_c *, uint8_t *ch);
uint8_t lncdc_get_line_state(lncdc_c *);
void lncdc_flush(lncdc_c *);
void lncdc_clear_input_buffers(lncdc_c *);
void lncdc_set_event_handler(lncdc_c *, cdc_event_handler *h, void *cookie);

// Direct port-level operations (ITF index 0..3)
bool lncdc_port_is_connected(uint32_t itf);
int lncdc_port_read_available(uint32_t itf);
int lncdc_port_write_available(uint32_t itf);
int lncdc_port_read(uint32_t itf, uint8_t *buffer, int maxSize);
int lncdc_port_write(uint32_t itf, const uint8_t *buffer, int maxSize);
int lncdc_port_write_no_block(uint32_t itf, const uint8_t *buffer, int maxSize);
void lncdc_port_flush(uint32_t itf);
void lncdc_port_clear_input(uint32_t itf);
uint8_t lncdc_port_get_line_state(uint32_t itf);
bool lncdc_port_peek(uint32_t itf, uint8_t *ch);

// EOF