/**
 * @file    dhcp_w5500.cpp
 * @brief   DHCP client implementation using shadow5Socket register access.
 *
 * Self-contained DHCP client that uses write_to_reg()/read_from_reg()
 * directly, avoiding the old WIZCHIP_READ/WIZCHIP_WRITE macro layer.
 *
 * Provides the same external API as the original WIZnet dhcp.h:
 *   DHCP_init(), DHCP_run(), DHCP_time_handler(), DHCP_stop()
 *   reg_dhcp_cbfunc(), getIPfromDHCP(), getGWfromDHCP(),
 *   getSNfromDHCP(), getDNSfromDHCP(), getDHCPLeasetime()
 *
 * @copyright (C) 2025
 * @license  See license file
 */

#include "dhcp_w5500.h"
#include "W5500/w5500_regs.h"
#include "dhcp_w5500_priv.h"
#include "esprit.h"
#include "lowlevel_w5500_helper.h"

// Debug logging
#undef DEBUGME
#if 0
#define DEBUGME Logger
#else
#define DEBUGME(...)                                                                                                   \
    {                                                                                                                  \
    }
#endif

/** @brief Buffer for sending &  receiving DHCP replies (parsed in-place). */
static uint8_t dhcp_reply_buffer[DHCP_REPLY_BUFFER_SIZE];

static const uint8_t bcast_mac[6] = {0xff, 0xff, 0xff, 0xff, 0xff, 0xff};
// Static DHCP client instance
/** @brief Single static instance holding all DHCP client state. */
static DHCPData g_dhcp;

// ---------------------------------------------------------------------------
// Low-level UDP send/recv using shadow5Socket
// ---------------------------------------------------------------------------

/**
 * @brief Send a UDP packet on the DHCP socket.
 *
 * Sets the destination IP and port, writes the data to the W5500 TX buffer,
 * issues the SEND command, and waits for SEND_OK (with a ~1 second timeout).
 *
 * @param buf      Data to send.
 * @param len      Number of bytes to send.
 * @param dst_ip   Destination IP address (4 bytes).
 * @param dst_port Destination UDP port.
 * @return Number of bytes sent on success, 0 on failure.
 */
static int dhcp_send_udp(const uint8_t *buf, uint16_t len, const uint8_t *dst_ip, uint16_t dst_port)
{
    shadow5Socket s(g_dhcp.sn);

    // Set destination IP and port
    s.setDestIp(dst_ip);
    s.setDestPort(dst_port);

    // For broadcast (255.255.255.255), set Sn_DHAR to ff:ff:ff:ff:ff:ff
    // The W5500 needs the destination MAC to transmit; without this the
    // frame is never sent.
    s.setDestMac(bcast_mac);

    // Check TX buffer space
    uint16_t freesize = s.getTxFreeBufferSize();
    if (freesize < len)
    {
        Logger("Cant send udp packet, tx buffer full \n");
        return 0;
    }

    DEBUGME("DHCP: send %d bytes to %d.%d.%d.%d:%d\n", len, dst_ip[0], dst_ip[1], dst_ip[2], dst_ip[3], dst_port);

    // Write data to TX buffer
    s.writeBlock(len, buf);

    // Issue SEND command
    s.sendCommand(Sn_CR_SEND);

    // Wait for SEND_OK interrupt (timeout ~1 second)
    // The W5500 sets Sn_IR_SENDOK when the packet has been transmitted.
    uint32_t timeout = 100000;
    while (timeout--)
    {
        uint8_t ir = s.getInterruptPending();
        if (ir & Sn_IR_SENDOK)
        {
            s.clearPendingInterrupt(Sn_IR_SENDOK); // clear
            break;
        }
        if (ir & Sn_IR_TIMEOUT)
        {
            s.clearPendingInterrupt(Sn_IR_TIMEOUT); // clear
            Logger("DHCP: SEND timeout on socket %d\n", g_dhcp.sn);
            return 0;
        }
    }
    if (timeout == 0)
    {
        Logger("DHCP: SEND timed out (no SENDOK) on socket %d\n", g_dhcp.sn);
        return 0;
    }

    return (int)len;
}

