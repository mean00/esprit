/**
 * @file
 * @brief [TODO:description]
 */

#pragma once

enum lnLwipEvent
{
    lnLwipEventNone,
    LwipDown,
    LwipReady
};
typedef void (*lnLwIpSysCallback)(lnLwipEvent evt, void *arg);

/**
 * @class lnLWIP
 * @brief [TODO:description]
 *
 */
class lnLWIP
{
  public:
    static bool start(lnLwIpSysCallback cb, void *arg);
};
enum lnSocketEvent
{
    SocketConnected, // listening socket connection notification ,you should accept
    SocketDisconnect,
    SocketDataAvailable,
    SocketWriteAvailable,
    SocketError,
};
typedef void (*lnSocketCb)(lnSocketEvent evt, void *arg);
/**
 * @class lnSocket
 * @brief [TODO:description]
 *
 */
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
    virtual status read(uint32_t &n, uint8_t **data)=0;
    virtual status invoke(lnSocketEvent evt);
    virtual status flush() = 0;
    virtual status disconnectClient() = 0;
    virtual status asyncMode() = 0;
    virtual status accept() = 0;
    virtual status freeReadData() =0;

  protected:
    lnSocket() {};

  protected:
};
// EOF
