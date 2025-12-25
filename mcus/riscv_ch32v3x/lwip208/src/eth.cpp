#include "stdint.h"
#include "esprit.h"
//
#include "eth.h"
#include <lwip/dhcp.h>
#include <lwip/etharp.h>
#include <lwip/init.h>
#include <lwip/timeouts.h>
#include <netif/ethernet.h>
#include <string.h>
//
#include "ch32v20x.h"
//
const uint32_t LED_PERSISTENCE = 50; // ms, time to leave LED on/off

extern ETH_DMADESCTypeDef *pDMARxSet;
extern ETH_DMADESCTypeDef *pDMATxSet;

struct netif gnetif;

volatile bool _hasFrame = 0, _linkChanged = 0;
struct pbuf *_pbuf = NULL;

uint32_t _lastTimeIsr = 0;
uint32_t _lastLedChange = 0;
bool _actLedDoBlink = 0, _lastActLedState = 0;

eth_led_cb_t _led_cb = NULL;

void ch32_eth_setLedCallback(eth_led_cb_t cb)
{
    _led_cb = cb;
}

netif *get_netif()
{
    return &gnetif;
}

// check if descriptor is not owned by ETH DMA
uint32_t eth_check_packet()
{
    if (pDMARxSet->Status & ETH_DMARxDesc_OWN)
    {
        return ETH_ERROR;
    }
    return ETH_SUCCESS;
}

// copied from eth_driver.c/RecDataPolling
uint32_t eth_get_packet(uint8_t **buffer, uint16_t *len)
{
    if (eth_check_packet() == ETH_ERROR)
    {
        return ETH_ERROR;
    }

    // while (!(pDMARxSet->Status & ETH_DMARxDesc_OWN)) {
    if (!(pDMARxSet->Status & ETH_DMARxDesc_ES) && (pDMARxSet->Status & ETH_DMARxDesc_LS) &&
        (pDMARxSet->Status & ETH_DMARxDesc_FS))
    {
        if (buffer && len)
        {
            /* Get the Frame Length of the received packet: substruct 4 bytes of the CRC */
            *len = ((pDMARxSet->Status & ETH_DMARxDesc_FL) >> 16) - 4;
            /* Get the addrees of the actual buffer */
            *buffer = (uint8_t *)pDMARxSet->Buffer1Addr;
        }
    }

    return ETH_SUCCESS;
    // }
}

// call when done with handling of packet
void eth_get_packet_done()
{
    pDMARxSet->Status = ETH_DMARxDesc_OWN;
    pDMARxSet = (ETH_DMADESCTypeDef *)pDMARxSet->Buffer2NextDescAddr;
}

// copied from eth_driver.c/MACRAW_Tx, but actually using the TX buffer, because of 4 byte alignment
// (number of DMATxDesc's are hardcoded to 1, so no actual buffering going on.)
uint32_t eth_send_packet(const uint8_t *buffer, uint16_t len)
{
    /* Check if the descriptor is owned by the ETHERNET DMA (when set) or CPU (when reset) */
    if (DMATxDescToSet->Status & ETH_DMATxDesc_OWN)
    {
        /* Return ERROR: OWN bit set */
        Logger("[ETH] ERROR: probably about to drop TX packet! Busy looping until previous TX done ");
        while (DMATxDescToSet->Status & ETH_DMATxDesc_OWN)
        {
            Logger(".");
        }
        Logger(" done.\n");
        // return ETH_ERROR;
    }
    DMATxDescToSet->Status |= ETH_DMATxDesc_OWN;
    memcpy((void *)DMATxDescToSet->Buffer1Addr, buffer, len);
    R16_ETH_ETXLN = len;
    R16_ETH_ETXST = DMATxDescToSet->Buffer1Addr;
    R8_ETH_ECON1 |= RB_ETH_ECON1_TXRTS; // start sending
    /* Update the ETHERNET DMA global Tx descriptor with next Tx descriptor */
    /* Chained Mode */
    /* Selects the next DMA Tx descriptor list for next buffer to send */
    DMATxDescToSet = (ETH_DMADESCTypeDef *)(DMATxDescToSet->Buffer2NextDescAddr);
    /* Return SUCCESS */
    return ETH_SUCCESS;
}

extern "C"
{
    INTERRUPT(ETH_IRQHandler)
    {
        // Logger("(%8d) [ETH] ISR\n", millis());

        uint8_t eth_irq_flag = R8_ETH_EIR;
        if (eth_irq_flag & RB_ETH_EIR_LINKIF)
        { // link changed
            _linkChanged = true;
        }

        WCHNET_ETHIsr();
    }
}

static err_t ch32_netif_output(struct netif *netif, struct pbuf *p)
{
    (void)netif;
    LINK_STATS_INC(link.xmit);
    _actLedDoBlink = true;

    if (eth_send_packet((const uint8_t *)p->payload, p->tot_len) == ETH_ERROR)
    {
        return ERR_IF;
    }
    return ERR_OK;
}

err_t ch32_netif_init(struct netif *netif)
{
    netif->linkoutput = ch32_netif_output;
    netif->output = etharp_output;
    netif->mtu = MAX_ETH_PAYLOAD;
    netif->flags = NETIF_FLAG_BROADCAST | NETIF_FLAG_ETHARP | NETIF_FLAG_LINK_UP;
    netif->hostname = "lwip";
    netif->name[0] = 'c';
    netif->name[1] = 'h';

    WCHNET_GetMacAddr(netif->hwaddr);
    netif->hwaddr_len = 6;

    return ERR_OK;
}

