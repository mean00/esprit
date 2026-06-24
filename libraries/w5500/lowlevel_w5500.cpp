/**
 * @file    lowlevel_w5500.cpp
 * @brief   W5500 SPI adapter — implementation.
 *
 * Provides the low-level SPI register access functions write_to_reg()
 * and read_from_reg() which are the ONLY functions that directly access
 * the SPI bus.  All higher-level code (shadow5Socket, network_w5500,
 * socket_high_w5500, dhcp_w5500) calls through these two functions.
 *
 * Contains the W5500 coordinator task (lnTaskW5500) which drives DHCP
 * internally and dispatches per-socket interrupts.  On each wakeup
 * (INTR pin edge or 100 ms timeout) the task reads the global SIR
 * register to discover which sockets have pending interrupts, then
 * calls dispatchPerSocket() which signals the corresponding socketRunner
 * via SocketCustom and SocketWriteAvailable events.
 *
 * Pin mapping (RP2040):
 *   MISO GPIO16, MOSI GPIO19, CLK GPIO18, CS GPIO17, RST GPIO21, INT GPIO20
 *
 * @copyright (C) 2025
 * @license  See license file
 */

#include "lnDebug.h"
#include "lnGPIO.h"
#include "lnSPI.h"
#include "lnSPIBidir.h"
#include "lnSPI_defines.h"
#include "lnSystemTime.h"
//
#include "lowlevel_w5500.h"
#include "lowlevel_w5500_helper.h"
#include "lnFreeRTOS.h"
#include "network_w5500.h"
#include "socket_w5500.h"
//
#include <cstring>

// ---------------------------------------------------------------------------
// W5500 SPI protocol constants
// ---------------------------------------------------------------------------

/** @brief SPI read operation code (bit 2 set = read). */
#define _W5500_SPI_READ_ (0x00 << 2)

/** @brief SPI write operation code (bit 2 set = write). */
#define _W5500_SPI_WRITE_ (0x01 << 2)

/** @brief Variable-data-length mode (0 = VDM). */
#define _W5500_SPI_VDM_OP_ 0x00

extern "C"
{
#include "dhcp_w5500.h"
}

// ---------------------------------------------------------------------------
// Static globals
// ---------------------------------------------------------------------------

/** @brief Dedicated UDP socket for DHCP. */
static const int DHCP_SOCKET = 7;

/** @brief DHCP message buffer size. */
static const int DHCP_BUF_SIZE = 600;

/** @brief Saved pin mapping pointer (set by W5500LowLevel::init()). */
static const lnW5500SPI *_pins = NULL;

/** @brief The SPI bidirectional driver instance. */
static lnSPIBidir *gSPI = nullptr;

/** @brief TX buffer size per socket (2 KB). */
static const uint8_t socket_txsize[8] = {4, 4, 4, 0, 0, 0, 0, 2};

/** @brief RX buffer size per socket (2 KB). */
static const uint8_t socket_rxsize[8] = {4, 4, 4, 0, 0, 0, 0, 2};

/** @brief SPI bus mutex (created in init()). */
static SemaphoreHandle_t gSpiMutex = NULL;

// ---------------------------------------------------------------------------
// Socket registry (for the W5500 task to poll)
// ---------------------------------------------------------------------------

/** @brief Maximum number of hardware sockets. */
static const int MAX_SOCKETS = 8;

/**
 * @brief Registered socket pointers, indexed by hardware socket number.
 *
 * nullptr means the slot is free.
 */
static lnSocket *gActiveSockets[MAX_SOCKETS] = {nullptr};

// ---------------------------------------------------------------------------
// LWIP-compatible callback storage
// ---------------------------------------------------------------------------

/** @brief User-registered network event callback (lnLWIP-compatible). */
static lnLwIpSysCallback gLwipCallback = nullptr;

/** @brief Opaque argument for the callback. */
static void *gLwipArg = nullptr;

/** @brief Stored MAC address (set via setMac()). */
static uint8_t gMac[6] = {0};

/** @brief Network information obtained from DHCP. */
static wiz_NetInfo gNetInfo;

// ---------------------------------------------------------------------------
// SPI register access — the ONLY functions that touch the SPI bus
// ---------------------------------------------------------------------------

