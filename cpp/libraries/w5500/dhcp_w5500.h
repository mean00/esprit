/**
 * @file    dhcp_w5500.h
 * @brief   DHCP client for W5500 — public API and internal state.
 *
 * Self-contained DHCP client that uses shadow5Socket register access
 * directly (no ioLibrary_Driver dependency).  Provides the same external
 * API as the original WIZnet dhcp.h:
 *   DHCP_init(), DHCP_run(), DHCP_time_handler(), DHCP_stop()
 *   reg_dhcp_cbfunc(), getIPfromDHCP(), getGWfromDHCP(),
 *   getSNfromDHCP(), getDNSfromDHCP(), getDHCPLeasetime()
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
    // DHCP network information structure
    // ---------------------------------------------------------------------------

    /**
     * @brief Network configuration obtained from DHCP.
     */
    typedef struct
    {
        uint8_t ip[4];       /**< Assigned IP address. */
        uint8_t gw[4];       /**< Gateway/router address. */
        uint8_t subnet[4];   /**< Subnet mask. */
        uint8_t dns[4];      /**< DNS server address. */
        uint32_t lease_time; /**< Lease duration in seconds. */
    } DHCPInfo;

    // ---------------------------------------------------------------------------
    // DHCP state machine return values
    // ---------------------------------------------------------------------------

    /**
     * @brief Return values from DHCP_run().
     */
    enum DHCPReturn
    {
        DHCP_FAILED = 0,     /**< Processing failed (retries exhausted). */
        DHCP_RUNNING = 1,    /**< DHCP still in progress. */
        DHCP_IP_ASSIGN = 2,  /**< IP address assigned for the first time. */
        DHCP_IP_CHANGED = 3, /**< IP address changed during renewal. */
        DHCP_IP_LEASED = 4,  /**< Lease is active, no action needed. */
        DHCP_STOPPED = 5     /**< DHCP has been stopped. */
    };

    // ---------------------------------------------------------------------------
    // Public API
    // ---------------------------------------------------------------------------

    /**
     * @brief Initialise the DHCP client.
     *
     * Opens a UDP socket on the DHCP client port (68) and generates a
     * transaction ID from the MAC address.
     *
     * @param s    W5500 hardware socket number (0-7).
     * @param buf  Buffer for constructing and receiving DHCP messages
     *             (must be at least 600 bytes).
     */
    void DHCP_init(uint8_t s);

    /**
     * @brief 1-second tick handler.
     *
     * Must be called once per second from a timer or the main loop.
     * Drives the DHCP timeout and lease-renewal logic.
     */
    void DHCP_time_handler(void);

    /**
     * @brief Register DHCP event callbacks.
     *
     * @param ip_assign    Called when an IP address is assigned for the first time.
     * @param ip_update    Called when the IP address changes during renewal.
     * @param ip_conflict  Called when an IP address conflict is detected.
     */
    void DHCP_reg_cbfunc(void (*ip_assign)(void), void (*ip_update)(void), void (*ip_conflict)(void));

    /**
     * @brief Run the DHCP state machine (call frequently from the main loop).
     *
     * @return One of @ref DHCPReturn:
     *         - @ref DHCP_RUNNING while in progress
     *         - @ref DHCP_IP_ASSIGN when a new IP is obtained
     *         - @ref DHCP_IP_LEASED while the lease is active
     *         - @ref DHCP_FAILED if retries are exhausted
     *         - @ref DHCP_STOPPED if DHCP_stop() was called
     */
    enum DHCPReturn DHCP_run(void);

    /**
     * @brief Stop DHCP processing and close the UDP socket.
     *
     * To restart, call DHCP_init() followed by DHCP_run().
     */
    void DHCP_stop(void);

    /**
     * @brief Get the current DHCP network configuration.
     *
     * @return A const pointer to a @ref DHCPInfo structure if DHCP has
     *         successfully obtained an IP address (state BOUND), or NULL
     *         if DHCP is still in progress or has failed.
     */
    const DHCPInfo *DHCP_getInfo(void);

#ifdef __cplusplus
}
#endif

// EOF
