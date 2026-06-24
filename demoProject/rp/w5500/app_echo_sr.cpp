/**
 * @file    app_echo_sr.cpp
 * @brief   W5500 Echo Server Demo using socketRunner only.
 *
 * Demonstrates the socketRunner framework on top of the W5500 hardware TCP
 * engine (via lnSocket_W5500).  No LWIP is used.
 *
 * The client code only uses the socketRunner API and the lnLWIP-compatible
 * W5500LowLevel interface.  All W5500 and DHCP internals are encapsulated
 * in modules/esprit_w5500.cpp.
 *
 * Flow:
 *   1. Initialise W5500 via SPI (esprit lnSPI)
 *   2. Set MAC address
 *   3. Start the W5500 task (drives DHCP internally)
 *   4. Wait for LwipReady callback → signal socketRunner::Up
 *   5. Pump events in a loop — echo back any received data
 *
 * Pin mapping (RP2040):
 *   MISO GPIO16, MOSI GPIO19, CLK GPIO18, CS GPIO17, RST GPIO21, INT GPIO20
 *
 * @copyright (C) 2025
 * @license  See license file
 */
#include "esprit.h"
//
#include "lnGPIO.h"
#include "lnGPIO_pins.h"
//
#include "LN_RTT.h"
#include "lnLWIP.h"
#include "lnSocketRunner.h"
#include "lowlevel_w5500.h"

#if 0
#define debugME Logger
#else
#define debugME(...)                                                                                                   \
    {                                                                                                                  \
    }
#endif

extern void rttLoggerFunction(int n, const char *data);

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------
static const uint8_t gMacAddr[6] = {0x00, 0x08, 0xDC, 0x12, 0x34, 0x56};

static const int SPI_INSTANCE = 0; /**< SPI0 on RP2040 */
static const int TCP_PORT = 3000;  /**< Echo server TCP port */

// ---------------------------------------------------------------------------
// Pin mapping
// ---------------------------------------------------------------------------

/** @brief W5500 pin assignment for the RP2040 board. */
const lnW5500SPI spi_pins = {
    .miso = GPIO16,
    .mosi = GPIO19,
    .clk = GPIO18,
    .cs = GPIO17,
    .reset = GPIO21,
    .intr = GPIO20,
};

// ---------------------------------------------------------------------------
// Event group (shared between lwip_cb and the main loop)
// ---------------------------------------------------------------------------

static lnFastEventGroup eg;

// ---------------------------------------------------------------------------
// LWIP-compatible callback
// ---------------------------------------------------------------------------

/**
 * @brief Called by the W5500 task when the network state changes.
 *
 * On LwipReady we signal socketRunner::Up so the event loop can start
 * the echo server.
 */
static void lwip_cb(lnLwipEvent evt, void * /*arg*/)
{
    switch (evt)
    {
    case LwipReady: {
        Logger("lwip_cb: LwipReady received, signalling Up\n");
        eg.setEvents(socketRunner::Up);
        break;
    }
    case LwipDown: {
        Logger("lwip_cb: LwipDown received, signalling Down\n");
        eg.setEvents(socketRunner::Down);
        break;
    }
    default:
        xAssert(0);
        break;
    }
}

// ---------------------------------------------------------------------------
// socketRunner subclass
// ---------------------------------------------------------------------------

/**
 * @brief Echo server using socketRunner.
 *
 * Listens on TCP port 3000 and echoes back any received data.
 * No hook_poll() needed — the W5500 task polls all sockets internally.
 */
class EchoSocketRunner : public socketRunner
{
  public:
    EchoSocketRunner(uint16_t port, lnFastEventGroup &eventGroup, uint32_t shift)
        : socketRunner(port, eventGroup, shift)
    {
    }

    // ---- socketRunner hooks ----

    virtual void hook_connected() override
    {
        Logger("EchoSocketRunner: client connected\n");
    }

    virtual void hook_disconnected() override
    {
        Logger("EchoSocketRunner: client disconnected\n");
    }

    /**
     * @brief No polling needed — the W5500 task handles it internally.
     */
    virtual void hook_poll() override
    {
        // Intentionally empty
    }

    /**
     * @brief Process incoming data — echo it back.
     *
     * Reads all available data from the socket and writes it back.
     */
    virtual void process_incoming_data() override
    {
        // If the client disconnected while we were processing events,
        // bail out immediately to avoid writing to a closed socket.
        if (!_connected)
        {
            Logger("EchoSocketRunner: not connected, discarding stale data\n");
            return;
        }

        // Loop until all data in the W5500 RX buffer has been drained.
        // The W5500 fires only one RECV interrupt per TCP segment, but
        // multiple segments may have arrived before we read.  Since our
        // _rxBufSize (2048) may be smaller than the total RX data, we
        // must read+echo in multiple passes within this single event.
        for (;;)
        {
            uint32_t n;
            uint8_t *data;

            debugME("Echo: readinging ...\n");
            if (!readData(n, &data))
            {
                Logger("EchoSocketRunner: readData failed\n");
                return;
            }

            if (n == 0)
                break;

            debugME("Echo: echo %lu bytes\n", (unsigned long)n);

            // Echo back — no need to copy, data is valid until releaseData()
            if (!writeData(n, data))
            {
                Logger("EchoSocketRunner: writeData failed, client likely disconnected\n");
                releaseData();
                return; // Exit immediately — socket is dead
            }
            else
            {
                flushWrite();
            }

            releaseData();
        }
    }
};

// ---------------------------------------------------------------------------
// Application entry points
// ---------------------------------------------------------------------------

/**
 * @brief Application setup.
 *
 * Initialises the RTT logger, the W5500 hardware, sets the MAC address,
 * and starts the W5500 network task (which drives DHCP internally).
 */
void setup()
{
    setLogger(rttLoggerFunction);
    LN_RTT_Init();

    Logger("\n--- W5500 Echo Server Demo (socketRunner) ---\n");
    Logger("Initializing W5500 on SPI%d...\n", SPI_INSTANCE);

    if (!W5500LowLevel::init(SPI_INSTANCE, &spi_pins))
    {
        Logger("FATAL: W5500 init failed!\n");
        while (1)
            ;
    }
    Logger("W5500 SPI initialized.\n");

    W5500LowLevel::setMac(gMacAddr);
}
/**
 * @brief Application main loop.
 *
 * Creates the socketRunner immediately (it does nothing until it receives
 * the Up event).  Once DHCP completes, the W5500 task fires LwipReady,
 * which sets socketRunner::Up in the event group.  process_events() then
 * creates the listening socket and starts accepting connections.
 */
void loop()
{

    if (!W5500LowLevel::start(lwip_cb, nullptr))
    {
        Logger("FATAL: W5500 start failed!\n");
        xAssert(0);
    }
    Logger("W5500 started. Waiting for DHCP...\n");
    eg.takeOwnership();
    EchoSocketRunner runner(TCP_PORT, eg, 0);

    while (1)
    {
        uint32_t events = eg.waitEvents(socketRunner::Mask | (3U << 30U), 100);

        if (events & socketRunner::Down)
        {
            Logger("Network down. Cleaning up...\n");
        }
        if (events & socketRunner::Up)
        {
            Logger("Network up. Setting up...\n");
        }
        runner.process_events(events);
    }
}
