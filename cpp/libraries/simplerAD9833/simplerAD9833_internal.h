/***************************************************
  This is a library for the MCP23017 i2c port expander

  These displays use I2C to communicate, 2 pins are required to
  interface
  Adafruit invests time and resources providing this open source code,
  please support Adafruit and open-source hardware by purchasing
  products from Adafruit!

  Written by Limor Fried/Ladyada for Adafruit Industries.
  BSD license, all text above must be included in any redistribution
 ****************************************************/
#pragma once

#define LN_AD9833_CTRL(x) ((x) << 14)
#define LN_AD9833_FREQ (1 << 14)    //
#define LN_AD9833_B28 (1 << 13)     // allows 2 14 bits writes to write a 28 bits register
#define LN_AD9833_HLB (1 << 12)     // Half write, only write 1/2 register
#define LN_AD9833_FSELECT (1 << 11) // Freq0 or Freq1 register
#define LN_AD9833_PSELECT (1 << 10) // Phase0 or Phase 1
#define LN_AD9833_RESET (1 << 8)    // When set, put to sleep
#define LN_AD9833_SLEEP1 (1 << 7)   // When set, put to sleep
#define LN_AD9833_SLEEP12 (1 << 6)  // When set, put *dac* to sleep
#define LN_AD9833_OPBITEN (1 << 5)  // When 1 send raw clock, when  0 sends waveform
#define LN_AD9833_DIV2 (1 << 3)     // Raw clock /2 when set
#define LN_AD9833_MODE (1 << 1)     // When 1 triangle, when 0 sin