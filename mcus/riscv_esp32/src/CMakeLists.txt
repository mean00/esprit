
add_library(esprit_lib OBJECT)
target_sources(esprit_lib PRIVATE ln_esp_gpio.cpp ln_esp_misc.cpp ln_esp_sysinit.cpp ln_esp_cpuId.cpp ln_esp_todo.cpp)
target_link_libraries(esprit_lib PRIVATE esprit_dev) # idf::driver idf::nvs_flash)