/**
 * @brief Receive a UDP packet on the DHCP socket (non-blocking).
 *
 * The W5500 prepends every received UDP datagram with an 8-byte header
 * in the RX buffer:
 *   bytes 0-3: source IP
 *   bytes 4-5: source port (big-endian)
 *   bytes 6-7: packet length (big-endian)
 *
 * We read the 8-byte header first, then the payload.  The read pointer
 * (Sn_RX_RD) is advanced by readData() after each read, so the second
 * read starts right after the header.
 *
 * @param[out] buf       Buffer to receive the UDP payload.
 * @param[in]  max_len   Maximum number of payload bytes to read.
 * @param[out] src_ip    Source IP address (4 bytes, may be NULL).
 * @param[out] src_port  Source UDP port (may be NULL).
 * @return Number of payload bytes received, or 0 if no data.
 */
static int dhcp_recv_udp(uint8_t *buf, uint16_t max_len, uint8_t *src_ip, uint16_t *src_port)
{
    shadow5Socket s(g_dhcp.sn);

    uint16_t rsvr = s.getReadAvailable();
    if (rsvr == 0)
    {
        return 0;
    }

    // Read the 8-byte UDP header from the RX buffer
    uint8_t rx_hdr[8];
    uint32_t rd_ptr_before = s.getReadPointer();
    s.readData(8, rx_hdr);
    uint32_t rd_ptr_after = s.getReadPointer();

    DEBUGME("DHCP: recv header: rd_ptr %lu -> %lu, hdr=%02x %02x %02x %02x %02x %02x %02x %02x\n",
            (unsigned long)rd_ptr_before, (unsigned long)rd_ptr_after, rx_hdr[0], rx_hdr[1], rx_hdr[2], rx_hdr[3],
            rx_hdr[4], rx_hdr[5], rx_hdr[6], rx_hdr[7]);

    // Parse source IP (bytes 0-3)
    if (src_ip)
    {
        src_ip[0] = rx_hdr[0];
        src_ip[1] = rx_hdr[1];
        src_ip[2] = rx_hdr[2];
        src_ip[3] = rx_hdr[3];
    }

    // Parse source port (bytes 4-5, big-endian)
    uint16_t peer_port = ((uint16_t)rx_hdr[4] << 8) | rx_hdr[5];
    if (src_port)
    {
        *src_port = peer_port;
    }

    // Packet length from header (bytes 6-7)
    uint16_t pack_len = ((uint16_t)rx_hdr[6] << 8) | rx_hdr[7];

    // Remaining bytes after the 8-byte header
    uint16_t payload_len = (pack_len > 0) ? pack_len : ((rsvr >= 8) ? (rsvr - 8) : 0);
    if (payload_len == 0)
    {
        s.sendCommand(Sn_CR_RECV);
        return 0;
    }

    // RECV command is issued automatically inside shadow5Socket::readData()
    // after each read, so the payload read will start from the correct
    // position (right after the 8-byte header).
    uint16_t toRead = (payload_len < max_len) ? payload_len : max_len;
    uint32_t rd_ptr_payload = s.getReadPointer();
    s.readData(toRead, buf);
    uint32_t rd_ptr_payload_end = s.getReadPointer();

    DEBUGME("DHCP: recv payload: rd_ptr %lu -> %lu, buf[0..3]=%02x %02x %02x %02x\n", (unsigned long)rd_ptr_payload,
            (unsigned long)rd_ptr_payload_end, buf[0], buf[1], buf[2], buf[3]);

    if (payload_len > max_len)
    {
        Logger("UDP msg received too big : %d vs %d\n", payload_len, max_len);
        s.skipData(payload_len - max_len);
    }

    DEBUGME("DHCP: recv %d bytes from %d.%d.%d.%d:%d\n", toRead, src_ip[0], src_ip[1], src_ip[2], src_ip[3], peer_port);

    return (int)toRead;
}

