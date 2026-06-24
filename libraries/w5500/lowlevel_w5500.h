/**
 * @file    lowlevel_w5500.h
 * @brief   W5500 SPI adapter — low-level hardware abstraction.
 *
 * Bridges the ioLibrary_Driver SPI callbacks to the esprit lnSPI API.
 * Provides initialisation, start-up, DHCP, and interrupt-driven socket
 * event dispatch for the W5500 Ethernet controller.
 *
 * The W5500 coordinator task (lnTaskW5500) runs DHCP internally and
 * polls the global SIR register on each INTR pin edge or 100 ms timeout.
 * When a socket has a pending Sn_IR interrupt, dispatchPerSocket()
 * signals the corresponding socketRunner via SocketCustom and
 * SocketWriteAvailable events so that the per-socket task can process
 * the interrupt inline (see lnSocket_W5500::processSocketInterrupt()).
 *
 * Usage:
 * @code
 *   W5500LowLevel::init(0, &pins);   // SPI0 with pin mapping
 *   W5500LowLevel::setMac(mac);
 *   W5500LowLevel::start(callback, arg);  // spawns coordinator task
 * @endcode
 *
 * @copyright (C) 2025
 * @license  See license file
 */
#pragma once

#include "lnGPIO.h"
#include "lnLWIP.h"
//
#include <stdbool.h>
#include <stdint.h>

class lnSocket;

class lnSPI;

/**
 * @brief Pin mapping structure for the W5500.
 *
 * Holds the GPIO pin assignments for all W5500 interface signals.
 */
class lnW5500SPI
{
  public:
    lnPin miso;  /**< MISO (Master-In Slave-Out) pin */
    lnPin mosi;  /**< MOSI (Master-Out Slave-In) pin */
    lnPin clk;   /**< SPI clock pin */
    lnPin cs;    /**< Chip-select pin (active low) */
    lnPin reset; /**< Hardware reset pin (active low) */
    lnPin intr;  /**< Interrupt pin from W5500 */
};

/**
 * @brief Static interface class for the W5500 Ethernet controller.
 *
 * Provides the three essential operations needed to bring up the W5500
 * and check its status.  All members are static because the underlying
 * ioLibrary_Driver uses global callbacks.
 *
 * The API mirrors lnLWIP so that the application can use the same
 * event-driven pattern without any LWIP dependency.
 */
class W5500LowLevel
{
  public:
    /**
     * @brief Initialise the W5500 on the given SPI instance.
     *
     * Configures GPIO pins, creates the SPI peripheral, registers the
     * ioLibrary_Driver callbacks and performs a hardware reset.
     *
     * @param spiInstance  SPI peripheral index (0 = SPI0, 1 = SPI1, etc.)
     * @param pins         Pointer to a @ref lnW5500SPI structure with the
     *                     desired pin assignments.
     * @retval true   Initialisation succeeded.
     * @retval false  Initialisation failed (e.g. SPI creation failed).
     */
    static bool init(int spiInstance, const lnW5500SPI *pins);

    /**
     * @brief Set the MAC address (must be called before start()).
     *
     * @param mac  Pointer to a 6-byte MAC address array.
     */
    static void setMac(const uint8_t mac[6]);

    /**
     * @brief Start the W5500 network stack.
     *
     * Performs a software reset, configures socket buffer sizes (2 KB TX
     * + 2 KB RX per socket), sets the MAC address, configures the PHY
     * for software-controlled auto-negotiation, and spawns a FreeRTOS
     * task that drives DHCP and polls all active sockets.
     *
     * When DHCP obtains an IP address the callback is invoked with
     * LwipReady.  If the link goes down later, LwipDown is fired.
     *
     * @param cb   Callback for network events (lnLwipEvent).
     * @param arg  Opaque argument passed to the callback.
     * @retval true   Start-up succeeded.
     * @retval false  Start-up failed (e.g. wizchip_init error).
     */
    static bool start(lnLwIpSysCallback cb, void *arg);

    /**
     * @brief Read data from a W5500 hardware socket.
     *
     * Directly reads from the W5500 RX buffer using wiz_recv_data() +
     * Sn_CR_RECV, bypassing the ioLibrary_Driver recv() function which
     * has a bug in non-blocking mode (sock_io_mode check before data check).
     *
     * @param sn      W5500 hardware socket number (0-7).
     * @param buf     Destination buffer.
     * @param maxSize Maximum number of bytes to read.
     * @return Number of bytes read on success, 0 if no data available,
     *         or -1 on error.
     */
    static int32_t readData(uint8_t sn, uint8_t *buf, uint16_t maxSize);

    // -- Internal (called from w5500_socket.cpp) ---------------------------

    /**
     * @brief Register a socket for periodic polling by the W5500 task.
     * @param s   Pointer to the lnSocket instance to register.
     * @param sn  W5500 hardware socket number (0-7).
     */
    static void _registerSocket(lnSocket *s, uint8_t sn);

    /**
     * @brief Unregister a socket from the polling list.
     * @param sn  W5500 hardware socket number (0-7) to unregister.
     */
    static void _unregisterSocket(uint8_t sn);

    /**
     * @brief Get the available TX buffer space for a socket.
     *
     * Queries the W5500 Sn_TX_FSR register to determine how many bytes
     * can be written without blocking.
     *
     * @param sn  W5500 hardware socket number (0-7).
     * @return Number of free bytes in the TX buffer.
     */
    static uint32_t tx_buffer_available(uint8_t sn);
};
// EOF
