#pragma once
#include "lnIRQ.h"
#include "lnPeripheral_priv.h"

#define LN_USBD_MAX_ENDPOINT 8 // should be enough

/**
 */

struct LN_USBD_Registersx
{
    uint32_t USBD_EPCS[8]; // 0x00--0x1c
    uint32_t filler[(0x40 - 0x20) / 4];
    uint32_t USBD_CTL;   // 0x40
    uint32_t USBD_INTF;  // 0x44
    uint32_t USBD_STAT;  // 0x48
    uint32_t USBD_DADDR; // 0x4C
    uint32_t USBD_BADDR; // 0x50
};

/**
 */
typedef volatile LN_USBD_Registersx LN_USBD_Registers;
/**
 */

//-------------- CTL ---------

#define LN_USBD_CTL_SETRST (1 << 0)
#define LN_USBD_CTL_CLOSE (1 << 1)
#define LN_USBD_CTL_LOWM (1 << 2)
#define LN_USBD_CTL_SETSPS (1 << 3) // suspend
#define LN_USBD_CTL_RSREQ (1 << 4)  // resume

// Only on GD32F3°xxx
#define LN_USBD_CTL_L1RSREQ_F3 (1 << 5) // Send Result signal to host
#define LN_USBD_CTL_L1REQIE_F3 (1 << 7) // Interrupt enable for the above

#define LN_USBD_CTL_ESOFIE (1 << 8)
#define LN_USBD_CTL_SOFIE (1 << 9)
#define LN_USBD_CTL_RSTIE (1 << 10)
#define LN_USBD_CTL_SPSIE (1 << 11)
#define LN_USBD_CTL_WKUPIE (1 << 12)
#define LN_USBD_CTL_ERRIE (1 << 13)
#define LN_USBD_CTL_PMOUIE (1 << 14)
#define LN_USBD_CTL_STIE (1 << 15)

//-------------- INTF ---------

#define LN_USBD_INTF_EPNUM(x) (x & 0x0f)
#define LN_USBD_INTF_DIR (1 << 4)
#define LN_USBD_INTF_L1REQIF_F3 (1 << 7)
#define LN_USBD_INTF_ESOFIF (1 << 8)
#define LN_USBD_INTF_SOFIF (1 << 9)
#define LN_USBD_INTF_RSTIF (1 << 10)
#define LN_USBD_INTF_SPSIF (1 << 11)
#define LN_USBD_INTF_WKUPIF (1 << 12)
#define LN_USBD_INTF_ERRIF (1 << 13)
#define LN_USBD_INTF_PMOUIF (1 << 14)
#define LN_USBD_INTF_STIF (1 << 15)

//-------------- STAT ---------

#define LN_USBD_STAT_FNCT(x) (x & 0x3ff)
#define LN_USBD_STAT_SOFLN(x) ((x & 3) << 11)
#define LN_USBD_STAT_LOCK (1 << 13)
#define LN_USBD_STAT_RX_DM (1 << 14)
#define LN_USBD_STAT_RX_DP (1 << 15)

//-------------- EPxCS ---------
#define LN_USBD_EPxCS_EPADDR_MASK (0xf)
#define LN_USBD_EPxCS_EPADDR(x) (x & 0xf)
#define LN_USBD_EPxCS_TX_DTG (1 << 6)
#define LN_USBD_EPxCS_TX_ST (1 << 7)
#define LN_USBD_EPxCS_EP_KCTL (1 << 8)
#define LN_USBD_EPxCS_SETUP (1 << 11)
#define LN_USBD_EPxCS_RX_DTG (1 << 14)
#define LN_USBD_EPxCS_RX_ST (1 << 15)

#define LN_USBD_EPxCS_RX_STA_MASK (3 << 12)
#define LN_USBD_EPxCS_RX_STA_DISABLED (0 << 12)
#define LN_USBD_EPxCS_RX_STA_STALL (1 << 12)
#define LN_USBD_EPxCS_RX_STA_NAK (2 << 12)
#define LN_USBD_EPxCS_RX_STA_VALID (3 << 12)

#define LN_USBD_EPxCS_RX_STA_BIT1 (1 << 12)
#define LN_USBD_EPxCS_RX_STA_BIT2 (2 << 12)

#define LN_USBD_EPxCS_EPCTL_MASK (3 << 9)
#define LN_USBD_EPxCS_EPCTL_BULK (0 << 9)
#define LN_USBD_EPxCS_EPCTL_CONTROL (1 << 9)
#define LN_USBD_EPxCS_EPCTL_ISO (2 << 9)
#define LN_USBD_EPxCS_EPCTL_INTERRUPT (3 << 9)

#define LN_USBD_EPxCS_TX_STA_MASK (3 << 4)
#define LN_USBD_EPxCS_TX_STA_DISABLED (0 << 4)
#define LN_USBD_EPxCS_TX_STA_STALL (1 << 4)
#define LN_USBD_EPxCS_TX_STA_NAK (2 << 4)
#define LN_USBD_EPxCS_TX_STA_VALID (3 << 4)

#define LN_USBD_EPxCS_TX_STA_BIT1 (1 << 4)
#define LN_USBD_EPxCS_TX_STA_BIT2 (2 << 4)

// EOF
