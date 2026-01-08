

MACRO(ADD_IDF_COMPONENT root suffix )
  foreach(comp ${ARGN})
    set(FULL_PATH "${IDF_PATH}/components/${root}/${comp}/${suffix}")
    if(EXISTS ${FULL_PATH})
      message(STATUS "SUCCESSX: Adding ${FULL_PATH}")
      SET(PPATH "${PPATH} -I${FULL_PATH}")
    else()
      message(FATAL_ERROR "FAILED: Path does not exist: ${FULL_PATH}")
    endif()

  endforeach()
ENDMACRO()

MESSAGE(STATUS "Targeting Espressif MCU : ${LN_ESP_MCU}")

ADD_IDF_COMPONENT("" "include" esp_driver_gpio esp_hw_support  soc esp_common esp_rom heap)
ADD_IDF_COMPONENT("" "include" esp_system hal nvs_flash esp_partition spi_flash)
ADD_IDF_COMPONENT("freertos" "" "config/include/freertos" "config/include" "esp_additions/include")
set(PPATH "${PPATH} -I${IDF_PATH}/components/newlib/platform_include")
set(PPATH "${PPATH} -I${IDF_PATH}/components/freertos/config/include/freertos/")
set(PPATH "${PPATH} -DLN_CUSTOM_FREERTOS=1 ")
IF(LN_ESP_MCU STREQUAL "ESP32C6") # ESP32C6
  ADD_IDF_COMPONENT("esp_rom" ""   "esp32c6/include/esp32c6/rom" )
  ADD_IDF_COMPONENT("" ""   "soc/esp32c6/register" )
  ADD_IDF_COMPONENT("" "include"  "soc/esp32c6"  riscv )
  ADD_IDF_COMPONENT("freertos" "" "FreeRTOS-Kernel/portable/riscv/include/freertos" )
  ADD_IDF_COMPONENT("freertos" ""  "config/riscv/include" )
ELSEIF(LN_ESP_MCU STREQUAL "ESP32C3") # ESP32C3
  ADD_IDF_COMPONENT("esp_rom" ""   "esp32c3/include/esp32c3/rom" )
  ADD_IDF_COMPONENT("" ""   "soc/esp32c3/register" )
  ADD_IDF_COMPONENT("" "include"  "soc/esp32c3"  riscv )
  ADD_IDF_COMPONENT("freertos" "" "FreeRTOS-Kernel/portable/riscv/include/freertos" )
  ADD_IDF_COMPONENT("freertos" ""  "config/riscv/include" )
ELSEIF(LN_ESP_MCU STREQUAL "ESP32S3") # ESP32S3
  ADD_IDF_COMPONENT("esp_rom" ""   "esp32s3/include/esp32s3/rom" )
  ADD_IDF_COMPONENT("" "include" "soc/esp32s3" xtensa )
  #ADD_IDF_COMPONENT("soc" "" "soc/esp32c6/register" )
  ADD_IDF_COMPONENT("freertos" ""  "FreeRTOS-Kernel/portable/xtensa/include/freertos" "config/xtensa/include"   )
  ADD_IDF_COMPONENT("xtensa" "include"  "esp32s3")
  ADD_IDF_COMPONENT("soc" ""  "esp32s3/register")
  SET(IDF_CPN_EXTRA "-DCRITICAL_SECTION_EXTRA_ARG=1")
ENDIF()
##
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${PPATH} ${IDF_CPN_EXTRA}" CACHE INTERNAL "")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${PPATH} ${IDF_CPN_EXTRA}" CACHE INTERNAL "")


