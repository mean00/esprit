#pragma once

enum lnLwipEvent
{
    lnLwipEventNone,
    LwipDown,
    LwipReady
};
typedef void (*lnLwIpSysCallback)(lnLwipEvent evt, void *arg);

class lnLWIP
{
  public:
    static bool start(lnLwIpSysCallback cb, void *arg);
};
enum lnSocketEvent
{

    SocketDisconnect,
};
typedef void (*lnSocketCb)(lnSocketEvent evt, void *arg);
class lnSocket
{
  public:
    enum status
    {
        Ok,
        Error,
    };
    virtual ~lnSocket() {};
    static lnSocket *create(uint16_t port, lnSocketCb cb, void *arg);
    virtual status write(uint32_t n, const uint8_t *data, uint32_t &done) = 0;
    virtual status read(uint32_t n, uint8_t *data, uint32_t &done) = 0;
    virtual status flush() = 0;
    virtual status close() = 0;
    virtual status accept() = 0;

  protected:
    lnSocket() {};

  protected:
};
