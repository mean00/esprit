/*
* Derived from dcd_stm32_fsdev.c

original header below

 * Copyright (c) 2019 Nathan Conrad
 *
 * Portions:
 * Copyright (c) 2016 STMicroelectronics
 * Copyright (c) 2019 Ha Thach (tinyusb.org)
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 * This file is part of the TinyUSB stack.

 */
#include "lnUSDB_tinyUSB.h"

#include "../private_include/lnUSBD_endPoints.h"
#include "ep0desc.h"

static TU_ATTR_ALIGNED(4) uint32_t _setup_packet[6];
extern lnUsbDevice *_usbInstance;

// into the stack.
void dcd_handle_bus_reset(void);
void dcd_transmit_packet(EndPoints::xfer_descriptor *xfer, uint16_t ep_ix);
void dcd_ep_ctr_handler(void);
void dcd_set_address(uint8_t rhport, uint8_t dev_addr);
void dcd_edpt0_status_complete(uint8_t rhport, tusb_control_request_t const *request);
void dcd_edpt_stall(uint8_t rhport, uint8_t ep_addr);
void dcd_edpt_clear_stall(uint8_t rhport, uint8_t ep_addr);
// Receive Set Address request, mcu port must also include status IN response
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void dcd_set_address(uint8_t rhport, uint8_t dev_addr)
{
    (void)rhport;
    (void)dev_addr;

    // Respond with status
    dcd_edpt_xfer(rhport, tu_edpt_addr(0, TUSB_DIR_IN), NULL, 0);

    // DCD can only set address after status for this request is complete.
    // do it at dcd_edpt0_status_complete()
}

/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void dcd_handle_bus_reset(void)
{
    _usbInstance->setAddress(-1); // disable
    // Clear all EPREG (or maybe this is automatic? I'm not sure)
    _usbInstance->resetEps();
    EndPoints::reset();
    dcd_edpt_open(0, &ep0OUT_desc);
    dcd_edpt_open(0, &ep0IN_desc);
    _usbInstance->setAddress(0);
}

