/*
 *
 *
 */
#include "esprit.h"
extern "C"
{
#include "lwip/api.h"
#include "lwip/dhcp.h"
#include "lwip/init.h"
#include "lwip/sys.h"
#include "lwip/tcpip.h"
}
#include "eth_pins.h"
#include "lnLWIP.h"
extern struct netif lwip_netif;

extern lnLwIpSysCallback _syscb;
extern void lnwip_common_init(lnLwIpSysCallback syscb, void *arg);
extern "C"
{
    err_t ethernetif_init(struct netif *netif);
    void ethernetif_input(struct netif *netif);
}

/**
 *
 *
 */
void ln_ethernet_rx(void)
{
    ethernetif_input(&lwip_netif);
}
//
void disable_eth_irq()
{
    lnDisableInterrupt(LN_IRQ_ETH);
}
//
void enable_eth_irq()
{
    lnEnableInterrupt(LN_IRQ_ETH);
}
void tiny_sleep_ms(unsigned int ms)
{
    lnDelayMs(ms);
}
void Ethernet_LED_LINKSET(uint8_t setbit)
{
    lnDigitalWrite(LN_ETH_LINKSET_LED, setbit);
}

void Ethernet_LED_DATASET(uint8_t setbit)
{
    lnDigitalWrite(LN_ETH_DATASET_LED, setbit);
}
/**
 *
 */
bool lnLWIP::start(lnLwIpSysCallback syscb, void *arg)
{
    Logger("Initializing LwIP \n");
    //
    lnPinMode(LN_ETH_LINKSET_LED, lnOUTPUT_OPEN_DRAIN, 10);
    lnPinMode(LN_ETH_DATASET_LED, lnOUTPUT_OPEN_DRAIN, 10);
    lnPeripherals::initEthClock();
    xDelay(5);
    lnPeripherals::enable(pETHERNET);
    lnPeripherals::reset(pETHERNET);
    tcpip_init(NULL, NULL);
    netif_add(&lwip_netif, NULL, NULL, NULL, NULL, &ethernetif_init, &tcpip_input);
    lnwip_common_init(syscb, arg);
    Logger(" LwIP Initialized \n");
    return true;
}

// EOF