/**
 * @brief RAII mutex guard for SPI register access.
 *
 * Acquires the SPI mutex on construction and releases it on destruction.
 */
class guard
{
  public:
    guard()
    {
        xAssert(gSpiMutex);
        xSemaphoreTake(gSpiMutex, portMAX_DELAY);
    }
    ~guard()
    {
        xSemaphoreGive(gSpiMutex);
    }
};

/**
 * @brief Write data to a W5500 register via SPI.
 *
 * Sends a 3-byte address (block select + offset) followed by the data
 * payload.  This is the ONLY function that writes to the SPI bus.
 *
 * @param adr    24-bit register address (block select + offset).
 * @param txlen  Number of bytes to write.
 * @param txdata Pointer to the data buffer.
 * @return true on success.
 */
bool write_to_reg(uint32_t adr, uint32_t txlen, const uint8_t *txdata)
{
    uint8_t spi_data[3];

    guard g;
    lnDigitalWrite(_pins->cs, false); // CS assert
    adr |= _W5500_SPI_WRITE_ | _W5500_SPI_VDM_OP_;
    spi_data[0] = (adr & 0x00FF0000) >> 16;
    spi_data[1] = (adr & 0x0000FF00) >> 8;
    spi_data[2] = (adr & 0x000000FF) >> 0;
    gSPI->blockWrite8(3, spi_data);              // address
    gSPI->blockWrite8(txlen, (uint8_t *)txdata); // data
    lnDigitalWrite(_pins->cs, true);             // CS de-assert
    return true;
}

/**
 * @brief Read data from a W5500 register via SPI.
 *
 * Sends a 3-byte address (block select + offset) then reads the response.
 * This is the ONLY function that reads from the SPI bus.
 *
 * @param adr    24-bit register address (block select + offset).
 * @param rxlen  Number of bytes to read.
 * @param rxdata Pointer to the destination buffer.
 * @return true on success.
 */
bool read_from_reg(uint32_t adr, uint32_t rxlen, uint8_t *rxdata)
{
    uint8_t spi_data[3];

    guard g;
    lnDigitalWrite(_pins->cs, false); // CS assert
    adr |= _W5500_SPI_READ_ | _W5500_SPI_VDM_OP_;
    spi_data[0] = (adr & 0x00FF0000) >> 16;
    spi_data[1] = (adr & 0x0000FF00) >> 8;
    spi_data[2] = (adr & 0x000000FF) >> 0;
    gSPI->blockWrite8(3, spi_data);  // address
    gSPI->read(rxlen, rxdata);       // read data
    lnDigitalWrite(_pins->cs, true); // CS de-assert
    return true;
}

// ---------------------------------------------------------------------------
// Socket TX/RX helpers (used by W5500LowLevel::readData)
// ---------------------------------------------------------------------------

/**
 * @brief Copy received data from the W5500 RX buffer and advance the RX read pointer.
 *
 * Reads @p len bytes from the W5500 internal RX memory at the current
 * Sn_RX_RD pointer into @p wizdata, then advances Sn_RX_RD by @p len.
 *
 * @param sn       Socket number (0-7).
 * @param wizdata  Destination buffer.
 * @param len      Number of bytes to read.
 */
static void wiz_recv_data(uint8_t sn, uint8_t *wizdata, uint16_t len)
{
    if (len == 0)
        return;
    shadow5Socket s(sn);
    s.readData((uint32_t)len, wizdata);
}

// ---------------------------------------------------------------------------
// W5500 coordinator task
// ---------------------------------------------------------------------------

/**
 * @brief FreeRTOS task that runs the W5500 network stack.
 *
 * Handles the DHCP state machine, fires the LwipReady callback once an IP
 * address is obtained, then enters a socket-polling loop that drives all
 * registered sockets.
 */

/** @brief Event flag: DHCP has assigned an IP address. */
#define W5_DHCP_EVT_ASSIGN 1

/**
 * @brief Event flag: W5500 INTR pin triggered.
 */
#define W5_INTR_EVT (1UL << 28)

class lnTaskW5500 : public lnTask
{
  public:
    lnTaskW5500(const char *name, uint32_t priority = 3, uint32_t taskSize = 100) : lnTask(name, priority, taskSize)
    {
        _state = W5_Idle;
    }
    void dispatchPerSocket(const uint8_t irq_socket_bitmap);
    virtual void run();

