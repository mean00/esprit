set(C ${CMAKE_CURRENT_SOURCE_DIR}/rust_esprit/c_interface)

include_directories(${C})

if(LN_ENABLE_USBD)
  include(${ESPRIT_ROOT}/libraries/tinyUsb/tiny.cmake)
  set(EXTRA ${EXTRA} ${C}/lnCDC_c.cpp ${C}/lnUSB_c.cpp)
endif()

if(LN_ENABLE_SPI)
  set(EXTRA ${EXTRA} ${C}/lnSPI_c.cpp)
endif()

if(USE_RP2040 OR USE_RP2350)

else()
  if(LN_ENABLE_TIMING_ADC)
    set(EXTRA ${EXTRA} ${C}/lnTiming_adc_c.cpp)
  endif()
  if(LN_ENABLE_I2C)
    set(EXTRA ${EXTRA} ${C}/lnI2C_c.cpp)
  endif()
endif()

add_library(lnarduino_c_bindings OBJECT ${EXTRA} ${C}/lnFast_EventGroup_c.cpp ${C}/lnSerial_c.cpp)
target_link_libraries(lnarduino_c_bindings PRIVATE esprit_dev)

if(LN_ENABLE_USBD)
  target_link_libraries(lnarduino_c_bindings PRIVATE tinyUsb)
endif()
