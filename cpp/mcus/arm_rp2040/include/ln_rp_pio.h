

#pragma once
#include "esprit.h"
/*
 * The usual sequence is
 * rpPio(0)
 * setSpeed()
 * configure()
 * uploadCode()
 * ...
 * execute()
 * ...
 * stop()
 * ...
 * execute()
 *
 */
/**
 *
 *
 */
class rpPIO_pinSet
{
  public:
    rpPIO_pinSet()
    {
        startPin = 0;
        pinNb = 0;
    }
    uint32_t startPin;
    uint32_t pinNb;
};

/**
 *
 */
class rpPIO_pinConfig
{
  public:
    rpPIO_pinSet inputs;
    rpPIO_pinSet outputs;
    rpPIO_pinSet sets;
};
/**
 *
 *
 *
 */
class rpPIO_SM
{
    friend class rpPIO;

  public:
    bool setBitOrder(bool transmitShiftToRight, bool ReceiveShiftToRight);
    bool setSpeed(uint32_t fq);
    bool configure(const rpPIO_pinConfig &config);
    bool configureSideSet(int startPin, int nbPin, int sideNbBits, bool optional);
    bool waitTxEmpty();
    bool waitTxReady();
    bool waitRxReady();
    bool setPinDir(lnPin pin, bool isOutput);
    bool setPinsValue(uint32_t value);
    bool execute();
    bool stop();
    bool write(uint32_t nb, const uint32_t *data);
    bool read(uint32_t nb, uint32_t *data);
    bool uploadCode(uint32_t codeSizeInHalfWord, const uint16_t *code, uint32_t wrapBegin, uint32_t wrapEnd);
    bool execute_jmp(uint32_t addr);
    void reset();

  protected:
    rpPIO_SM(uint32_t unit, uint32_t sm);
    uint32_t _sm;
    uint32_t _unit;
    uint32_t _codeOffset;
    uint32_t _codeSize;
    uint32_t _wrapStart;
    uint32_t _wrapEnd;
};
/**
 *
 *
 */
class rpPIO
{
    friend class rpPio_SM;

  public:
    rpPIO(uint32_t unit);
    virtual ~rpPIO();
    // size is in 16 bits instrucitons
    rpPIO_SM *getSm(uint32_t sm);

  protected:
    int _unit;
    int _codeSize;
};
