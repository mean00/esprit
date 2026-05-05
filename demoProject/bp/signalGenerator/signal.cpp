#include "signal.h"
#include "esprit.h"
#include "math.h"

/**
 *
 * @param pin
 */
SignalGenerator::SignalGenerator(lnPin pin, int dac)
{
    lnPinMode(pin, lnDAC_MODE);
    _dac = new lnDAC(dac);
}
/**
 *
 */
SignalGenerator::~SignalGenerator()
{
    delete _dac;
    _dac = NULL;
}
/**
 *
 */
void SignalGenerator::stop()
{
    _dac->stopDmaMode();
}
/**
 *
 * @param fq
 * @param form
 */
void SignalGenerator::start(int fq, SignalForm form)
{
    switch (form)
    {
    case SignalSine: {
        _dac->startDmaMode(fq * 100);
        int actualFq = _dac->getDmaFrequency();
        float pointFloat = (float)(actualFq + fq / 2) / (float)fq;
        _nbPoints = floor(pointFloat + 0.49f);
        // Logger("In Fq=%d outFq=%d, # points=%d pointsF=%f\n", fq, actualFq, _nbPoints, pointFloat);
        Logger("In Fq=%d outFq=%d, # points=%d pointsF=\n", fq, actualFq, _nbPoints);

        for (int i = 0; i < _nbPoints; i++)
        {
            float angle = 2.f * (float)M_PI;
            angle /= (float)_nbPoints;
            angle *= (float)i;
            _waveForm[i] = 2048.f + 2047.f * (float)sin(angle);
        }
    }
    break;
    default:
        xAssert(0);
        break;
    }
    _dac->dmaWrite(_nbPoints, _waveForm, true);
}
/**
 *
 * @return
 */
int SignalGenerator::getActualFrequency()
{
    return _dac->getDmaFrequency();
}
// EOF
