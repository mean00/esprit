set(C ${CMAKE_CURRENT_SOURCE_DIR}/rust_esprit/c_interface)

add_library(esprit_c_bindings OBJECT)
target_include_directories(esprit_c_bindings PRIVATE ${C})
target_link_libraries(esprit_c_bindings PRIVATE esprit_dev)

target_sources(esprit_c_bindings PRIVATE ${EXTRA} ${C}/lnFast_EventGroup_c.cpp ${C}/lnSerial_c.cpp ${C}/lnGPIO_c.cpp ${C}/lnExti_c.cpp)
if(LN_ENABLE_USBD)
  include(${ESPRIT_ROOT}/libraries/tinyUsb/tiny.cmake)
  target_sources(esprit_c_bindings PRIVATE  ${C}/lnCDC_c.cpp ${C}/lnUSB_c.cpp)
  target_link_libraries(esprit_c_bindings PRIVATE tinyUsb)
endif()

macro(cond_build FLAG SRC)
  if(${FLAG})
    target_sources(esprit_c_bindings PRIVATE ${C}/${SRC})
  endif()
endmacro()

if(NOT LN_ENABLE_RUST)
  cond_build(LN_ENABLE_SPI lnSPI_c.cpp)
endif()

if(NOT (USE_RP2040 OR USE_RP2350))
  cond_build(LN_ENABLE_TIMING_ADC lnTiming_adc_c.cpp)
  # When LN_ENABLE_RUST is active, the pure native Rust driver rs_i2c_bluepill
  # provides the lni2c_* C ABI symbols, so the C++ bridge lnI2C_c.cpp must not be compiled.
  if(NOT LN_ENABLE_RUST)
    cond_build(LN_ENABLE_I2C lnI2C_c.cpp)
  endif()
  target_sources(esprit_c_bindings PRIVATE ${C}/lnTimer_c.cpp ${C}/lnMultiPulse_c.cpp)
else()
  # RP2xxx has no lnTimingAdc, but it does have the C++ lnSimpleADC driver
  # (mcus/arm_rp2xxx/src/ln_rp_adc.cpp).  This bridge lets rs_adc_rp2xxx, the
  # Rust ADC backend used on RP2040/RP2350, drive it.
  target_sources(esprit_c_bindings PRIVATE ${C}/ln_rp_simple_adc_c.cpp)
endif()
target_link_libraries(esprit_c_bindings PRIVATE esprit_dev)
