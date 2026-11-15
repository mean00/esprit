/**
 * @file    socket_w5500.cpp
 * @brief   lnSocket_W5500 implementation.
 *
 * Bridges the lnSocket pure-virtual interface to the ioLibrary_Driver
 * C socket API.  The factory function lnSocket::create() is defined here
 * so that socketRunner can instantiate a W5500-backed socket without
 * any LWIP dependency.
 *
 * ## Interrupt-driven event model
 *
 * All socket event detection is interrupt-driven via
 * processSocketInterrupt().  The W5500 coordinator task (lnTaskW5500)
 * reads the global SIR register, masks the socket in SIMR, and signals
 * the per-socket task via SocketCustom / SocketWriteAvailable events.
 * processSocketInterrupt() then reads Sn_IR, clears bits, fires the
 * appropriate callbacks, and re-enables the socket in SIMR.
 *
 * ## TX state machine
 *
 * The write path uses a simple two-state model:
 *
 *   - **TxIdle**:    Data is copied to the TX buffer and Sn_CR_SEND is
 *                    issued.  _txState transitions to TxSending.
 *   - **TxSending**: write() returns done=0 (would-block).  The caller
 *                    (socketRunner::_forcedWrite()) waits for CanWrite
 *                    via waitForWrite().  When SEND_OK fires, the
 *                    coordinator dispatches SocketWriteAvailable +
 *                    SocketCustom.  The waitForWrite() override in the
 *                    application (EchoSocketRunner) processes CustomEvent
 *                    inline, calling processSocketInterrupt() which
 *                    transitions _txState back to TxIdle and fires
 *                    SocketWriteAvailable (CanWrite).
 *
 * This avoids a deadlock where _forcedWrite() is blocked waiting for
 * CanWrite, but CanWrite can only be set by processSocketInterrupt()
 * which is only reachable via CustomEvent dispatch from the main loop.
 *
 * @copyright (C) 2025
 * @license  See license file
 */

#include "socket_w5500.h"
#include "lnDebug.h"
#include "lowlevel_w5500.h"
#include "lowlevel_w5500_helper.h"
#include "socket_high_w5500.h"
#define debugME(...)                                                                                                   \
    {                                                                                                                  \
    }
// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** @brief Default RX buffer size (1.5 KB — fits one full TCP segment). */
static const uint16_t DEFAULT_RX_BUF_SIZE = 2048;

/** @brief W5500 socket flag: non-blocking I/O. */
static const uint8_t SOCK_FLAG = SF_IO_NONBLOCK;

/** @brief Auto-incrementing counter for allocating W5500 hardware socket numbers. */
static uint8_t nextSocket = 0;
// ---------------------------------------------------------------------------
// Factory: lnSocket::create()
// ---------------------------------------------------------------------------

/**
 * @brief Create a W5500-backed lnSocket instance.
 *
 * Allocates an lnSocket_W5500 and calls init() to open the hardware
 * socket and start listening.  Returns nullptr on failure (no leak).
 *
 * @param port  TCP port to listen on.
 * @param cb    Event callback.
 * @param arg   Opaque argument for the callback.
 * @return Pointer to the new lnSocket, or nullptr on failure.
 *
 * @note This function replaces the LWIP-based implementation when LWIP
 *       is not linked.
 */
lnSocket *lnSocket::create(uint16_t port, lnSocketCb cb, void *arg)
{
    lnSocket_W5500 *sock = new lnSocket_W5500(port, cb, arg);
    if (!sock->init())
    {
        Logger("lnSocket::create: failed to create socket\n");
        delete sock;
        return nullptr;
    }
    return sock;
}

// ---------------------------------------------------------------------------
// lnSocket_W5500 implementation
// ---------------------------------------------------------------------------

/**
 * @brief Construct a new lnSocket_W5500.
 *
 * Allocates the internal RX buffer (DEFAULT_RX_BUF_SIZE bytes).
 * The hardware socket is not opened until init() is called.
 *
 * @param port  TCP port to listen on.
 * @param cb    Event callback.
 * @param arg   Opaque argument for the callback.
 */
