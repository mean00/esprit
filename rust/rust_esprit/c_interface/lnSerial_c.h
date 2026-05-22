#pragma once
#include "stdbool.h"
#include "stdint.h"

#ifdef __cplusplus
extern "C"
{
#endif

    // ---------------------------------------------------------------------------
    //  Opaque handle types
    // ---------------------------------------------------------------------------

    typedef struct // opaque — transmit‑only serial
    {
        void *dummy;
    } ln_serial_tx_c;

    typedef struct // opaque — bidirectional serial (Rx + Tx)
    {
        void *dummy;
    } ln_serial_rx_c;

    // ---------------------------------------------------------------------------
    //  Callback type (matches lnSerialCallback in lnSerial.h)
    // ---------------------------------------------------------------------------
    //  event: 0 = dataAvailable, 1 = txDone
    typedef void ln_serial_event_cb(void *cookie, int event);

    // ---------------------------------------------------------------------------
    //  Tx‑only API
    // ---------------------------------------------------------------------------

    ln_serial_tx_c *lnserial_tx_create(uint32_t instance, bool dma, bool buffered);
    void lnserial_tx_delete(ln_serial_tx_c *s);
    bool lnserial_tx_init(ln_serial_tx_c *s);
    bool lnserial_tx_set_speed(ln_serial_tx_c *s, uint32_t speed);
    bool lnserial_tx_transmit(ln_serial_tx_c *s, uint32_t size, const uint8_t *buffer);
    bool lnserial_tx_raw_write(ln_serial_tx_c *s, uint32_t size, const uint8_t *buffer);

    // ---------------------------------------------------------------------------
    //  Rx+Tx API
    // ---------------------------------------------------------------------------

    ln_serial_rx_c *lnserial_rx_create(uint32_t instance, uint32_t rxBufferSize, bool dma);
    void lnserial_rx_delete(ln_serial_rx_c *s);
    bool lnserial_rx_init(ln_serial_rx_c *s);
    bool lnserial_rx_set_speed(ln_serial_rx_c *s, uint32_t speed);
    bool lnserial_rx_transmit(ln_serial_rx_c *s, uint32_t size, const uint8_t *buffer);
    int lnserial_rx_transmit_no_block(ln_serial_rx_c *s, uint32_t size, const uint8_t *buffer);
    bool lnserial_rx_enable_rx(ln_serial_rx_c *s, bool enabled);
    void lnserial_rx_purge_rx(ln_serial_rx_c *s);
    int lnserial_rx_read(ln_serial_rx_c *s, uint32_t max, uint8_t *to);
    void lnserial_rx_set_callback(ln_serial_rx_c *s, ln_serial_event_cb *cb, void *cookie);
    int lnserial_rx_get_read_pointer(ln_serial_rx_c *s, uint8_t **to);
    void lnserial_rx_consume(ln_serial_rx_c *s, uint32_t n);

#ifdef __cplusplus
}
#endif

// EOF
