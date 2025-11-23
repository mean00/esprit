/*
 *
 *
 */
#include "esprit.h"
extern "C"
{
#include "lwip/api.h"
#include "lwip/dhcp.h"
#include "lwip/init.h"
#include "lwip/sys.h"
#include "lwip/tcpip.h"
}
#include "lnLWIP.h"
struct netif lwip_netif;
NETIF_DECLARE_EXT_CALLBACK(netif_callback);

lnLwIpSysCallback _syscb = NULL;
void *_sysarg = NULL;

static void netif_ext_callback_esprit(struct netif *netif, netif_nsc_reason_t reason,
                                      const netif_ext_callback_args_t *args);
static void lwip_status_change_callback(struct netif *netif);

#if 0
#define DEBUGME Logger
#else
#define DEBUGME(...)                                                                                                   \
    {                                                                                                                  \
    }
#endif

/**
 * @brief [TODO:description]
 */
void lnwip_common_init(lnLwIpSysCallback syscb, void *arg)

{
    _syscb = syscb;
    _sysarg = arg;
    netif_set_default(&lwip_netif);
    netif_set_link_callback(&lwip_netif, lwip_status_change_callback);
    netif_add_ext_callback(&netif_callback, netif_ext_callback_esprit);
    netif_set_up(&lwip_netif);
    dhcp_start(&lwip_netif);
}

/**
 *
 *
 */
void lwip_status_change_callback(struct netif *netif)
{
    Logger("Status change\n");
}
/**
 *
 */
void netif_ext_callback_esprit(struct netif *netif, netif_nsc_reason_t reason, const netif_ext_callback_args_t *args)
{
    Logger(" Callback : 0x%x\n", reason);
    uint32_t reduced = reason & 0xff;
    if (reduced & LWIP_NSC_IPV4_ADDRESS_CHANGED)
    {
        if (reason & LWIP_NSC_IPV4_ADDR_VALID)
        {
            uint32_t a = netif->ip_addr.addr;
            Logger(" IP : %d.%d.%d.%d\n", a & 0xff, (a >> 8) & 0xff, (a >> 16) & 0xff, (a >> 24));
            _syscb(LwipReady, _sysarg);
        }
        else
        {
            _syscb(LwipDown, _sysarg);
        }
        return;
    }
}
#define LNSOCKET_WRITE_BUFFER_SIZE 256
/**
 * @class lnSocket_impl
 * @brief [TODO:description]
 *
 */
class lnSocket_impl : public lnSocket
{
  public:
    lnSocket_impl(lnSocketCb cb, void *arg);
    virtual ~lnSocket_impl();
    status bind(uint16_t port);
    status read(uint32_t n, uint8_t *data, uint32_t &done);
    status accept();
    status write(uint32_t n, const uint8_t *data, uint32_t &done);
    status flush(); // force flushing the write buffer
    status close();

    enum conn_state
    {
        CON_IDLE = 0,
        CON_LISTEN = 1,
        CON_WORK = 2
    };

  protected:
    void disconnect();

  protected:
    struct netconn *_conn;
    struct netconn *_work_conn;
    conn_state _state;
    struct netbuf *_receiveBuf;
    uint32_t _receiveIndex;
    uint8_t _writeBuffer[LNSOCKET_WRITE_BUFFER_SIZE];
    uint32_t _writeBufferIndex;
    lnSocketCb _sockCb;
    void *_cbArg;
};

/**
 * @brief [TODO:description]
 *
 * @param port [TODO:parameter]
 * @return [TODO:return]
 */
lnSocket *lnSocket::create(uint16_t port, lnSocketCb cb, void *arg)
{
    lnSocket_impl *s = new lnSocket_impl(cb, arg);
    if (s->bind(port) != lnSocket::Ok)
    {
        Logger("Cannot bind to port %u\n", port);
        delete s;
        return NULL;
    }
    return s;
}
/**
 *
 */
lnSocket_impl::lnSocket_impl(lnSocketCb cb, void *arg) : lnSocket()
{
    _conn = NULL;
    _work_conn = NULL;
    _state = CON_IDLE;
    _receiveBuf = NULL;
    _writeBufferIndex = 0;
    _receiveIndex = 0;
    _cbArg = arg;
    _sockCb = cb;
}
/**
 * @brief [TODO:description]
 */
lnSocket_impl::~lnSocket_impl()
{
    close();
    if (_conn)
    {
        netconn_close(_conn);
        netconn_delete(_work_conn);
        _conn = NULL;
    }
    _state = CON_IDLE;
}
/**
 * @brief [TODO:description]
 */
lnSocket::status lnSocket_impl::close()
{
    if (_work_conn)
    {
        netconn_close(_work_conn);
        netconn_delete(_work_conn);
        _work_conn = NULL;
        _state = CON_LISTEN;
    }
    if (_receiveBuf)
    {
        netbuf_delete(_receiveBuf);
    }
    _receiveIndex = 0;
    return lnSocket::Ok;
}
/**
 * @brief [TODO:description]
 *
 * @param port [TODO:parameter]
 * @return [TODO:return]
 */
