#
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../../private_include)

set(OVERLAY_SRCS
    lnSerialCore.cpp
    lnGPIO.cpp
    lnDma.cpp
    lnSystemInit.cpp
    lnRCU.cpp
    lnExti.cpp
    lnBasicTimer.cpp
    lnBasicDelayTimer.cpp
    lnDelayTimer.cpp
    lnTimer.cpp
    lnDmaTimer.cpp
    lnMultiPulse.cpp
    # lnStopWatch.cpp
    lnDAC.cpp
    lnADC.cpp
    lnADC_base.cpp
    lnADC_timing.cpp
    lnFMC.cpp
    lnStubs.cpp
    # lnSerialTxOnly*.cpp live further down (USE_RUST_LOGGER)
    # ${LN_MCU_FOLDER}/boards/${LN_BOARD_NAME}/lnPinMapping.cpp
    CACHE INTERNAL "")

# Sources whose C++ symbols have no caller left once the Rust backends own the
# peripheral. They are simply not compiled in a Rust build (LN_ENABLE_RUST, which
# swindle always sets), and they are back in every non-Rust build (the esprit demo
# projects can turn LN_ENABLE_RUST off). Their Rust counterparts:
#   lnADC.cpp / lnADC_base.cpp / lnADC_timing.cpp -> rs_adc_bluepill (simple + timing ADC)
#   lnDmaTimer.cpp                                -> rs_dma_bluepill (WS2812B, ws2812b crate)
#   lnMultiPulse.cpp                              -> rs_multi_pulse_bluepill
#   lnDAC.cpp                                     -> nothing at all on these boards (no DAC)
#   lnStubs.cpp                                   -> libc glue nothing references any more
# Nothing here is deleted: the files stay for the non-Rust builds.
set(OVERLAY_SRCS_RUST_REPLACED
    lnADC.cpp
    lnADC_base.cpp
    lnADC_timing.cpp
    lnDmaTimer.cpp
    lnMultiPulse.cpp
    lnDAC.cpp
    lnStubs.cpp)
set(OVERLAY_SRCS_BUILD ${OVERLAY_SRCS})
if(LN_ENABLE_RUST)
  target_compile_definitions(esprit_dev INTERFACE LN_RUST_BLUEPILL=1)
  message(STATUS "Rust backends own them, not compiled: ${OVERLAY_SRCS_RUST_REPLACED}")
  list(REMOVE_ITEM OVERLAY_SRCS_BUILD ${OVERLAY_SRCS_RUST_REPLACED})
  if(USE_RUST_LOGGER)
    list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnDma.cpp)
    # lnSerialCore.cpp holds createLnSerialTxOnly() - whose only caller is the C++
    # logger that USE_RUST_LOGGER replaces - and the USART0..3_IRQHandler entries,
    # which rs_uart_bluepill exports itself (rs_uart_bluepill/src/irq.rs). The C++
    # Tx-only drivers it would serve are gated on USE_RUST_LOGGER too (below), so the
    # file leaves the build only when they do. Every non-Rust build keeps it.
    list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnSerialCore.cpp)
  endif()
  # The general/basic timer drivers are only kept alive by the ARM vector table
  # (`TIMER1..3` from lnDelayTimer.cpp, `TIMER5..6` from lnBasicDelayTimer.cpp, see
  # lnIRQ_arm_vector.h): no C++ code in a Rust build instantiates lnBasicTimer,
  # lnBasicDelayTimer or lnDelayTimer. rs_timer_bluepill exports those entries
  # (rs_timer_bluepill/src/irq.rs) and owns the delay-timer engine.
  list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnBasicTimer.cpp lnBasicDelayTimer.cpp lnDelayTimer.cpp)
  # lnExti.cpp: rs_exti_bluepill owns the engine (attach/detach/enable/disable, the
  # per-line dispatch) and exports the ARM vector entries plus the two C++ entries
  # `lnSystemInit.cpp` and `lnRunTime_arm_gd32fx.cpp` still reference
  # (rs_exti_bluepill/src/irq.rs).
  list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnExti.cpp)
  # lnTimer.cpp: only its `abTimers[]` table is still referenced (by the inline
  # lnTimerWatch the SDI tap uses). rs_timer_bluepill exports that table
  # (rs_timer_bluepill/src/irq.rs) and owns the timer engine; the lnTimer class has no
  # user left in a Rust build.
  list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnTimer.cpp)
  # lnGPIO.cpp: rs_gpio_bluepill owns the pins (write/read/toggle/mode/open-drain, the
  # port register accessors) and rs_afio_bluepill owns lnRemapTimerPin; the C++ entries
  # are exported from those crates (rs_gpio_bluepill/src/c_api.rs, rs_afio_bluepill) and
  # lnGPIO.h declares them `extern "C"` in a Rust build (LN_RUST_BLUEPILL).
  # lnFastIO's constructor is inline in lnFastGpio.h for the same reason.
  list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnGPIO.cpp)
  # lnRCU.cpp: rs_rcu_bluepill owns the per-family APB/AHB enable/reset tables and their
  # AHB bit maps, and exports the three actions plus the `arcu` register pointer the
  # family files dereference (rs_rcu_bluepill/src/c_api.rs). lnRCU.h forwards to them.
  list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnRCU.cpp)
  # lnFMC.cpp is the 90 byte shim that #includes the family lnFMC_impl.cpp. The Rust
  # build needs three of those entries - FMC_hasFastUnlock() for lnCpuID_arm_gd32fx.cpp,
  # lnInitFlash_ch32v3x() for lnRunTime_riscv_ch32v3x.cpp - and rs_fmc_bluepill owns
  # them (a new crate: the family impl had no Rust twin). Both C++ files stay on disk.
  list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnFMC.cpp)
  if(LN_EXT STREQUAL "arm_gd32fx" OR LN_EXT STREQUAL "riscv_ch32v3x")
    list(REMOVE_ITEM OVERLAY_SRCS_BUILD lnSystemInit.cpp)
  endif()