lnSocket_W5500::lnSocket_W5500(uint16_t port, lnSocketCb cb, void *arg)
    : _port(port), _cb(cb), _cbArg(arg), _state(StateError), _rxBuf(nullptr), _rxBufSize(0), _rxDataLen(0)
{
    _sn = nextSocket++;
    xAssert(nextSocket <= 4);
    _rxBuf = (uint8_t *)malloc(DEFAULT_RX_BUF_SIZE);
    if (_rxBuf)
        _rxBufSize = DEFAULT_RX_BUF_SIZE;
}

/**
 * @brief Destroy the lnSocket_W5500.
 *
 * Unregisters from the W5500 polling task, frees the RX buffer,
 * and closes the hardware socket.
 */
lnSocket_W5500::~lnSocket_W5500()
{
    // Unregister from the W5500 task
    W5500LowLevel::_unregisterSocket(_sn);

    if (_rxBuf)
        free(_rxBuf);
    _rxBuf = nullptr;
    close(_sn);
}

// ---------------------------------------------------------------------------
// init()
// ---------------------------------------------------------------------------

/**
 * @brief Open the hardware socket and start listening.
 *
 * Calls socket() with SF_IO_NONBLOCK, then listen().  Registers this
 * socket with W5500LowLevel so the polling task drives it.
 *
 * @retval true   Socket opened and listening.
 * @retval false  Failed to open or listen (socket already in use, etc.).
 */
bool lnSocket_W5500::init()
{
    // Open TCP socket in non-blocking mode
    if (socket(_sn, Sn_MR_TCP, _port, SOCK_FLAG) != (int8_t)_sn)
    {
        Logger("lnSocket_W5500::init: socket(%d) failed\n", _sn);
        return false;
    }

    // Start listening
    if (listen(_sn) != SOCK_OK)
    {
        Logger("lnSocket_W5500::init: listen(%d) failed\n", _sn);
        close(_sn);
        return false;
    }
    shadow5Socket s(_sn);
    s.setSocketInterruptMask(Sn_IR_RECV | Sn_IR_CON | Sn_IR_DISCON | Sn_IR_TIMEOUT | Sn_IR_SENDOK);

    _state = StateListening;
    W5500LowLevel::_registerSocket(this, _sn);
    Logger("lnSocket_W5500::init: socket %d listening on port %d\n", _sn, _port);
    return true;
}

// ---------------------------------------------------------------------------
// accept()
// ---------------------------------------------------------------------------

/**
 * @brief Re-accept after a disconnect.
 *
 * The W5500 hardware automatically accepts connections — we just need
 * to detect the state change in poll().  This method re-opens the socket
 * as a listener if it was previously connected and then disconnected.
 *
 * @return lnSocket::Ok on success, lnSocket::Error on failure.
 */
