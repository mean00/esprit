#pragma once

typedef void(lnBasicTimerIrq)(void *cookie);
/**
 * \brief Careful all the channels of a given timer share the same frequency!
 * Max frequency is around 100 kHz in PWM mode !
 *
 *
 * In all mode , after setup the channel is disabled
 * You have to explicitely call timer->enable()
 *
 * @param timer
 * @param channel
 */
class lnBasicTimer
{
  public:
    lnBasicTimer(int timer);
    virtual ~lnBasicTimer();
    // PWM
    void setTimerFrequency(
        int fqInHz); // This program the timer fq with a tick of 1024, i.e. internal fq is 1024* the given fq
    void setTimerFrequency(int prediv, int reload);
    int getTimerFrequency();
    void disable();
    void enable();

  protected:
    int _timer;
};
/**
 *
 * @param timer
 * @return
 */
class lnBasicDelayTimer : lnBasicTimer
{
  public:
    lnBasicDelayTimer(int timer);
    virtual ~lnBasicDelayTimer();
    void arm(int delayUs);
    void setInterrupt(lnBasicTimerIrq *handler, void *cookie);
    //--
    static void interruptHandler(int timer);

  protected:
    void irq();
    void enableInterrupt();
    void disableInterrupt();

    lnBasicTimerIrq *_handler;
    void *_cookie;
};