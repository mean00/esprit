/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once

struct LN_USART_Registersx
{
    uint32_t STAT;
    uint32_t DATA;
    uint32_t BAUD;
    uint32_t CTL0;
    uint32_t CTL1;
    uint32_t CTL2;
    uint32_t GP;
};

typedef volatile LN_USART_Registersx LN_USART_Registers;

// Registers STAT...
#define LN_USART_STAT_PERR (1 << 0)  // parity
#define LN_USART_STAT_FERR (1 << 1)  // frame
#define LN_USART_STAT_NERR (1 << 2)  // noise
#define LN_USART_STAT_ORERR (1 << 3) // OVerrun

#define LN_USART_STAT_ERROR_MASK (0xf)

#define LN_USART_STAT_OVERR (1 << 3) // Overrun
#define LN_USART_STAT_IDLEF (1 << 4) // Frame detect
#define LN_USART_STAT_RBNE (1 << 5)  // Read buffer not empty
#define LN_USART_STAT_TC (1 << 6)    // Tx complete
#define LN_USART_STAT_TBE (1 << 7)   // Tx buffer empty
#define LN_USART_STAT_LBDF (1 << 8)  // Break detect
#define LN_USART_STAT_CTSF (1 << 9)  // CTS change

// Registers BAUD...
#define LN_USART_BAUD_KEEP_MASK 0xFFFF0000
#define LN_USART_BAUD_BAUD(intg, frac) (intg << 4 + frac)

// GP , dont care

// CTL0
#define LN_USART_CTL0_KEEP_MASK (0xffffc000)
#define LN_USART_CTL0_SBKCMD (1 << 0)
#define LN_USART_CTL0_RWU (1 << 1)
#define LN_USART_CTL0_REN (1 << 2)
#define LN_USART_CTL0_TEN (1 << 3)
#define LN_USART_CTL0_IDLEIE (1 << 4)
#define LN_USART_CTL0_RBNEIE (1 << 5)
#define LN_USART_CTL0_TCIE (1 << 6)
#define LN_USART_CTL0_TBIE (1 << 7)
#define LN_USART_CTL0_PERRIE (1 << 8)
#define LN_USART_CTL0_PM (1 << 9)
#define LN_USART_CTL0_PCEN (1 << 10)
#define LN_USART_CTL0_WM (1 << 11)
#define LN_USART_CTL0_WL (1 << 12)
#define LN_USART_CTL0_UEN (1 << 13)

// CTL2
#define LN_USART_CTL2_DMA_TX (1 << 7)
#define LN_USART_CTL2_DMA_RX (1 << 6)
