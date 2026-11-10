#include "esprit.h"
#include "lnLWIP.h"
#include "lnSocket_c.h"

#define TO_CPP(s) (reinterpret_cast<lnSocket *>(s))
#define TO_C(s)   (reinterpret_cast<ln_socket_c *>(s))

// Weak fallback so builds without W5500/LWIP still link cleanly
__attribute__((weak)) lnSocket *lnSocket::create(uint16_t port, lnSocketCb cb, void *arg)
{
    (void)port;
    (void)cb;
    (void)arg;
    return nullptr;
}

extern "C" __attribute__((weak)) void ln_set_connected_state(bool state)
{
    (void)state;
}

static ln_socket_status_c to_c_status(lnSocket::status s)
{
    return (s == lnSocket::Ok) ? LN_SOCKET_STATUS_OK : LN_SOCKET_STATUS_ERROR;
}

ln_socket_c *ln_socket_create(uint16_t port, ln_socket_cb_c cb, void *arg)
{
    lnSocket *sock = lnSocket::create(port, reinterpret_cast<lnSocketCb>(cb), arg);
    return TO_C(sock);
}

void ln_socket_delete(ln_socket_c *sock)
{
    if (sock) {
        delete TO_CPP(sock);
    }
}

ln_socket_status_c ln_socket_async_mode(ln_socket_c *sock)
{
    if (!sock) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->asyncMode());
}

ln_socket_status_c ln_socket_accept(ln_socket_c *sock)
{
    if (!sock) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->accept());
}

ln_socket_status_c ln_socket_disconnect_client(ln_socket_c *sock)
{
    if (!sock) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->disconnectClient());
}

ln_socket_status_c ln_socket_read(ln_socket_c *sock, uint32_t *n, uint8_t **data)
{
    if (!sock || !n || !data) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->read(*n, data));
}

ln_socket_status_c ln_socket_free_read_data(ln_socket_c *sock)
{
    if (!sock) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->freeReadData());
}

ln_socket_status_c ln_socket_write(ln_socket_c *sock, uint32_t n, const uint8_t *data, uint32_t *done)
{
    if (!sock || !done) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->write(n, data, *done));
}

ln_socket_status_c ln_socket_write_buffer_available(ln_socket_c *sock, uint32_t *avail)
{
    if (!sock || !avail) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->writeBufferAvailable(*avail));
}

ln_socket_status_c ln_socket_flush(ln_socket_c *sock)
{
    if (!sock) return LN_SOCKET_STATUS_ERROR;
    return to_c_status(TO_CPP(sock)->flush());
}
