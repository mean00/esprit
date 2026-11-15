/**
 * @file    app_echo_sr.cpp
 * @brief   W5500 Echo Server Demo using Rust rs_w5500 + rs_socketrunner.
 *
 * Pin mapping (RP2040 / RP2350):
 *   MISO GPIO16, MOSI GPIO19, CLK GPIO18, CS GPIO17, RST GPIO21, INT GPIO20
 */
#include "esprit.h"
//
#include "lnGPIO.h"
#include "lnGPIO_pins.h"
//
#include "LN_RTT.h"
#include "lnLWIP.h"
#include "lowlevel_w5500.h"

extern void rttLoggerFunction(int n, const char *data);

extern "C" {
void rust_w5500_net_cb(uint32_t evt, void *arg);
void rust_w5500_loop(void);
}

static const uint8_t gMacAddr[6] = {0x00, 0x08, 0xDC, 0x12, 0x34, 0x56};
static const int SPI_INSTANCE = 0;

static const lnW5500SPI spi_pins = {
    .miso = GPIO16,
    .mosi = GPIO19,
    .clk = GPIO18,
    .cs = GPIO17,
    .reset = GPIO21,
    .intr = GPIO20,
};

static void lwip_cb(lnLwipEvent evt, void *arg)
{
    rust_w5500_net_cb(static_cast<uint32_t>(evt), arg);
}

void setup()
{
    setLogger(rttLoggerFunction);
    LN_RTT_Init();

    Logger("\n--- Rust W5500 Echo Server Demo ---\n");
    Logger("Initializing W5500 on SPI%d...\n", SPI_INSTANCE);

    if (!W5500LowLevel::begin(SPI_INSTANCE, &spi_pins, gMacAddr, lwip_cb, nullptr))
    {
        Logger("FATAL: W5500 init failed!\n");
        while (1)
            ;
    }
    Logger("W5500 started. Waiting for DHCP...\n");
}

void loop()
{
    rust_w5500_loop();
}