// ---------------------------------------------------------------------------
// DHCP message helpers
// ---------------------------------------------------------------------------

/**
 * @brief Build a DHCP message header in the buffer.
 *
 * Constructs a BOOTP header (240 bytes) with the magic cookie at offset 236,
 * then appends the DHCP message type option (option 53).
 *
 * @param buf      Output buffer (must be at least 240 bytes).
 * @param msg_type DHCP message type (DHCP_DISCOVER, DHCP_REQUEST, etc.).
 * @param xid      Transaction ID.
 * @param ciaddr   Client IP (NULL = 0.0.0.0 for DISCOVER).
 * @return Total length of the DHCP message (header + first option).
 */
static uint16_t dhcp_build_header(uint8_t *buf, uint8_t msg_type, uint32_t xid, const uint8_t *ciaddr)
{
    // Zero the buffer
    memset(buf, 0, 240);

    // BOOTP header
    buf[BOOTP_OP_OFFSET] = 1;    // op: BOOTREQUEST
    buf[BOOTP_HTYPE_OFFSET] = 1; // htype: Ethernet
    buf[BOOTP_HLEN_OFFSET] = 6;  // hlen: MAC length
    buf[BOOTP_HOPS_OFFSET] = 0;  // hops
    buf[BOOTP_XID_OFFSET + 0] = (uint8_t)(xid >> 24);
    buf[BOOTP_XID_OFFSET + 1] = (uint8_t)(xid >> 16);
    buf[BOOTP_XID_OFFSET + 2] = (uint8_t)(xid >> 8);
    buf[BOOTP_XID_OFFSET + 3] = (uint8_t)(xid);
    // secs = 0
    // flags = 0 (unicast)

    if (ciaddr)
    {
        buf[BOOTP_CIADDR_OFFSET + 0] = ciaddr[0];
        buf[BOOTP_CIADDR_OFFSET + 1] = ciaddr[1];
        buf[BOOTP_CIADDR_OFFSET + 2] = ciaddr[2];
        buf[BOOTP_CIADDR_OFFSET + 3] = ciaddr[3];
    }

    // yiaddr, siaddr, giaddr = 0

    // chaddr = MAC (read from W5500 SHAR register via shadow5System)
    {
        shadow5System sys;
        uint8_t mac[6];
        sys.getMac(mac);
        memcpy(&buf[BOOTP_CHADDR_OFFSET], mac, 6);
    }

    // sname + file = 0 (already zeroed)

    // Magic cookie
    buf[BOOTP_MAGIC_OFFSET + 0] = 0x63;
    buf[BOOTP_MAGIC_OFFSET + 1] = 0x82;
    buf[BOOTP_MAGIC_OFFSET + 2] = 0x53;
    buf[BOOTP_MAGIC_OFFSET + 3] = 0x63;

    // DHCP message type option
    uint16_t off = BOOTP_OPTIONS_OFFSET;
    buf[off++] = DHCP_OPT_MSG_TYPE;
    buf[off++] = 1; // length
    buf[off++] = msg_type;

    return off;
}

/**
 * @brief Add a DHCP option to the message buffer.
 *
 * Writes the tag, length, and data at the given offset and advances
 * the offset past the option.
 *
 * @param buf   Message buffer.
 * @param off   Current offset (updated in place).
 * @param tag   Option tag (e.g. DHCP_OPT_ROUTER).
 * @param len   Option data length.
 * @param data  Option data.
 */
static void dhcp_add_option(uint8_t *buf, uint16_t *off, uint8_t tag, uint8_t len, const uint8_t *data)
{
    buf[(*off)++] = tag;
    buf[(*off)++] = len;
    memcpy(&buf[*off], data, len);
    *off += len;
}

