
macro(ADD_IDF_COMPONENT root suffix)
  foreach(comp ${ARGN})
    set(FULL_PATH "${IDF_PATH}/components/${root}/${comp}/${suffix}")
    if(EXISTS ${FULL_PATH})
      message(STATUS "SUCCESSX: Adding ${FULL_PATH}")
      set(PPATH "${PPATH} -I${FULL_PATH}")
    else()
      message(FATAL_ERROR "FAILED: Path does not exist: ${FULL_PATH}")
    endif()

  endforeach()
endmacro()

message(STATUS "Targeting Espressif MCU : ${LN_ESP_MCU}")

add_idf_component(
  ""
  "include"
  esp_driver_spi
  esp_driver_gpio
  esp_hw_support
  soc
  esp_common
  esp_rom
  heap)
add_idf_component(
  ""
  "include"
  esp_system
  hal
  nvs_flash
  esp_partition
  spi_flash
  esp_adc)
add_idf_component("freertos" "" "config/include/freertos" "config/include" "esp_additions/include")
# IDF 6.x restructure: these headers live in their own components now (hal/gpio_types.h,
# hal/spi_flash_types.h, esp_blockdev.h) and are pulled in by driver/gpio.h, esp_flash.h,
# esp_partition.h respectively.
add_idf_component("" "include" esp_hal_gpio esp_hal_mspi esp_blockdev esp_hal_ana_conv esp_timer)
add_idf_component("esp_hw_support" "include" etm)
set(PPATH "${PPATH} -I${IDF_PATH}/components/esp_libc/platform_include")
set(PPATH "${PPATH} -I${IDF_PATH}/components/freertos/config/include/freertos/")
set(PPATH "${PPATH} -DLN_CUSTOM_FREERTOS=1 ")
if(LN_ESP_MCU STREQUAL "ESP32C6") # ESP32C6
  add_idf_component("esp_rom" "" "esp32c6/include/esp32c6/rom")
  add_idf_component("esp_adc" "" "esp32c6/include") #
  add_idf_component("" "" "soc/esp32c6/register")
  add_idf_component("" "include" "soc/esp32c6" riscv)
  add_idf_component("hal" "" "esp32c6/include/")
  add_idf_component("" "include" "esp_hal_gpio/esp32c6")
  add_idf_component("freertos" "" "FreeRTOS-Kernel/portable/riscv/include/freertos")
  add_idf_component("freertos" "" "config/riscv/include")
  set(CUSTOM_TARGET_FLAGS
      "-march=rv32imac_zicsr"
      CACHE INTERNAL "")
elseif(LN_ESP_MCU STREQUAL "ESP32C3") # ESP32C3
  add_idf_component("esp_rom" "" "esp32c3/include/esp32c3/rom")
  add_idf_component("esp_adc" "" "esp32c3/include") #
  add_idf_component("" "" "soc/esp32c3/register")
  add_idf_component("hal" "" "esp32c3/include/")
  add_idf_component("" "include" "soc/esp32c3" riscv)
  add_idf_component("" "include" "esp_hal_gpio/esp32c3")
  add_idf_component("freertos" "" "FreeRTOS-Kernel/portable/riscv/include/freertos")
  add_idf_component("freertos" "" "config/riscv/include")
  # ESP32-C3 is RV32IMC (no M extension); the swindle fast-GPIO code needs zicsr+zifencei
  set(CUSTOM_TARGET_FLAGS "-march=rv32imc_zicsr_zifencei" CACHE INTERNAL "")
elseif(LN_ESP_MCU STREQUAL "ESP32S3") # ESP32S3
  add_idf_component("esp_rom" "" "esp32s3/include/esp32s3/rom")
  add_idf_component("esp_adc" "" "esp32s3/include") #
  add_idf_component("" "include" "soc/esp32s3" xtensa)
  add_idf_component("hal" "" "esp32s3/include/")
  # ADD_IDF_COMPONENT("soc" "" "soc/esp32c6/register" )
  add_idf_component("" "include" "esp_hal_gpio/esp32s3")
  add_idf_component("freertos" "" "FreeRTOS-Kernel/portable/xtensa/include/freertos" "config/xtensa/include")
  add_idf_component("xtensa" "include" "esp32s3")
  add_idf_component("soc" "" "esp32s3/register")
  set(IDF_CPN_EXTRA "-DCRITICAL_SECTION_EXTRA_ARG=1 -mlongcalls")
endif()
#
# LN_ESPRESSIF marks Espressif/ESP-IDF builds (set for all ESP32 C3/C6/S3) so that
# __riscv-specific Espressif workarounds in the generic esprit headers stay off for
# the other RISC-V targets (CH32V3x, GD32) compiled from the same sources.
set(CMAKE_C_FLAGS
    "${CMAKE_C_FLAGS} ${PPATH} ${IDF_CPN_EXTRA} ${CUSTOM_TARGET_FLAGS} -DLN_ESPRESSIF=1"
    CACHE INTERNAL "")
set(CMAKE_CXX_FLAGS
    "${CMAKE_CXX_FLAGS} ${PPATH} ${IDF_CPN_EXTRA} ${CUSTOM_TARGET_FLAGS} -DLN_ESPRESSIF=1"
    CACHE INTERNAL "")
