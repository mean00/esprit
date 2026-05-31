set(C ${CMAKE_CURRENT_SOURCE_DIR}/rust_esprit/c_interface)

add_library(lnarduino_c_bindings OBJECT)
target_include_directories(lnarduino_c_bindings PRIVATE ${C})
target_link_libraries(lnarduino_c_bindings PRIVATE esprit_dev)

target_sources(lnarduino_c_bindings PRIVATE ${EXTRA} ${C}/lnFast_EventGroup_c.cpp ${C}/lnSerial_c.cpp ${C}/lnGPIO_c.cpp)
if(LN_ENABLE_USBD)
  include(${ESPRIT_ROOT}/libraries/tinyUsb/tiny.cmake)
  target_sources(lnarduino_c_bindings PRIVATE  ${C}/lnCDC_c.cpp ${C}/lnUSB_c.cpp)
  target_link_libraries(lnarduino_c_bindings PRIVATE tinyUsb)
endif()

macro(cond_build FLAG SRC)
  if(${FLAG})
    target_sources(lnarduino_c_bindings PRIVATE ${C}/${SRC})
  endif()
endmacro()

cond_build(LN_ENABLE_SPI $lnSPI_c.cpp)

if(NOT (USE_RP2040 OR USE_RP2350))
  cond_build(LN_ENABLE_TIMING_ADC lnTiming_adc_c.cpp)
  cond_build(LN_ENABLE_I2C lnI2C_c.cpp)
  target_sources(lnarduino_c_bindings PRIVATE ${C}/lnTimer_c.cpp ${C}/lnMultiPulse_c.cpp)
endif()
target_link_libraries(lnarduino_c_bindings PRIVATE esprit_dev)
