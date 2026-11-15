/**
 * @file    socket_w5500.h
 * @brief   lnSocket implementation backed by the W5500 hardware TCP engine.
 *
 * Provides an lnSocket subclass that wraps the ioLibrary_Driver C socket API
 * (socket(), listen(), send(), recv(), ...) so that socketRunner can operate
 * on top of the W5500 without any LWIP dependency.
 *
 * All socket event detection is interrupt-driven via
 * lnSocket_W5500::processSocketInterrupt().  The W5500 coordinator task
 * reads the global SIR register, masks the socket in SIMR, and signals
 * the per-socket task via SocketCustom / SocketWriteAvailable events.
 * processSocketInterrupt() then reads Sn_IR, clears bits, fires the
 * appropriate callbacks, and re-enables the socket in SIMR.
 *
 * The TX state machine uses a simple two-state model:
 *   - TxIdle:    Data is copied to the TX buffer and Sn_CR_SEND is issued.
 *   - TxSending: write() returns done=0 (would-block).  The caller
 *                (socketRunner::_forcedWrite()) waits for CanWrite via
 *                waitForWrite().  When SEND_OK fires, the coordinator
 *                dispatches SocketWriteAvailable + SocketCustom, and the
 *                waitForWrite() override processes CustomEvent inline to
 *                transition _txState back to TxIdle.
 *
 * Usage:
 * @code
 *   // Create an lnSocket via the factory (called by socketRunner)
 *   lnSocket *sock = lnSocket::create(port, callback, arg);
 * @endcode
 *
 * @copyright (C) 2025
 * @license  See license file
 */
#pragma once
#include "esprit.h"
#include "lnLWIP.h"
#include <stdint.h>

/**
 * @brief lnSocket implementation for the W5500 hardware TCP engine.
 *
 * Manages one W5500 hardware socket slot (0-7) in TCP server mode.
 * State machine: IDLE -> LISTEN -> CONNECTED -> ERROR
 *
 * The user must call poll() (or W5500Esprit::pollSocket()) periodically
 * from hook_poll() to detect state transitions and generate callbacks.
 */
class lnSocket_W5500 : public lnSocket
{
  public:
    /**
     * @brief Construct a W5500-backed socket (cannot fail).
     *
     * Allocates the RX buffer and initialises members.  Does NOT touch
     * the W5500 hardware — call init() after construction to open the
     * hardware socket and start listening.
     * @param port  TCP port to listen on.
     * @param cb    Callback for socket events.
     * @param arg   Opaque argument passed to the callback.
     */
    lnSocket_W5500(uint16_t port, lnSocketCb cb, void *arg);

    /**
     * @brief Destructor — closes the W5500 socket.
     */
    virtual ~lnSocket_W5500();

    // ---- lnSocket pure virtual interface ----

    virtual lnSocket::status write(uint32_t n, const uint8_t *data, uint32_t &done) override;
    virtual lnSocket::status read(uint32_t &n, uint8_t **data) override;
    virtual lnSocket::status invoke(lnSocketEvent evt) override;
    virtual lnSocket::status flush() override;
    virtual lnSocket::status disconnectClient() override;
    virtual lnSocket::status asyncMode() override;
    virtual lnSocket::status accept() override;
    virtual lnSocket::status freeReadData() override;
    virtual lnSocket::status writeBufferAvailable(uint32_t &n) override;

    /**
     * @brief Initialise the W5500 hardware socket and start listening.
     *
     * Opens the W5500 hardware socket in TCP server mode and calls
     * listen().  Must be called once after construction.
     * @retval true   Initialisation succeeded.
     * @retval false  Initialisation failed (socket() or listen() error).
     */
    bool init();

    // ---- W5500-specific ----

    /**
     * @brief Poll the W5500 socket state machine.
     *
     * Must be called periodically (e.g. from hook_poll()) to detect:
     *   - Connection established (LISTEN -> ESTABLISHED)
     *   - Data available (RX_RSR > 0)
     *   - Disconnection / timeout
     *
     * @note Events generated here are delivered on the *next* call to
     *       socketRunner::process_events() — one scan-cycle latency.
     *       This is acceptable for typical embedded TCP use-cases but
     *       should be revisited if tight latency is required.
     */
    void poll();

    /**
     * @brief Get the W5500 hardware socket number.
     * @return Socket number (0-7).
     */
    uint8_t getSn() const
    {
        return _sn;
    }

    /**
     * @brief Check if a client is currently connected.
     * @retval true   Client connected (ESTABLISHED).
     * @retval false  No client connected.
     */
    bool isConnected() const
    {
        return _state == StateConnected;
    }

  private:
    /**
     * @brief Reset the hardware socket state for re-use.
     *
     * Drains any stale RX data left in the hardware buffer from the
     * previous connection and clears all pending Sn_IR interrupt flags.
     * Must be called after re-listen (accept()) and on unexpected close
     * while listening to prevent stale data/events from firing.
     */
    void resetSocket();

    /** @brief Single-client state machine for the W5500 hardware socket. */
    enum SocketState : uint8_t
    {
        StateListening, /**< listen() called, waiting for client connection. */
        StateConnected, /**< SOCK_ESTABLISHED, client connected. */
        StateClosing,   /**< disconnect() called, waiting for HW to close. */
        StateError      /**< Socket error occurred (e.g. SOCK_CLOSED while listening). */
    };

    uint8_t _sn;         /**< W5500 hardware socket number (0-7). */
    uint16_t _port;      /**< TCP port number. */
    lnSocketCb _cb;      /**< Event callback. */
    void *_cbArg;        /**< Opaque callback argument. */
    SocketState _state;  /**< Current state of the socket state machine. */
    uint8_t *_rxBuf;     /**< Dynamically allocated RX buffer. */
    uint16_t _rxBufSize; /**< Size of the RX buffer. */
    uint16_t _rxDataLen; /**< Number of valid bytes in _rxBuf. */
};

// ---- Factory declaration (defined in w5500_socket.cpp) ----
// lnSocket::create() is declared in lnLWIP.h and must be defined
// exactly once in the project.  We provide it here.
