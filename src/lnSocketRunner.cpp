#include "esprit.h"
//
#include "lnLWIP.h"
//
#include "lnSocketRunner.h"

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
socketRunner::socketRunner()
{
    _connected = false;
    _writeBufferIndex = 0;
    lnLWIP::start(NetCb_c, this);
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
    default:
        xAssert(0);
    }
}

void socketRunner::run()
{
    _eventGroup.takeOwnership();
    while (1)
    {
        uint32_t events = _eventGroup.waitEvents(~CanWrite, 100);
        process_events(events);
    }
}
#define BEGIN_EVENT(e)                                                                                                 \
    if (events & e)                                                                                                    \
    {                                                                                                                  \
        local_event = e;
#define END_EVENT()                                                                                                    \
    events &= ~local_event;                                                                                            \
    }
void socketRunner::process_events(uint32_t events)
{
    // link up
    uint32_t local_event;
    //--
    BEGIN_EVENT(Up)
    Logger("Got link up event\n");
    cleanup();
    _current_connection = lnSocket::create(2000, socketCb_c, this);
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
    disconnectClient();
    END_EVENT()
    //--
    BEGIN_EVENT(Connected)
    Logger("Got tcp connect \n");
    _connected = true;
    END_EVENT()
    //--
    BEGIN_EVENT(DataAvailable)
    process_incoming_data();
    END_EVENT()
    if (!_connected && events != 0)
    {
        Logger("Warning: not connected and got event 0x%x\n", events);
        xAssert(0);
    }
}
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
    DEBUGME("Flushing before writing\n");
    flushWrite();
    DEBUGME("Actually writing %d bytes\n", n);
    return _forcedWrite(n, data);
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
bool socketRunner::_forcedWrite(uint32_t n, const uint8_t *data)
{
    uint32_t done;
    DEBUGME("  Forcing  write %d byte\n", n);
    while (n > 0)
    {
        done = 0;
        clearWrite();
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
            DEBUGME("Wr OK\n");
        }
        n -= done;
        data += done;
    }
    return true;
}
// EOF
