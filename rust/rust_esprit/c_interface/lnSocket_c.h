#pragma once
#include "stdint.h"
#include "stdbool.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ln_socket_c ln_socket_c;

typedef enum {
    LN_SOCKET_EVENT_IGNORE = 0,
    LN_SOCKET_EVENT_CONNECTED = 1,
    LN_SOCKET_EVENT_DISCONNECT = 2,
    LN_SOCKET_EVENT_DATA_AVAILABLE = 3,
    LN_SOCKET_EVENT_WRITE_AVAILABLE = 4,
    LN_SOCKET_EVENT_ERROR = 5,
    LN_SOCKET_EVENT_CUSTOM = 6,
} ln_socket_event_c;

typedef enum {
    LN_SOCKET_STATUS_OK = 0,
    LN_SOCKET_STATUS_ERROR = 1,
} ln_socket_status_c;

typedef void (*ln_socket_cb_c)(ln_socket_event_c evt, void *arg);

ln_socket_c *ln_socket_create(uint16_t port, ln_socket_cb_c cb, void *arg);
void ln_socket_delete(ln_socket_c *sock);

ln_socket_status_c ln_socket_async_mode(ln_socket_c *sock);
ln_socket_status_c ln_socket_accept(ln_socket_c *sock);
ln_socket_status_c ln_socket_disconnect_client(ln_socket_c *sock);

ln_socket_status_c ln_socket_read(ln_socket_c *sock, uint32_t *n, uint8_t **data);
ln_socket_status_c ln_socket_free_read_data(ln_socket_c *sock);

ln_socket_status_c ln_socket_write(ln_socket_c *sock, uint32_t n, const uint8_t *data, uint32_t *done);
ln_socket_status_c ln_socket_write_buffer_available(ln_socket_c *sock, uint32_t *avail);
ln_socket_status_c ln_socket_flush(ln_socket_c *sock);

#ifdef __cplusplus
}
#endif
