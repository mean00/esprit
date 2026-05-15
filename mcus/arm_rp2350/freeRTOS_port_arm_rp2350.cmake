set(FOS ${ESPRIT_ROOT}/FreeRTOS)

include_directories(${PICO_SDK_PATH}/src/common/pico_base${RP_SUFFIX_INCLUDE}/include)
include_directories(${PICO_SDK_PATH}/src/common/pico_base_headers/include)
include_directories(${PICO_SDK_PATH}/)
include_directories(${PICO_SDK_PATH}/src/rp2350/pico_platform/include)
include_directories(${PICO_SDK_PATH}/src/rp2350/hardware_regs/include)
include_directories(${PICO_SDK_PATH}/src/rp2350/hardware_structs/include)

include_directories(${PICO_SDK_PATH}/src/rp2_common/pico_platform_compiler/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/pico_platform_sections/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/pico_platform_panic/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/hardware_sync/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/hardware_base/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/hardware_sync_spin_lock/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/hardware_clocks/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/hardware_exception/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/hardware_irq/include)
include_directories(${PICO_SDK_PATH}/src/rp2_common/hardware_timer/include)

include_directories(${PICO_SDK_PATH}/src/common/pico_sync/include)
include_directories(${PICO_SDK_PATH}/src/common/pico_time/include)

set(RP2350_PORT_DIR ${LN_MCU_FOLDER}/freertos_2350_port)
set(LN_FREERTOS_PORT
    ${RP2350_PORT_DIR}/
    CACHE INTERNAL "")
set(LN_FREERTOS_PORT_SOURCES ${LN_FREERTOS_PORT}/port.c ${LN_FREERTOS_PORT}/portasm.c)
