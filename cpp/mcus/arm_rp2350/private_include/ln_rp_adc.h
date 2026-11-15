#include "esprit.h"
#include "ln_rp_memory_map.h"
/**
 */
/**
 */
struct LN_RP_ADCx
{
    uint32_t CS;      // 0x00
    uint32_t RESULT;  // 0x04
    uint32_t FCS;     // 0x08
    uint32_t FIFO;    // 0x0C
    uint32_t DIV;     // 0x10
    uint32_t INTR;    // 0x14
    uint32_t INTE;    // 0x18
    uint32_t INTF;    // 0x1C
    uint32_t INTS;    // 0x20
};

typedef volatile LN_RP_ADCx LN_RP_ADC;

// Control and STATUS
#define LN_RP_ADC_CS_ENABLE         (1 << 0)
#define LN_RP_ADC_CS_TS_EN          (1 << 1) // temperature enable
#define LN_RP_ADC_CS_START_ONCE     (1 << 2)
#define LN_RP_ADC_CS_START_MABNY    (1 << 3)
#define LN_RP_ADC_CS_READY          (1 << 8)
#define LN_RP_ADC_CS_ERR            (1 << 9)
#define LN_RP_ADC_CS_ERR_STICKY     (1 << 10)
#define LN_RP_ADC_CS_AINSEL(x)      ((x) << 12) // select analog mux
#define LN_RP_ADC_CS_RROBIN(x)      ((x) << 16)

// FIfo control and status
#define LN_RP_ADC_FCS_EN            ( 1<< 0)
#define LN_RP_ADC_FCS_SHIFT         ( 1<< 1)
#define LN_RP_ADC_FCS_ERR           ( 1<< 2)
#define LN_RP_ADC_FCS_DREQ_EN       ( 1<< 3)
#define LN_RP_ADC_FCS_EMPTY         ( 1<< 8)
#define LN_RP_ADC_FCS_FULL          ( 1<< 9)
#define LN_RP_ADC_FCS_UNDER         ( 1<< 10)
#define LN_RP_ADC_FCS_OVER          ( 1<< 11)
#define LN_RP_ADC_FCS_LEVEL(x)      ( (x)<< 16)
#define LN_RP_ADC_FCS_THRESH(x)     ( (x)<< 24)


//  FIFO

#define LN_RP_ADC_FIFO_VALUE_MASK   (0xfff)
#define LN_RP_ADC_FIFO_ERROR        ( 1<<15 )


// DIV REGISTER
#define LN_RP_ADC_DIV_FRAC(x)       ((x)<<0) 
#define LN_RP_ADC_DIV_INT(x)        ((x)<<8) 

// INTR              works also for INTE  INTS, INTF
#define LN_RP_ADC_INTx_FIFO         (1<<0) 

LN_RP_ADC *lnAdc = (LN_RP_ADC *)LN_RP_ADC0;

// EOF