/**
 * @brief Finish the DHCP options with the END tag.
 *
 * Writes the END option (255) at the current offset.
 *
 * @param buf   Message buffer.
 * @param off   Current offset (updated in place).
 */
static void dhcp_finish_options(uint8_t *buf, uint16_t *off)
{
    buf[(*off)++] = DHCP_OPT_END;
}

/**
 * @brief Parse a received DHCP reply and extract options.
 *
 * Validates the magic cookie and transaction ID, then walks the option
 * list starting at offset 240 to extract the message type, offered IP,
 * server id, subnet mask, gateway, DNS, and lease time.
 *
 * @param[in]  buf        Received DHCP message.
 * @param[in]  len        Message length.
 * @param[out] msg_type   DHCP message type (ACK, NAK, OFFER).
 * @param[out] yiaddr     Offered IP address (from yiaddr field).
 * @param[out] server_ip  DHCP server id (option 54).
 * @param[out] subnet     Subnet mask (option 1, optional).
 * @param[out] gateway    Gateway/router (option 3, optional).
 * @param[out] dns        DNS server (option 6, optional).
 * @param[out] lease      Lease time in seconds (option 51, optional).
 * @return true if the message is valid and the message type was found.
 */
static bool dhcp_parse_reply(const uint8_t *buf, uint16_t len, uint8_t *msg_type, uint8_t *yiaddr, uint8_t *server_ip,
                             uint8_t *subnet, uint8_t *gateway, uint8_t *dns, uint32_t *lease)
{
    if (len < 240)
    {
        return false;
    }

    // Check magic cookie at offset BOOTP_MAGIC_OFFSET
    if (buf[BOOTP_MAGIC_OFFSET + 0] != 0x63 || buf[BOOTP_MAGIC_OFFSET + 1] != 0x82 ||
        buf[BOOTP_MAGIC_OFFSET + 2] != 0x53 || buf[BOOTP_MAGIC_OFFSET + 3] != 0x63)
    {
        return false;
    }

    // Check transaction ID (xid) at offset BOOTP_XID_OFFSET
    uint32_t rx_xid = ((uint32_t)buf[BOOTP_XID_OFFSET + 0] << 24) | ((uint32_t)buf[BOOTP_XID_OFFSET + 1] << 16) |
                      ((uint32_t)buf[BOOTP_XID_OFFSET + 2] << 8) | buf[BOOTP_XID_OFFSET + 3];
    if (rx_xid != g_dhcp.xid)
    {
        return false;
    }

    // Extract yiaddr (offered IP) at offset BOOTP_YIADDR_OFFSET
    if (yiaddr)
    {
        yiaddr[0] = buf[BOOTP_YIADDR_OFFSET + 0];
        yiaddr[1] = buf[BOOTP_YIADDR_OFFSET + 1];
        yiaddr[2] = buf[BOOTP_YIADDR_OFFSET + 2];
        yiaddr[3] = buf[BOOTP_YIADDR_OFFSET + 3];
    }

    // Parse options starting at offset BOOTP_OPTIONS_OFFSET
    uint16_t off = BOOTP_OPTIONS_OFFSET;
    bool got_msg_type = false;

    while (off < len)
    {
        uint8_t tag = buf[off++];
        if (tag == DHCP_OPT_END)
            break;
        if (tag == 0) // PAD
            continue;

        uint8_t opt_len = buf[off++];
        if (off + opt_len > len)
            break;

        switch (tag)
        {
        case DHCP_OPT_MSG_TYPE:
            if (opt_len >= 1 && msg_type)
            {
                *msg_type = buf[off];
                got_msg_type = true;
            }
            break;
        case DHCP_OPT_SERVER_ID:
            if (opt_len >= 4 && server_ip)
            {
                server_ip[0] = buf[off];
                server_ip[1] = buf[off + 1];
                server_ip[2] = buf[off + 2];
                server_ip[3] = buf[off + 3];
            }
            break;
        case DHCP_OPT_SUBNET_MASK:
            if (opt_len >= 4 && subnet)
            {
                subnet[0] = buf[off];
                subnet[1] = buf[off + 1];
                subnet[2] = buf[off + 2];
                subnet[3] = buf[off + 3];
            }
            break;
        case DHCP_OPT_ROUTER:
            if (opt_len >= 4 && gateway)
            {
                gateway[0] = buf[off];
                gateway[1] = buf[off + 1];
                gateway[2] = buf[off + 2];
                gateway[3] = buf[off + 3];
            }
            break;
        case DHCP_OPT_DNS:
            if (opt_len >= 4 && dns)
            {
                dns[0] = buf[off];
                dns[1] = buf[off + 1];
                dns[2] = buf[off + 2];
                dns[3] = buf[off + 3];
            }
            break;
        case DHCP_OPT_LEASE_TIME:
            if (opt_len >= 4 && lease)
            {
                *lease = ((uint32_t)buf[off] << 24) | ((uint32_t)buf[off + 1] << 16) | ((uint32_t)buf[off + 2] << 8) |
                         buf[off + 3];
            }
            break;
        }
        off += opt_len;
    }

    return got_msg_type;
}

