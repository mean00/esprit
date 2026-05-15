# Select  RP2350
set(RP_CHIP rp2350)
set(RP_CHIP2 RP2350)

macro(ADD_RP_INCDIR)
  foreach(subdir ${ARGN})
    list(APPEND rp_inc_dir ${PICO_SDK_PATH}/src/rp2_common/${subdir}/include)
  endforeach()
endmacro()
#
set(RP_COMMON_FOLDER ${CMAKE_CURRENT_SOURCE_DIR}/../../rp_common/src)
#
if(EXISTS "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  message(STATUS "Detected SDK v2.x")
  set(RP_SDK_VERSION 2)
  set(RP_SUFFIX_INCLUDE "_headers")
  set(VERSION_FILE "${PICO_SDK_PATH}/src/common/pico_base_headers/include/pico/version.h.in")
  set(RP_PLATFORM_INCLUDE
      "${PICO_SDK_PATH}/src/${RP_CHIP}/pico_platform/include"
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
      "${PICO_SDK_PATH}/src/rp2_common/hardware_rcp/include")
  set(RP_EXTRA_SOURCES ${PICO_SDK_PATH}/src/rp2_common/pico_runtime_init/runtime_init_clocks.c
                       ${PICO_SDK_PATH}/src/rp2_common/hardware_ticks/ticks.c)
else()
  message(FATAL_ERROR "Cannot detect pico sdk v2.X")
endif()

configure_file(${VERSION_FILE} ${CMAKE_BINARY_DIR}/pico/version.h)

set(RP2350_SDK ${PICO_SDK_PATH})
set(SDK_SRCS
    ${RP2350_SDK}/src/rp2_common/hardware_irq/irq.c
    # ${RP2350_SDK}/src/rp2_common/pico_platform_panic/panic.c
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
    # ${RP2350_SDK}/src/rp2_common/pico_stdlib/stdlib.c
    # ${RP2350_SDK}/src/rp2_common/pico_cxx_options/new_delete.cpp
    # ${RP2350_SDK}/src/rp2_common/pico_divider/divider_compiler.c
    # ${RP2350_SDK}/src/rp2_common/pico_double/double_math.c
    # ${RP2350_SDK}/src/rp2_common/pico_double/double_aeabi_dcp.S
    # ${RP2350_SDK}/src/rp2_common/pico_double/double_fma_dcp.S
    # ${RP2350_SDK}/src/rp2_common/pico_double/double_sci_m33.S
    # ${RP2350_SDK}/src/rp2_common/pico_double/double_conv_m33.S
    # ${RP2350_SDK}/src/rp2_common/pico_float/float_math.c
    # ${RP2350_SDK}/src/rp2_common/pico_float/float_sci_m33_vfp.S
    # ${RP2350_SDK}/src/rp2_common/pico_float/float_conv_m33.S
    # ${RP2350_SDK}/src/rp2_common/pico_malloc/malloc.c
    # ${RP2350_SDK}/src/rp2_common/pico_printf/printf.c
    # ${RP2350_SDK}/src/rp2_common/pico_clib_interface/newlib_interface.c
    # ${RP2350_SDK}/src/rp2_common/pico_stdio/stdio.c
    # ${RP2350_SDK}/src/rp2_common/pico_stdio_usb/reset_interface.c
    # ${RP2350_SDK}/src/rp2_common/pico_stdio_usb/stdio_usb.c
    # ${RP2350_SDK}/src/rp2_common/pico_stdio_usb/stdio_usb_descriptors.c
)

set(RP_LN_SOURCES
    ${RP_COMMON_FOLDER}/ln_rp_gpio.cpp ${RP_COMMON_FOLDER}/ln_rp_dma.cpp ${RP_COMMON_FOLDER}/ln_rp_pio.cpp
    ${RP_COMMON_FOLDER}/lnStubs.cpp ln_rp_spi.cpp ln_rp_adc.cpp
    # boot.s
)

set(RP_LN_SRCS
    ${RP_COMMON_FOLDER}/rp_serial.cpp ${RP_COMMON_FOLDER}/rp_rcu.cpp
    # ${CMAKE_CURRENT_SOURCE_DIR}/minimum_arm_image_def_block.s
    ${CMAKE_CURRENT_SOURCE_DIR}/rp_irq.cpp)
add_library(esprit_lib OBJECT rp.cpp ${SDK_SRCS} ${RP_LN_SOURCES} ${RP_LN_SRCS})

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
  pico_bootrom
  pico_double
  pico_float
  pico_int64_ops
  pico_malloc
  pico_mem_ops
  pico_printf
  pico_runtime
  pico_stdio
  pico_stdio_uart
  pico_time_adapter
  hardware_flash
  pico_unique_id
  hardware_xip_cache)

list(APPEND rp_inc_dir ${PICO_SDK_PATH}/src/common/boot_picoboot_headers/include)

target_include_directories(esprit_lib PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../sdk_copy)
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
            ${PICO_SDK_PATH}/src/${RP_CHIP}/hardware_regs/include
            ${PICO_SDK_PATH}/src/${RP_CHIP}/hardware_structs/include
            ${PICO_SDK_PATH}/src/rp2_common/boot_stage2/asminclude
            ${PICO_SDK_PATH}/src/common/boot_picobin_headers/include/
            ${RP_PLATFORM_INCLUDE})
target_include_directories(esprit_lib PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/../include)
target_include_directories(esprit_lib PRIVATE ../private_include)
target_link_libraries(esprit_lib PRIVATE esprit_dev)
target_compile_definitions(esprit_lib PUBLIC PICO_${RP_CHIP2}=1 RP_SDK_VERSION=${RP_SDK_VERSION})
