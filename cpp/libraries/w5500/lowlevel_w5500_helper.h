/**
 * @file    lowlevel_w5500_helper.h
 * @brief   W5500 register access helpers and shadow5Socket wrapper.
 *
 * Provides low-level SPI register read/write functions (write_to_reg,
 * read_from_reg) and the shadow5Socket C++ wrapper class that gives
 * inline access to W5500 socket registers (Sn_MR, Sn_CR, Sn_SR, Sn_IR,
 * Sn_IMR, Sn_TX_FSR, Sn_RX_RSR, etc.) via the SPI functions.
 *
 * The shadow5Socket class is used internally by lnSocket_W5500 and
 * W5500LowLevel to avoid the overhead and bugs of the full
 * ioLibrary_Driver socket API.  It includes double-read consistency
 * checks for 16-bit registers that the W5500 hardware may update
 * asynchronously (Sn_TX_FSR, Sn_RX_RSR, Sn_TX_WR, Sn_RX_RD).
 *
 * @copyright (C) 2025
 * @license  See license file
 */
#pragma once

#include "lnGPIO.h"
#include "lnLWIP.h"
#include "lnFreeRTOS.h"
//
#include "W5500/w5500_regs.h"
#include "socket_high_w5500.h"
//
#include <stdbool.h>
#include <stdint.h>

// ---------------------------------------------------------------------------
// W5500 TX/RX buffer block address calculation
// (provided by w5500_regs.h: WIZCHIP_TXBUF_BLOCK / WIZCHIP_RXBUF_BLOCK)
// ---------------------------------------------------------------------------

#if 1
#define HDEBUG(...)                                                                                                    \
    {                                                                                                                  \
    }
#else
#define HDEBUG Logger
#endif

/**
 * @brief Write data to a W5500 register via SPI.
 * @param adr    Register address (24-bit, including block select).
 * @param txlen  Number of bytes to write.
 * @param txdata Pointer to the data buffer.
 * @return true on success.
 */
bool write_to_reg(uint32_t adr, uint32_t txlen, const uint8_t *txdata);

/**
 * @brief Read data from a W5500 register via SPI.
 * @param adr    Register address (24-bit, including block select).
 * @param rxlen  Number of bytes to read.
 * @param rxdata Pointer to the destination buffer.
 * @return true on success.
 */
bool read_from_reg(uint32_t adr, uint32_t rxlen, uint8_t *rxdata);

/**
 * @brief Bitmask of sockets currently busy sending.
 *
 * Bit n is set when socket n has issued a SEND command and is waiting
 * for the SEND_OK interrupt.
 */
extern volatile uint16_t sock_is_sending;

/**
 * @brief Bitmask of sockets in non-blocking (async) I/O mode.
 *
 * Bit n is set when socket n should return immediately rather than
 * spin-waiting for TX buffer space.
 */
extern volatile uint16_t sock_io_mode;

/**
 * @brief Lightweight C++ wrapper around the W5500 system (common) register set.
 *
 * Provides inline read/write access to the W5500 common registers
 * (MR, IR, SIR, SIMR, etc.) via the low-level SPI functions
 * write_to_reg() / read_from_reg().
 *
 * Used internally by lnTaskW5500 to read the global SIR register
 * and discover which sockets have pending interrupts.
 */
class shadow5System
{
  protected:
    /**
     * @brief Write a single byte to a W5500 register.
     * @param reg  24-bit register address.
     * @param data Byte value to write.
     */
    void write8(uint32_t reg, uint8_t data)
    {
        uint8_t x[1] = {data};
        write_to_reg(reg, 1, x);
    }

    /**
     * @brief Read a single byte from a W5500 register.
     * @param reg  24-bit register address.
     * @return The byte value read.
     */
    uint8_t read8(uint32_t reg)
    {
        uint8_t x[1] = {0};
        read_from_reg(reg, 1, x);
        return x[0];
    }

  public:
    shadow5System()
    {
    }
    ~shadow5System()
    {
    }

