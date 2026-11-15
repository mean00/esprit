/**
 * @file    network_w5500.h
 * @brief   W5500 network configuration API — replacement for ioLibrary wizchip_conf.h.
 *
 * Provides functions to configure MAC address, IP, subnet, gateway,
 * PHY settings, and interrupt masks using shadow5System register access.
 * No ioLibrary_Driver dependency.
 *
 * @copyright (C) 2025
 * @license  See license file
 */

#pragma once

#include <stdint.h>
#include <string.h>

// ---------------------------------------------------------------------------
// Network info structure (matches ioLibrary wiz_NetInfo layout)
// ---------------------------------------------------------------------------

typedef struct wiz_NetInfo_t
{
    uint8_t mac[6]; /**< Source MAC address */
    uint8_t ip[4];  /**< Source IP address */
    uint8_t sn[4];  /**< Subnet mask */
    uint8_t gw[4];  /**< Gateway IP address */
    uint8_t dns[4]; /**< DNS server IP address */
    uint8_t dhcp;   /**< 1 = static, 2 = DHCP */
} wiz_NetInfo;

// ---------------------------------------------------------------------------
// PHY configuration structure
// ---------------------------------------------------------------------------

typedef struct wiz_PhyConf_t
{
    uint8_t by;     /**< PHY_CONFBY_HW or PHY_CONFBY_SW */
    uint8_t mode;   /**< PHY_MODE_MANUAL or PHY_MODE_AUTONEGO */
    uint8_t speed;  /**< PHY_SPEED_10 or PHY_SPEED_100 */
    uint8_t duplex; /**< PHY_DUPLEX_HALF or PHY_DUPLEX_FULL */
} wiz_PhyConf;

// ---------------------------------------------------------------------------
// PHY configuration constants
// ---------------------------------------------------------------------------

#define PHY_CONFBY_SW 0
#define PHY_CONFBY_HW 1
#define PHY_MODE_MANUAL 0
#define PHY_MODE_AUTONEGO 1
#define PHY_SPEED_10 0
#define PHY_SPEED_100 1
#define PHY_DUPLEX_HALF 0
#define PHY_DUPLEX_FULL 1

// ---------------------------------------------------------------------------
// DHCP mode constants
// ---------------------------------------------------------------------------

#define NETINFO_STATIC 1
#define NETINFO_DHCP 2

// ---------------------------------------------------------------------------
// Interrupt mask constants
// ---------------------------------------------------------------------------

#define IK_SOCK_ALL 0xFF /**< Enable interrupts for all sockets */

// ---------------------------------------------------------------------------
// API functions
// ---------------------------------------------------------------------------

/**
 * @brief Set the MAC address (SHAR register).
 *
 * @param mac  6-byte MAC address.
 */
void w5500_set_mac(const uint8_t mac[6]);

/**
 * @brief Read back the MAC address (SHAR register).
 *
 * @param mac  Output buffer for 6-byte MAC address.
 */
void w5500_get_mac(uint8_t mac[6]);

/**
 * @brief Set IP, subnet, gateway, and MAC in hardware registers.
 *
 * Writes SHAR, SIPR, SUBR, GAR registers.
 *
 * @param info  Network configuration.
 */
void w5500_set_netinfo(const wiz_NetInfo *info);

/**
 * @brief Configure the integrated Ethernet PHY.
 *
 * Writes the PHYCFGR register.
 *
 * @param phyconf  PHY configuration.
 */
void w5500_set_phyconf(const wiz_PhyConf *phyconf);

/**
 * @brief Read the current PHY status (link, speed, duplex).
 *
 * Reads the PHYCFGR register.
 *
 * @param phyconf  Output structure for PHY status.
 */
void w5500_get_phystat(wiz_PhyConf *phyconf);

/**
 * @brief Enable global socket interrupt mask (SIMR).
 *
 * @param mask  Bitmask of sockets to enable interrupts for (0-7).
 */
void w5500_set_socket_intr_mask(uint8_t mask);
