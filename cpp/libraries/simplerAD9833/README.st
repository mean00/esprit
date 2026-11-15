
This is a derivative of the Adafruit MCP23017 i2c expander board driver.
The problem with that chip is that it generates an interrupt to notify pin(s) have changed,
*BUT* to acknowledge/get pin change informations you have to an i2c request

It means you have 3 ways to deal with that :
* Do i2c requests under interrupt, sort of fishy because i2c also uses interrupt.
* Do a polling method, but there is a significant risk you'll miss some stuff. For example with a rotary encoder.
* Use threads

This library is split in two parts.
The common parts just deals with the chip itself and offers a "high" level interface. It is not autonomous.
Noticable changes :
* Axx are all inputs
* Bxx are all outputs
* Built in support for rotary encoders
* Build in support for input lines
* Can use hw i2c or sw i2c


That low level lib has a "process" method that will check the pins and update the cliens (rotary encoder/buttons)


The 2nd part of the library is a freertos wrapper on top of the low level lib. It is using the FreeRtos1000_pp wrapper to have a
more readable C++ interface.
The 2nd library  has a small thread that will do the i2c operations , that thread is woken up by an interrupt

Usage :

* Create the lib
* Hook in clients (rotary  encoders/button/....)
* call process() or use the freertos wrapper

In case of freerts wrapper, make sure you use the i2c mutex on ALL i2c accesses to avoid races there

The free_rtos wrapper is in example, to that it is not systematically built
