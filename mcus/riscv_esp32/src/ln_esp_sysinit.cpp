#include "esprit.h"
#include "lnDebug.h"
#include <nvs_flash.h>
//
#undef printf
//
extern void LoggerInitMutex();
// Custom printf function for ESP, prints 'n' characters from 'data' followed by a newline
void espPrintf(int n, const char *data)
{
    printf("%.*s", n, data);
}

// System initialization function for ESP
void lnEspSysInit()
{
    LoggerInitMutex();
    setLogger(espPrintf); // Set the logger function to espPrintf
                          // make sure nvm is working
#if 0
    esp_err_t ret = nvs_flash_init();
    if (ret == ESP_ERR_NVS_NO_FREE_PAGES || ret == ESP_ERR_NVS_NEW_VERSION_FOUND)
    {
        ESP_ERROR_CHECK(nvs_flash_erase());
        ret = nvs_flash_init();
    }
#endif
}
