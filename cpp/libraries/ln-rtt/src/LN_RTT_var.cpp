
#include "LN_RTT.h"
#include "esprit.h"
#include "ln-rtt_priv.h"
#include "string.h"

__attribute__((used)) MY_RTT_DESC my_rtt;
extern MY_RTT_DESC _SEGGER_RTT __attribute__((alias("my_rtt"))) __attribute__((used));
