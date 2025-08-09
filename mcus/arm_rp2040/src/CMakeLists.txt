SET(RP_CHIP rp2040)
SET(RP_CHIP2 RP2040)
SET(RP_COMMON_FOLDER  ${CMAKE_CURRENT_SOURCE_DIR}/../../rp_common/src)

MACRO(ADD_RP_INCDIR)
  FOREACH(subdir ${ARGN})
    LIST(APPEND rp_inc_dir  ${PICO_SDK_PATH}/src/rp2_common/${subdir}/include)
  ENDFOREACH()
ENDMACRO()
#

IF(EXISTS "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  MESSAGE(STATUS "Detected SDK v2.x")
  SET(RP_SDK_VERSION 2)
  SET(RP_SUFFIX_INCLUDE "_headers")
  SET(VERSION_FILE "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  SET(RP_PLATFORM_INCLUDE "${PICO_SDK_PATH}/src/${RP_CHIP}/pico_platform/include"
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
                          "${PICO_SDK_PATH}/src/rp2_common/boot_bootrom_headers/include"
                          )
  SET(RP_EXTRA_SOURCES ${PICO_SDK_PATH}/src/rp2_common/pico_runtime_init/runtime_init_clocks.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_ticks/ticks.c)
ELSE()
  MESSAGE(FATAL_ERROR "Cannot detect pico sdk v2.X")
ENDIF()

CONFIGURE_FILE(${VERSION_FILE} ${CMAKE_BINARY_DIR}/pico/version.h)


IF(USE_RP2040_PURE_RAM)
  SET(CRT0 ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/crt0_ram.S
           ${CMAKE_CURRENT_SOURCE_DIR}/ln_sys_init_ram.cpp)
ELSE()
  SET(CRT0 ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/crt0.S)
ENDIF()


SET(RP_LN_SOURCES
        ${RP_COMMON_FOLDER}/ln_rp_gpio.cpp
        ${RP_COMMON_FOLDER}/ln_rp_dma.cpp
        ln_rp_spi.cpp
        ln_rp_adc.cpp
        ${RP_COMMON_FOLDER}/ln_rp_pio.cpp
        ${RP_COMMON_FOLDER}/lnStubs.cpp
)
SET(BASE_SOURCES
    ${PICO_SDK_PATH}/src/rp2_common/hardware_timer/timer.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_uart/uart.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_clocks/clocks.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_pll/pll.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_vreg/vreg.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_xosc/xosc.c
    ${PICO_SDK_PATH}/src/rp2_common/hardware_irq/irq_handler_chain.S
    ${PICO_SDK_PATH}/src/rp2_common/hardware_pio/pio.c
)

SET(RP_SDK_SOURCES
  ${BASE_SOURCES}
  ${RP_COMMON_FOLDER}/rp_rcu.cpp
  ${RP_COMMON_FOLDER}/rp_serial.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/ln_rp_stubs.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/ln_rp_stubs_sdk${RP_SDK_VERSION}.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/rp_irq.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/platform.c
  ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy/runtime.c
  ${CRT0}
)
add_library(esprit_lib OBJECT rp.cpp  ${RP_LN_SOURCES} ${RP_SDK_SOURCES} ${RP_EXTRA_SOURCES})

#
ADD_RP_INCDIR(hardware_adc hardware_base hardware_claim hardware_clocks hardware_divider
  hardware_gpio hardware_irq hardware_pll hardware_resets hardware_sync
  hardware_timer hardware_uart hardware_vreg hardware_watchdog hardware_xosc hardware_dma hardware_pio)

ADD_RP_INCDIR(boot_stage2 pico_bootrom pico_double pico_float pico_int64_ops pico_malloc pico_mem_ops
  pico_printf pico_runtime pico_stdio pico_stdio_uart)

target_include_directories(esprit_dev INTERFACE
        ${rp_inc_dir}
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
        ${RP_PLATFORM_INCLUDE}
  )
TARGET_INCLUDE_DIRECTORIES(esprit_dev INTERFACE  ${CMAKE_CURRENT_SOURCE_DIR}/../include)
TARGET_INCLUDE_DIRECTORIES(esprit_dev  INTERFACE ../private_include)
TARGET_COMPILE_DEFINITIONS(esprit_dev INTERFACE PICO_${RP_CHIP2}=1)
TARGET_COMPILE_DEFINITIONS(esprit_dev INTERFACE RP_SDK_VERSION=${RP_SDK_VERSION})
TARGET_LINK_LIBRARIES(esprit_lib  PRIVATE esprit_dev)
