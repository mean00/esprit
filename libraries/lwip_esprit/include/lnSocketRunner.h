#include "esprit.h"
//
#include "lnLWIP.h"

#define RUNNER_WRITE_BUFFER_SIZE 64
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
class socketRunner
{
  public:
#define BBITS(x) (1 << x)
    /**
     * @brief [TODO:description]
     */
    enum RunnerEvent
    {
        Up = BBITS(0),
        Down = BBITS(1),
        ConnectedServerSide = BBITS(2),
        ConnectedClientSide = BBITS(3),
        Disconnected = BBITS(8),
        DataAvailable = BBITS(9),
        CanWrite = BBITS(10),
        Error = BBITS(16),
    };
    /**
     * @brief [TODO:description]
     */
    socketRunner()
    {
        _connected = false;
        _writeBufferIndex = 0;
        lnLWIP::start(NetCb_c, this);
    }
    /**
     *
     * @param evt [TODO:parameter]
     * @param arg [TODO:parameter]
     */
    static void NetCb_c(lnLwipEvent evt, void *arg)
    {
        socketRunner *me = (socketRunner *)arg;
        me->sendEvent(Up);
    }
    /**
     * @brief [TODO:description]
     *
     * @param ev [TODO:parameter]
     */
    void sendEvent(RunnerEvent ev)
    {
        _eventGroup.setEvents(ev);
    }
    /**
     * @brief [TODO:description]
     *
     * @param evt [TODO:parameter]
     */
    void socketEvent(lnSocketEvent evt)
    {
        switch (evt)
        {
        case SocketConnectServer:
            sendEvent(ConnectedServerSide);
            break;
        case SocketConnectClient:
            sendEvent(ConnectedClientSide);
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
    /**
     * @brief [TODO:description]
     *
     * @param evt [TODO:parameter]
     * @param arg [TODO:parameter]
     */
    static void socketCb_c(lnSocketEvent evt, void *arg)
    {
        socketRunner *s = (socketRunner *)arg;
        s->socketEvent(evt);
    }
    void run()
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
    void process_events(uint32_t events)
    {
        // link up
        uint32_t local_event;
        //--
        BEGIN_EVENT(Up)
        Logger("Got link up event\n");
        lnDigitalWrite(DHCP_LED, true);
        cleanup();
        _current_connection = lnSocket::create(2000, socketCb_c, this);
        _current_connection->asyncMode();
        //_current_connection->accept();
        Logger("Server ready \n");
        END_EVENT()
        // link down
        BEGIN_EVENT(Down)
        Logger("Got link down event\n");
        cleanup();
        lnDigitalWrite(DHCP_LED, false);
        return;
        END_EVENT()
        //--
        BEGIN_EVENT(ConnectedServerSide)
        Logger("Got tcp connect server side\n");
        // that will trigger the client side
        xAssert(lnSocket::Ok == _current_connection->accept());
        //_current_connection->hook_working();
        END_EVENT()
        //--
        BEGIN_EVENT(ConnectedClientSide)
        Logger("Got tcp connect client side\n");
        _connected = true;
        END_EVENT()
        BEGIN_EVENT(Disconnected)
        Logger("Got disconnect \n");
        cleanup();
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
    virtual void process_incoming_data() = 0;

    /**
     * @brief [TODO:description]
     */
    virtual ~socketRunner()
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
    bool readData(uint32_t n, uint8_t *data, uint32_t &done)
    {
        if (lnSocket::Ok == _current_connection->read(n, data, done))
            return true;
        return false;
    }
    /**
     * @brief [TODO:description]
     *
     * @param n [TODO:parameter]
     * @param data [TODO:parameter]
     * @param done [TODO:parameter]
     * @return [TODO:return]
     */
    bool writeData(uint32_t n, uint8_t *data)
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
    bool flushWrite()
    {
        DEBUGME("Flushing write buffer (%d)\n", _writeBufferIndex);
        bool r = _forcedWrite(_writeBufferIndex, _writeBuffer);
        _writeBufferIndex = 0;
        return r;
    }

  protected:
    void cleanup()
    {
        _connected = false;
        if (_current_connection)
        {
            delete _current_connection;
            _current_connection = NULL;
        }
    }

  protected:
    void clearWrite()
    {
        _eventGroup.readEvents(CanWrite);
    }
    void waitForWrite()
    {
        _eventGroup.waitEvents(CanWrite, 100);
    }

    /**
     * @brief [TODO:description]
     *
     * @param n [TODO:parameter]
     * @param data [TODO:parameter]
     * @return [TODO:return]
     */
    bool _forcedWrite(uint32_t n, uint8_t *data)
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

  protected:
    lnFastEventGroup _eventGroup;
    lnSocket *_current_connection;
    bool _connected;
    uint8_t _writeBuffer[RUNNER_WRITE_BUFFER_SIZE];
    uint32_t _writeBufferIndex;
};