    /**
     * @brief Read the global Socket Interrupt Register (SIR).
     *
     * SIR is an 8-bit register where bit n is set when socket n
     * has a pending Sn_IR interrupt.
     *
     * @return Bitmask of sockets with pending interrupts.
     */
    uint8_t getPendingInterrupts()
    {
        return read8(SIR);
    }

    // ---- Common register typed helpers ----

    /**
     * @brief Set the source MAC address (SHAR, 6 bytes).
     */
    void setMac(const uint8_t mac[6])
    {
        write_to_reg(SHAR, 6, mac);
    }

    /**
     * @brief Get the source MAC address (SHAR, 6 bytes).
     */
    void getMac(uint8_t mac[6])
    {
        read_from_reg(SHAR, 6, mac);
    }

    /**
     * @brief Set the source IP address (SIPR, 4 bytes).
     */
    void setSourceIp(const uint8_t ip[4])
    {
        write_to_reg(SIPR, 4, ip);
    }

    /**
     * @brief Get the source IP address (SIPR, 4 bytes).
     */
    void getSourceIp(uint8_t ip[4])
    {
        read_from_reg(SIPR, 4, ip);
    }

    /**
     * @brief Set the gateway IP address (GAR, 4 bytes).
     */
    void setGateway(const uint8_t ip[4])
    {
        write_to_reg(GAR, 4, ip);
    }

    /**
     * @brief Set the subnet mask (SUBR, 4 bytes).
     */
    void setSubnet(const uint8_t ip[4])
    {
        write_to_reg(SUBR, 4, ip);
    }

    /**
     * @brief Set the global socket interrupt mask (SIMR).
     */
    void setSocketInterruptMask(uint8_t mask)
    {
        write_to_reg(SIMR, 1, &mask);
    }

    /**
     * @brief Get the global socket interrupt mask (SIMR).
     */
    uint8_t getSocketInterruptMask()
    {
        uint8_t mask = 0;
        read_from_reg(SIMR, 1, &mask);
        return mask;
    }

    /**
     * @brief Set the retransmission timeout (RTR, 100us units).
     */
    void setRetransmissionTimeout(uint16_t timeout)
    {
        uint8_t buf[2] = {(uint8_t)(timeout >> 8), (uint8_t)(timeout & 0xFF)};
        write_to_reg(_RTR_, 2, buf);
    }

    /**
     * @brief Set the retry count (RCR).
     */
    void setRetryCount(uint8_t count)
    {
        write_to_reg(_RCR_, 1, &count);
    }
};

/**
 * @brief Lightweight C++ wrapper around a W5500 hardware socket register set.
 *
 * Provides inline read/write access to the W5500 socket registers
 * (Sn_MR, Sn_CR, Sn_SR, Sn_IR, Sn_IMR, Sn_TX_FSR, Sn_RX_RSR, etc.)
 * via the low-level SPI functions write_to_reg() / read_from_reg().
 *
 * Used internally by lnSocket_W5500 and W5500LowLevel to avoid
 * the overhead and bugs of the full ioLibrary_Driver socket API.
 */
class shadow5Socket
{
  public:
    /**
     * @brief Read a 16-bit register value with double-read consistency check.
     *
     * Reads both bytes in a single SPI burst (CS held low for both bytes),
     * then re-reads and verifies the values match.  The W5500 does not latch
     * the register at the start of the transaction, so the hardware can still
     * update the counter between the two byte-shifts within the same burst.
     * The double-read guards against this intra-transaction tear.
     * @param r  Base register address (high-byte offset).
     * @return Stable 16-bit register value.
     */
    inline uint32_t getStable16bits(uint32_t r)
    {
        uint8_t buf1[2], buf2[2];
        uint32_t val = 0, val1 = 0;
        do
        {
            read_from_reg(r, 2, buf1);
            val1 = ((uint32_t)buf1[0] << 8) | buf1[1];
            if (val1 != 0)
            {
                read_from_reg(r, 2, buf2);
                val = ((uint32_t)buf2[0] << 8) | buf2[1];
            }
        } while (val != val1);
        return val;
    }

