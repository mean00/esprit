// C‑interface bridge for lnSerial (UART)
//
// Wraps the C++ lnSerialTxOnly / lnSerialRxTx classes in extern "C" functions
// so that Rust (via bindgen) can call them.

#include "lnSerial_c.h"
#include "esprit.h"
#include "lnSerial.h"

// ---------------------------------------------------------------------------
//  Tx‑only
// ---------------------------------------------------------------------------

#define WRAP_TX(x)  (reinterpret_cast<lnSerialTxOnly *>(x))
#define WRAP_RX(x)  (reinterpret_cast<lnSerialRxTx *>(x))

ln_serial_tx_c *lnserial_tx_create(int instance, bool dma, bool buffered)
{
    auto *p = createLnSerialTxOnly(instance, dma, buffered);
    return reinterpret_cast<ln_serial_tx_c *>(p);
}

void lnserial_tx_delete(ln_serial_tx_c *s)
{
    delete WRAP_TX(s);
}

bool lnserial_tx_init(ln_serial_tx_c *s)
{
    return WRAP_TX(s)->init();
}

bool lnserial_tx_set_speed(ln_serial_tx_c *s, int speed)
{
    return WRAP_TX(s)->setSpeed(speed);
}

bool lnserial_tx_transmit(ln_serial_tx_c *s, int size, const uint8_t *buffer)
{
    return WRAP_TX(s)->transmit(size, buffer);
}

bool lnserial_tx_raw_write(ln_serial_tx_c *s, int size, const uint8_t *buffer)
{
    return WRAP_TX(s)->rawWrite(size, buffer);
}

// ---------------------------------------------------------------------------
//  Rx+Tx
// ---------------------------------------------------------------------------

ln_serial_rx_c *lnserial_rx_create(int instance, int rxBufferSize, bool dma)
{
    auto *p = createLnSerialRxTx(instance, rxBufferSize, dma);
    return reinterpret_cast<ln_serial_rx_c *>(p);
}

void lnserial_rx_delete(ln_serial_rx_c *s)
{
    delete WRAP_RX(s);
}

bool lnserial_rx_init(ln_serial_rx_c *s)
{
    return WRAP_RX(s)->init();
}

bool lnserial_rx_set_speed(ln_serial_rx_c *s, int speed)
{
    return WRAP_RX(s)->setSpeed(speed);
}

bool lnserial_rx_transmit(ln_serial_rx_c *s, int size, const uint8_t *buffer)
{
    return WRAP_RX(s)->transmit(size, buffer);
}

int lnserial_rx_transmit_no_block(ln_serial_rx_c *s, int size, const uint8_t *buffer)
{
    return WRAP_RX(s)->transmitNoBlock(size, buffer);
}

bool lnserial_rx_enable_rx(ln_serial_rx_c *s, bool enabled)
{
    return WRAP_RX(s)->enableRx(enabled);
}

void lnserial_rx_purge_rx(ln_serial_rx_c *s)
{
    WRAP_RX(s)->purgeRx();
}

int lnserial_rx_read(ln_serial_rx_c *s, int max, uint8_t *to)
{
    return WRAP_RX(s)->read(max, to);
}

void lnserial_rx_set_callback(ln_serial_rx_c *s, ln_serial_event_cb *cb, void *cookie)
{
    // Both lnSerialCallback and ln_serial_event_cb have the same ABI:
    //   void callback(void *cookie, int event)
    // where event maps 1:1 to lnSerialCore::Event (dataAvailable=0, txDone=1).
    auto c_cb = reinterpret_cast<void (*)(void *, int)>(cb);
    WRAP_RX(s)->setCallback(reinterpret_cast<lnSerialCallback *>(c_cb), cookie);
}

int lnserial_rx_get_read_pointer(ln_serial_rx_c *s, uint8_t **to)
{
    return WRAP_RX(s)->getReadPointer(to);
}

void lnserial_rx_consume(ln_serial_rx_c *s, int n)
{
    WRAP_RX(s)->consume(n);
}

// EOF
