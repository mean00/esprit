#pragma once

typedef void lnDFUCb( uint32_t block_num, uint8_t const *data, uint32_t length);
/**
 * 
 */
class lnUsbDFU
{
public:
        static void addDFURTCb(lnDFUCb *cb);

};