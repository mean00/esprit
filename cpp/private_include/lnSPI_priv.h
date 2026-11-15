/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */

#pragma once
#include "stdint.h"
struct LN_SPI_Registersx
{
    uint32_t CTL0;    // 0
    uint32_t CTL1;    // 4
    uint32_t STAT;    // 8
    uint32_t DATA;    // c
    uint32_t CRCPOLY; // 10
    uint32_t RXCRC;   // 14
    uint32_t TXCRC;   // 18
    uint32_t I2SCTL;  // 1C
    uint32_t I2SPSC;  // 20
};

typedef volatile LN_SPI_Registersx LN_SPI_Registers;

//----------- CTL0 --------------------

#define LN_SPI_CTL0_CKPH (1 << 0)
#define LN_SPI_CTL0_CKPL (1 << 1)
#define LN_SPI_CTL0_MSTMODE (1 << 2)      // MAster
#define LN_SPI_CTL0_PCS(x) ((x & 7) << 3) //
#define LN_SPI_CTL0_SPIEN (1 << 6)        //
#define LN_SPI_CTL0_LSB (1 << 7)          //
#define LN_SPI_CTL0_SWNSS (1 << 8)        //
#define LN_SPI_CTL0_SWNSSEN (1 << 9)      //
#define LN_SPI_CTL0_RO (1 << 10)          //
#define LN_SPI_CTL0_FF16 (1 << 11)        //
#define LN_SPI_CTL0_CRCNT (1 << 12)       //
#define LN_SPI_CTL0_CRCEN (1 << 13)       //
#define LN_SPI_CTL0_BDOEN (1 << 14)       //
#define LN_SPI_CTL0_BDEN (1 << 15)        //

#define LN_SPI_CTL0_MASK 0xffff0000U

//----------- CTL1 --------------------
#define LN_SPI_CTL1_DMAREN (1 << 0)
#define LN_SPI_CTL1_DMATEN (1 << 1)
#define LN_SPI_CTL1_NSSDRV (1 << 2) //
#define LN_SPI_CTL1_NSSP (1 << 3)   //
#define LN_SPI_CTL1_TMOD (1 << 4)   //
#define LN_SPI_CTL1_ERRIE (1 << 5)  //
#define LN_SPI_CTL1_RBNEIE (1 << 6) //
#define LN_SPI_CTL1_TBEIE (1 << 7)  //
#define LN_SPI_CTL1_MASK 0xffffff00U

//----------- STAT --------------------
#define LN_SPI_STAT_RBNE (1 << 0)
#define LN_SPI_STAT_TBE (1 << 1)
#define LN_SPI_STAT_I2SCH (1 << 2)   //
#define LN_SPI_STAT_TXURERR (1 << 3) //
#define LN_SPI_STAT_CRCERR (1 << 4)  //
#define LN_SPI_STAT_CONFERR (1 << 5) //
#define LN_SPI_STAT_RXORERR (1 << 6) //
#define LN_SPI_STAT_TRANS (1 << 7)   //
#define LN_SPI_STAT_FERR (1 << 7)    //

#define LN_SPI_STAT_MASK 0xfffffe00U

//-- Others are CRC/I2S registers, wec dont use that for now ---

#define senable()                                                                                                      \
    {                                                                                                                  \
        if (1 && (_regs->CTL0 & LN_SPI_CTL0_SPIEN))                                                                    \
            xAssert(0);                                                                                                \
        (_regs->CTL0 |= LN_SPI_CTL0_SPIEN);                                                                            \
    }
#define txbusy() (!(_regs->STAT & LN_SPI_STAT_TBE))

#define sdisable()                                                                                                     \
    {                                                                                                                  \
        _regs->CTL0 &= ~LN_SPI_CTL0_SPIEN;                                                                             \
    }
#define sbusy() (_regs->STAT & LN_SPI_STAT_TRANS)

enum lnSpiDirection
{
    lnDuplex = 0,
    lnTxOnly = 1,
    lnRxOnly = 2
};

void updateMode(LN_SPI_Registers *d, lnSpiDirection dir);
void updateDataSize(LN_SPI_Registers *d, int bits);
void updateDmaTX(LN_SPI_Registers *d, bool onoff);
