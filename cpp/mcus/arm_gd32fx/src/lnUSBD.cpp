
/*
This is derived from the stm sample code from tinyusb
Original Copyright (c) 2019 Nathan Conrad
    *
    * Portions:
    * Copyright (c) 2016 STMicroelectronics
    * Copyright (c) 2019 Ha Thach (tinyusb.org)

    * Permission is hereby granted, free of charge, to any person obtaining a copy
    * of this software and associated documentation files (the "Software"), to deal
    * in the Software without restriction, including without limitation the rights
    * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    * copies of the Software, and to permit persons to whom the Software is
    * furnished to do so, subject to the following conditions:


 * The modifications are (C) 2022 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "esprit.h"

#include "lnPinMapping.h"
#include "lnUSBD.h"
#include "lnUSBD_priv.h"

LN_USBD_Registers *aUSBD0 = (LN_USBD_Registers *)(LN_USBD0_ADR);
lnUsbDevice *_usbInstance = NULL;
volatile uint32_t *aUSBD0_SRAM = (volatile uint32_t *)LN_USBD0_RAM_ADR;
#include "lnUSBD_endPoints.h"
EndPoints::xfer_descriptor EndPoints::xfer_status[LN_USBD_MAX_ENDPOINT * 2];
extern "C" void dcd_int_handler(uint8_t rhport);
// this is mapped at the start of of the SrAM

int EndPoints::ep_bufferTail;
int EndPoints::ep_nbEp;

#define CLEAR_INTERRUPT(x)                                                                                             \
    {                                                                                                                  \
        aUSBD0->USBD_INTF &= ~(LN_USBD_INTF_##x##IF);                                                                  \
    }
#define DISABLE_INTERRUPT(x)                                                                                           \
    {                                                                                                                  \
        aUSBD0->USBD_CTL &= ~(LN_USBD_CTL_##x##IE);                                                                    \
    }
#define CLEAR_CONTROL(x)                                                                                               \
    {                                                                                                                  \
        aUSBD0->USBD_CTL &= ~(LN_USBD_CTL_##x);                                                                        \
    }
#define SET_CONTROL(x)                                                                                                 \
    {                                                                                                                  \
        aUSBD0->USBD_CTL |= (LN_USBD_CTL_##x);                                                                         \
    }

/**
 */
lnUsbDevice::lnUsbDevice(int port)
{
    xAssert(port == 0);
    _handler = NULL;
    _usbInstance = this;
    _remoteWakeCountdown = 4;
}
/**
 */
lnUsbDevice::~lnUsbDevice()
{
    power(false);
    _usbInstance = NULL;
}
/**
 */
bool lnUsbDevice::init()
{
    // Only do low level hw setup here
    // the follow up will be in hwReset
    // first disable interrupt
    irqEnabled(false);
    // 48 Mhz input to usb
    lnPeripherals::enableUsb48Mhz();
    //
    EndPoints::reset();
    // Switch pins... PA11 and PA12
    lnPinMode(PA12, lnINPUT_FLOATING); // D+
    lnPinMode(PA11, lnINPUT_FLOATING); // D-
    power(true);
    return true;
}
/**
 */
bool lnUsbDevice::power(bool onoff)
{
    if (onoff)
    {
        lnPeripherals::enable(Peripherals::pUSB);
        lnPeripherals::reset(Peripherals::pUSB);
    }
    else
    {
        lnPeripherals::disable(Peripherals::pUSB);
    }
    return true;
}
/**
 */
