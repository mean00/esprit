#ifndef NET__H
#define NET__H
#include "stdint.h"
//
#include "ch32v30x_eth.h"

#if 0
#define USE_LOOP_STRUCT 1
#else
#define USE_CHAIN_STRUCT 1
#endif

typedef struct
{
    void *next;
    uint32_t length;
    uint32_t buffer;
    ETH_DMADESCTypeDef *descriptor;
} FrameTypeDef;

extern "C" bool ETH_TxPkt_ChainMode(uint16_t FrameLength);
void mac_send(uint8_t *content_ptr, uint16_t content_len);

extern "C" FrameTypeDef ETH_RxPkt_ChainMode(void);

extern void PHY_control_pin_init(void);
extern void GETH_pin_init(void);
extern void FETH_pin_init(void);

#define ROM_CFG_USERADR_ID 0x1FFFF7E8

extern "C" void WCH_GetMacAddr(uint8_t *p);

extern "C" void ch32v3_init_phy(void);

#endif