lnSocket::status lnSocket_impl::bind(uint16_t port)
{
    _conn = netconn_new(NETCONN_TCP);
    if (!_conn)
        return lnSocket::Error;
    if (ERR_OK != netconn_bind(_conn, IP_ADDR_ANY, port))
    {
        return lnSocket::Error;
    }
    if (ERR_OK != netconn_listen(_conn))
    {
        return lnSocket::Error;
    }
    _state = CON_LISTEN;
    return lnSocket::Ok;
}
/**
 * @brief [TODO:description]
 *
 * @return [TODO:return]
 */
lnSocket::status lnSocket_impl::accept()
{
    xAssert(_state == CON_LISTEN);
    if (netconn_accept(_conn, &_work_conn) != ERR_OK)
    {
        return lnSocket::Error;
    }
    _state = CON_WORK;
    _writeBufferIndex = 0;
    _receiveIndex = 0;
    return lnSocket::Ok;
}
/**
 * @brief [TODO:description]
 *
 * @param n [TODO:parameter]
 * @param data [TODO:parameter]
 * @param done [TODO:parameter]
 * @return [TODO:return]
 */
// FIXME [TODO  iomplement multi read]
lnSocket::status lnSocket_impl::read(uint32_t n, uint8_t *data, uint32_t &done)
{
    xAssert(_state == CON_WORK);
    //
again:
    if (_receiveBuf)
    {
        uint32_t buf_len = _receiveBuf->p->len;
        xAssert(_receiveIndex < buf_len);
        uint32_t avail = buf_len - _receiveIndex;
        DEBUGME("avail %d, asked  %d\n", avail, n);
        uint32_t mx = avail;
        if (mx > n)
        {
            mx = n;
        }
        DEBUGME("processing %d bytes \n", mx);
        memcpy(data, (uint8_t *)_receiveBuf->p->payload + _receiveIndex, mx);
        done = mx;
        _receiveIndex += mx;
        if (_receiveIndex == buf_len)
        {
            netbuf_delete(_receiveBuf);
            _receiveBuf = NULL;
        }
        DEBUGME("read ok  \n");
        return lnSocket::Ok;
    }
    _receiveIndex = 0;
    if (ERR_OK != netconn_recv(_work_conn, &_receiveBuf))
    {
        disconnect();
        return lnSocket::Error;
    }
    DEBUGME("Receive packet of size  %d\n", _receiveBuf->p->len);
    goto again;
}
/**
 * @brief [TODO:description]
 *
 * @param n [TODO:parameter]
 * @param data [TODO:parameter]
 * @param done [TODO:parameter]
 * @return [TODO:return]
 */
lnSocket::status lnSocket_impl::write(uint32_t n, const uint8_t *data, uint32_t &done)
{
    xAssert(_state == CON_WORK);
    DEBUGME("write %d bytes   \n", n);
    size_t w = 0;
    // does it fit in write buffer ?
    if ((_writeBufferIndex + n) < LNSOCKET_WRITE_BUFFER_SIZE)
    {
        memcpy(_writeBuffer + _writeBufferIndex, data, n);
        _writeBufferIndex += n;
        done = n;
        DEBUGME("buffering, now buffer is %d bytes   \n", _writeBufferIndex);
        return lnSocket::Ok;
    }
    // does not fit
    this->flush();
    uint32_t start = 0;
    while (start < n)
    {
        uint32_t w = 0;
        if (ERR_OK != netconn_write_partly(_work_conn, data + start, n - start, NETCONN_COPY, &w))
        {
            disconnect();
            return lnSocket::Error;
        }
        start += w;
        DEBUGME("chunk of size %d sent, progress is %d vs %d\n", w, start, n);
    }
    done = n;
    return lnSocket::Ok;
}
/**
 * @brief [TODO:description]
 *
 * @return [TODO:return]
 */
lnSocket::status lnSocket_impl::flush()
{
    xAssert(_state == CON_WORK);
    DEBUGME("Flushing  %d bytes\n", _writeBufferIndex);
    uint32_t start = 0;
    if (!_writeBufferIndex)
        return lnSocket::Ok;
    while (start < _writeBufferIndex)
    {
        uint32_t w = 0;
        if (ERR_OK !=
            netconn_write_partly(_work_conn, _writeBuffer + start, _writeBufferIndex - start, NETCONN_COPY, &w))
        {
            disconnect();
            _writeBufferIndex = 0;
            return lnSocket::Error;
        }
        start += w;
        DEBUGME("chunk of size %d sent, progress is %d vs %d\n", w, start, _writeBufferIndex);
    }
    _writeBufferIndex = 0;
    DEBUGME("Flushing   doine\n");
    return lnSocket::Ok;
}
/**
 * @brief [TODO:description]
 */
void lnSocket_impl::disconnect()
{
    close();
    _sockCb(SocketDisconnect, _cbArg);
}
// EOF
