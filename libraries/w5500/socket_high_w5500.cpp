/**
 * @file    socket_high_w5500.cpp
 * @brief   High-level W5500 socket API — implementation.
 *
 * Implements socket(), listen(), close(), disconnect() using
 * shadow5Socket register access.  No ioLibrary_Driver dependency.
 *
 * @copyright (C) 2025
 * @license  See license file
 */

#include "socket_high_w5500.h"
#include "W5500/w5500_regs.h"
#include "lnDebug.h"
#include "lnSystemTime.h"
#include "lnFreeRTOS.h"
#include "lowlevel_w5500_helper.h"

// Global variables (previously defined in ioLibrary_Driver socket.c)

/** @brief Bitmask of sockets currently busy sending. */
volatile uint16_t sock_is_sending = 0;

/** @brief Bitmask of sockets in non-blocking (async) I/O mode. */
volatile uint16_t sock_io_mode = 0;

// Timeout (ms) for waiting on Sn_CR to clear and Sn_SR transitions

/** @brief Max time to wait for a command to be accepted (Sn_CR clears). */
static const uint32_t CR_TIMEOUT_MS = 100;

/** @brief Max time to wait for Sn_SR to leave SOCK_CLOSED after OPEN. */
static const uint32_t OPEN_TIMEOUT_MS = 100;

/** @brief Max time to wait for Sn_SR to reach SOCK_LISTEN after LISTEN. */
static const uint32_t LISTEN_TIMEOUT_MS = 100;

// Helper: wait for Sn_CR to clear (command accepted by hardware)

/**
 * @brief Busy-wait until Sn_CR returns to 0x00.
 * @param sn  Socket number.
 * @return true if CR cleared within timeout, false on timeout.
 */
static bool wait_cr_clear(uint8_t sn)
{
    shadow5Socket s(sn);
    uint32_t start = lnGetMs();
    while (s.getCommand() != 0)
    {
        if (lnGetMs() - start >= CR_TIMEOUT_MS)
        {
            Logger("socket_high: CR timeout on socket %d\n", sn);
            return false;
        }
    }
    return true;
}

// ---------------------------------------------------------------------------
// socket()
// ---------------------------------------------------------------------------

int8_t socket(uint8_t sn, uint8_t protocol, uint16_t port, uint8_t flag)
{
    // We only support TCP on W5500
    if (protocol != Sn_MR_TCP)
    {
        Logger("socket_high: only TCP supported (protocol=0x%02x)\n", protocol);
        return SOCK_FATAL;
    }

    shadow5Socket s(sn);

    // 1. Set mode register: protocol | upper flag bits
    s.setMode(protocol | (flag & 0xF0));

    // 2. Set port (big-endian)
    s.setLocalPort(port);

    // 3. Clear any pending interrupts
    s.clearPendingInterrupt(0xFF);

    // 4. Issue OPEN command
    s.setCommand(Sn_CR_OPEN);
    if (!wait_cr_clear(sn))
    {
        Logger("socket_high: OPEN command failed on socket %d\n", sn);
        return SOCK_FATAL;
    }

    // 5. Wait for socket to leave SOCK_CLOSED
    uint32_t start = lnGetMs();
    while (s.getStatus() == SOCK_CLOSED)
    {
        if (lnGetMs() - start >= OPEN_TIMEOUT_MS)
        {
            Logger("socket_high: socket %d failed to open (stayed SOCK_CLOSED)\n", sn);
            return SOCK_FATAL;
        }
    }

    // 6. Clear sock_io_mode and sock_is_sending bits (global variables
    //    used by shadow5Socket::writeData — must be initialised)
    sock_io_mode &= ~(1 << sn);
    sock_io_mode |= ((flag & SF_IO_NONBLOCK) << sn);
    s.clearTxSending();

    // enable interrupt for that socket
    shadow5System sys;
    uint8_t mask = sys.getSocketInterruptMask();
    mask |= (1 << sn);
    sys.setSocketInterruptMask(mask);

    return (int8_t)sn;
}

// ---------------------------------------------------------------------------
// listen()
// ---------------------------------------------------------------------------

int8_t listen(uint8_t sn)
{
    shadow5Socket s(sn);

    // Issue LISTEN command
    s.setCommand(Sn_CR_LISTEN);
    if (!wait_cr_clear(sn))
    {
        Logger("socket_high: LISTEN command failed on socket %d\n", sn);
        return SOCK_FATAL;
    }

    // Wait for SOCK_LISTEN or SOCK_ESTABLISHED (client may connect fast)
    uint32_t start = lnGetMs();
    while (true)
    {
        uint8_t status = s.getStatus();
        if (status == SOCK_LISTEN || status == SOCK_ESTABLISHED)
        {
            break;
        }
        if (status == SOCK_CLOSED)
        {
            Logger("socket_high: socket %d closed during LISTEN\n", sn);
            return SOCK_FATAL;
        }
        if (lnGetMs() - start >= LISTEN_TIMEOUT_MS)
        {
            Logger("socket_high: LISTEN timeout on socket %d (status=0x%02x)\n", sn, status);
            return SOCK_FATAL;
        }
    }

    return SOCK_OK;
}

// ---------------------------------------------------------------------------
// close()
// ---------------------------------------------------------------------------

int8_t close(uint8_t sn)
{
    shadow5Socket s(sn);

    // Issue CLOSE command
    s.setCommand(Sn_CR_CLOSE);
    if (!wait_cr_clear(sn))
    {
        Logger("socket_high: CLOSE command failed on socket %d\n", sn);
        return SOCK_FATAL;
    }
    shadow5System sys;
    uint8_t mask = sys.getSocketInterruptMask();
    mask &= ~(1 << sn);
    sys.setSocketInterruptMask(mask);

    return SOCK_OK;
}

// ---------------------------------------------------------------------------
// disconnect()
// ---------------------------------------------------------------------------

int8_t disconnect(uint8_t sn)
{
    shadow5Socket s(sn);

    // Issue DISCON command
    s.setCommand(Sn_CR_DISCON);
    if (!wait_cr_clear(sn))
    {
        Logger("socket_high: DISCON command failed on socket %d\n", sn);
        return SOCK_FATAL;
    }

    return SOCK_OK;
}
