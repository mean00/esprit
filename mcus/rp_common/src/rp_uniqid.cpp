/*
 *  (C) 2021 MEAN00 fixounet@free.fr
 *  See license file
 */
#include "lnCpuID.h"
#include "pico/unique_id.h"

/**
 * @brief Return the unique board ID as a 32-bit value.
 *
 * The SDK's pico_get_unique_board_id() reads the 8-byte unique ID from the
 * external SPI flash chip via flash_get_unique_id(). This is the standard
 * RP2040 method — there is no pre-programmed chip serial in OTP.
 *
 * On RP2350, it reads from ROM sys_info (chip info).
 */
uint32_t lnCpuID::getSerialID()
{
    pico_unique_board_id_t id;
    pico_get_unique_board_id(&id);
    // Take first 4 bytes as uint32_t (little-endian)
    return (uint32_t)id.id[0] |
           ((uint32_t)id.id[1] << 8) |
           ((uint32_t)id.id[2] << 16) |
           ((uint32_t)id.id[3] << 24);
}
