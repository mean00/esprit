/**
 * @file    dhcp_w5500_priv.h
 * @brief   DHCP client for W5500 — private internal state and constants.
 *
 * Contains all internal implementation details of the DHCP client:
 * state machine enum, internal data structure, protocol constants,
 * option tags, and the reply buffer.  External code should never
 * include this file directly.
 *
 * @copyright (C) 2025
 * @license  See license file
 */
#pragma once

#include <stdint.h>

// ---------------------------------------------------------------------------
// DHCP protocol constants
// ---------------------------------------------------------------------------

/** @brief DHCP server UDP port. */
#define DHCP_SERVER_PORT 67

/** @brief DHCP client UDP port. */
#define DHCP_CLIENT_PORT 68

/** @brief Magic cookie value (0x63825363). */
#define MAGIC_COOKIE 0x63825363

/** @brief Maximum number of DHCP retries before giving up. */
#define MAX_DHCP_RETRY 2

/** @brief Timeout per state in seconds (10 s). */
#define DHCP_WAIT_TIME 10

// ---------------------------------------------------------------------------
// DHCP message types (option 53)
// ---------------------------------------------------------------------------

#define DHCP_DISCOVER 1
#define DHCP_OFFER 2
#define DHCP_REQUEST 3
#define DHCP_DECLINE 4
#define DHCP_ACK 5
#define DHCP_NAK 6
#define DHCP_RELEASE 7

// ---------------------------------------------------------------------------
// DHCP option tags
// ---------------------------------------------------------------------------

#define DHCP_OPT_SUBNET_MASK 1
#define DHCP_OPT_ROUTER 3
#define DHCP_OPT_DNS 6
#define DHCP_OPT_HOST_NAME 12
#define DHCP_OPT_REQUESTED_IP 50
#define DHCP_OPT_LEASE_TIME 51
#define DHCP_OPT_MSG_TYPE 53
#define DHCP_OPT_SERVER_ID 54
#define DHCP_OPT_PARAM_REQ 55
#define DHCP_OPT_END 255

// ---------------------------------------------------------------------------
// BOOTP header field offsets (within the 240-byte BOOTP message)
// ---------------------------------------------------------------------------

#define BOOTP_OP_OFFSET      0   /**< op (1=BOOTREQUEST, 2=BOOTREPLY). */
#define BOOTP_HTYPE_OFFSET   1   /**< htype (1=Ethernet). */
#define BOOTP_HLEN_OFFSET    2   /**< hlen (6 for MAC). */
#define BOOTP_HOPS_OFFSET    3   /**< hops. */
#define BOOTP_XID_OFFSET     4   /**< xid (transaction ID, 4 bytes). */
#define BOOTP_SECS_OFFSET    8   /**< secs. */
#define BOOTP_FLAGS_OFFSET   10  /**< flags. */
#define BOOTP_CIADDR_OFFSET  12  /**< ciaddr (client IP, 4 bytes). */
#define BOOTP_YIADDR_OFFSET  16  /**< yiaddr (your/offered IP, 4 bytes). */
#define BOOTP_SIADDR_OFFSET  20  /**< siaddr (server IP, 4 bytes). */
#define BOOTP_GIADDR_OFFSET  24  /**< giaddr (relay agent IP, 4 bytes). */
#define BOOTP_CHADDR_OFFSET  28  /**< chaddr (client MAC, 16 bytes). */
#define BOOTP_SNAME_OFFSET   44  /**< sname (server host name, 64 bytes). */
#define BOOTP_FILE_OFFSET    108 /**< file (boot file name, 128 bytes). */
#define BOOTP_MAGIC_OFFSET   236 /**< magic cookie (4 bytes: 0x63825363). */
#define BOOTP_OPTIONS_OFFSET 240 /**< start of DHCP options. */

// ---------------------------------------------------------------------------
// Internal buffers
// ---------------------------------------------------------------------------

/** @brief Size of the DHCP reply buffer (must hold a full BOOTP message + options). */
#define DHCP_REPLY_BUFFER_SIZE 600

// ---------------------------------------------------------------------------
// DHCP client state machine
// ---------------------------------------------------------------------------

/**
 * @brief Internal states of the DHCP client state machine.
 */
enum DHCPState
{
    DHCP_STATE_IDLE = 0,        /**< Idle — no DHCP in progress. */
    DHCP_STATE_DISCOVERING = 1, /**< DISCOVER sent, waiting for OFFER. */
    DHCP_STATE_REQUESTING = 2,  /**< REQUEST sent, waiting for ACK/NAK. */
    DHCP_STATE_BOUND = 3        /**< Lease active, IP address valid. */
};

// ---------------------------------------------------------------------------
// DHCP internal data
// ---------------------------------------------------------------------------

/**
 * @brief All internal state of the DHCP client.
 *
 * A single static instance of this structure is kept in dhcp_w5500.cpp.
 * External code should never access these fields directly; use the getter
 * functions declared in dhcp_w5500.h instead.
 */
typedef struct
{
    /** @brief Current state machine state (@ref DHCPState). */
    uint8_t state;

    /** @brief W5500 hardware socket number used for DHCP (usually 7). */
    uint8_t sn;

    /** @brief Pointer to the DHCP message buffer (supplied by caller). */
    uint8_t *buf;

    /** @brief Retry counter (incremented on each timeout). */
    uint8_t retry_count;

    /** @brief Seconds elapsed since entering the current state. */
    uint32_t tick;

    /** @brief Lease time in seconds (from DHCP ACK option 51). */
    uint32_t lease_time;

    /** @brief Transaction ID (xid) for the current DHCP session. */
    uint32_t xid;

    /** @brief DHCP server identifier (option 54). */
    uint8_t server_ip[4];

    /** @brief IP address requested in the REQUEST message (option 50). */
    uint8_t requested_ip[4];

    /** @brief Public network info (ip, gw, subnet, dns, lease_time). */
    DHCPInfo info;

    /** @brief Callback: IP address assigned for the first time. */
    void (*ip_assign)(void);

    /** @brief Callback: IP address changed during renewal. */
    void (*ip_update)(void);

    /** @brief Callback: IP address conflict detected. */
    void (*ip_conflict)(void);
} DHCPData;
// EOF