/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once

/**
 *
 * @param p
 */
class lnFastIO
{
  public:
    lnFastIO(lnPin p);
    void on() volatile
    {
        *_onoff = _onbit;
    }
    void off() volatile
    {
        *_onoff = _offbit;
    }
    void pulseLow() volatile __attribute__((always_inline))
    {
        *_onoff = _offbit;
        *_onoff = _onbit;
    }
    void pulseHigh() volatile __attribute__((always_inline))
    {
        *_onoff = _onbit;
        *_onoff = _offbit;
    }
    /**
     * The pin's level, read straight from the port's input status register: one
     * load and a mask, the same cost on()/off() have as stores. It reads the
     * same register field lnDigitalRead() does, without that function's port
     * lookup and call, which is what a timing critical loop needs (the SDI read
     * cell has ~24 timer ticks of slack after its sample, and the call was most
     * of them).
     */
    bool read() volatile
    {
        return (*_in & _inbit) != 0;
    }
    /**
     * What on() writes, and where: the BOP register and the two set/clear
     * words. For a caller that cannot make the store itself - the SDI read
     * frame fires the level shifter's DIR turn-around from a timer compare
     * through the DMA, which has to write exactly these words to this register
     * at the instant the channel releases the wire.
     */
    volatile uint32_t *fastRegister() volatile
    {
        return _onoff;
    }
    uint32_t fastSetWord() volatile
    {
        return _onbit;
    }
    uint32_t fastClearWord() volatile
    {
        return _offbit;
    }

  protected:
    volatile uint32_t *_onoff;
    volatile uint32_t _onbit, _offbit;
    volatile uint32_t *_in; // the port's input status register
    volatile uint32_t _inbit;
};

// EOF