    /**
     * @brief Write a 16-bit value to a register pair (big-endian) in a single SPI burst.
     * @param r    Base register address (high-byte offset).
     * @param val  16-bit value to write.
     */
    inline void setStable16bits(uint32_t r, uint32_t val)
    {
        uint8_t buf[2] = {(uint8_t)(val >> 8), (uint8_t)(val & 0xff)};
        write_to_reg(r, 2, buf);
    }

    /**
     * @brief Construct a shadow5Socket wrapper for a given hardware socket.
     * @param s  W5500 hardware socket number (0-7).
     */
    shadow5Socket(uint32_t s) : _sn(s)
    {
    }

    /** @brief Destructor (no-op). */
    ~shadow5Socket()
    {
    }

    /**
     * @brief Set the global sock_is_sending bit for this socket under critical section.
     */
    inline void setTxSending()
    {
        taskENTER_CRITICAL();
        sock_is_sending |= (1 << _sn);
        taskEXIT_CRITICAL();
    }

    /**
     * @brief Clear the global sock_is_sending bit for this socket under critical section.
     */
    inline void clearTxSending()
    {
        taskENTER_CRITICAL();
        sock_is_sending &= ~(1 << _sn);
        taskEXIT_CRITICAL();
    }

    /**
     * @brief Check if the global sock_is_sending bit is set for this socket.
     * @return true if sending, false otherwise.
     */
    inline bool isTxSending() const
    {
        return (sock_is_sending & (1 << _sn)) != 0;
    }

    /**
     * @brief Write a single byte to a W5500 register.
     * @param reg  24-bit register address.
     * @param data Byte value to write.
     */
    void write8(uint32_t reg, uint8_t data)
    {
        uint8_t x[1] = {data};
        write_to_reg(reg, 1, x);
    }

    /**
     * @brief Read a single byte from a W5500 register.
     * @param reg  24-bit register address.
     * @return The byte value read.
     */
    uint8_t read8(uint32_t reg)
    {
        uint8_t x[1] = {0};
        read_from_reg(reg, 1, x);
        return x[0];
    }

    // ---- Socket register typed helpers ----

    /**
     * @brief Set the socket mode register (Sn_MR).
     * @param mode  Mode value (Sn_MR_TCP, Sn_MR_UDP, etc.).
     */
    void setMode(uint8_t mode)
    {
        write8(Sn_MR(_sn), mode);
    }

    /**
     * @brief Get the socket mode register (Sn_MR).
     * @return Current mode value.
     */
    uint8_t getMode()
    {
        return read8(Sn_MR(_sn));
    }

    /**
     * @brief Issue a command to the socket (Sn_CR).
     * Writes the command byte and busy-waits until the hardware clears it.
     * @param cmd  Command byte (Sn_CR_OPEN, Sn_CR_SEND, Sn_CR_RECV, etc.).
     */
    void setCommand(uint8_t cmd)
    {
        write8(Sn_CR(_sn), cmd);
        while (read8(Sn_CR(_sn)))
        {
            __asm__("nop");
        }
    }

    /**
     * @brief Get the current command register value (Sn_CR).
     * @return Current command byte (0 if idle).
     */
    uint8_t getCommand()
    {
        return read8(Sn_CR(_sn));
    }

    /**
     * @brief Set the local source port (Sn_PORT, 16-bit big-endian).
     * @param port  Port number.
     */
    void setLocalPort(uint16_t port)
    {
        setStable16bits(Sn_PORT(_sn), port);
    }

    /**
     * @brief Get the local source port (Sn_PORT).
     * @return Port number.
     */
    uint16_t getLocalPort()
    {
        return (uint16_t)getStable16bits(Sn_PORT(_sn));
    }

    /**
     * @brief Set the destination port (Sn_DPORT, 16-bit big-endian).
     * @param port  Destination port number.
     */
    void setDestPort(uint16_t port)
    {
        setStable16bits(Sn_DPORT(_sn), port);
    }

    /**
     * @brief Get the destination port (Sn_DPORT).
     * @return Destination port number.
     */
    uint16_t getDestPort()
    {
        return (uint16_t)getStable16bits(Sn_DPORT(_sn));
    }

