#pragma once

typedef void lnDFUCb(void);
/**
 * 
 */
class lnUsbDFURT
{
public:
        static void addDFURTCb(lnDFUCb *cb);

};