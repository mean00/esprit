#include "esprit.h"
//
#include "lnLWIP.h"

#define RUNNER_WRITE_BUFFER_SIZE 64
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
        Connected = BBITS(3),
        Disconnected = BBITS(8),
        DataAvailable = BBITS(9),
        CanWrite = BBITS(10),
        Error = BBITS(16),
    };
    /**
     * @brief [TODO:description]
     */
    socketRunner();

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
    void socketEvent(lnSocketEvent evt);

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
    void run();

    void process_events(uint32_t events);

    void disconnectClient();

    virtual void process_incoming_data() = 0;

    /**
     * @brief [TODO:description]
     */
    virtual ~socketRunner();
    /**
     * @brief [TODO:description]
     *
     * @param n [TODO:parameter]
     * @param data [TODO:parameter]
     * @param done [TODO:parameter]
     * @return [TODO:return]
     */
    bool readData(uint32_t &n, uint8_t **data);
    bool releaseData();
    /**
     * @brief [TODO:description]
     *
     * @param n [TODO:parameter]
     * @param data [TODO:parameter]
     * @param done [TODO:parameter]
     * @return [TODO:return]
     */
    bool writeData(uint32_t n, const uint8_t *data);

    /**
     * @brief [TODO:description]
     *
     * @return [TODO:return]
     */
    bool flushWrite();

  protected:
    void cleanup();

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
    bool _forcedWrite(uint32_t n, const uint8_t *data);

  protected:
    lnFastEventGroup _eventGroup;
    lnSocket *_current_connection;
    bool _connected;
    uint8_t _writeBuffer[RUNNER_WRITE_BUFFER_SIZE];
    uint32_t _writeBufferIndex;
};

// EOF
