#include "esprit.h"
//
#include "lnLWIP.h"

#define RUNNER_WRITE_BUFFER_SIZE 64
/**

*/

class socketRunner
{
  public:
#define BBITS(x) (1UL << x)
    /**
     * @brief [TODO:description]
     */
    enum RunnerEvent
    {
        Up = BBITS(30),
        Down = BBITS(31),
        Connected = BBITS(0),
        Disconnected = BBITS(1),
        DataAvailable = BBITS(2),
        CanWrite = BBITS(3),
        Error = BBITS(4),
        Mask = (0x1F),
    };
    /**
     * @brief [TODO:description]
     */
    socketRunner(lnFastEventGroup &eventGroup, uint32_t shift);
    /*
     *
     *
     */
    virtual void hook_connected() = 0;
    virtual void hook_disconnected() = 0;
    virtual void hook_poll() = 0;
    /**
     * @brief [TODO:description]
     *
     * @param ev [TODO:parameter]
     */
    void sendEvent(RunnerEvent ev)
    {
        uint32_t v = ev;
        if (ev < Up)
        {
            v = v << _shift;
        }
        _eventGroup.setEvents(v);
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
        _eventGroup.readEvents(CanWrite << _shift);
    }
    void waitForWrite()
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
    bool _forcedWrite(uint32_t n, const uint8_t *data);

  protected:
    lnFastEventGroup &_eventGroup;
    lnSocket *_current_connection;
    bool _connected;
    uint8_t _writeBuffer[RUNNER_WRITE_BUFFER_SIZE];
    uint32_t _writeBufferIndex;
    uint32_t _shift;
};

// EOF