    enum W5500State
    {
        W5_Idle,
        W5_DHCPING,
        W5_ON,
    };
    W5500State _state;
    lnFastEventGroup _events;

    void invoke(uint32_t ev)
    {
        _events.setEvents(ev);
    }

    void intr()
    {
        invoke(W5_INTR_EVT);
    }
};

#define CHANGE_STATE(x)                                                                                                \
    {                                                                                                                  \
        _state = W5_##x;                                                                                               \
        Logger("Changing state to %d\n", _state);                                                                      \
    }

static lnTaskW5500 *_w500Task = NULL;

// ---------------------------------------------------------------------------
// DHCP callbacks
// ---------------------------------------------------------------------------

static void dhcp_ip_assign(void)
{
    _w500Task->invoke(W5_DHCP_EVT_ASSIGN);
}

static void dhcp_ip_update(void)
{
    Logger("ip update!\n");
}

static void dhcp_ip_conflict(void)
{
    Logger("DHCP: IP CONFLICT detected!\n");
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

void intr_pin_callback(lnPin pin, void *cookie)
{
    xAssert(_w500Task);
    if (_w500Task->_state == lnTaskW5500::W5_ON)
    {
        lnExtiDisableInterrupt(pin);
    }
    _w500Task->intr();
}

bool W5500LowLevel::init(int spiInstance, const lnW5500SPI *pins)
{
    _pins = pins;

    // Create the SPI bus mutex
    gSpiMutex = xSemaphoreCreateMutex();

    // Configure GPIOs for SPI function
    lnPinMode(_pins->mosi, lnSPI_MODE);
    lnPinMode(_pins->miso, lnSPI_MODE);
    lnPinMode(_pins->intr, lnINPUT_PULLUP);
    lnPinMode(_pins->clk, lnSPI_MODE);

    // Chip-select as GPIO output, initially de-asserted
    lnDigitalWrite(_pins->cs, true);
    lnPinMode(_pins->cs, lnOUTPUT);

    // Reset as GPIO output, initially de-asserted
    lnDigitalWrite(_pins->reset, true);
    lnPinMode(_pins->reset, lnOUTPUT);
    lnPinMode(_pins->intr, lnINPUT_PULLUP);
    lnDelay(5);
    // Attach falling-edge interrupt
    lnExtiAttachInterrupt(_pins->intr, LN_EDGE_FALLING, intr_pin_callback, NULL);

    // Create SPI instance
    gSPI = lnSPIBidir::createBiDir(spiInstance, -1);
    if (!gSPI)
        return false;

    // Configure SPI: 40 MHz, MSB first, MODE0
    gSPI->setDataMode(SPI_MODE0);
    gSPI->setSpeed(40 * 1000 * 1000);
    gSPI->setBitOrder(SPI_MSBFIRST);
    gSPI->begin();

    // Hardware reset via RST pin
    lnDigitalWrite(_pins->reset, false);
    lnDelayMs(10);
    lnDigitalWrite(_pins->reset, true);
    lnDelayMs(50);

    return true;
}

void W5500LowLevel::setMac(const uint8_t mac[6])
{
    memcpy(gMac, mac, 6);
}

static void print_mac(const uint8_t *mac)
{
    for (int i = 0; i < 6; i++)
        Logger("%02X ", mac[i]);
    Logger("\n");
}

bool W5500LowLevel::start(lnLwIpSysCallback cb, void *arg)
{
    gLwipCallback = cb;
    gLwipArg = arg;

    // Software reset: set MR_RST bit in MR register
    {
        uint8_t mr_rst = 0x80;
        write_to_reg(MR, 1, &mr_rst);
        lnDelayMs(10);
    }

    // Init socket buffer sizes
    for (int i = 0; i < 8; i++)
    {
        write_to_reg(Sn_TXBUF_SIZE(i), 1, &socket_txsize[i]);
        write_to_reg(Sn_RXBUF_SIZE(i), 1, &socket_rxsize[i]);
    }

    _w500Task = new lnTaskW5500("w5500", 5, 8 * 1024);
    _w500Task->start();
    return true;
}

int32_t W5500LowLevel::readData(uint8_t sn, uint8_t *buf, uint16_t maxSize)
{
    shadow5Socket s(sn);
    uint16_t rsvr = s.getReadAvailable();
    if (rsvr == 0)
        return 0;

    uint16_t toRead = (rsvr < maxSize) ? rsvr : maxSize;
    wiz_recv_data(sn, buf, toRead);
    // RECV command is now issued automatically inside shadow5Socket::readData()
    return (int32_t)toRead;
}

void W5500LowLevel::_registerSocket(lnSocket *s, uint8_t sn)
{
    if (sn < MAX_SOCKETS)
    {
        gActiveSockets[sn] = s;
        Logger("W5500LowLevel: registered socket %d\n", sn);
    }
    else
    {
        Logger("W5500LowLevel: SOCKET INDEX OVERFLOW socket %d\n", sn);
    }
}

void W5500LowLevel::_unregisterSocket(uint8_t sn)
{
    if (sn < MAX_SOCKETS)
    {
        gActiveSockets[sn] = nullptr;
        Logger("W5500LowLevel: unregistered socket %d\n", sn);
    }
    else
    {
        Logger("W5500LowLevel: SOCKET INDEX OVERFLOW socket %d\n", sn);
    }
}

// ---------------------------------------------------------------------------
// Sn_IR event dispatch
// ---------------------------------------------------------------------------
// The "SocketIgnore" entries means it is managed by the poll or elsewhere
static const lnSocketEvent kSnIrEventMap[8] = {
    SocketIgnore,        // SocketConnected,      // bit 0: CON
    SocketIgnore,        // SocketDisconnect,     // bit 1: DISCON
    SocketDataAvailable, // bit 2: RECV
    SocketError,         // bit 3: TIMEOUT
    SocketWriteAvailable, // bit 4: SEND_OK
    SocketIgnore,        // SocketError,          // bit 5: unused
    SocketIgnore,        // SocketError,          // bit 6: unused
    SocketIgnore,        // SocketError,          // bit 7: unused
};
/*
 * We have all the sockets pending interrupt as parameter, wake the threads attached to
 * those sockets.
 */
void lnTaskW5500::dispatchPerSocket(const uint8_t irq_socket_bitmap)
{
    for (int sn = 0; sn < MAX_SOCKETS; sn++)
    {
        if (irq_socket_bitmap & (1 << sn))
        {
            shadow5Socket s(sn);
            uint8_t sn_ir = s.getInterruptPending();
            if (sn_ir)
            {
                uint8_t clear_mask = sn_ir;
                if (clear_mask)
                    s.clearPendingInterrupt(clear_mask);

                if (sn_ir & Sn_IR_SENDOK)
                {
                    s.clearTxSending();
                }

                lnSocket_W5500 *skt = static_cast<lnSocket_W5500 *>(gActiveSockets[sn]);
                if (skt)
                {
                    for (int bit = 0; bit < 5; bit++)
                    {
                        lnSocketEvent event = kSnIrEventMap[bit];
                        if (sn_ir & (1 << bit) && (event != SocketIgnore))
                            skt->invoke(event);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// lnTaskW5500::run()
// ---------------------------------------------------------------------------

void lnTaskW5500::run()
{
    _events.takeOwnership();

    // Set MAC address
    w5500_set_mac(gMac);

    // Readback MAC
    uint8_t read_mac[6] = {0};
    w5500_get_mac(read_mac);

    Logger("Asked MAC ");
    print_mac(gMac);
    Logger("In use MAC ");
    print_mac(read_mac);

    if (memcmp(read_mac, gMac, 6))
    {
        xAssert(0);
    }

    // Configure PHY: software control, 100M Full Duplex
    wiz_PhyConf phy;
    phy.by = PHY_CONFBY_SW;
    phy.mode = PHY_MODE_MANUAL;
    phy.speed = PHY_SPEED_100;
    phy.duplex = PHY_DUPLEX_FULL;
    w5500_set_phyconf(&phy);

    lnDelayMs(200);
    w5500_get_phystat(&phy);
    Logger("\t Speed : %d (0: 10M, 1:100M)\n", phy.speed);
    Logger("\t Duplex : %d (0: Half, 1:Full)\n", phy.duplex);

    // Register DHCP callbacks
    DHCP_reg_cbfunc(dhcp_ip_assign, dhcp_ip_update, dhcp_ip_conflict);

    // Enable interrupts for all sockets
    shadow5System sys;
    sys.setSocketInterruptMask(0); // IK_SOCK_ALL);

    uint32_t counter = 0;
    lnExtiEnableInterrupt(_pins->intr);

    while (1)
    {
        uint32_t event = _events.waitEvents(W5_DHCP_EVT_ASSIGN | W5_INTR_EVT, 100);

        counter++;
        if (counter >= 10)
        {
            counter = 0;
            DHCP_time_handler();
        }

        // Read global SIR and dispatch per-socket interrupts
        {
#if 0
            shadow5System s;
            uint8_t irq_socket_bitmap = s.getPendingInterrupts();
            if (irq_socket_bitmap)
                dispatchPerSocket(irq_socket_bitmap);
#endif

            switch (_state)
            {
            case W5_Idle:
                Logger("DHCP started. Waiting for IP...\n");
                CHANGE_STATE(DHCPING);
                DHCP_init(DHCP_SOCKET);
                break;

            case W5_DHCPING: {
                enum DHCPReturn dhcp_ret = DHCP_run();
                if (dhcp_ret == DHCP_IP_ASSIGN || dhcp_ret == DHCP_IP_CHANGED)
                {
                    const DHCPInfo *info = DHCP_getInfo();
                    if (info)
                    {
                        memcpy(gNetInfo.ip, info->ip, 4);
                        memcpy(gNetInfo.gw, info->gw, 4);
                        memcpy(gNetInfo.sn, info->subnet, 4);
                        memcpy(gNetInfo.dns, info->dns, 4);
                        gNetInfo.dhcp = NETINFO_DHCP;
                        w5500_set_netinfo(&gNetInfo);
                        Logger("DHCP OK: \n\tIP %d.%d.%d.%d  \n\tGW %d.%d.%d.%d  \n\tSN %d.%d.%d.%d\n", gNetInfo.ip[0],
                               gNetInfo.ip[1], gNetInfo.ip[2], gNetInfo.ip[3], gNetInfo.gw[0], gNetInfo.gw[1],
                               gNetInfo.gw[2], gNetInfo.gw[3], gNetInfo.sn[0], gNetInfo.sn[1], gNetInfo.sn[2],
                               gNetInfo.sn[3]);

                        Logger("DHCP complete. Firing LwipReady callback.\n");
                        CHANGE_STATE(ON);
                        if (gLwipCallback)
                            gLwipCallback(LwipReady, gLwipArg);
                    }
                    else
                    {
                        Logger("DHCP failed. Restarting...\n");
                        CHANGE_STATE(Idle);
                    }
                }
                else if (dhcp_ret == DHCP_FAILED)
                {
                    Logger("DHCP failed. Restarting...\n");
                    CHANGE_STATE(Idle);
                }
                break;
            }

            case W5_ON: {
                shadow5System sys;
                uint8_t irq_socket_bitmap = sys.getPendingInterrupts();
                if (irq_socket_bitmap)
                {
                    dispatchPerSocket(irq_socket_bitmap);
                }

                // Poll all active sockets on every tick (timeout or interrupt)
                // to immediately detect and process state transitions (such as close handshake completion)
                for (int sn = 0; sn < MAX_SOCKETS; sn++)
                {
                    lnSocket_W5500 *skt = static_cast<lnSocket_W5500 *>(gActiveSockets[sn]);
                    if (skt)
                    {
                        skt->poll();
                    }
                }

                // Re-enable EXTI interrupt on the RP2040
                lnExtiEnableInterrupt(_pins->intr);

                // Check physical pin state to prevent missed edge transitions
                if (lnDigitalRead(_pins->intr) == false)
                {
                    // Pin is still low. Disable interrupt again and trigger our task.
                    lnExtiDisableInterrupt(_pins->intr);
                    _w500Task->intr();
                    // Yield the CPU for 1 ms to allow lower-priority application tasks
                    // (like socketRunner at Prio 3) to run and drain W5500 buffers,
                    // preventing priority starvation/livelocks.
                    lnDelayMs(1);
                }
                break;
            }
            }
        }
    }
}
// EOF
