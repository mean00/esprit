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
static void server_callback(struct netconn *conn, enum netconn_evt evt, u16_t len);
//
void server_callback(struct netconn *conn, enum netconn_evt evt, u16_t len)
{
    // case NETCONN_EVT_ACCEPT:
    Logger("Server callback 0x%x\n", evt);
}
/**
 * @brief [TODO:description]
 *
 * @param netif [TODO:parameter]
 * @param reason [TODO:parameter]
 * @param args [TODO:parameter]
 */
void netif_ext_callback_esprit(struct netif *netif, netif_nsc_reason_t reason, const netif_ext_callback_args_t *args)
{
#define UP_AND_RUNNING_EVT (LWIP_NSC_IPV4_ADDRESS_CHANGED + LWIP_NSC_IPV4_ADDR_VALID)
#define DOWN_EVT (LWIP_NSC_IPV4_ADDRESS_CHANGED + 0 * LWIP_NSC_IPV4_ADDR_VALID)

    if ((reason & UP_AND_RUNNING_EVT) == UP_AND_RUNNING_EVT)
    {
        Logger("IP up\n");
        _syscb(LwipReady, _sysarg);
        return;
    }
    if ((reason & DOWN_EVT) == DOWN_EVT)
    {
        Logger("IP down\n");
        _syscb(LwipDown, _sysarg);
        return;
    }
    Logger("Unhandled ext callback event 0x%x\n", reason);
}
//
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
    netif_add_ext_callback(&netif_callback, netif_ext_callback_esprit);
    netif_set_up(&lwip_netif);
    dhcp_start(&lwip_netif);
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
    status asyncMode();
    status write(uint32_t n, const uint8_t *data, uint32_t &done);
    status flush(); // force flushing the write buffer
    status close();
    status invoke(lnSocketEvent evt)
    {
        _sockCb(evt, _cbArg);
        return lnSocket::Ok;
    }
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
 *
 */
static void _conn_callback_srv(struct netconn *con, enum netconn_evt evt, u16_t len)
{
    Logger("Received evt 0x%x\n", evt);
    lnSocket_impl *sock = (lnSocket_impl *)netconn_get_callback_arg(con);
    xAssert(sock);
    switch (evt)
    {
    case NETCONN_EVT_RCVPLUS:
        if (len == 0)
            sock->invoke(SocketConnect);
        else
        {
            sock->invoke(SocketDataAvailable);
        }
        break;
    default:
        xAssert(0);
    }
}

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
    _conn = netconn_new_with_callback(NETCONN_TCP, _conn_callback_srv);
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
    netconn_set_callback_arg(_conn, this);
    Logger("Socket bound and ok\n");
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
    netconn_set_nonblocking(_conn, true);
    switch (netconn_accept(_conn, &_work_conn))
    {

    case ERR_OK: {
        Logger("Accepted...\n");
        _state = CON_WORK;
        _writeBufferIndex = 0;
        _receiveIndex = 0;
        netconn_set_callback_arg(_conn, this);
        return lnSocket::Ok;
    }
    case ERR_WOULDBLOCK:
        return lnSocket::Error;
    default:
        return lnSocket::Error;
    }
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
 */
lnSocket::status lnSocket_impl::asyncMode()
{
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
#if 0
#include "lwip/api.h"
#include "lwip/sys.h"
#include <stdio.h>
#include <string.h>

// Callback function for netconn events
static void server_callback(struct netconn *conn, enum netconn_evt evt, u16_t len) {
    if (evt == NETCONN_EVT_ACCEPT) {
        // 🔔 Event: new client connection pending
        printf("New client connection event!\n");
    } else if (evt == NETCONN_EVT_RCVPLUS) {
        // 🔔 Event: new data available to read
        printf("Data available event (len=%d)\n", len);
    } else if (evt == NETCONN_EVT_RCVMINUS) {
        // 🔔 Event: data buffer consumed
        printf("Data consumed event\n");
    } else if (evt == NETCONN_EVT_SENDPLUS) {
        // 🔔 Event: ready to send more data
        printf("Send ready event\n");
    }
}

// Server task
void tcp_server_task(void *arg) {
    struct netconn *listen_conn, *new_conn;
    struct netbuf *buf;
    err_t err;

    // 1. Create new TCP connection handle
    listen_conn = netconn_new(NETCONN_TCP);
    if (listen_conn == NULL) {
        printf("Failed to create netconn\n");
        return;
    }

    // 2. Bind to port
    netconn_bind(listen_conn, IP_ADDR_ANY, 8080);

    // 3. Start listening
    netconn_listen(listen_conn);

    // 4. Register callback for async events
    netconn_set_callback(listen_conn, server_callback);

    printf("Async TCP server listening on port 8080\n");

    // 5. Accept loop
    while (1) {
        // Accept new connection (blocking until one arrives)
        err = netconn_accept(listen_conn, &new_conn);
        if (err == ERR_OK) {
            printf("Accepted new client!\n");

            // Register callback for this client
            netconn_set_callback(new_conn, server_callback);

            // Handle client in loop
            while ((err = netconn_recv(new_conn, &buf)) == ERR_OK) {
                void *data;
                u16_t len;
                netbuf_data(buf, &data, &len);

                printf("Received: %.*s\n", len, (char*)data);

                // Echo back
                netconn_write(new_conn, data, len, NETCONN_COPY);

                netbuf_delete(buf);
            }

            // Close client connection
            netconn_close(new_conn);
            netconn_delete(new_conn);
            printf("Client disconnected\n");
        }
    }
}

#endif
