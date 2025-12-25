#pragma once

#include <lwip/netif.h>
//
#include "eth_driver.h"

#define INTERRUPT(name) __attribute__((interrupt)) void name(void)

uint32_t ch32_eth_init(uint8_t *mac = nullptr, const uint8_t *ip = nullptr, const uint8_t *gw = nullptr,
                       const uint8_t *netmask = nullptr);

void ch32_eth_loop(uint32_t deltaMs = 0);

netif *get_netif();

// LwIP driver
err_t ch32_netif_init(struct netif *netif);

uint8_t dhcp_get_state();

#define ETH_LED_LINK 0
#define ETH_LED_ACT 1

typedef void (*eth_led_cb_t)(uint8_t ledId, uint8_t state);

void ch32_eth_setLedCallback(eth_led_cb_t cb);