// ---------------------------------------------------------------------------
// DHCP state machine actions
// ---------------------------------------------------------------------------

/**
 * @brief Send a DHCP DISCOVER message.
 *
 * Builds a DISCOVER with a parameter request list (subnet, router, DNS,
 * lease time) and a host name option, then broadcasts it to 255.255.255.255:67.
 */
static const uint8_t broadcast[4] = {255, 255, 255, 255};
static void dhcp_send_discover(void)
{
    uint16_t off = dhcp_build_header(g_dhcp.buf, DHCP_DISCOVER, g_dhcp.xid, nullptr);

    // Parameter request list
    uint8_t param_req[] = {DHCP_OPT_SUBNET_MASK, DHCP_OPT_ROUTER, DHCP_OPT_DNS, DHCP_OPT_LEASE_TIME};
    dhcp_add_option(g_dhcp.buf, &off, DHCP_OPT_PARAM_REQ, sizeof(param_req), param_req);

    // Host name
    dhcp_add_option(g_dhcp.buf, &off, DHCP_OPT_HOST_NAME, 6, (const uint8_t *)"w5500");

    dhcp_finish_options(g_dhcp.buf, &off);

    // Send to 255.255.255.255:67
    dhcp_send_udp(g_dhcp.buf, off, broadcast, DHCP_SERVER_PORT);
}

/**
 * @brief Send a DHCP REQUEST message.
 *
 * Builds a REQUEST with the requested IP (option 50), server id
 * (option 54), parameter request list, and host name, then broadcasts it
 * to 255.255.255.255:67.
 */
static void dhcp_send_req(void)
{
    uint16_t off = dhcp_build_header(g_dhcp.buf, DHCP_REQUEST, g_dhcp.xid, nullptr);

    // Requested IP
    dhcp_add_option(g_dhcp.buf, &off, DHCP_OPT_REQUESTED_IP, 4, g_dhcp.requested_ip);

    // Server id
    dhcp_add_option(g_dhcp.buf, &off, DHCP_OPT_SERVER_ID, 4, g_dhcp.server_ip);

    // Parameter request list
    uint8_t param_req[] = {DHCP_OPT_SUBNET_MASK, DHCP_OPT_ROUTER, DHCP_OPT_DNS, DHCP_OPT_LEASE_TIME};
    dhcp_add_option(g_dhcp.buf, &off, DHCP_OPT_PARAM_REQ, sizeof(param_req), param_req);

    // Host name
    dhcp_add_option(g_dhcp.buf, &off, DHCP_OPT_HOST_NAME, 6, (const uint8_t *)"w5500");

    dhcp_finish_options(g_dhcp.buf, &off);

    dhcp_send_udp(g_dhcp.buf, off, broadcast, DHCP_SERVER_PORT);
}

