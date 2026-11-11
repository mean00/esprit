/**
 * @file    lowlevel_w5500.h
 * @brief   W5500 SPI adapter — Rust driver C++ interface.
 *
 * Thin header forwarding W5500LowLevel operations to the pure-Rust
 * rs_w5500 driver (w5500_init_c, w5500_set_mac_c, w5500_start_c).
 *
 * @copyright (C) 2025-2026
 * @license  See license file
 */
#pragma once

#include "lnGPIO.h"
#include "lnLWIP.h"
#include <stdbool.h>
#include <stdint.h>

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

extern "C" {
    bool w5500_init_c(int spiInstance, const lnW5500SPI *pins);
    void w5500_set_mac_c(const uint8_t mac[6]);
    bool w5500_start_c(lnLwIpSysCallback cb, void *arg);
}

class W5500LowLevel
{
  public:
    static inline bool init(int spiInstance, const lnW5500SPI *pins)
    {
        return w5500_init_c(spiInstance, pins);
    }

    static inline void setMac(const uint8_t mac[6])
    {
        w5500_set_mac_c(mac);
    }

    static inline bool start(lnLwIpSysCallback cb, void *arg)
    {
        return w5500_start_c(cb, arg);
    }
};
