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
#include "lwip/tcp.h"
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
#if 0
#define DEBUGME Logger
#else
#define DEBUGME(...)                                                                                                   \
    {                                                                                                                  \
    }
#endif

/**
 * @brief [TODO:description]
 *
 * @param conn [TODO:parameter]
 * @param evt [TODO:parameter]
 * @param len [TODO:parameter]
 */
void server_callback(struct netconn *conn, enum netconn_evt evt, u16_t len)
{
    // case NETCONN_EVT_ACCEPT:
    DEBUGME("Server callback 0x%x\n", evt);
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
    if (reason == LWIP_NSC_IPV4_ADDR_VALID)
        return;
    Logger("Unhandled ext callback event 0x%x\n", reason);
}
//
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
    struct netconn *conn()
    {
        return _conn;
    };
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
    lnSocketCb _sockCb;
    void *_cbArg;
};
/**
 * @brief [TODO:description]
 *
 * @param ev [TODO:parameter]
 * @return [TODO:return]
 */
static const char *evt2string(enum netconn_evt ev)
{
#define SCASE(x)                                                                                                       \
    case NETCONN_EVT_##x:                                                                                              \
        return #x;                                                                                                     \
        break;
    switch (ev)
    {
        SCASE(SENDPLUS)
        SCASE(SENDMINUS)
        SCASE(RCVMINUS)
        SCASE(RCVPLUS)
        SCASE(ERROR)
    default:
        return "???";
        break;
    }
}
/**
 *
 */
static void _conn_callback_srv(struct netconn *con, enum netconn_evt evt, u16_t len)
{
    DEBUGME("Received evt 0x%x, %s, len=%d\n", evt, evt2string(evt), len);
    lnSocket_impl *sock = (lnSocket_impl *)netconn_get_callback_arg(con);
    xAssert(sock);
    switch (evt)
    {
    case NETCONN_EVT_SENDMINUS: // it means the next send would block
        break;
    case NETCONN_EVT_SENDPLUS: // can send, high & low threshold
        sock->invoke(SocketWriteAvailable);
        break;
    case NETCONN_EVT_ERROR:
        sock->invoke(SocketError);
        break;
    case NETCONN_EVT_RCVPLUS: // if 0, incoming connectio server side
        if (len == 0)
            sock->invoke(SocketConnectServer);
        else
        {
            sock->invoke(SocketDataAvailable);
        }
        break;
    case NETCONN_EVT_RCVMINUS: // if 0, that's the new socket
        if (len == 0)
        {
            sock->invoke(SocketConnectClient);
        }
        else
        {
            sock->invoke(SocketDataAvailable);
        }
        break;
    default:
        xAssert(0);
    }
}
static void _conn_callback(struct netconn *con, enum netconn_evt evt, u16_t len)
{
    lnSocket_impl *sock = (lnSocket_impl *)netconn_get_callback_arg(con);
    if (sock->conn() == con)
        _conn_callback_srv(con, evt, len);
    else
        _conn_callback_srv(con, evt, len); // TODO FIMXE TODO!
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
    Logger("Closing socket\n");
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
    _conn = netconn_new_with_callback(NETCONN_TCP, _conn_callback);
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
    err_t er = netconn_accept(_conn, &_work_conn);
    Logger("Accept : 0x%x\n", er);
    switch (er)
    {

    case ERR_OK: {
        Logger("Accepted...\n");
        _state = CON_WORK;
        _receiveIndex = 0;
        netconn_set_nonblocking(_work_conn, true);
        netconn_set_callback_arg(_work_conn, this);
        tcp_nagle_disable(_work_conn->pcb.tcp);
        return lnSocket::Ok;
    }
    case ERR_WOULDBLOCK:
        Logger("Would block...\n");
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
        DEBUGME("  read buffer , avail %d, asked  %d\n", avail, n);
        uint32_t mx = avail;
        if (mx > n)
        {
            mx = n;
        }
        DEBUGME("  processing %d bytes \n", mx);
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
    err_t er = netconn_recv(_work_conn, &_receiveBuf);
    switch (er)
    {
    case ERR_OK:
        break;
    case ERR_WOULDBLOCK:
        done = 0;
        return lnSocket::Ok;
    default:
        Logger(">>>Receive Error\n");
        disconnect();
        return lnSocket::Error;
    }
    DEBUGME("Received new  packet of size  %d\n", _receiveBuf->p->len);
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
    uint32_t start = 0;
    DEBUGME("Socket::write %d bytes   \n", n);
    while (start < n)
    {
        DEBUGME("  Socket::writing chunk of size %d bytes   \n", n - start);
        uint32_t w = 0;
        err_t er = netconn_write_partly(_work_conn, data + start, n - start, NETCONN_COPY | NETCONN_DONTBLOCK, &w);
        switch (er)
        {
        case ERR_OK:
            break;
        case ERR_WOULDBLOCK:
            DEBUGME("Blk\n");
            return lnSocket::Ok;
            break;
        default:
            Logger("***Socket::write >>>>Go write error 0x%x\n", er);
            disconnect();
            return lnSocket::Error;
            break;
        }
        start += w;
        done += w;
        DEBUGME("  Socket::write chunk of size %d sent, progress is %d vs %d\n", w, start, n);
    }
    DEBUGME("Socket::write done, total %d\n", done);
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
static void flush_tcp_output(void *arg)
{
    struct tcp_pcb *pcb = (struct tcp_pcb *)arg;
    if (pcb != NULL)
    {
        tcp_output(pcb);
    }
}
lnSocket::status lnSocket_impl::flush()
{
    err_t err = tcpip_callback(flush_tcp_output, _work_conn->pcb.tcp);
    if (err != ERR_OK)
    {
        Logger(">>> FLUSH FAILED\n");
    }
    //        netconn_write_partly(_work_conn, _writeBuffer + start, _writeBufferIndex - start, NETCONN_COPY, &w))
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