/**
 * @brief Retry from DISCOVER after a timeout.
 *
 * Increments the retry counter.  If retries are exhausted, transitions to
 * IDLE and returns DHCP_FAILED.  Otherwise resets the tick, increments the
 * transaction ID, sends a new DISCOVER, and returns DHCP_RUNNING.
 *
 * @return DHCP_FAILED if retries exhausted, DHCP_RUNNING otherwise.
 */
static enum DHCPReturn dhcp_retry(void)
{
    g_dhcp.retry_count++;
    if (g_dhcp.retry_count >= MAX_DHCP_RETRY)
    {
        g_dhcp.state = DHCP_STATE_IDLE;
        return DHCP_FAILED;
    }
    g_dhcp.tick = 0;
    g_dhcp.xid++;
    dhcp_send_discover();
    return DHCP_RUNNING;
}

// ---------------------------------------------------------------------------
// Exported API (extern "C" for C callers)
// ---------------------------------------------------------------------------

/**
 * @brief Initialise the DHCP client on a given W5500 socket.
 *
 * Clears all state, generates a semi-random transaction ID from the MAC
 * address, and opens the socket in UDP mode on DHCP_CLIENT_PORT.
 *
 * @param s  W5500 socket number (0-7) to use for DHCP traffic.
 */
extern "C" void DHCP_init(uint8_t s)
{
    memset(&g_dhcp, 0, sizeof(g_dhcp));
    g_dhcp.sn = s;
    g_dhcp.buf = dhcp_reply_buffer;

    // Generate a semi-random transaction ID from the low bits of the MAC
    {
        shadow5System sys;
        uint8_t mac[6];
        sys.getMac(mac);
        g_dhcp.xid = ((uint32_t)mac[2] << 24) | ((uint32_t)mac[3] << 16) | ((uint32_t)mac[4] << 8) | mac[5];
    }

    // Open UDP socket on DHCP client port
    {
        shadow5Socket s(g_dhcp.sn);
        s.setMode(Sn_MR_UDP);
        s.setLocalPort(DHCP_CLIENT_PORT);
        s.clearPendingInterrupt(0xFF);
        s.setCommand(Sn_CR_OPEN);

        uint8_t sn_sr = s.getStatus();
        DEBUGME("DHCP_init: socket %d opened, Sn_SR=0x%02x\n", g_dhcp.sn, sn_sr);
    }
}

/**
 * @brief Stop the DHCP client and close the socket.
 *
 * Transitions to IDLE state and issues the CLOSE command on the W5500
 * socket.  The socket can be re-started by calling DHCP_init() again.
 */
extern "C" void DHCP_stop(void)
{
    g_dhcp.state = DHCP_STATE_IDLE;

    // Close the socket
    {
        shadow5Socket s(g_dhcp.sn);
        s.setCommand(Sn_CR_CLOSE);
    }
}

/**
 * @brief Periodic tick handler — must be called every second.
 *
 * Increments the internal tick counter used for timeout and lease
 * tracking.  The caller should invoke this from a 1 Hz timer ISR or
 * equivalent periodic context.
 */
extern "C" void DHCP_time_handler(void)
{
    g_dhcp.tick++;
}

/**
 * @brief Run the DHCP state machine — call this periodically.
 *
 * Checks the socket status (re-opening if needed), then advances the
 * DHCP state machine: IDLE → DISCOVERING → REQUESTING → BOUND.
 *
 * @retval DHCP_RUNNING   State machine is still active.
 * @retval DHCP_IP_ASSIGN IP address has been assigned (BOUND).
 * @retval DHCP_IP_LEASED Already BOUND, lease still valid.
 * @retval DHCP_FAILED    Retries exhausted, giving up.
 */