/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */
void dcd_ep_ctr_tx_handler(int ep)
{
    /* clear int flag */
    _usbInstance->clearTxRx(ep, true);
    EndPoints::xfer_descriptor *xfer = EndPoints::getDescriptor(ep, TUSB_DIR_IN);
    if ((xfer->total_len != xfer->queued_len)) /* TX not complete */
        dcd_transmit_packet(xfer, ep);
    else /* TX Complete */
        dcd_event_xfer_complete(0, (uint8_t)(0x80 + ep), xfer->total_len, XFER_RESULT_SUCCESS, true);
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */
void dcd_ep_ctr_rx_handler(int ep)
{

    uint32_t wEPRegVal = _usbInstance->getEpStatusReg(ep);
    uint32_t count = EndPoints::getBTable(ep)->RxSize & 0x3ffU; // pcd_get_ep_rx_cnt(EPindex);

    EndPoints::xfer_descriptor *xfer = EndPoints::getDescriptor(ep, TUSB_DIR_OUT);

    if ((ep == 0U) && ((wEPRegVal & LN_USBD_EPxCS_SETUP) != 0U)) /* Setup packet */
    {
        // The setup_received function uses memcpy, so this must first copy the setup data into
        // user memory, to allow for the 32-bit access that memcpy performs.
        uint8_t userMemBuf[8];
        /* Get SETUP Packet*/
        if (count == 8) // Setup packet should always be 8 bytes. If not, ignore it, and try again.
        {
            // Must reset EP to NAK (in case it had been stalling) (though, maybe too late here)
            _usbInstance->setEpStatus(0, false, LN_USBD_EPxCS_RX_STA_NAK);
            _usbInstance->setEpStatus(0, true, LN_USBD_EPxCS_TX_STA_NAK);
            _usbInstance->copyFromSRAM(userMemBuf, EndPoints::getBTable(ep)->RxAdr, 8);
            dcd_event_setup_received(0, (uint8_t *)userMemBuf, true);
        }
    }
    else
    {
        // Clear RX CTR interrupt flag
        if (ep)
        {
            _usbInstance->clearTxRx(ep, false);
        }

        if (count != 0U)
        {
            _usbInstance->copyFromSRAM((uint8_t *)&(xfer->buffer[xfer->queued_len]),
                                       EndPoints::getBTable(ep)->RxAdr /* *pcd_ep_rx_address_ptr(EPindex)*/, count);
            xfer->queued_len = (uint16_t)(xfer->queued_len + count);
        }

        if (((int)count < xfer->max_packet_size) || (xfer->queued_len == xfer->total_len))
        {
            /* RX COMPLETE */
            dcd_event_xfer_complete(0, ep, xfer->queued_len, XFER_RESULT_SUCCESS, true);
            // Though the host could still send, we don't know.
            // Does the bulk pipe need to be reset to valid to allow for a ZLP?
        }
        else
        {
            uint32_t remaining = (uint32_t)xfer->total_len - (uint32_t)xfer->queued_len;
            int sz = remaining;
            if (sz > xfer->max_packet_size)
                sz = xfer->max_packet_size;
            EndPoints::setRxBufferSize(ep, sz);
            _usbInstance->setEpStatus(ep, false, LN_USBD_EPxCS_RX_STA_VALID);
        }
    }

    // For EP0, prepare to receive another SETUP packet.
    // Clear CTR last so that a new packet does not overwrite the packing being read.
    // (Based on the docs, it seems SETUP will always be accepted after CTR is cleared)
    if (!ep)
    {
        // Always be prepared for a status packet...
        EndPoints::setRxBufferSize(ep, CFG_TUD_ENDPOINT0_SIZE);
        _usbInstance->clearTxRx(ep, false);
    }
}

/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void dcd_edpt0_status_complete(uint8_t rhport, tusb_control_request_t const *request)
{
    (void)rhport;

    if (request->bmRequestType_bit.recipient == TUSB_REQ_RCPT_DEVICE &&
        request->bmRequestType_bit.type == TUSB_REQ_TYPE_STANDARD && request->bRequest == TUSB_REQ_SET_ADDRESS)
    {
        // Setting new address after the whole request is complete
        _usbInstance->setAddress(request->wValue);
    }
}

// The STM32F0 doesn't seem to like |= or &= to manipulate the EP#R registers,
// so I'm using the #define from HAL here, instead.

bool dcd_edpt_open(uint8_t rhport, tusb_desc_endpoint_t const *p_endpoint_desc)
{
    (void)rhport;
    uint8_t const epnum = tu_edpt_number(p_endpoint_desc->bEndpointAddress);
    uint8_t const dir = tu_edpt_dir(p_endpoint_desc->bEndpointAddress);
    const uint16_t epMaxPktSize = tu_edpt_packet_size(p_endpoint_desc);
    uint16_t pma_addr;
    uint32_t wType;

    // Isochronous not supported (yet), and some other driver assumptions.
    TU_ASSERT(p_endpoint_desc->bmAttributes.xfer != TUSB_XFER_ISOCHRONOUS);
    TU_ASSERT(epnum < (MAX_EP_COUNT + 2)); // ep0 is pre-reserved

    // Set type
    switch (p_endpoint_desc->bmAttributes.xfer)
    {
    case TUSB_XFER_CONTROL:
        wType = LN_USBD_EPxCS_EPCTL_CONTROL;
        break;
#if (0)
    case TUSB_XFER_ISOCHRONOUS: // FIXME: Not yet supported
        wType = LN_USBD_EPxCS_EPCTL_ISO;
        break;
#endif

    case TUSB_XFER_BULK:
        wType = LN_USBD_EPxCS_EPCTL_CONTROL;
        break;

    case TUSB_XFER_INTERRUPT:
        wType = LN_USBD_EPxCS_EPCTL_INTERRUPT;
        break;

    default:
        TU_ASSERT(false);
    }
    _usbInstance->setEpType(epnum, wType);
    _usbInstance->setEpAddress(epnum, epnum);

    // Be normal, for now, instead of only accepting zero-byte packets (on control endpoint)
    // or being double-buffered (bulk endpoints)
    _usbInstance->setEpKind(0, false);

    pma_addr = EndPoints::initEpRam(p_endpoint_desc->bEndpointAddress, epMaxPktSize);
    if (dir == TUSB_DIR_IN)
    {
        EndPoints::setBlock(epnum, true, pma_addr, epMaxPktSize);
        _usbInstance->clearDTG(epnum, true);
        _usbInstance->setEpStatus(epnum, true, LN_USBD_EPxCS_TX_STA_NAK);
    }
    else
    {
        EndPoints::getBTable(epnum)->RxAdr = pma_addr;
        EndPoints::setRxBufferSize(epnum, epMaxPktSize);
        _usbInstance->clearDTG(epnum, false);
        _usbInstance->setEpStatus(epnum, false, LN_USBD_EPxCS_RX_STA_NAK);
    }

    EndPoints::getDescriptor(epnum, dir)->max_packet_size = epMaxPktSize;
    return true;
}

/**
 * Close an endpoint.
 *
 * This function may be called with interrupts enabled or disabled.
 *
 * This also clears transfers in progress, should there be any.
 */
void dcd_edpt_close(uint8_t rhport, uint8_t ep_addr)
{
    (void)rhport;
    uint32_t const epnum = tu_edpt_number(ep_addr);
    uint32_t const dir = tu_edpt_dir(ep_addr);

    if (dir == TUSB_DIR_IN)
    {
        _usbInstance->setEpStatus(epnum, true, LN_USBD_EPxCS_TX_STA_DISABLED);
    }
    else
    {
        _usbInstance->setEpStatus(epnum, false, LN_USBD_EPxCS_RX_STA_DISABLED);
    }
    EndPoints::freeEndpoint(ep_addr);
}

// Currently, single-buffered, and only 64 bytes at a time (max)

void dcd_transmit_packet(EndPoints::xfer_descriptor *xfer, uint16_t ep_ix)
{
    uint16_t len = (uint16_t)(xfer->total_len - xfer->queued_len);

    if (len > xfer->max_packet_size) // max packet size for FS transfer
    {
        len = xfer->max_packet_size;
    }
    uint16_t oldAddr = EndPoints::getBTable(ep_ix)->TxAdr; //*pcd_ep_tx_address_ptr(ep_ix);
    _usbInstance->copyToSRAM(oldAddr, (uint8_t *)&(xfer->buffer[xfer->queued_len]), len);
    xfer->queued_len = (uint16_t)(xfer->queued_len + len);
    EndPoints::getBTable(ep_ix)->TxSize = len;
    // pcd_set_ep_tx_cnt(ep_ix,len);
    _usbInstance->setEpStatus(ep_ix, true, LN_USBD_EPxCS_TX_STA_VALID);
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

bool dcd_edpt_xfer(uint8_t rhport, uint8_t ep_addr, uint8_t *buffer, uint16_t total_bytes)
{
    (void)rhport;

    uint8_t const epnum = tu_edpt_number(ep_addr);
    uint8_t const dir = tu_edpt_dir(ep_addr);

    EndPoints::xfer_descriptor *xfer = EndPoints::getDescriptor(epnum, dir);

    xfer->buffer = buffer;
    // xfer->ff     = NULL; // TODO support dcd_edpt_xfer_fifo API
    xfer->total_len = total_bytes;
    xfer->queued_len = 0;

    if (dir == TUSB_DIR_OUT)
    {
        // A setup token can occur immediately after an OUT STATUS packet so make sure we have a valid
        // buffer for the control endpoint.
        if (epnum == 0 && buffer == NULL)
        {
            xfer->buffer = (uint8_t *)_setup_packet;
        }
        int sz = total_bytes;
        if (sz > xfer->max_packet_size)
            sz = xfer->max_packet_size;
        EndPoints::setRxBufferSize(epnum, sz);
        _usbInstance->setEpStatus(epnum, false, LN_USBD_EPxCS_RX_STA_VALID);
    }
    else // IN
    {
        dcd_transmit_packet(xfer, epnum);
    }
    return true;
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void dcd_edpt_stall(uint8_t rhport, uint8_t ep_addr)
{
    (void)rhport;
    if (ep_addr & 0x80)
        _usbInstance->setEpStatus(ep_addr & 0x7f, true, LN_USBD_EPxCS_TX_STA_STALL);
    else
        _usbInstance->setEpStatus(ep_addr, false, LN_USBD_EPxCS_RX_STA_STALL);
}
/**
 * @brief <brief>
 * @param [in] <name> <parameter_description>
 * @return <return_description>
 * @details <details>
 */

void dcd_edpt_clear_stall(uint8_t rhport, uint8_t ep_addr)
{
    (void)rhport;

    if (ep_addr & 0x80)
    { // IN
        ep_addr &= 0x7F;
        /* Reset to DATA0 if clearing stall condition. */
        _usbInstance->setEpStatus(ep_addr & 0x7f, true, LN_USBD_EPxCS_TX_STA_NAK);
        _usbInstance->clearDTG(ep_addr, true);
    }
    else
    { // OUT
        /* Reset to DATA0 if clearing stall condition. */
        _usbInstance->clearDTG(ep_addr, false);
        _usbInstance->setEpStatus(ep_addr, false, LN_USBD_EPxCS_RX_STA_NAK);
    }
}
void dcd_sof_enable(uint8_t rhport, bool en)
{
    (void)rhport;
    (void)en;

    // TODO implement later
}

// EOF
