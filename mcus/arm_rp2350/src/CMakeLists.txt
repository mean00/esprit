# Select  RP2350
SET(RP_CHIP rp2350)
SET(RP_CHIP2 RP2350)

MACRO(ADD_RP_INCDIR)
  FOREACH(subdir ${ARGN})
    LIST(APPEND rp_inc_dir  ${PICO_SDK_PATH}/src/rp2_common/${subdir}/include)
  ENDFOREACH()
ENDMACRO()
#
SET(RP_COMMON_FOLDER  ${CMAKE_CURRENT_SOURCE_DIR}/../../rp_common/src)
#
IF(EXISTS "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  MESSAGE(STATUS "Detected SDK v2.x")
  SET(RP_SDK_VERSION 2)
  SET(RP_SUFFIX_INCLUDE "_headers")
  SET(VERSION_FILE "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  SET(RP_PLATFORM_INCLUDE "${PICO_SDK_PATH}/src/${RP_CHIP}/pico_platform/include"
                          "${PICO_SDK_PATH}/src/${RP_CHIP}/boot_stage2/include"
                          "${PICO_SDK_PATH}/src/common/hardware_claim/include/"
                          "${PICO_SDK_PATH}/src/rp2_common/pico_platform_compiler/include"
                          "${PICO_SDK_PATH}/src/rp2_common/pico_platform_common/include"
                          "${PICO_SDK_PATH}/src/rp2_common/pico_platform_sections/include"
                          "${PICO_SDK_PATH}/src/rp2_common/pico_platform_panic/include"
                          "${PICO_SDK_PATH}/src/rp2_common/pico_flash/include"
                          "${PICO_SDK_PATH}/src/rp2_common/hardware_sync_spin_lock/include/"
                          "${PICO_SDK_PATH}/src/rp2_common/hardware_ticks/include/"
                          "${PICO_SDK_PATH}/src/rp2_common/hardware_boot_lock/include/"
                          "${PICO_SDK_PATH}/src/rp2_common/pico_runtime_init/include/"
                          "${PICO_SDK_PATH}/src/rp2_common/boot_bootrom_headers/include"
                          )
  SET(RP_EXTRA_SOURCES ${PICO_SDK_PATH}/src/rp2_common/pico_runtime_init/runtime_init_clocks.c
${PICO_SDK_PATH}/src/rp2_common/hardware_ticks/ticks.c)
ELSE()
  MESSAGE(FATAL_ERROR "Cannot detect pico sdk v2.X")
ENDIF()

CONFIGURE_FILE(${VERSION_FILE} ${CMAKE_BINARY_DIR}/pico/version.h)


SET(RP2350_SDK ${PICO_SDK_PATH})
SET(SDK_SRCS
  ${RP2350_SDK}/src/rp2_common/hardware_irq/irq.c
  #${RP2350_SDK}/src/rp2_common/pico_platform_panic/panic.c
  ${RP2350_SDK}/src/rp2_common/pico_crt0/crt0.S
  ${RP2350_SDK}/src/rp2_common/hardware_gpio/gpio.c
  ${RP2350_SDK}/src/rp2350/pico_platform/platform.c
  ${RP2350_SDK}/src/common/hardware_claim/claim.c
  ${RP2350_SDK}/src/rp2_common/hardware_sync/sync.c
  ${RP2350_SDK}/src/rp2_common/hardware_sync_spin_lock/sync_spin_lock.c
  ${RP2350_SDK}/src/rp2_common/hardware_irq/irq_handler_chain.S
  ${RP2350_SDK}/src/common/pico_sync/sem.c
  ${RP2350_SDK}/src/common/pico_sync/lock_core.c
  ${RP2350_SDK}/src/common/pico_sync/mutex.c
  ${RP2350_SDK}/src/common/pico_sync/critical_section.c
  ${RP2350_SDK}/src/common/pico_time/time.c
  ${RP2350_SDK}/src/common/pico_time/timeout_helper.c
  ${RP2350_SDK}/src/rp2_common/hardware_timer/timer.c
  ${RP2350_SDK}/src/common/pico_util/datetime.c
  ${RP2350_SDK}/src/common/pico_util/pheap.c
  ${RP2350_SDK}/src/common/pico_util/queue.c
  ${RP2350_SDK}/src/rp2_common/hardware_uart/uart.c
  ${RP2350_SDK}/src/rp2_common/hardware_clocks/clocks.c
  ${RP2350_SDK}/src/rp2_common/hardware_pll/pll.c
  ${RP2350_SDK}/src/rp2_common/hardware_vreg/vreg.c
  ${RP2350_SDK}/src/rp2_common/hardware_watchdog/watchdog.c
  ${RP2350_SDK}/src/rp2_common/hardware_ticks/ticks.c
  ${RP2350_SDK}/src/rp2_common/pico_bootrom/bootrom.c
  ${RP2350_SDK}/src/rp2_common/pico_bootrom/bootrom_lock.c
  ${RP2350_SDK}/src/rp2_common/hardware_boot_lock/boot_lock.c
  ${RP2350_SDK}/src/rp2_common/pico_flash/flash.c
  ${RP2350_SDK}/src/rp2_common/hardware_xosc/xosc.c
  ${RP2350_SDK}/src/rp2_common/hardware_divider/divider.c
  ${RP2350_SDK}/src/rp2_common/pico_runtime/runtime.c
  ${RP2350_SDK}/src/rp2_common/pico_runtime_init/runtime_init.c
  ${RP2350_SDK}/src/rp2_common/pico_runtime_init/runtime_init_clocks.c
  ${RP2350_SDK}/src/rp2_common/pico_runtime_init/runtime_init_stack_guard.c
  ${RP2350_SDK}/src/rp2_common/pico_bit_ops/bit_ops_aeabi.S
  ${RP2350_SDK}/src/rp2_common/pico_atomic/atomic.c
  ${RP2350_SDK}/src/rp2_common/pico_standard_binary_info/standard_binary_info.c
  ${RP2350_SDK}/src/rp2_common/pico_unique_id/unique_id.c
  ${RP2350_SDK}/src/rp2_common/hardware_flash/flash.c
  ${RP2350_SDK}/src/rp2_common/hardware_xip_cache/xip_cache.c
#${RP2350_SDK}/src/rp2_common/pico_stdlib/stdlib.c
#${RP2350_SDK}/src/rp2_common/pico_cxx_options/new_delete.cpp
#${RP2350_SDK}/src/rp2_common/pico_divider/divider_compiler.c
#${RP2350_SDK}/src/rp2_common/pico_double/double_math.c
#${RP2350_SDK}/src/rp2_common/pico_double/double_aeabi_dcp.S
#${RP2350_SDK}/src/rp2_common/pico_double/double_fma_dcp.S
#${RP2350_SDK}/src/rp2_common/pico_double/double_sci_m33.S
#${RP2350_SDK}/src/rp2_common/pico_double/double_conv_m33.S
#${RP2350_SDK}/src/rp2_common/pico_float/float_math.c
#${RP2350_SDK}/src/rp2_common/pico_float/float_sci_m33_vfp.S
#${RP2350_SDK}/src/rp2_common/pico_float/float_conv_m33.S
#${RP2350_SDK}/src/rp2_common/pico_malloc/malloc.c
#${RP2350_SDK}/src/rp2_common/pico_printf/printf.c
#${RP2350_SDK}/src/rp2_common/pico_clib_interface/newlib_interface.c
#${RP2350_SDK}/src/rp2_common/pico_stdio/stdio.c
#${RP2350_SDK}/src/rp2_common/pico_stdio_usb/reset_interface.c
#${RP2350_SDK}/src/rp2_common/pico_stdio_usb/stdio_usb.c
#${RP2350_SDK}/src/rp2_common/pico_stdio_usb/stdio_usb_descriptors.c
)





SET(RP_LN_SOURCES
        ${RP_COMMON_FOLDER}/ln_rp_gpio.cpp
        ${RP_COMMON_FOLDER}/ln_rp_dma.cpp
        ${RP_COMMON_FOLDER}/ln_rp_pio.cpp
        ${RP_COMMON_FOLDER}/lnStubs.cpp
        ln_rp_spi.cpp
        ln_rp_adc.cpp
        #boot.s
)

SET(RP_LN_SRCS
 ${RP_COMMON_FOLDER}/rp_serial.cpp
 ${RP_COMMON_FOLDER}/rp_rcu.cpp
  #${CMAKE_CURRENT_SOURCE_DIR}/minimum_arm_image_def_block.s
 ${CMAKE_CURRENT_SOURCE_DIR}/rp_irq.cpp
)
ADD_LIBRARY(esprit_lib OBJECT rp.cpp
  ${SDK_SRCS}
  ${RP_LN_SOURCES}
  ${RP_LN_SRCS}
)

#
ADD_RP_INCDIR(hardware_adc hardware_base hardware_claim hardware_clocks hardware_divider
  hardware_gpio hardware_irq hardware_pll hardware_resets hardware_sync
  hardware_timer hardware_uart hardware_vreg hardware_watchdog hardware_xosc hardware_dma hardware_pio)

ADD_RP_INCDIR(pico_bootrom pico_double pico_float pico_int64_ops pico_malloc pico_mem_ops
  pico_printf pico_runtime pico_stdio pico_stdio_uart pico_time_adapter hardware_flash pico_unique_id hardware_xip_cache)

LIST(APPEND rp_inc_dir  ${PICO_SDK_PATH}/src/common/boot_picoboot_headers/include)

TARGET_INCLUDE_DIRECTORIES(esprit_lib PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy
)
TARGET_INCLUDE_DIRECTORIES(esprit_dev INTERFACE
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
        ${PICO_SDK_PATH}/src/${RP_CHIP}/hardware_regs/include
        ${PICO_SDK_PATH}/src/${RP_CHIP}/hardware_structs/include
        ${PICO_SDK_PATH}/src/rp2_common/boot_stage2/asminclude
        ${PICO_SDK_PATH}/src/common/boot_picobin_headers/include/
        ${RP_PLATFORM_INCLUDE}
  )
TARGET_INCLUDE_DIRECTORIES(esprit_lib PUBLIC  ${CMAKE_CURRENT_SOURCE_DIR}/../include)
TARGET_INCLUDE_DIRECTORIES(esprit_lib  PRIVATE ../private_include)
TARGET_LINK_LIBRARIES(esprit_lib  PRIVATE esprit_dev)
TARGET_COMPILE_DEFINITIONS(esprit_lib PUBLIC PICO_${RP_CHIP2}=1 RP_SDK_VERSION=${RP_SDK_VERSION})