bool lnUsbDevice::irqEnabled(bool onoff)
{
    if (onoff)
    {
        lnEnableInterrupt(LN_IRQ_USB_HP_CAN_TX);
        lnEnableInterrupt(LN_IRQ_USB_LP_CAN_RX0);
        lnEnableInterrupt(LN_IRQ_USBWAKEUP);
    }
    else
    {
        lnDisableInterrupt(LN_IRQ_USB_HP_CAN_TX);
        lnDisableInterrupt(LN_IRQ_USB_LP_CAN_RX0);
        lnDisableInterrupt(LN_IRQ_USBWAKEUP);
    }
    return true;
}
/**
  Copy from SRAM to user memory
  The SRAM is only using 16 bits every 32 bits


*/
void lnUsbDevice::copyFromSRAM(uint8_t *dest, int srcOffset, int bytes)
{
    xAssert(!(srcOffset & 1));
    int nbWord = bytes / 2;
    volatile uint16_t *ram = (volatile uint16_t *)aUSBD0_SRAM;
    ram += srcOffset;

    if (((uint32_t)dest) & 1) // not aligned
    {
        uint8_t *target = dest;

        for (int i = 0; i < nbWord; i++)
        {
            int temp = *ram;
            ram += 2;
            *target++ = ((temp >> 0) & 0xFF); // copy 16 bits every 32 bits
            *target++ = ((temp >> 8) & 0xFF);
        }
    }
    else // aligned, copy 16 bits at a time
    {
        uint16_t *target16 = (uint16_t *)dest;

        int pack4 = nbWord / 8;
        for (int i = 0; i < pack4; i++)
        {
            target16[0] = ram[0];
            target16[1] = ram[2];
            target16[2] = ram[4];
            target16[3] = ram[6];
            target16[4] = ram[8];
            target16[5] = ram[10];
            target16[6] = ram[12];
            target16[7] = ram[14];
            ram += 16;
            target16 += 8;
        }
        int l = nbWord & 7;
        for (int i = 0; i < l; i++)
        {
            target16[0] = ram[0];
            ram += 2;
            target16 += 1;
        }
    }

    if (bytes & 1)
    {
        int temp = *ram;
        dest[bytes - 1] = temp & 0xff;
    }
    return;
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void lnUsbDevice::copyToSRAM(int destOffset, uint8_t *src, int bytes)
{
    int nbWords = (bytes + 1) / 2; // might overread by one byte,not an issue
    volatile uint16_t *ram = (volatile uint16_t *)aUSBD0_SRAM;
    xAssert(!(destOffset & 1));
    ram += destOffset;
    if (((uint32_t)src) & 1) // not aligned
    {
        uint8_t *s8 = src;
        for (int i = 0; i < nbWords; i++)
        {
            int temp = (int)s8[0] + ((int)s8[1] << 8);
            *ram = temp;
            ram += 2;
            s8 += 2;
        }
    }
    else
    { // aligned
        uint16_t *s16 = (uint16_t *)src;
        int pack8 = nbWords / 8;
        for (int i = 0; i < pack8; i++)
        {
            ram[0] = s16[0];
            ram[2] = s16[1];
            ram[4] = s16[2];
            ram[6] = s16[3];
            ram[8] = s16[4];
            ram[10] = s16[5];
            ram[12] = s16[6];
            ram[14] = s16[7];
            ram += 16;
            s16 += 8;
        }
        int l = nbWords & 7;
        for (int i = 0; i < l; i++)
        {
            *ram = *s16++;
            ram += 2;
        }
    }
    // We copy one extra byte in case it is not even, no need to take care of it
}

/**
 */
bool lnUsbDevice::registerEventHandler(const lnUsbEventHandler *h)
{
    _handler = h;
    return true;
}

/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */
#define MASK_CLEAR                                                                                                     \
    (0xffff & ~LN_USBD_EPxCS_RX_DTG & ~LN_USBD_EPxCS_TX_DTG & ~LN_USBD_EPxCS_RX_STA_MASK & ~LN_USBD_EPxCS_TX_STA_MASK)
void lnUsbDevice::clearTxRx(int ep, bool isTx)
{
    uint32_t reg = aUSBD0->USBD_EPCS[ep & 7] & 0xffff;
    reg &= MASK_CLEAR;
    if (isTx)
    {
        reg &= ~LN_USBD_EPxCS_TX_ST;
        reg |= LN_USBD_EPxCS_RX_ST;
    }
    else // Rx
    {
        reg &= ~LN_USBD_EPxCS_RX_ST;
        reg |= LN_USBD_EPxCS_TX_ST;
    }
    aUSBD0->USBD_EPCS[ep & 7] = reg & 0xffff;
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void lnUsbDevice::toggleDTG(int ep, bool isTx)
{
    uint32_t reg = aUSBD0->USBD_EPCS[ep & 7] & 0xffff;
    reg &= MASK_CLEAR;
    uint32_t change = LN_USBD_EPxCS_RX_DTG;
    if (isTx)
        change = LN_USBD_EPxCS_TX_DTG;
    reg |= LN_USBD_EPxCS_RX_ST + LN_USBD_EPxCS_TX_ST + change;
    aUSBD0->USBD_EPCS[ep & 7] = reg & 0xffff;
}

/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void lnUsbDevice::setEpKind(int ep, bool set)
{
    uint32_t reg = aUSBD0->USBD_EPCS[ep & 7] & 0xffff;

    reg &= MASK_CLEAR;
    if (set)
        reg |= LN_USBD_EPxCS_EP_KCTL;
    else
        reg &= ~LN_USBD_EPxCS_EP_KCTL;
    reg |= LN_USBD_EPxCS_RX_ST + LN_USBD_EPxCS_TX_ST;
    aUSBD0->USBD_EPCS[ep & 7] = reg & 0xffff;
}

void lnUsbDevice::setEpAddress(int ep, int adr)
{
    uint32_t reg = aUSBD0->USBD_EPCS[ep & 7] & 0xffff;

    reg &= MASK_CLEAR;
    reg |= adr;
    reg |= LN_USBD_EPxCS_RX_ST + LN_USBD_EPxCS_TX_ST;

    aUSBD0->USBD_EPCS[ep & 7] = reg & 0xffff;
}

/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void lnUsbDevice::setEpStatus(int ep, bool isTx, int status)
{
    // rx

    uint32_t reg = aUSBD0->USBD_EPCS[ep & 7] & 0xffff;
    uint32_t c;

    if (isTx)
    {
        reg &= MASK_CLEAR + LN_USBD_EPxCS_TX_STA_MASK;
        c = LN_USBD_EPxCS_TX_STA_MASK & status;
    }
    else
    {
        reg &= MASK_CLEAR + LN_USBD_EPxCS_RX_STA_MASK;
        c = LN_USBD_EPxCS_RX_STA_MASK & status;
    }
    reg ^= c;
    reg |= LN_USBD_EPxCS_RX_ST + LN_USBD_EPxCS_TX_ST;
    aUSBD0->USBD_EPCS[ep & 7] = reg & 0xffff;
}

/**
 */
void lnUsbDevice::clearDTG(int ep, bool isTx)
{
    uint32_t reg = aUSBD0->USBD_EPCS[ep & 7] & 0xffff;
    uint32_t change = 0;
    if (isTx)
    {
        if (reg & LN_USBD_EPxCS_TX_DTG)
            change = LN_USBD_EPxCS_TX_DTG;
    }
    else
    {
        if (reg & LN_USBD_EPxCS_RX_DTG)
            change = LN_USBD_EPxCS_RX_DTG;
    }
    if (change)
    {
        reg &= MASK_CLEAR;
        reg |= LN_USBD_EPxCS_RX_ST + LN_USBD_EPxCS_TX_ST + change;
        aUSBD0->USBD_EPCS[ep & 7] = reg & 0xffff;
    }
}

/**
 */
void lnUsbDevice::setEpType(int ep, int type)
{
    uint32_t reg = aUSBD0->USBD_EPCS[ep & 7] & 0xffff;
    reg &= MASK_CLEAR & ~LN_USBD_EPxCS_EPCTL_MASK;
    reg |= type;
    reg |= LN_USBD_EPxCS_RX_ST + LN_USBD_EPxCS_TX_ST;
    aUSBD0->USBD_EPCS[ep & 7] = reg & 0xffff;
}

/**

*/
int lnUsbDevice::getEpStatusReg(int ep)
{
    return aUSBD0->USBD_EPCS[ep & 7] & 0xffff;
}
/**

*/
void lnUsbDevice::setEpStatusReg(int ep, int value)
{
    aUSBD0->USBD_EPCS[ep & 7] = value & 0xffff;
}
/**
 */
bool lnUsbDevice::wakeUpHost()
{
    aUSBD0->USBD_CTL |= LN_USBD_CTL_RSREQ; // clear suspend
    return true;
}
void lnUsbDevice::resetEps()
{
    for (uint32_t i = 0; i < LN_USBD_MAX_ENDPOINT; i++)
        _usbInstance->setEpStatusReg(i, 0);
    EndPoints::reset();
}
/**

*/
void lnUsbDevice::hwReset()
{
#if 0 // According to GD doc...
1. Clear the CLOSE bit in USBD_CTL register, then clear the SETRST bit.
2. Clear USBD_INTF register to remove any spurious pending interrupt.
3. Program USBD_BADDR register to set endpoint buffer base address.
4. Set USBD_CTL register to enable interrupts.
5. Wait for the reset interrupt (RSTIF).
6. In the reset interrupt, initialize default control endpoint 0 to start enumeration process and
program USBD_ADDR to set the device address to 0 and enable USB module function.
7. Configure endpoint 0 and prepare to receive SETUP packet.
#endif
    // Force host reset by asserting PA12 low for 10 ms
    lnDigitalWrite(PA12, 0);
    lnPinMode(PA12, lnOUTPUT); // D+
    lnDigitalWrite(PA12, 0);
    lnDelay(10);
    lnPinMode(PA12, lnINPUT_FLOATING); // D+

    CLEAR_CONTROL(CLOSE);
    CLEAR_CONTROL(SETRST);
    aUSBD0->USBD_INTF &= LN_USBD_INTF_RSTIF; // clear all except reset
    // Clear all interrupts
    aUSBD0->USBD_BADDR = 0; // Descriptors at the begining
    aUSBD0->USBD_DADDR = 0; // address 0, disabled
    aUSBD0->USBD_STAT = 0;

#define GG(x) LN_USBD_CTL_##x
    uint32_t inter = aUSBD0->USBD_CTL;
    inter |= GG(RSTIE) + GG(PMOUIE) + GG(STIE); // + GG(WKUPIE) + GG(SPSIE); //+ GG(ESOFIE); //Ignore ESOFIE/SOFIE
                                                // +GG(ESOFIE)++GG(SOFIE);  GG(ERRIE) +
    aUSBD0->USBD_CTL = inter;
}
/**
 */
bool lnUsbDevice::setAddress(int address)
{
    aUSBD0->USBD_DADDR = 0;
    if (address == -1)
    {
        return true;
    }
    aUSBD0->USBD_DADDR = 0x80 + (address & 0x7f); // address 0, enabled
    return true;
}
/**
 */
void USB_TX_IRQHandler()
{
    xAssert(_usbInstance);
    _usbInstance->irq();
}
/**
 */
void USB_RX_IRQHandler()
{
    xAssert(_usbInstance);
    _usbInstance->irq();
}
void USB_WAKEUP_IRQHandler()
{
    xAssert(_usbInstance);
    _usbInstance->irq();
}

/**
 */

void lnUsbDevice::irq()
{
    // What interrupt triggered ?
    uint32_t flags_original = aUSBD0->USBD_INTF;
    uint32_t flags_mask = aUSBD0->USBD_CTL;
    uint32_t flags = (flags_original & flags_mask & 0xff00);
    // static uint32_t flags=(flags_original);

    if (flags & LN_USBD_INTF_RSTIF /*|| (aUSBD0->USBD_CTL & LN_USBD_CTL_SETRST)*/)
    {
        CLEAR_INTERRUPT(RST);
        //  CLEAR_CONTROL(SETRST);
        //  CLEAR_CONTROL(RSTIE);
        _handler->event(lnUsbEventHandler::UsbReset);
        return;
    }

    if (flags & LN_USBD_INTF_STIF) // transfer complete
    {

        // The endpoint identity lives in the low bits of USBD_INTF, which
        // `flags` above masked off (it only keeps the interrupt *types*).
        // Dispatching this first completion with the masked value reported it
        // as an EP0 IN completion: tinyusb then saw a bogus control transfer
        // completion and the real endpoint's TX_ST was left set. Use the value
        // read at interrupt entry, like the re-read in the loop below.
        uint32_t f = flags_original;
        flags &= ~LN_USBD_INTF_STIF;
        while (f & LN_USBD_INTF_STIF)
        {
            if (f & LN_USBD_INTF_DIR) // OUT Type PC-> DEV
            {
                uint32_t reg = _usbInstance->getEpStatusReg(LN_USBD_INTF_EPNUM(f));
                if (reg & LN_USBD_EPxCS_RX_ST)
                {
                    _handler->event(lnUsbEventHandler::UsbTransferRxCompleted, LN_USBD_INTF_EPNUM(f));
                }
            }
            else // In Type Ddev->PC
            {
                uint32_t reg = _usbInstance->getEpStatusReg(LN_USBD_INTF_EPNUM(f));
                if ((reg & LN_USBD_EPxCS_TX_ST))
                {
                    _handler->event(lnUsbEventHandler::UsbTransferTxCompleted, LN_USBD_INTF_EPNUM(f));
                }
            }
            f = aUSBD0->USBD_INTF;
        }
    }

    if (flags & LN_USBD_INTF_ERRIF) // error
    {
        CLEAR_INTERRUPT(ERR);
        flags &= ~LN_USBD_INTF_ERRIF;
    }

    if (flags & LN_USBD_INTF_WKUPIF) // Wakeup
    {
        CLEAR_CONTROL(LOWM);
        CLEAR_CONTROL(SETSPS);
        CLEAR_INTERRUPT(WKUP);
        flags &= ~LN_USBD_INTF_WKUPIF;
        _handler->event(lnUsbEventHandler::UsbResume);
    }

    if (flags & LN_USBD_INTF_SPSIF) // suspend
    {
        SET_CONTROL(SETSPS);
        SET_CONTROL(LOWM);
        CLEAR_INTERRUPT(SPS);
        flags &= ~LN_USBD_INTF_SPSIF;
        _handler->event(lnUsbEventHandler::UsbSuspend);
    }

    if (flags & LN_USBD_INTF_SOFIF) // start of frame, not used
    {
        CLEAR_INTERRUPT(SOF);
        DISABLE_INTERRUPT(SOF);
        flags &= ~LN_USBD_INTF_SOFIF;
    }

    if (flags & LN_USBD_INTF_ESOFIF) // expected end of frame
    {
        if (_remoteWakeCountdown == 1)
        {
            CLEAR_CONTROL(RSREQ);
        }
        if (_remoteWakeCountdown > 0)
        {
            _remoteWakeCountdown--;
        }
        CLEAR_INTERRUPT(ESOF);
        flags &= ~LN_USBD_INTF_ESOFIF;
    }
    if (flags)
        xAssert(0);
}

// EOF
