

add_library(esprit_lib OBJECT)
TARGET_SOURCES(esprit_lib PRIVATE ln_esp_gpio.cpp ln_esp_misc.cpp ln_esp_sysinit.cpp)
TARGET_LINK_LIBRARIES(esprit_lib PRIVATE esprit_dev) # idf::driver idf::nvs_flash)