    /**
     * @brief Set the destination IP address (Sn_DIPR, 4 bytes).
     * @param ip  IP address (4 bytes).
     */
    void setDestIp(const uint8_t ip[4])
    {
        writeN(Sn_DIPR(_sn), 4, ip);
    }

    /**
     * @brief Get the destination IP address (Sn_DIPR, 4 bytes).
     * @param ip  Buffer to receive the IP address (4 bytes).
     */
    void getDestIp(uint8_t ip[4])
    {
        readN(Sn_DIPR(_sn), 4, ip);
    }

    /**
     * @brief Set the destination MAC address (Sn_DHAR, 6 bytes).
     * @param mac  MAC address (6 bytes).
     */
    void setDestMac(const uint8_t mac[6])
    {
        writeN(Sn_DHAR(_sn), 6, mac);
    }

    /**
     * @brief Get the destination MAC address (Sn_DHAR, 6 bytes).
     * @param mac  Buffer to receive the MAC address (6 bytes).
     */
    void getDestMac(uint8_t mac[6])
    {
        readN(Sn_DHAR(_sn), 6, mac);
    }

    /**
     * @brief Set the socket interrupt mask (Sn_IMR).
     * @param mask  Interrupt mask bits to enable.
     */
    void setSocketInterruptMask(uint8_t mask)
    {
        write8(Sn_IMR(_sn), mask);
    }

    /**
     * @brief Get the current socket interrupt mask (Sn_IMR).
     * @return Current interrupt mask.
     */
    uint8_t getSocketInterruptMask()
    {
        return read8(Sn_IMR(_sn));
    }

    /**
     * @brief Get the free TX buffer size (Sn_TX_FSR).
     * @return Number of bytes available in the TX buffer.
     */
    uint32_t getTxFreeBufferSize()
    {
        return getStable16bits(Sn_TX_FSR(_sn));
    }

    /**
     * @brief Get the TX write pointer (Sn_TX_WR).
     * @return Current TX write pointer value.
     */
    uint32_t getTxPointer()
    {
        return getStable16bits(Sn_TX_WR(_sn));
    }

    /**
     * @brief Set the TX write pointer (Sn_TX_WR).
     * @param ptr  New TX write pointer value.
     */
    void setTxPointer(uint32_t ptr)
    {
        setStable16bits(Sn_TX_WR(_sn), ptr);
    }

    /**
     * @brief Get the number of bytes received and ready to read (Sn_RX_RSR).
     * @return Number of bytes available in the RX buffer.
     */
    uint32_t getReadAvailable()
    {
        return getStable16bits(Sn_RX_RSR(_sn));
    }

    /**
     * @brief Get the RX read pointer (Sn_RX_RD).
     * @return Current RX read pointer value.
     */
    uint32_t getReadPointer()
    {
        return getStable16bits(Sn_RX_RD(_sn));
    }

    /**
     * @brief Set the RX read pointer (Sn_RX_RD).
     * @param ptr  New RX read pointer value.
     */
    void setReadPointer(uint32_t ptr)
    {
        return setStable16bits(Sn_RX_RD(_sn), ptr);
    }

    /**
     * @brief Get the socket status register (Sn_SR).
     * @return Socket status (SOCK_CLOSED, SOCK_ESTABLISHED, etc.).
     */
    uint8_t getStatus()
    {
        return read8(Sn_SR(_sn));
    }

    /**
     * @brief Issue a command to the socket (Sn_CR) and wait for acceptance.
     *
     * Writes the command byte to Sn_CR and busy-waits until the hardware
     * clears the register, indicating the command has been accepted.
     * @param cmd  Command byte (Sn_CR_OPEN, Sn_CR_SEND, Sn_CR_RECV, etc.).
     */
    void sendCommand(uint8_t cmd)
    {
        write8(Sn_CR(_sn), cmd);
        while (read8(Sn_CR(_sn))) // the CR register will go back to zero
                                  // when the command is accepted
        {
            __asm__("nop");
        }
    }