endif()
if(NOT OVERLAY_SRCS_BUILD AND USE_RUST_LOGGER AND (NOT LN_ENABLE_UART OR LN_ENABLE_RUST) AND (NOT LN_ENABLE_USBD OR LN_RUST_USB))
  add_library(esprit_lib INTERFACE)
  target_link_libraries(esprit_lib INTERFACE esprit_dev)
  target_include_directories(esprit_lib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/include)
else()
  add_library(esprit_lib OBJECT ${OVERLAY_SRCS_BUILD})
  if(LN_ENABLE_RUST AND USE_RUST_LOGGER)
    target_compile_definitions(esprit_lib PRIVATE LN_RUST_DMA=1)
  endif()
  target_link_libraries(esprit_lib PUBLIC esprit_dev)
  target_include_directories(esprit_lib PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include)

  if(NOT USE_RUST_LOGGER)
    target_sources(esprit_lib PRIVATE lnSerialTxOnly.cpp lnSerialTxOnlyDma.cpp lnSerialTxOnlyDmaBuffer.cpp)
  endif()

  if(LN_ENABLE_UART)
    if(NOT LN_ENABLE_RUST)
      target_sources(esprit_lib PRIVATE lnSerialRxTx.cpp lnSerialRxTxDma.cpp)
    endif()
    target_compile_options(esprit_lib PRIVATE "-DLN_ENABLE_UART=1")
  endif()

  macro(ALTER nm text)
    if(${nm})
      message(STATUS "Altering ${text} to ${${nm}}")
      target_compile_definitions(esprit_lib PRIVATE ${nm}=${${nm}})
    endif()
  endmacro()

  alter(LN_INITIAL_STACK_SIZE "initial stack size")
  alter(LN_INITIAL_TASK_PRIORITY "initial task priority")

  target_compile_definitions(esprit_lib PRIVATE LN_MCU_SPEED=${LN_MCU_SPEED})
  target_include_directories(
    esprit_lib
    PRIVATE arduinoLayer/include
            include
            FreeRTOS/include
            .
            ${LN_MCU_FOLDER}/boards
            private_include
            ${LN_MCU_FOLDER}/include)
  if(LN_USE_INTERNAL_CLOCK)
    target_compile_definitions(esprit_lib PRIVATE LN_USE_INTERNAL_CLOCK)
  endif(LN_USE_INTERNAL_CLOCK)

  if(LN_ENABLE_I2C)
    if(NOT LN_ENABLE_RUST)
      target_sources(esprit_lib PRIVATE lnI2C.cpp lnI2C_impl.cpp)
    endif()
  endif(LN_ENABLE_I2C)

  if(LN_ENABLE_SPI)
    if(NOT LN_ENABLE_RUST)
      target_sources(esprit_lib PRIVATE lnSPI_bp.cpp lnSPI_bp_dma.cpp)
    endif()
  endif(LN_ENABLE_SPI)
endif()

#
