/**
 * @file    lowlevel_w5500.h
 * @brief   W5500 SPI adapter — Rust driver entry point.
 *
 * The driver itself is the Rust crate rs_w5500; this header only hands the board
 * pin table, the MAC address and the network callback to its single C entry
 * point, which initialises the controller and starts the coordinator task.
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
bool w5500_begin_c(uint32_t spiInstance, const lnW5500SPI *pins, const uint8_t mac[6],
                   lnLwIpSysCallback cb, void *arg);
}

class W5500LowLevel
{
  public:
    static inline bool begin(int spiInstance, const lnW5500SPI *pins, const uint8_t mac[6],
                             lnLwIpSysCallback cb, void *arg)
    {
        return w5500_begin_c((uint32_t)spiInstance, pins, mac, cb, arg);
    }
};
