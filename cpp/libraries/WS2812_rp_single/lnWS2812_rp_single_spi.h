#include "esprit.h"
#include "lnGPIO.h"
#include "lnSerial.h"
#include "ln_rp_spi.h"

/**
 * @brief
 *
 */
class WS2812_rp2040_single
{
  public:
    WS2812_rp2040_single(int spi_instance, lnPin pin)
    {
        _spi = new rpSPI(spi_instance);
        lnPinMode(pin, lnSPI_MODE);
        _spi->setSpeed(3470 * 1000); // 3.47 Meg; to=288 ns
        _spi->begin(16);
    }
    virtual ~WS2812_rp2040_single()
    {
        delete _spi;
        _spi = NULL;
    }
    void setColor(uint8_t r, uint8_t g, uint8_t b);

  protected:
    rpSPI *_spi;
    uint16_t _buffer[6]; // 24*4=96 bits = 6*16 bits = 12 bytes
};

// EOF
