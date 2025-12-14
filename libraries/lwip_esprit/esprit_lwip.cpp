/*
 * This is an implementation of a "socket pair"
 * The server part is oinly acceptint a single "client" socket
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
/* Forward declarations */
static err_t tcp_server_accept(void *arg, struct tcp_pcb *newpcb, err_t err);
static err_t tcp_client_recv(void *arg, struct tcp_pcb *tpcb, struct pbuf *p, err_t err);
static err_t tcp_client_sent(void *arg, struct tcp_pcb *tpcb, u16_t len);
static void tcp_client_error(void *arg, err_t err);
static err_t tcp_client(struct tcp_pcb *tpcb, struct conn_state *state);
//
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

#define NOFAIL(x)                                                                                                      \
    if (ERR_OK != (x))                                                                                                 \
        xAssert(0);

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
        uint32_t ip = ip4_addr_get_u32(netif_ip4_addr(&lwip_netif));
        Logger("IP up %d.%d.%d.%d\n", ip & 0xff, (ip >> 8) & 0xff, (ip >> 16) & 0xff, (ip >> 24));
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

*/
#define BUFFER_FIFO_SIZE 8
class Bufferfifo
{
  public:
    Bufferfifo(struct tcp_pcb *tcp)
    {
        head = 0;
        tail = 0;
        _tcp = tcp;
    }
    ~Bufferfifo()
    {
        struct pbuf *p;
        while (1)
        {
            p = pop();
            if (p == nullptr)
                return;
            LOCK_TCPIP_CORE();
            tcp_recved(_tcp, p->len);
            pbuf_free(p);
            UNLOCK_TCPIP_CORE();
        }
    }
    // Push from ISR (producer)
    bool push(struct pbuf *item)
    {
        size_t next = (head + 1) & (BUFFER_FIFO_SIZE - 1);
        if (next == tail)
            return false; // FIFO full
        buffer[head] = item;
        head = next;
        return true;
    }
    // Peek
    struct pbuf *peek()
    {
        if (head == tail)
            return nullptr; // FIFO empty
        struct pbuf *item = buffer[tail];
        return item;
    }
    // Pop from main loop (consumer)
    struct pbuf *pop()
    {
        if (head == tail)
            return nullptr; // FIFO empty
        struct pbuf *item = buffer[tail];
        tail = (tail + 1) & (BUFFER_FIFO_SIZE - 1);
        return item;
    }

    bool isEmpty() const
    {
        return head == tail;
    }
    bool isFull() const
    {
        return ((head + 1) & (BUFFER_FIFO_SIZE - 1)) == tail;
    }

  private:
    struct pbuf *buffer[BUFFER_FIFO_SIZE]; // fixed-size array of pointers
    volatile size_t head = 0;              // write index (ISR)
    volatile size_t tail = 0;              // read index (main loop)
    struct tcp_pcb *_tcp;
};

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
    status read(uint32_t &n, uint8_t **data);
    status freeReadData();
    status accept();
    status asyncMode();
    status write(uint32_t n, const uint8_t *data, uint32_t &done);
    status flush(); // force flushing the write buffer
    status disconnectClient();
    status writeBufferAvailable(uint32_t &n);
    void accepted(struct tcp_pcb *t)
    {
        xAssert(!_client_tcp);

        _client_tcp = t;
        xAssert(!_fifo);
        _fifo = new Bufferfifo(t);
        _state = CON_WORK;
        invoke(SocketConnected);
    }
    void disconnected()
    {
        DEBUGME("Err or disconnect\n");
        _state = CON_ERROR;
        invoke(SocketDisconnect);
    }
    void incomingData(pbuf *p, err_t err)
    {
        if (_state != CON_WORK)
            return;
        xAssert(_fifo);
        if (p) // actual data
        {
            _fifo->push(p);
            invoke(SocketDataAvailable);
        } // disconnect
        else
        {
            disconnected();
        }
    }
    status invoke(lnSocketEvent evt)
    {
        _sockCb(evt, _cbArg);
        return lnSocket::Ok;
    }
    enum conn_state
    {
        CON_IDLE = 0,
        CON_LISTEN = 1,
        CON_WORK = 2,
        CON_ERROR = 3,
    };

  protected:
    void disconnect();

  public:
    struct tcp_pcb *_server_tcp;
    struct tcp_pcb *_client_tcp;

  protected:
    conn_state _state;
    lnSocketCb _sockCb;
    void *_cbArg;
    Bufferfifo *_fifo;
};
/* Accept new connection */
static err_t tcp_server_accept(void *arg, struct tcp_pcb *newpcb, err_t err)
{
    static int next_id = 1;

    lnSocket_impl *socket = (lnSocket_impl *)arg;
    if (socket->_client_tcp)
    {
        // if we already have a connection, refuse a new one
        tcp_abort(newpcb);
        return ERR_ABRT;
    }

    tcp_arg(newpcb, socket); // attach connection context
    tcp_recv(newpcb, tcp_client_recv);
    tcp_sent(newpcb, tcp_client_sent);
    tcp_err(newpcb, tcp_client_error);

    DEBUGME("TCP:Accepted connection\n");
    socket->accepted(newpcb);
    return ERR_OK;
}

static void tcp_client_error(void *arg, err_t err)
{
    lnSocket_impl *socket = (lnSocket_impl *)arg;
    socket->disconnected();
}

