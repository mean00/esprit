#include "esprit.h"
//
#include "lnLWIP.h"
//
#include "lnSocketRunner.h"
#include "modules/socket_w5500.h"

//
#undef DEBUGME
#if 0
#define DEBUGME Logger
#else
#define DEBUGME(...)                                                                                                   \
    {                                                                                                                  \
    }
#endif
/**

*/
/**
 * @brief [TODO:description]
 */
socketRunner::socketRunner(uint16_t port, lnFastEventGroup &eventGroup, uint32_t shift) : _eventGroup(eventGroup)
{
    _shift = shift;
    _current_connection = NULL;
    _connected = false;
    _writeBufferIndex = 0;
    _port = port;
}

/**
 * @brief [TODO:description]
 *
 * @param evt [TODO:parameter]
 */
void socketRunner::socketEvent(lnSocketEvent evt)
{
    switch (evt)
    {
    case SocketConnected:
        sendEvent(Connected);
        break;
    case SocketDisconnect:
        sendEvent(Disconnected);
        break;
    case SocketDataAvailable:
        sendEvent(DataAvailable);
        break;
    case SocketWriteAvailable:
        sendEvent(CanWrite);
        break;
    case SocketError:
        sendEvent(Error);
        break;
    case SocketCustom:
        sendEvent(CustomEvent);
        break;
    default:
        xAssert(0);
    }
}

#define BEGIN_EVENT(e)                                                                                                 \
    if (events & e)                                                                                                    \
    {                                                                                                                  \
        local_event = e;
#define END_EVENT()                                                                                                    \
    events &= ~local_event;                                                                                            \
    }
/**
 * @brief [TODO:description]
 *
 * @param events [TODO:parameter]
 */
void socketRunner::process_events(uint32_t events)
{
    hook_poll();
    events &= ~CustomEvent; // managed before
    // link up
    uint32_t local_event;
    //--
    BEGIN_EVENT(Up)
    Logger(">>>>>>>>>>>>>>Got link up event\n");
    Logger(">>>>>>>>>>>>>>Got link up event\n");
    Logger(">>>>>>>>>>>>>>Got link up event\n");
    cleanup();
    _current_connection = lnSocket::create(_port, socketCb_c, this);
    _current_connection->asyncMode();
    _current_connection->accept();
    Logger("Server ready \n");
    END_EVENT()
    // link down
    BEGIN_EVENT(Down)
    Logger("Got link down event\n");
    cleanup();
    return;
    END_EVENT()
    //--
    BEGIN_EVENT(Disconnected)
    Logger("Got disconnect \n");
    _connected = false;
    hook_disconnected();
    disconnectClient();
    END_EVENT()
    //--
    BEGIN_EVENT(Connected)
    Logger("Got tcp connect \n");
    _connected = true;
    hook_connected();
    END_EVENT()
    //--
    BEGIN_EVENT(DataAvailable)
    if (_connected)
    {
        process_incoming_data();
    }
    else
    {
        Logger("Warning: DataAvailable while not connected, discarding\n");
    }
    END_EVENT()
    //--
    if (!_connected && events != 0)
    {
        Logger("Warning: not connected and got event 0x%x\n", events);
        // we may get some leftovers can write or data available events
    }
}
/**
 * @brief [TODO:description]
 */
void socketRunner::disconnectClient()
{
    _current_connection->disconnectClient();
}

/**
 * @brief [TODO:description]
 */
socketRunner::~socketRunner()
{
}
/**
 * @brief [TODO:description]
 *
 * @param n [TODO:parameter]
 * @param data [TODO:parameter]
 * @param done [TODO:parameter]
 * @return [TODO:return]
 */
bool socketRunner::readData(uint32_t &n, uint8_t **data)
{
    lnSocket::status r;
    r = _current_connection->read(n, data);
    if (r != lnSocket::Ok)
    {
        n = 0;
        return true;
    }
    return true;
}

/**
 * @brief [TODO:description]
 *
 * @return [TODO:return]
 */
bool socketRunner::releaseData()
{
    _current_connection->freeReadData();
    return true;
}
/**
 * @brief [TODO:description]
 *
 * @param n [TODO:parameter]
 * @param data [TODO:parameter]
 * @param done [TODO:parameter]
 * @return [TODO:return]
 */
bool socketRunner::writeData(uint32_t n, const uint8_t *data)
{
    if (_writeBufferIndex + n < RUNNER_WRITE_BUFFER_SIZE)
    {
        DEBUGME("Buffering %d bytes\n", n);
        memcpy(_writeBuffer + _writeBufferIndex, data, n);
        _writeBufferIndex += n;
        return true;
    }
    if (_writeBufferIndex != 0)
    {
        DEBUGME("Flushing before writing\n");
        flushWrite();
    }
    DEBUGME("Actually writing %d bytes\n", n);
    bool r = _forcedWrite(n, data);
    DEBUGME("Write done\n");
    return r;
}
/**
 * @brief [TODO:description]
 *
 * @return [TODO:return]
 */
bool socketRunner::flushWrite()
{
    DEBUGME("Flushing write buffer (%d)\n", _writeBufferIndex);
    bool r = _forcedWrite(_writeBufferIndex, _writeBuffer);
    _writeBufferIndex = 0;
    return r;
}

/**
 * @brief [TODO:description]
 */
void socketRunner::cleanup()
{
    _connected = false;
    if (_current_connection)
    {
        delete _current_connection;
        _current_connection = NULL;
    }
}

/**
 * @brief [TODO:description]
 *
 * @param n [TODO:parameter]
 * @param data [TODO:parameter]
 * @return [TODO:return]
 */
uint32_t socketRunner::writeBufferAvailable()
{
    uint32_t n = 0;
    if (lnSocket::Ok == _current_connection->writeBufferAvailable(n))
        return n;
    return 0;
}
/**
 * @brief Wait for the socket to become writable (default implementation).
 *
 * Simply waits for the CanWrite event.  Subclasses that need to process
 * pending interrupts inline (e.g. W5500 SEND_OK) should override this.
 */
void socketRunner::waitForWrite()
{
    _eventGroup.waitEvents(CanWrite << _shift, 100);
}

/**
 * @brief [TODO:description]
 *
 * @param n [TODO:parameter]
 * @param data [TODO:parameter]
 * @return [TODO:return]
 */
bool socketRunner::_forcedWrite(uint32_t n, const uint8_t *data)
{
    uint32_t done;
    DEBUGME("_forcedWrite %d byte\n", n);
    while (n > 0)
    {
        done = 0;
        DEBUGME("  WR!!\n");
        lnSocket::status s = _current_connection->write(n, data, done);
        if (lnSocket::Ok != s)
        {
            Logger("** SOCKET WRITE ERROR 0x:%x\n", s);
            return false;
        }
        if (!done)
        {
            DEBUGME("Waiting for Wr\n");
            waitForWrite();
            DEBUGME("Got canwrite \n");
        }
        DEBUGME("Wr OK\n");
        n -= done;
        data += done;
    }
    return true;
}
// EOF
