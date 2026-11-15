/**
 * @file    network_w5500.cpp
 * @brief   W5500 network configuration API — implementation.
 *
 * Implements MAC, IP, PHY, and interrupt mask configuration using
 * shadow5System register access.  No ioLibrary_Driver dependency.
 *
 * @copyright (C) 2025
 * @license  See license file
 */

#include "network_w5500.h"
#include "W5500/w5500_regs.h"
#include "lowlevel_w5500_helper.h"

// ---------------------------------------------------------------------------
// w5500_set_mac()
// ---------------------------------------------------------------------------

void w5500_set_mac(const uint8_t mac[6])
{
    shadow5System sys;
    sys.setMac(mac);
}

// ---------------------------------------------------------------------------
// w5500_get_mac()
// ---------------------------------------------------------------------------

void w5500_get_mac(uint8_t mac[6])
{
    shadow5System sys;
    sys.getMac(mac);
}

// ---------------------------------------------------------------------------
// w5500_set_netinfo()
// ---------------------------------------------------------------------------

void w5500_set_netinfo(const wiz_NetInfo *info)
{
    shadow5System sys;

    // Write MAC (SHAR)
    sys.setMac(info->mac);

    // Write IP (SIPR)
    sys.setSourceIp(info->ip);

    // Write subnet (SUBR)
    sys.setSubnet(info->sn);

    // Write gateway (GAR)
    sys.setGateway(info->gw);
}

// ---------------------------------------------------------------------------
// w5500_set_phyconf()
// ---------------------------------------------------------------------------

void w5500_set_phyconf(const wiz_PhyConf *phyconf)
{
    uint8_t phycfgr = 0;

    if (phyconf->by == PHY_CONFBY_SW)
    {
        // Software control: set OPMD bit
        phycfgr |= PHYCFGR_OPMD;

        if (phyconf->mode == PHY_MODE_AUTONEGO)
        {
            phycfgr |= PHYCFGR_OPMDC_ALLA; // Auto-negotiation, all speeds
        }
        else
        {
            // Manual mode
            if (phyconf->speed == PHY_SPEED_100 && phyconf->duplex == PHY_DUPLEX_FULL)
                phycfgr |= PHYCFGR_OPMDC_100F;
            else if (phyconf->speed == PHY_SPEED_100 && phyconf->duplex == PHY_DUPLEX_HALF)
                phycfgr |= PHYCFGR_OPMDC_100H;
            else if (phyconf->speed == PHY_SPEED_10 && phyconf->duplex == PHY_DUPLEX_FULL)
                phycfgr |= PHYCFGR_OPMDC_10F;
            else
                phycfgr |= PHYCFGR_OPMDC_10H; // 10M Half
        }
    }

    write_to_reg(PHYCFGR, 1, &phycfgr);
}

// ---------------------------------------------------------------------------
// w5500_get_phystat()
// ---------------------------------------------------------------------------

void w5500_get_phystat(wiz_PhyConf *phyconf)
{
    uint8_t phycfgr = 0;
    read_from_reg(PHYCFGR, 1, &phycfgr);

    // Decode speed
    phyconf->speed = (phycfgr & (1 << 1)) ? PHY_SPEED_100 : PHY_SPEED_10;

    // Decode duplex
    phyconf->duplex = (phycfgr & (1 << 2)) ? PHY_DUPLEX_FULL : PHY_DUPLEX_HALF;

    // Link status (bit 0)
    // phyconf->link = (phycfgr & 0x01) ? PHY_LINK_ON : PHY_LINK_OFF;
}

// ---------------------------------------------------------------------------
// w5500_set_socket_intr_mask()
// ---------------------------------------------------------------------------

void w5500_set_socket_intr_mask(uint8_t mask)
{
    shadow5System sys;
    sys.setSocketInterruptMask(mask);
}