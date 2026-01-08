#include "esprit.h"
#include "lnCpuID.h"
#include "lnFreeRTOS_pp.h"
//
#include "esp_chip_info.h"
#include "esp_clk_tree.h"
#include "esp_flash.h"
#include "esp_heap_caps.h"
//
static lnCpuID::LN_MCU _mcu;
static lnCpuID::LN_VENDOR _vendor;
static int _flashSize = 0;
static int _ramSize = 0;

enum MCU_IDENTIFICATION
{
    MCU_NONE,
    MCU_ESP32_C3,
    MCU_ESP32_C6,
    MCU_ESP32_S3,
};
static MCU_IDENTIFICATION _chipId;
/**
 *
 */
void lnCpuID::identify()
{
    if (_chipId != MCU_NONE)
        return; // already done
    esp_chip_info_t chip_info;
    esp_chip_info(&chip_info);
    switch (chip_info.model)
    {
    case CHIP_ESP32C3:
        _chipId = MCU_ESP32_C3;
        break;
    case CHIP_ESP32C6:
        _chipId = MCU_ESP32_C6;
        break;
    case CHIP_ESP32S3:
        _chipId = MCU_ESP32_S3;
        break;
    default:
        _chipId = MCU_NONE;
        break;
    }
    uint32_t flash_size;
    if (esp_flash_get_size(NULL, &flash_size) == ESP_OK)
    {
        _flashSize = flash_size / 1024;
    }
    else
    {
        _flashSize = 2048; // minimum
    }
    // this is very approximate
    _ramSize = heap_caps_get_free_size(MALLOC_CAP_8BIT | MALLOC_CAP_INTERNAL) / 1024;
}
/**
 *
 * @return
 */
lnCpuID::LN_VENDOR lnCpuID::vendor()
{
    return lnCpuID::LN_VENDOR_ESP;
}
/**
 *
 * @return
 */
lnCpuID::LN_MCU lnCpuID::mcu()
{
    switch (_chipId)
    {
    case MCU_ESP32_C3:
    case MCU_ESP32_C6:
        return LN_MCU_RISCV;
        break;
    case MCU_ESP32_S3:
        return LN_MCU_XTENSA;
        break;
    default:
        return LN_MCU_UNKNOWN;
        break;
    }
}
/**
 */
const char *lnCpuID::mcuAsString()
{
    switch (_mcu)
    {
    case LN_MCU_RISCV:
        return "RISCV";
        break;
    case LN_MCU_XTENSA:
        return "XTENSA";
        break;
    default:
        return "????";
    }
}

/**
 *
 * @return
 */
int lnCpuID::flashSize()
{
    return _flashSize;
}
/**
 *
 * @return
 */
int lnCpuID::ramSize()
{
    return _ramSize;
}
/**
 *
 * @return
 */
const char *lnCpuID::idAsString()
{
    switch (_chipId)
    {
    case MCU_ESP32_C3:
        return "ESP32C3";
        break;
    case MCU_ESP32_C6:
        return "ESP32C6";
        break;
    case MCU_ESP32_S3:
        return "ESP32S3";
        break;
    default:
        break;
    }
    return "???";
}
/**
 */
int lnCpuID::clockSpeed()
{
    uint32_t cpu_freq_hz;
    if (ESP_OK == esp_clk_tree_src_get_freq_hz((soc_module_clk_t)SOC_MOD_CLK_CPU, ESP_CLK_TREE_SRC_FREQ_PRECISION_EXACT,
                                               &cpu_freq_hz))
    {
        return cpu_freq_hz;
    }
    Logger("Cannot get clock speed\n");
    return 120000000;
}
// EOF