    /**
     * @brief Get the socket status register (Sn_SR).
     * @return Socket status.
     * @deprecated Use getStatus() instead.
     */
    uint8_t getStatusRegister()
    {
        return read8(Sn_SR(_sn));
    }

    /**
     * @brief Get the pending interrupt flags (Sn_IR).
     * @return Bitmask of pending interrupts.
     */
    uint8_t getInterruptPending()
    {
        return read8(Sn_IR(_sn));
    }

    /**
     * @brief Clear specific interrupt flags (Sn_IR).
     * @param i  Bitmask of interrupts to clear.
     */
    void clearPendingInterrupt(uint8_t i)
    {
        write8(Sn_IR(_sn), i);
    }

    /**
     * @brief Get the maximum TX buffer size for this socket.
     * @return Maximum TX buffer size in bytes (Sn_TXBUF_SIZE << 10).
     */
    uint32_t getMaxTxSize()
    {
        return read8(Sn_TXBUF_SIZE(_sn)) << 10;
    }

    /**
     * @brief Read received data from the W5500 RX buffer.
     *
     * Reads @p len bytes from the W5500 internal RX memory into @p buffer
     * and advances the RX read pointer (Sn_RX_RD).
     * @param len     Number of bytes to read.
     * @param buffer  Destination buffer to fill with received data.
     */
    void readData(uint32_t len, uint8_t *buffer)
    {
        uint16_t ptr = 0;
        uint32_t addrsel = 0;

        if (len == 0)
        {
            return;
        }
        ptr = getReadPointer();
        // M20140501 : implict type casting -> explict type casting
        // addrsel = ((ptr << 8) + (WIZCHIP_RXBUF_BLOCK(sn) << 3);
        addrsel = ((uint32_t)ptr << 8) + (WIZCHIP_RXBUF_BLOCK(_sn) << 3);
        //
        read_from_reg(addrsel, len, buffer);
        ptr += len;

        setReadPointer(ptr);

        // Issue RECV command to release the buffer space.  The W5500's
        // internal RX read pointer is only advanced when the RECV command
        // is issued — writing Sn_RX_RD alone is not sufficient.  Without
        // this, a subsequent readData() would read from the same position
        // as the previous one, causing data corruption.
        sendCommand(Sn_CR_RECV);
    }

    /**
     * @brief Advance the RX read pointer without reading data (discard).
     *
     * Skips @p len bytes in the W5500 RX buffer by advancing Sn_RX_RD
     * without performing any SPI data reads.  Used to discard received
     * data, e.g. after a timeout or when the application wants to flush
     * the RX buffer.
     *
     * @param len  Number of bytes to skip.
     */
    void skipData(uint32_t len)
    {
        uint16_t ptr = 0;
        uint32_t addrsel = 0;

        if (len == 0)
        {
            return;
        }
        ptr = getReadPointer();
        ptr += len;

        setReadPointer(ptr);

        // Issue RECV command to release the buffer space (same rationale
        // as in readData()).
        sendCommand(Sn_CR_RECV);
    }