extern "C" enum DHCPReturn DHCP_run(void)
{
    // Ensure the UDP socket is open (the W5500 may close it on error)
    {
        shadow5Socket s(g_dhcp.sn);
        uint8_t sr = s.getStatus();
        if (sr != SOCK_UDP)
        {
            DEBUGME("DHCP_run: socket %d not in SOCK_UDP (0x%02x), re-opening\n", g_dhcp.sn, sr);
            // Close first
            s.setCommand(Sn_CR_CLOSE);
            // Open fresh
            s.setMode(Sn_MR_UDP);
            s.setLocalPort(DHCP_CLIENT_PORT);
            s.clearPendingInterrupt(0xFF);
            s.setCommand(Sn_CR_OPEN);

            sr = s.getStatus();
            DEBUGME("DHCP_run: socket %d re-opened, Sn_SR=0x%02x\n", g_dhcp.sn, sr);
        }
    }

    switch (g_dhcp.state)
    {
    case DHCP_STATE_IDLE:
        // IDLE -> start DISCOVER
        DEBUGME("DHCP: state IDLE -> DISCOVER (xid=0x%08lx)\n", g_dhcp.xid + 1);
        g_dhcp.state = DHCP_STATE_DISCOVERING;
        g_dhcp.retry_count = 0;
        g_dhcp.tick = 0;
        g_dhcp.xid++; // new transaction
        dhcp_send_discover();
        return DHCP_RUNNING;

    case DHCP_STATE_DISCOVERING: {
        // DISCOVER sent, waiting for OFFER
        if (g_dhcp.tick >= DHCP_WAIT_TIME)
        {
            return dhcp_retry();
        }

        // Try to receive OFFER
        uint8_t src_ip[4] = {0};
        uint16_t src_port = 0;
        int rlen = dhcp_recv_udp(dhcp_reply_buffer, sizeof(dhcp_reply_buffer), src_ip, &src_port);
        if (rlen <= 0)
        {
            return DHCP_RUNNING;
        }

        // Parse the reply directly into g_dhcp fields
        uint8_t msg_type = 0;
        if (dhcp_parse_reply(dhcp_reply_buffer, rlen, &msg_type, g_dhcp.requested_ip, g_dhcp.server_ip,
                             nullptr, nullptr, nullptr, nullptr))
        {
            DEBUGME("DHCP: got msg_type=%d (OFFER=%d)\n", msg_type, DHCP_OFFER);
            if (msg_type == DHCP_OFFER)
            {
                DEBUGME("DHCP: OFFER from %d.%d.%d.%d, IP=%d.%d.%d.%d\n",
                        g_dhcp.server_ip[0], g_dhcp.server_ip[1], g_dhcp.server_ip[2], g_dhcp.server_ip[3],
                        g_dhcp.requested_ip[0], g_dhcp.requested_ip[1], g_dhcp.requested_ip[2], g_dhcp.requested_ip[3]);
                // Move to REQUEST state
                g_dhcp.state = DHCP_STATE_REQUESTING;
                g_dhcp.tick = 0;
                dhcp_send_req();
            }
        }
        else
        {
            Logger("DHCP: received %d bytes but parse failed\n", rlen);
        }
        return DHCP_RUNNING;
    }

    case DHCP_STATE_REQUESTING: {
        // REQUEST sent, waiting for ACK
        if (g_dhcp.tick >= DHCP_WAIT_TIME)
        {
            // On timeout in REQUESTING, fall back to DISCOVER
            g_dhcp.state = DHCP_STATE_DISCOVERING;
            return dhcp_retry();
        }

        // Try to receive ACK/NAK
        uint8_t src_ip[4] = {0};
        uint16_t src_port = 0;
        int rlen = dhcp_recv_udp(dhcp_reply_buffer, sizeof(dhcp_reply_buffer), src_ip, &src_port);
        if (rlen <= 0)
        {
            return DHCP_RUNNING;
        }

        // Parse the reply directly into g_dhcp.info fields
        uint8_t msg_type = 0;
        if (dhcp_parse_reply(dhcp_reply_buffer, rlen, &msg_type, g_dhcp.info.ip, g_dhcp.server_ip,
                             g_dhcp.info.subnet, g_dhcp.info.gw, g_dhcp.info.dns, &g_dhcp.info.lease_time))
        {
            DEBUGME("DHCP: state2 got msg_type=%d (ACK=%d, NAK=%d)\n", msg_type, DHCP_ACK, DHCP_NAK);
            if (msg_type == DHCP_ACK)
            {
                DEBUGME("DHCP: ACK — IP=%d.%d.%d.%d gw=%d.%d.%d.%d sn=%d.%d.%d.%d dns=%d.%d.%d.%d lease=%lu\n",
                        g_dhcp.info.ip[0], g_dhcp.info.ip[1], g_dhcp.info.ip[2], g_dhcp.info.ip[3],
                        g_dhcp.info.gw[0], g_dhcp.info.gw[1], g_dhcp.info.gw[2], g_dhcp.info.gw[3],
                        g_dhcp.info.subnet[0], g_dhcp.info.subnet[1], g_dhcp.info.subnet[2], g_dhcp.info.subnet[3],
                        g_dhcp.info.dns[0], g_dhcp.info.dns[1], g_dhcp.info.dns[2], g_dhcp.info.dns[3],
                        (unsigned long)g_dhcp.info.lease_time);
                g_dhcp.lease_time = g_dhcp.info.lease_time; // keep for tick comparison
                g_dhcp.state = DHCP_STATE_BOUND;
                g_dhcp.tick = 0; // Reset tick for lease timing
                // Fire callback
                if (g_dhcp.ip_assign)
                    g_dhcp.ip_assign();
                return DHCP_IP_ASSIGN;
            }
            else if (msg_type == DHCP_NAK)
            {
                Logger("DHCP: NAK received\n");
                if (g_dhcp.ip_conflict)
                    g_dhcp.ip_conflict();
                // Retry from DISCOVER
                g_dhcp.state = DHCP_STATE_DISCOVERING;
                g_dhcp.tick = 0;
                g_dhcp.xid++;
                dhcp_send_discover();
                return DHCP_RUNNING;
            }
        }
        else
        {
            Logger("DHCP: state2 received %d bytes but parse failed\n", rlen);
        }
        return DHCP_RUNNING;
    }

    case DHCP_STATE_BOUND:
        // BOUND — lease is active
        // Check for lease renewal (T1 = 50% of lease time)
        if (g_dhcp.lease_time > 0 && g_dhcp.tick >= (g_dhcp.lease_time / 2))
        {
            // Lease is half-expired, send REQUEST to renew
            g_dhcp.state = DHCP_STATE_REQUESTING;
            g_dhcp.tick = 0;
            dhcp_send_req();
            return DHCP_RUNNING;
        }
        return DHCP_IP_LEASED;

    default:
        g_dhcp.state = DHCP_STATE_IDLE;
        return DHCP_FAILED;
    }
}

/**
 * @brief Register DHCP event callbacks.
 *
 * @param ip_assign  Called when an IP address is assigned (ACK received).
 * @param ip_update  Called when the IP is renewed (reserved for future use).
 * @param ip_conflict Called when a NAK is received (IP conflict).
 */
extern "C" void DHCP_reg_cbfunc(void (*ip_assign)(void), void (*ip_update)(void), void (*ip_conflict)(void))
{
    g_dhcp.ip_assign = ip_assign;
    g_dhcp.ip_update = ip_update;
    g_dhcp.ip_conflict = ip_conflict;
}

/**
 * @brief Get the current DHCP lease info.
 *
 * Returns a pointer to the DHCPInfo structure (IP, subnet, gateway, DNS,
 * lease time) if the state is BOUND, or NULL otherwise.
 *
 * @return Pointer to DHCPInfo, or NULL if no lease is active.
 */
extern "C" const DHCPInfo *DHCP_getInfo(void)
{
    if (g_dhcp.state != DHCP_STATE_BOUND)
    {
        return NULL;
    }
    return &g_dhcp.info;
}