lnSocket::status lnSocket_W5500::accept()
{
    // The W5500 hardware automatically accepts connections — we just need
    // to detect the state change in poll().
    //
    // If the socket was previously connected and then disconnected, we
    // re-open it here.
    if (_state != StateListening)
    {
        // Re-open as listener
        close(_sn);
        int8_t ret = socket(_sn, Sn_MR_TCP, _port, SOCK_FLAG);
        if (ret != (int8_t)_sn)
            return lnSocket::Error;
        ret = listen(_sn);
        if (ret != SOCK_OK)
        {
            close(_sn);
            return lnSocket::Error;
        }
        shadow5Socket s(_sn);
        s.setSocketInterruptMask(Sn_IR_RECV | Sn_IR_CON | Sn_IR_DISCON | Sn_IR_TIMEOUT | Sn_IR_SENDOK);

        _state = StateListening;
        Logger("lnSocket_W5500::accept: re-listening on socket %d port %d\n", _sn, _port);
    }
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// read()
// ---------------------------------------------------------------------------

/**
 * @brief Read incoming data from the W5500 hardware socket.
 *
 * Uses W5500LowLevel::readData() which bypasses the buggy ioLibrary_Driver
 * recv() function (sock_io_mode check before data check).
 *
 * If data is available it is copied into the internal RX buffer and
 * @p data is set to point to it.  The caller must call freeReadData()
 * when done.
 *
 * @param[out] n     Number of bytes read (0 if no data).
 * @param[out] data  Pointer to the received data.
 * @return lnSocket::Ok (always — no-data is not an error).
 */
lnSocket::status lnSocket_W5500::read(uint32_t &n, uint8_t **data)
{
    n = 0;
    *data = nullptr;

    if (_state != StateConnected)
        return lnSocket::Ok;

    // Read data directly via W5500LowLevel (bypasses buggy ioLibrary recv())
    int32_t recvd = W5500LowLevel::readData(_sn, _rxBuf, _rxBufSize);
    if (recvd > 0)
    {
        _rxDataLen = (uint16_t)recvd;
        n = _rxDataLen;
        *data = _rxBuf;
        return lnSocket::Ok;
    }
    // recvd == 0 → no data available (not an error)
    // recvd < 0 → error (should not happen with our implementation)
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// freeReadData()
// ---------------------------------------------------------------------------

/**
 * @brief Release the data buffer returned by read().
 *
 * W5500LowLevel::readData() already advanced the RX read pointer and
 * issued the RECV command.  We just reset our tracking.
 *
 * @return lnSocket::Ok.
 */
lnSocket::status lnSocket_W5500::freeReadData()
{
    // W5500LowLevel::readData() already advanced the RX read pointer and
    // issued the RECV command.  We just reset our tracking.
    _rxDataLen = 0;
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// write()
// ---------------------------------------------------------------------------

/**
 * @brief Write data to the W5500 hardware socket.
 *
 * Checks TX_FSR before sending.  If the TX buffer is full (would block),
 * returns Ok with done=0 so the caller can retry later.
 *
 * @param[in]  n     Number of bytes to write.
 * @param[in]  data  Pointer to the data to send (const-qualified).
 * @param[out] done  Number of bytes actually written.
 * @return lnSocket::Ok on success (or would-block), lnSocket::Error on failure.
 */
lnSocket::status lnSocket_W5500::write(uint32_t n, const uint8_t *data, uint32_t &done)
{
    done = 0;

    // Guard: if poll() has already detected disconnection, reject writes
    // immediately.  Without this check, writeData() may still attempt to
    // send while the hardware socket is in SOCK_CLOSE_WAIT (which it
    // allows), causing repeated SEND failures and a tight error loop.
    if (_state != StateConnected)
    {
        return lnSocket::Error;
    }

    shadow5Socket s(_sn);
    int32_t sent = s.writeData(n, data); // the function already caps the size and checks FSR

    if (sent > 0)
    {
        done = (uint32_t)sent;
        return lnSocket::Ok;
    }

    if (sent == 0)
        return lnSocket::Ok; // try again later (TX buffer full or SEND_OK pending)

    Logger("lnSocket_W5500::write: send returned %ld\n", (long)sent);
    return lnSocket::Error;
}

// ---------------------------------------------------------------------------
// flush()
// ---------------------------------------------------------------------------

/**
 * @brief Flush the TX buffer.
 *
 * W5500 hardware sends data automatically after the SEND command
 * (which is issued inside send()).  Nothing to do here.
 *
 * @return lnSocket::Ok (always).
 */
lnSocket::status lnSocket_W5500::flush()
{
    // W5500 hardware sends data automatically after the SEND command
    // (which is issued inside send()).  Nothing to do here.
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// disconnectClient()
// ---------------------------------------------------------------------------

/**
 * @brief Disconnect the connected client.
 *
 * Sends a TCP FIN to the peer via the ioLibrary disconnect() function.
 * Transitions to StateClosing; poll() will detect SOCK_CLOSED later.
 *
 * @return lnSocket::Ok on success, lnSocket::Error if not connected.
 */
lnSocket::status lnSocket_W5500::disconnectClient()
{
    if (_state != StateConnected)
        return lnSocket::Ok;

    Logger("lnSocket_W5500::disconnectClient: disconnecting socket %d\n", _sn);
    disconnect(_sn);
    _state = StateClosing;
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// writeBufferAvailable()
// ---------------------------------------------------------------------------

/**
 * @brief Query the available TX buffer space.
 *
 * @param[out] n  Number of bytes that can be written without blocking.
 * @return lnSocket::Ok (always).
 */
lnSocket::status lnSocket_W5500::writeBufferAvailable(uint32_t &n)
{
    if (_state != StateConnected)
    {
        n = 0;
        return lnSocket::Ok;
    }
    shadow5Socket s(_sn);
    n = s.getTxFreeBufferSize();
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// asyncMode()
// ---------------------------------------------------------------------------

/**
 * @brief Enable asynchronous (non-blocking) mode.
 *
 * The socket is always non-blocking — no-op.
 *
 * @return lnSocket::Ok (always).
 */
lnSocket::status lnSocket_W5500::asyncMode()
{
    // Always non-blocking — no-op.
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// invoke()
// ---------------------------------------------------------------------------

/**
 * @brief Fire a socket event callback.
 *
 * @param evt  The event to report (SocketConnected, SocketDisconnect, etc.).
 * @return lnSocket::Ok (always).
 */
lnSocket::status lnSocket_W5500::invoke(lnSocketEvent evt)
{
    if (_cb)
        _cb(evt, _cbArg);
    return lnSocket::Ok;
}

// ---------------------------------------------------------------------------
// poll()
// ---------------------------------------------------------------------------

/**
 * @brief Poll the hardware socket status and drive the state machine.
 *
 * Called periodically by the W5500 polling task (lnTaskW5500).
 * Detects connection establishment, disconnection, incoming data,
 * and TX buffer availability.
 *
 * @note Events generated here are delivered on the *next* call to
 *       socketRunner::process_events() — one scan-cycle latency.
 */
void lnSocket_W5500::poll()
{
    shadow5Socket s(_sn);
    uint8_t status = s.getStatus();

    switch (_state)
    {
    case StateListening:
        // Waiting for a client to connect
        if (status == SOCK_ESTABLISHED)
        {
            _state = StateConnected;
            Logger("lnSocket_W5500::poll: socket %d connected\n", _sn);
            invoke(SocketConnected);
        }
        else if (status == SOCK_CLOSED)
        {
            // Socket closed unexpectedly (e.g. timeout while listening)
            Logger("lnSocket_W5500::poll: socket %d closed while listening\n", _sn);
            _state = StateError;
            invoke(SocketError);
        }
        break;

    case StateConnected: {
        // Check for disconnection
        if (status == SOCK_CLOSED || status == SOCK_CLOSE_WAIT)
        {
            Logger("lnSocket_W5500::poll: socket %d disconnected (status=0x%02x)\n", _sn, status);
            // Transition immediately to StateClosing so that:
            //   - Subsequent poll() calls won't re-fire SocketDisconnect
            //   - write() will return Error (guarded by _state != StateConnected)
            //   - read() will return no data
            _state = StateClosing;
            // Only send FIN if peer hasn't already closed (SOCK_CLOSE_WAIT means
            // peer sent FIN first, so we just need to close locally).
            if (status == SOCK_CLOSE_WAIT)
            {
                disconnect(_sn);
            }
            else
            {
                close(_sn);
            }
            invoke(SocketDisconnect);
            break;
        }
#if 0
        // Check for incoming data
        uint16_t rsvr = s.getReadAvailable();
        if (rsvr > 0)
        {
            invoke(SocketDataAvailable);
            debugME("Data available on socket %d, size = %d\n", _sn, rsvr);
        }

        // Check for write space available
        uint16_t freesize = s.getTxFreeBufferSize();
        if (freesize > 0)
        {
            // Logger("write available %d bytes \n", freesize);
            invoke(SocketWriteAvailable);
            // FIXME
#warning FIXME
        }
#endif
    }
    break;

    case StateClosing:
        // Waiting for hardware to fully close
        if (status == SOCK_CLOSED)
        {
            Logger("lnSocket_W5500::poll: socket %d closed after disconnect, re-listening\n", _sn);
            accept(); // re-opens as listener
        }
        break;

    case StateError:
        // Socket is dead — should not happen with auto re-listen in StateClosing.
        break;
    }
}
// EOF
