/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once
#include "lnPeripheral_priv.h"
struct LN_I2C_Registersx
{
    uint32_t CTL0;
    uint32_t CTL1;
    uint32_t SADDR0;
    uint32_t SADDR1;
    uint32_t DATA;
    uint32_t STAT0;
    uint32_t STAT1;
    uint32_t CKCFG;
    uint32_t RT;
};

typedef volatile LN_I2C_Registersx LN_I2C_Registers;

// CTL0

#define LN_I2C_CTL0_I2CEN (1 << 0)
#define LN_I2C_CTL0_SMBEN (1 << 1)  // SMB protocol enable
#define LN_I2C_CTL0_SMBSEL (1 << 3) // SMB mode sel : device/host
#define LN_I2C_CTL0_ARPEN (1 << 4)  // ARP protocol enable (smb)
#define LN_I2C_CTL0_PECEN (1 << 5)  // PEC calculation on
#define LN_I2C_CTL0_GCEN (1 << 6)   // respond to general call (slave)
#define LN_I2C_CTL0_SS (1 << 7)     // strech SCL when data not ready in slave mode
#define LN_I2C_CTL0_START (1 << 8)
#define LN_I2C_CTL0_STOP (1 << 9)
#define LN_I2C_CTL0_ACKEN (1 << 10)    //
#define LN_I2C_CTL0_POAP (1 << 11)     // position of Ack and PEC
#define LN_I2C_CTL0_PECTRANS (1 << 12) // PEC transfer
#define LN_I2C_CTL0_SALT (1 << 13)     // smbus alert
#define LN_I2C_CTL0_SRESET (1 << 15)   // reset

// CTL1

#define LN_I2C_CTL1_I2CCLK_MASK (~(0xcf)) // APB1 frequency in MHZ
#define LN_I2C_CTL1_ERRIE (1 << 8)        // Error interrupt enable
#define LN_I2C_CTL1_EVIE (1 << 9)         // Event interrupt enable
#define LN_I2C_CTL1_BUFIE (1 << 10)       // Interrupt enable when TBE=1/RBNE=1&EVIE=1
#define LN_I2C_CTL1_DMAON (1 << 11)       // DMA enable
#define LN_I2C_CTL1_DMALST (1 << 12)      // DMA last

// STAT0

#define LN_I2C_STAT0_SBSEND (1 << 0)    // Start condition sent (master)
#define LN_I2C_STAT0_ADDSEND (1 << 1)   // Address send (master)
#define LN_I2C_STAT0_BTC (1 << 2)       // Byte tranmision complete
#define LN_I2C_STAT0_ADD10SEND (1 << 3) // Header in 10 bits (master)
#define LN_I2C_STAT0_STPDET (1 << 4)    // Stop detected (slave)
#define LN_I2C_STAT0_RBNE (1 << 6)      // RX not empty
#define LN_I2C_STAT0_TBE (1 << 7)       // Transmit empty
#define LN_I2C_STAT0_BERR (1 << 8)      // Bus error
#define LN_I2C_STAT0_LOSTARB (1 << 9)   // Lost arbitration
#define LN_I2C_STAT0_AERR (1 << 10)     // Ack error
#define LN_I2C_STAT0_OUERR (1 << 11)    // Overrun/underrun error
#define LN_I2C_STAT0_PECERR (1 << 12)   // PEC error
#define LN_I2C_STAT0_SMBTO (1 << 14)    // SMB timeout
#define LN_I2C_STAT0_SMBALT (1 << 15)   // SMB bus altert

// STAT1

#define LN_I2C_STAT1_MASTER (1 << 0)
#define LN_I2C_STAT1_I2CBSY (1 << 1)    // busy
#define LN_I2C_STAT1_TR (1 << 2)        // transmitter/receiver state
#define LN_I2C_STAT1_RXGC (1 << 4)      // General call received
#define LN_I2C_STAT1_DEFSMB (1 << 5)    // default SMB adress
#define LN_I2C_STAT1_HSTSMB (1 << 6)    // smb host header
#define LN_I2C_STAT1_DUOMODF (1 << 7)   // dual flag (slave)
#define LN_I2C_STAT1_PEC_MASK (~(0xff)) // packet error checking value (CRC)

// CKCFG

#define LN_I2C_CKCFG_DTCY (1 << 14) // duty cycle in fast mode
#define LN_I2C_CKCFG_FAST (1 << 15) // Fast mode (mster)
#define LN_I2C_CKCFG_CLKC_MASK ((0xc000))

// I2C RT
#define LN_I2C_RT_RISETIME_MASK (~(0xcf))

#define LN_FMP_FMPEN (1 << 0)

// EOF