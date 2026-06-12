set(RP_CHIP rp2040)
set(RP_CHIP2 RP2040)
set(RP_COMMON_FOLDER ${CMAKE_CURRENT_SOURCE_DIR}/../../rp_common/src)

macro(ADD_RP_INCDIR)
  foreach(subdir ${ARGN})
    list(APPEND rp_inc_dir ${PICO_SDK_PATH}/src/rp2_common/${subdir}/include)
  endforeach()
endmacro()
#

if(EXISTS "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  message(STATUS "Detected SDK v2.x")
  set(RP_SDK_VERSION 2)
  set(RP_SUFFIX_INCLUDE "_headers")
  set(VERSION_FILE "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  set(RP_PLATFORM_INCLUDE
      "${PICO_SDK_PATH}/src/${RP_CHIP}/pico_platform/include"
      "${PICO_SDK_PATH}/src/rp2_common/pico_platform_compiler/include"
      "${PICO_SDK_PATH}/src/rp2_common/pico_platform_common/include"
      "${PICO_SDK_PATH}/src/rp2_common/pico_platform_sections/include"
      "${PICO_SDK_PATH}/src/rp2_common/pico_platform_panic/include"
      "${PICO_SDK_PATH}/src/rp2_common/pico_flash/include"
      "${PICO_SDK_PATH}/src/rp2_common/hardware_sync_spin_lock/include/"
      "${PICO_SDK_PATH}/src/rp2_common/hardware_ticks/include/"
      "${PICO_SDK_PATH}/src/rp2_common/hardware_boot_lock/include/"
      "${PICO_SDK_PATH}/src/rp2_common/pico_runtime_init/include/"
      "${PICO_SDK_PATH}/src/common/hardware_claim/include/"
      "${PICO_SDK_PATH}/src/rp2_common/boot_bootrom_headers/include")
  set(RP_EXTRA_SOURCES ${PICO_SDK_PATH}/src/rp2_common/pico_runtime_init/runtime_init_clocks.c
                       ${PICO_SDK_PATH}/src/rp2_common/hardware_ticks/ticks.c)
else()
  message(FATAL_ERROR "Cannot detect pico sdk v2.X")
endif()

configure_file(${VERSION_FILE} ${CMAKE_BINARY_DIR}/pico/version.h)

if(USE_RP2040_PURE_RAM)
  set(CRT0 ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/crt0_ram.S ${CMAKE_CURRENT_SOURCE_DIR}/ln_sys_init_ram.cpp)
else()
  set(CRT0 ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/crt0.S)
endif()

set(RP_LN_SOURCES ${RP_COMMON_FOLDER}/ln_rp_gpio.cpp ${RP_COMMON_FOLDER}/ln_rp_dma.cpp ${RP_COMMON_FOLDER}/ln_rp_spi.cpp ln_rp_adc.cpp
                  ${RP_COMMON_FOLDER}/ln_rp_pio.cpp ${RP_COMMON_FOLDER}/lnStubs.cpp)
set(BASE_SOURCES
    ${PICO_SDK_PATH}/src/rp2_common/hardware_timer/timer.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_uart/uart.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_clocks/clocks.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_pll/pll.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_vreg/vreg.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_xosc/xosc.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_irq/irq_handler_chain.S
    ${PICO_SDK_PATH}/src/rp2_common/hardware_pio/pio.c)

set(RP_SDK_SOURCES
    ${BASE_SOURCES}
    ${RP_COMMON_FOLDER}/rp_rcu.cpp
    ${RP_COMMON_FOLDER}/rp_serial.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/ln_rp_stubs.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/ln_rp_stubs_sdk${RP_SDK_VERSION}.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/rp_irq.cpp
    ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/platform.c
    ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/runtime.c
    ${CRT0})
add_library(esprit_lib OBJECT rp.cpp ${RP_LN_SOURCES} ${RP_SDK_SOURCES} ${RP_EXTRA_SOURCES})

#
add_rp_incdir(
  hardware_adc
  hardware_base
  hardware_claim
  hardware_clocks
  hardware_divider
  hardware_gpio
  hardware_irq
  hardware_pll
  hardware_resets
  hardware_sync
  hardware_timer
  hardware_uart
  hardware_vreg
  hardware_watchdog
  hardware_xosc
  hardware_dma
  hardware_pio)

add_rp_incdir(
  boot_stage2
  pico_bootrom
  pico_double
  pico_float
  pico_int64_ops
  pico_malloc
  pico_mem_ops
  pico_printf
  pico_runtime
  pico_stdio
  pico_stdio_uart)

target_include_directories(
  esprit_dev
  INTERFACE ${rp_inc_dir}
            ${PICO_SDK_PATH}
            ${PICO_SDK_PATH}/src/boards/include
            ${PICO_SDK_PATH}/src/common/pico_base${RP_SUFFIX_INCLUDE}/include
            ${PICO_SDK_PATH}/src/common/pico_binary_info/include
            ${PICO_SDK_PATH}/src/common/pico_bit_ops${RP_SUFFIX_INCLUDE}/include
            ${PICO_SDK_PATH}/src/common/pico_divider${RP_SUFFIX_INCLUDE}/include
            ${PICO_SDK_PATH}/src/common/pico_stdlib${RP_SUFFIX_INCLUDE}/include
            ${PICO_SDK_PATH}/src/common/pico_sync/include
            ${PICO_SDK_PATH}/src/common/pico_time/include
            ${PICO_SDK_PATH}/src/common/pico_util/include
            "${PICO_SDK_PATH}/src/${RP_CHIP}/hardware_regs/include"
            "${PICO_SDK_PATH}/src/${RP_CHIP}/hardware_structs/include"
            ${PICO_SDK_PATH}/src/rp2_common/boot_stage2/asminclude
            ${RP_PLATFORM_INCLUDE})
target_include_directories(esprit_dev INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/../include)
target_include_directories(esprit_dev INTERFACE ../private_include)
target_include_directories(esprit_dev INTERFACE ${RP_COMMON_FOLDER}/../include)
target_include_directories(esprit_dev INTERFACE ${RP_COMMON_FOLDER}/../private_include)
target_compile_definitions(esprit_dev INTERFACE PICO_${RP_CHIP2}=1)
target_compile_definitions(esprit_dev INTERFACE RP_SDK_VERSION=${RP_SDK_VERSION})
target_link_libraries(esprit_lib PRIVATE esprit_dev)
