/**
 * @file    socket_high_w5500.h
 * @brief   High-level W5500 socket API — replacement for ioLibrary socket.h.
 *
 * Provides socket(), listen(), close(), disconnect() using the
 * shadow5Socket register-access wrapper (lowlevel_w5500_helper.h).
 * No ioLibrary_Driver dependency.
 *
 * All functions operate on W5500 hardware socket numbers (0-7).
 * Return values follow ioLibrary convention:
 *   - socket() returns the socket number on success, negative on error.
 *   - listen()/close()/disconnect() return 1 (SOCK_OK) on success,
 *     negative on error.
 *
 * @copyright (C) 2025
 * @license  See license file
 */

#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C"
{
#endif

    // ---------------------------------------------------------------------------
    // Socket status constants (from W5500 Sn_SR register)
    // ---------------------------------------------------------------------------

#define SOCK_CLOSED 0x00
#define SOCK_INIT 0x13
#define SOCK_LISTEN 0x14
#define SOCK_ESTABLISHED 0x17
#define SOCK_CLOSE_WAIT 0x1C

    // ---------------------------------------------------------------------------
    // Return value constants
    // ---------------------------------------------------------------------------

#define SOCK_OK 1
#define SOCK_BUSY 0
#define SOCK_ERROR 0
#define SOCK_FATAL (-1)

    // ---------------------------------------------------------------------------
    // Flag constants (for socket() flag parameter)
    // ---------------------------------------------------------------------------

#define SF_IO_NONBLOCK 0x01 /**< Non-blocking I/O mode */

    // ---------------------------------------------------------------------------
    // API functions
    // ---------------------------------------------------------------------------

    /**
     * @brief Open a TCP socket and bind to a port.
     *
     * Sets Sn_MR to TCP, writes the port, issues Sn_CR_OPEN, and waits
     * for the socket to leave SOCK_CLOSED state.
     *
     * @param sn       Hardware socket number (0-7).
     * @param protocol Sn_MR_TCP (only TCP is supported).
     * @param port     Local TCP port to bind.
     * @param flag     Flags (SF_IO_NONBLOCK or 0).
     * @return sn on success, negative on error.
     */
    int8_t socket(uint8_t sn, uint8_t protocol, uint16_t port, uint8_t flag);

    /**
     * @brief Start listening for incoming TCP connections.
     *
     * Issues Sn_CR_LISTEN and waits for Sn_SR to reach SOCK_LISTEN.
     *
     * @param sn  Hardware socket number.
     * @return SOCK_OK on success, negative on error.
     */
    int8_t listen(uint8_t sn);

    /**
     * @brief Close a socket.
     *
     * Issues Sn_CR_CLOSE.
     *
     * @param sn  Hardware socket number.
     * @return SOCK_OK on success, negative on error.
     */
    int8_t close(uint8_t sn);

    /**
     * @brief Send a TCP FIN to the peer.
     *
     * Issues Sn_CR_DISCON.
     *
     * @param sn  Hardware socket number.
     * @return SOCK_OK on success, negative on error.
     */
    int8_t disconnect(uint8_t sn);

#ifdef __cplusplus
}
#endif