    /**
     * @brief Write data to the W5500 TX buffer and issue SEND command.
     *
     * Handles the full send sequence:
     * - Checks socket is in ESTABLISHED or CLOSE_WAIT state.
     * - Waits for previous SEND_OK if sock_is_sending is set.
     * - Waits for TX buffer space (respects sock_io_mode for non-blocking).
     * - Copies data directly to the W5500 TX memory via write_to_reg()
     *   (bypassing wiz_send_data()) and issues Sn_CR_SEND.
     *
     * @param len  Number of bytes to send.
     * @param buf  Pointer to the data to send (const-qualified).
     * @return Number of bytes sent on success, 0 if busy, negative on error.
     */
    int32_t writeData(uint32_t len, const uint8_t *buf)
    {
        uint8_t tmp = 0;
        uint32_t freesize = 0;
        bool had_sendok = false;

        // Optimization 1: Skip getStatusRegister() check since lnSocket_W5500::write()
        // already guards the write call by verifying socket is in StateConnected.

        if (isTxSending())
        {
            tmp = getInterruptPending();
            if (tmp & Sn_IR_SENDOK)
            {
                // Don't clear SEND_OK yet — we need to verify the TX buffer
                // actually has space.  If we clear SEND_OK but
                // getTxFreeBufferSize() still returns 0 (hardware hasn't
                // updated Sn_TX_FSR yet), we'd deadlock because no more
                // SEND_OK will fire.  Just clear sock_is_sending so the
                // freesize check below can proceed.
                had_sendok = true;
                clearTxSending();
            }
            else if (tmp & Sn_IR_TIMEOUT)
            {
                HDEBUG("Hlp: timeout \n");
                close(_sn);
                return -1;
            }
            else
            {
                return 0; // still busy...
            }
        }
        HDEBUG("Hlp: got clear\n");
        freesize = getTxFreeBufferSize();
        if (len > freesize)
            len = freesize;
        if (len == 0)
        {
            HDEBUG("Hlp: FULL \n");
            // Sn_IR.  SEND_OK is still pending in Sn_IR (we didn't clear
            // it), so the next call will find it and try again.
            setTxSending();
            return 0;
        }
        // TX buffer has space.  Now consume SEND_OK if it was pending.
        // Optimization 2: Reuse cached had_sendok flag to avoid re-reading Sn_IR.
        if (had_sendok)
        {
            clearPendingInterrupt(Sn_IR_SENDOK);
        }
        writeBlock(len, buf);

        // Set sock_is_sending BEFORE issuing SEND to prevent a race:
        // if SEND_OK fires between sendCommand() and the next writeData()
        // call, sock_is_sending must already be set so writeData() checks
        // Sn_IR for SEND_OK instead of returning 0 (deadlocking waitForWrite).
        setTxSending();
        sendCommand(Sn_CR_SEND);
        HDEBUG("Write+Send=%d\n", after1 - before);

        return len;
    }

    /**
     * @brief Write N bytes to a W5500 register (burst).
     * @param reg  24-bit register address.
     * @param len  Number of bytes to write.
     * @param buf  Pointer to the data.
     */
    void writeN(uint32_t reg, uint32_t len, const uint8_t *buf)
    {
        write_to_reg(reg, len, buf);
    }

    /**
     * @brief Read N bytes from a W5500 register (burst).
     * @param reg  24-bit register address.
     * @param len  Number of bytes to read.
     * @param buf  Destination buffer.
     */
    void readN(uint32_t reg, uint32_t len, uint8_t *buf)
    {
        read_from_reg(reg, len, buf);
    }

    /**
     * @brief Write data to the W5500 TX buffer (without issuing SEND).
     *
     * Copies @p len bytes into the W5500 internal TX memory at the current
     * Sn_TX_WR pointer, then advances Sn_TX_WR by @p len.  Does NOT issue
     * the Sn_CR_SEND command — the caller must do that separately.
     *
     * This is a lower-level operation than writeData(); it skips the
     * state checks and SEND_OK wait, making it suitable for use by
     * wiz_send_data() and other internal paths where the caller manages
     * the send state machine.
     *
     * @param len  Number of bytes to write.
     * @param buf  Pointer to the data to write.
     */
    void writeBlock(uint32_t len, const uint8_t *buf)
    {

        uint16_t ptr = 0;
        uint32_t addrsel = 0;

        ptr = getTxPointer();
        // ptr = getSn_TX_WR(sn);
        //  M20140501 : implict type casting -> explict type casting
        //  addrsel = (ptr << 8) + (WIZCHIP_TXBUF_BLOCK(sn) << 3);
        addrsel = ((uint32_t)ptr << 8) + (WIZCHIP_TXBUF_BLOCK(_sn) << 3);
        //
        /// WIZCHIP_WRITE_BUF(addrsel, wizdata, len);
        write_to_reg(addrsel, len, buf);

        ptr += len;
        // setSn_TX_WR(sn, ptr);
        setTxPointer(ptr);
    }

  protected:
    /** @brief W5500 hardware socket number (0-7). */
    uint32_t _sn;
};

// EOF