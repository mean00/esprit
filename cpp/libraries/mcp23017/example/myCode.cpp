/***************************************************
 Digitally controlled power supply
 *  * GPL v2
 * (c) mean 2018 fixounet@free.fr
 ****************************************************/

#include "mcp23017.h"
#include <Wire.h>

myMcp23017 *mcp;

myMcpButtonInput *pushButton;
myMcpRotaryEncoder *rotary;

void mySetup()
{
    Wire.begin();
    Serial.begin(57600);
    Serial.println("Init");
    // Reset the chip, connected to PA1
    pinMode(PA1, OUTPUT);
    digitalWrite(PA1, 0);
    lnDelayMs(100);
    digitalWrite(PA1, 1);

    //

    mcp = myMcp23017::create(PA2);
    // continue init here, add clients
    pushButton = new myMcpButtonInput(mcp, 2); // A2
    rotary = new myMcpRotaryEncoder(mcp, 0, 1);

    // then go
    mcp->start();
}
/**
 */

void myLoop(void)
{
    static bool pin = false;
    static int blk = 0;
    static int counter = 10;

    for (int i = 0; i < 10; i++)
    {
        mcp->process();
        lnDelayMs(10);
    }
    int inc = rotary->count();
    if (inc)
    {
        counter += inc;
        ;
    }
    if (pushButton->changed())
    {
        pin = !pin;
        mcp->digitalWrite(0, pin);
    }
    mcp->digitalWrite(7, blk & 4);
    blk++;
    {
        printf("Count=%d\n", counter);
    }
}
//-
