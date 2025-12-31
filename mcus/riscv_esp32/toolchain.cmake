

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

ADD_IDF_COMPONENT(  "" "include" esp_driver_gpio esp_hw_support "soc/esp32c6" soc riscv esp_common esp_rom heap esp_system hal nvs_flash esp_partition)
ADD_IDF_COMPONENT(  "freertos" "" "FreeRTOS-Kernel/include" "FreeRTOS-Kernel/include/freertos" "FreeRTOS-Kernel/portable/riscv/include/freertos" )
ADD_IDF_COMPONENT(  "freertos" "" "config/include/freertos" "config/include" "config/riscv/include" "esp_additions/include")
set(PPATH "${PPATH} -I${IDF_PATH}/components/newlib/platform_include")
set(PPATH "${PPATH} -I${IDF_PATH}/components/freertos/config/include/freertos/")
#
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} ${PPATH}" CACHE INTERNAL "")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} ${PPATH}" CACHE INTERNAL "")