static err_t tcp_client_recv(void *arg, struct tcp_pcb *tpcb, struct pbuf *p, err_t err)
{
    lnSocket_impl *socket = (lnSocket_impl *)arg;
    socket->incomingData(p, err);
    return ERR_OK;
}
static err_t tcp_client_sent(void *arg, struct tcp_pcb *tpcb, u16_t len)
{
    lnSocket_impl *socket = (lnSocket_impl *)arg;
    socket->invoke(SocketWriteAvailable);
    return ERR_OK;
}

#define NO_ERROR_PLEASE()                                                                                              \
    {                                                                                                                  \
        if (_state != CON_WORK)                                                                                        \
            return lnSocket::Error;                                                                                    \
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
    // s->accept();
    return s;
}
/**
 *
 */
lnSocket_impl::lnSocket_impl(lnSocketCb cb, void *arg) : lnSocket()
{
    _server_tcp = NULL;
    _client_tcp = NULL;
    _state = CON_IDLE;
    _cbArg = arg;
    _sockCb = cb;
    _fifo = NULL;
}
/**
 * @brief [TODO:description]
 */
lnSocket_impl::~lnSocket_impl()
{
    disconnectClient();
    // close master socket also
    if (_server_tcp)
    {
        LOCK_TCPIP_CORE();
        tcp_close(_server_tcp);
        if (_client_tcp)
            tcp_close(_client_tcp);
        UNLOCK_TCPIP_CORE();
        _server_tcp = NULL;
        _client_tcp = NULL;
    }
    if (_fifo)
    {
        delete _fifo;
        _fifo = nullptr;
    }
    _state = CON_IDLE;
}
/**
 * @brief [TODO:description]
 */
lnSocket::status lnSocket_impl::disconnectClient()
{
    Logger("Closing socket\n");
    if (_client_tcp)
    {
        LOCK_TCPIP_CORE();
        tcp_close(_client_tcp);
        _client_tcp = NULL;
        UNLOCK_TCPIP_CORE();
    }
    if (_fifo)
    {
        delete _fifo;
        _fifo = NULL;
    }
    _state = CON_LISTEN; // back to listening
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

    LOCK_TCPIP_CORE();
    struct tcp_pcb *pcb = tcp_new();
    if (!pcb)
        xAssert(0);

    NOFAIL(tcp_bind(pcb, IP_ADDR_ANY, port));
    UNLOCK_TCPIP_CORE();
    this->_server_tcp = pcb;
    tcp_arg(pcb, this);
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
    // convert to listening socket
    LOCK_TCPIP_CORE();
    _server_tcp = tcp_listen(_server_tcp);
    UNLOCK_TCPIP_CORE();
    tcp_accept(_server_tcp, tcp_server_accept);
    return lnSocket::Ok;
}
/**
 * @brief [TODO:description]
 */
lnSocket::status lnSocket_impl::freeReadData()
{
    xAssert(_fifo);
    xAssert(!_fifo->isEmpty());
    struct pbuf *t = _fifo->pop();
    xAssert(t);
    LOCK_TCPIP_CORE();
    tcp_recved(_client_tcp, t->len);
    pbuf_free(t);
    UNLOCK_TCPIP_CORE();
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
lnSocket::status lnSocket_impl::read(uint32_t &n, uint8_t **data)
{
    NO_ERROR_PLEASE();
    xAssert(_fifo);
    auto *t = _fifo->peek();
    if (!t)
    {
        n = 0;
        return lnSocket::Ok;
    }
    n = t->len;
    *data = (uint8_t *)t->payload;
    return lnSocket::Ok;
}
// return # of bytes that can be written without blocking. This is not atomic
lnSocket::status lnSocket_impl::writeBufferAvailable(uint32_t &n)
{
    NO_ERROR_PLEASE();
    LOCK_TCPIP_CORE();
    uint32_t available = tcp_sndbuf(_client_tcp);
    UNLOCK_TCPIP_CORE();
    n = available;
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
lnSocket::status lnSocket_impl::write(uint32_t n, const uint8_t *data, uint32_t &done)
{
    NO_ERROR_PLEASE();
    LOCK_TCPIP_CORE();
    uint32_t available = tcp_sndbuf(_client_tcp);
    UNLOCK_TCPIP_CORE();
    if (!available)
    {
        done = 0;
        return lnSocket::Ok;
    }
    if (n > available)
        n = available;
    LOCK_TCPIP_CORE();
    err_t err = tcp_write(_client_tcp, data, n, TCP_WRITE_FLAG_COPY);
    if (err == ERR_OK)
    {
        tcp_output(_client_tcp);
        UNLOCK_TCPIP_CORE();
        done = n;
        return lnSocket::Ok;
    }
    UNLOCK_TCPIP_CORE();
    _state = CON_ERROR;
    return lnSocket::Error;
}
lnSocket::status lnSocket_impl::asyncMode()
{
    return lnSocket::Ok;
}
lnSocket::status lnSocket_impl::flush()
{
    NO_ERROR_PLEASE();
    LOCK_TCPIP_CORE();
    tcp_output(this->_client_tcp);
    UNLOCK_TCPIP_CORE();
    return lnSocket::Ok;
}
/**
 * @brief [TODO:description]
 */
void lnSocket_impl::disconnect()
{
    xAssert(0);
}

// EOF
