#pragma once
#include "stdint.h"

void LN_RTT_Init(void);
uint32_t LN_RTT_Write(uint32_t bufferIndex, const uint8_t *buffer, uint32_t size);
