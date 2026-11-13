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
# The GPIO / CPU-id bridges exist *only* for the Rust side. When the pure-Rust
# backends own those peripherals (LN_ENABLE_RUST on the bluepill and RP2xxx targets:
# rs_gpio_bluepill / rs_gpio_rp2xxx, the Rust CpuId), nothing calls them any more
# and the linker drops every one of their symbols. ESP32 keeps them: its Rust path
# goes through these esprit c_api bindings.
#
# The EXTI bridge (lnExti_c.cpp) is gone: Rust reaches pin interrupts through
# `rs_mcu_traits::exti::Exti`, implemented by `rs_exti_bluepill`,
# `rs_exti_rp2xxx` (IO_BANK0) and `rs_exti_wrapped` — nothing on the Rust side
# calls `lnExti*_c` any more. C++ still uses `lnExtiAttachInterrupt` directly.
set(_esprit_rust_c_bridges ${C}/lnGPIO_c.cpp ${C}/lnCpuID_c.cpp)
if(LN_ENABLE_RUST)
  if(USE_RP2040 OR USE_RP2350)
    set(_esprit_rust_c_bridges ${C}/lnGPIO_c.cpp)
  elseif(NOT ("${LN_MCU}" STREQUAL "ESP32"))
    message(STATUS "Rust GPIO/CPU-id bridges: not built (the Rust backends own them)")
    set(_esprit_rust_c_bridges "")
  endif()
endif()
target_sources(esprit_c_bindings PRIVATE ${EXTRA} ${C}/lnFast_EventGroup_c.cpp ${_esprit_rust_c_bridges})
# lnSerial_c.cpp is only for the backends that reach the UART through it: the
# ESP32 `rs_uart_wrapped` crate and every non-Rust build. On a Rust build of the
# bluepill / RP2xxx the UART is `rs_uart_bluepill` / `rs_uart_rp2xxx`, i.e. the
# `rs_mcu_traits::uart::Uart` super trait, and nothing calls `lnserial_*`.
if(NOT LN_ENABLE_RUST OR "${LN_MCU}" STREQUAL "ESP32")
  target_sources(esprit_c_bindings PRIVATE ${C}/lnSerial_c.cpp)
endif()
if(LN_ENABLE_USBD)
  include(${ESPRIT_ROOT}/libraries/tinyUsb/tiny.cmake)
  # lnUSB_c.cpp is the C ABI the Rust USB stack sits on: `rs_tinyusb` only *calls*
  # lnusb_create/lnusb_start/lnusb_setEventHandler/... (rs_tinyusb/src/c_usb.rs
  # declares them `extern`), so the bridge has to be compiled on a Rust build too.
  # lnCDC_c.cpp still has live C callers, so it stays as well.
  target_sources(esprit_c_bindings PRIVATE ${C}/lnCDC_c.cpp ${C}/lnUSB_c.cpp)
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
endif()
# No ADC bridge on RP2xxx any more: `rs_adc_rp2xxx` is a pure-Rust driver on the
# ADC block, so neither the C++ `lnSimpleADC` (mcus/arm_rp2xxx/src/ln_rp_adc.cpp)
# nor the `ln_rp_simple_adc_c.cpp` shim that exposed it is part of a Rust build.
target_link_libraries(esprit_c_bindings PRIVATE esprit_dev)
