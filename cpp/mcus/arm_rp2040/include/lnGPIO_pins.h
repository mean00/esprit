/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#pragma once

#define PPINS(val) GPIO##val = val,
#define LN_SYSTEM_LED GPIO25
/**
 */
enum lnPin : int
{
    PPINS(0)
    PPINS(1) PPINS(2) PPINS(3) PPINS(4) PPINS(5) PPINS(6) PPINS(7) PPINS(8) PPINS(9) PPINS(10) PPINS(11) PPINS(12)
        PPINS(13) PPINS(14) PPINS(15) PPINS(16) PPINS(17) PPINS(18) PPINS(19) PPINS(20) PPINS(21) PPINS(22) PPINS(23)
            PPINS(24) PPINS(25) PPINS(26) PPINS(27) PPINS(28) PPINS(29) PPINS(99) NoPin = -1
};