void netifCfg(const ip4_addr_t *ipaddr, const ip4_addr_t *netmask, const ip4_addr_t *gw)
{
    netif_remove(&gnetif);

    netif_add(&gnetif, ipaddr, netmask, gw, NULL, &ch32_netif_init, &ethernet_input);

    netif_set_default(&gnetif);

    if (netif_is_link_up(&gnetif))
    {
        netif_set_up(&gnetif); // When the netif is fully configured this function must be called
    }
    else
    {
        netif_set_down(&gnetif); // When the netif link is down this function must be called
    }

// MEANX #if LWIP_NETIF_LINK_CALLBACK
// MEANX    netif_set_link_callback(&gnetif, ethernetif_update_config); // TODO
// MEANX #endif
}

#define IP_ADDR4_U8ARR(ipaddr, u8ptr) IP_ADDR4((ipaddr), (u8ptr)[0], (u8ptr)[1], (u8ptr)[2], (u8ptr)[3])

uint32_t ch32_eth_init(uint8_t *mac, const uint8_t *ip, const uint8_t *gw, const uint8_t *netmask)
{
    if ((SystemCoreClock != 60000000) && (SystemCoreClock != 120000000))
    {
        Logger("[ETH] ERROR! SysClock has to be 60MHz or 120MHz when using Ethernet! \nCurrent SysClock: %d\n",
               SystemCoreClock);
        return ETH_ERROR;
    }
    if (mac == nullptr)
    {
        uint8_t wchMac[6];
        WCHNET_GetMacAddr(wchMac);
        mac = wchMac;
    }

    ip_addr_t ipAddr, gwAddr, netmaskAddr;
    ip_addr_set_zero_ip4(&ipAddr);
    ip_addr_set_zero_ip4(&gwAddr);
    ip_addr_set_zero_ip4(&netmaskAddr);

    if (ip != nullptr)
    {
        IP_ADDR4_U8ARR(&ipAddr, ip);
    }

    if (gw != nullptr)
    {
        IP_ADDR4_U8ARR(&gwAddr, gw);
    }

    if (netmask != nullptr)
    {
        IP_ADDR4_U8ARR(&netmaskAddr, netmask);
    }

    ETH_Init(mac);

    lwip_init();

    netifCfg(&ipAddr, &netmaskAddr, &gwAddr);

    if (ip != nullptr)
    {
        dhcp_inform(&gnetif);
    }
    else
    {
        dhcp_start(&gnetif);
    }

    return ETH_SUCCESS;
}

// set _actLedDoBlink to true to trigger a LED blink
// correctly handles multiple blink requests in quick succession
void actLedHandling(uint32_t ms)
{
    if (!_led_cb)
        return;

    if (ms - _lastLedChange >= LED_PERSISTENCE)
    {
        _lastLedChange = ms;

        bool newLedState = !_lastActLedState && _actLedDoBlink; // turn on if packet waiting and was off previously
        if (newLedState)
        {
            _actLedDoBlink = false; // clear waiting LED turn-on request
        }
        if (newLedState != _lastActLedState)
        {
            _led_cb(ETH_LED_ACT, newLedState);
        }
        _lastActLedState = newLedState;
    }
}

void ch32_eth_loop(uint32_t ms)
{
    if (eth_check_packet())
    {
        uint8_t *buf;
        uint16_t len;
        if (eth_get_packet(&buf, &len) == ETH_SUCCESS)
        {
            _pbuf = pbuf_alloc(PBUF_RAW, len, PBUF_POOL);
            if (_pbuf != NULL)
            {
                pbuf_take(_pbuf, buf, len);
                eth_get_packet_done();

                LINK_STATS_INC(link.recv);
                _actLedDoBlink = true;
                if (gnetif.input(_pbuf, &gnetif) != ERR_OK)
                {
                    pbuf_free(_pbuf);
                    Logger("[ETH] netif input error\n");
                }
            }
            else
            {
                // if this error occurs, try decreasing FREE_BUFFER_THRESHHOLD in EthernetClient
                Logger("[ETH] ERROR: Buffer overrun!\n");
            }
        }
    }

    actLedHandling(ms);

    if (ms != _lastTimeIsr)
    {
        WCHNET_TimeIsr(ms - _lastTimeIsr);
        WCHNET_HandlePhyNegotiation();
        _lastTimeIsr = ms;
    }

    if (_linkChanged)
    {
        _linkChanged = false;
        uint16_t phyBMSR = ReadPHYReg(PHY_BMSR);
        bool linkState = phyBMSR & PHY_Linked_Status /*&& phyBMSR & PHY_AutoNego_Complete*/;
        if (linkState)
        {
            Logger("[ETH] Link Up.\n");
            netif_set_link_up(&gnetif);
        }
        else
        {
            Logger("[ETH] Link Down.\n");
            netif_set_link_down(&gnetif);
        }

        if (_led_cb)
        {
            _led_cb(ETH_LED_LINK, linkState);
        }
    }

    sys_check_timeouts();
}

uint8_t dhcp_get_state()
{
    return ((struct dhcp *)netif_get_client_data(&gnetif, LWIP_NETIF_CLIENT_DATA_INDEX_DHCP))->state;
}
