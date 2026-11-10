set(C ${CMAKE_CURRENT_SOURCE_DIR}/rust_esprit/c_interface)

add_library(esprit_c_bindings OBJECT)
target_include_directories(esprit_c_bindings PRIVATE ${C})
target_link_libraries(esprit_c_bindings PRIVATE esprit_dev)

# `make rust_bindings_check` re-runs bindgen into a temporary directory and diffs
# the result against the checked-in files (rs_rtos: src/c_freertos.rs,
# src/c_fast_event.rs).  A manual target, never part of ALL: it needs python3 +
# bindgen + libclang, which a plain firmware build does not.
find_package(Python3 COMPONENTS Interpreter QUIET)
if(Python3_Interpreter_FOUND)
  add_custom_target(
    rust_bindings_check
    COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/rs_rtos/c_interface/bindgen.py
            --check
    COMMENT "Checking the checked-in rs_rtos bindings against their headers"
    USES_TERMINAL
    VERBATIM)
endif()

# lnFast_EventGroup_c.cpp is not dead: the Rust side (rs_rtos / event.rs) calls it.
#
# The GPIO / EXTI / CPU-id bridges exist *only* for the Rust side. When the pure-Rust
# backends own those peripherals (LN_ENABLE_RUST on the bluepill and RP2xxx targets:
# rs_gpio_bluepill / rs_gpio_rp2xxx, rs_exti_bluepill, the Rust CpuId), nothing calls
# them any more and the linker drops every one of their symbols. ESP32 keeps them:
# its Rust path goes through these esprit c_api bindings.
set(_esprit_rust_c_bridges ${C}/lnGPIO_c.cpp ${C}/lnExti_c.cpp ${C}/lnCpuID_c.cpp)
if(LN_ENABLE_RUST AND NOT ("${LN_MCU}" STREQUAL "ESP32"))
  message(STATUS "Rust GPIO/EXTI/CPU-id bridges: not built (the Rust backends own them)")
  set(_esprit_rust_c_bridges "")
endif()
target_sources(esprit_c_bindings PRIVATE ${EXTRA} ${C}/lnFast_EventGroup_c.cpp ${C}/lnSocket_c.cpp ${_esprit_rust_c_bridges})
if(NOT (LN_ENABLE_RUST AND NOT (USE_RP2040 OR USE_RP2350)))
  target_sources(esprit_c_bindings PRIVATE ${C}/lnSerial_c.cpp)
endif()
if(LN_ENABLE_USBD)
  include(${ESPRIT_ROOT}/libraries/tinyUsb/tiny.cmake)
  # lnUSB_c.cpp is a Rust-facing bridge too, and on a Rust build the USB C ABI is
  # rs_tinyusb's. lnCDC_c.cpp still has live users, so it stays.
  set(_esprit_usb_c_bridge ${C}/lnCDC_c.cpp ${C}/lnUSB_c.cpp)
  if(LN_ENABLE_RUST AND NOT ("${LN_MCU}" STREQUAL "ESP32"))
    set(_esprit_usb_c_bridge ${C}/lnCDC_c.cpp)
  endif()
  target_sources(esprit_c_bindings PRIVATE ${_esprit_usb_c_bridge})
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
  # Timing-ADC / timer / multi-pulse bridges: same story as the GPIO ones above.
  # With LN_ENABLE_RUST the pure-Rust bluepill backends own that C ABI
  # (rs_adc_bluepill for the timing ADC, rs_timer_bluepill, rs_multi_pulse_bluepill)
  # and nothing links these files. ESP32 keeps them (c_api path).
  if((NOT LN_ENABLE_RUST) OR ("${LN_MCU}" STREQUAL "ESP32"))
    cond_build(LN_ENABLE_TIMING_ADC lnTiming_adc_c.cpp)
    target_sources(esprit_c_bindings PRIVATE ${C}/lnTimer_c.cpp ${C}/lnMultiPulse_c.cpp)
  endif()
  # When LN_ENABLE_RUST is active, the pure native Rust driver rs_i2c_bluepill
  # provides the lni2c_* C ABI symbols, so the C++ bridge lnI2C_c.cpp must not be compiled.
  if(NOT LN_ENABLE_RUST)
    cond_build(LN_ENABLE_I2C lnI2C_c.cpp)
  endif()
else()
  # RP2xxx has no lnTimingAdc, but it does have the C++ lnSimpleADC driver
  # (mcus/arm_rp2xxx/src/ln_rp_adc.cpp).  This bridge lets rs_adc_rp2xxx, the
  # Rust ADC backend used on RP2040/RP2350, drive it.
  target_sources(esprit_c_bindings PRIVATE ${C}/ln_rp_simple_adc_c.cpp)
endif()
target_link_libraries(esprit_c_bindings PRIVATE esprit_dev)
