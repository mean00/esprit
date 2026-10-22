set(FOS
    ${ESPRIT_ROOT}/FreeRTOS
    CACHE INTERNAL "")
include(${LN_MCU_FOLDER}/freeRTOS_port_${LN_EXT}.cmake)

foreach(common event_groups.c list.c queue.c stream_buffer.c tasks.c timers.c)
  list(APPEND CMN ${FOS}/${common})
endforeach(
  common
  event_groups.c
  list.c
  queue.c
  stream_buffer.c
  tasks.c
  timers.c)

add_definitions("-DLN_FREERTOS_HEAP_SIZE=(${LN_MCU_RAM_SIZE}-${LN_MCU_STATIC_RAM}-1)")

set(MEM ${FOS}/portable/MemMang/heap_4.c)
#
set(FOS_SOURCES
    ${CMN} ${LN_FREERTOS_PORT_SOURCES} ${MEM}
    CACHE INTERNAL "")
message(
  STATUS
    "Using internal freertos with config in ${ESPRIT_ROOT}/freertos_config for target ${LN_EXT} boards ${LN_BOARD_NAME}"
)
#
target_include_directories(esprit_dev INTERFACE ${ESPRIT_ROOT}/freertos_config)
target_include_directories(esprit_dev INTERFACE ${LN_FREERTOS_PORT})
target_include_directories(esprit_dev INTERFACE ${CMAKE_SOURCE_DIR})
target_include_directories(esprit_dev INTERFACE ${FOS}/)
target_include_directories(esprit_dev INTERFACE ${FOS}/..)
target_include_directories(esprit_dev INTERFACE ${FOS}/include)
#
add_library(FreeRTOS OBJECT)
target_sources(FreeRTOS PRIVATE ${FOS_SOURCES})
target_link_libraries(FreeRTOS PUBLIC esprit_dev)
#
# ---------------------------------------------------------------------------
# FreeRTOS object sizes -> ${CMAKE_BINARY_DIR}/freertos_struct.h
#                    and -> ${ESPRIT_ROOT}/rust/rs_rtos/src/kernel/esprit/generated.rs
#
# The probe is compiled with the very same flags/includes/config as the kernel
# (it consumes esprit_dev) and is never linked into the firmware: it only exists
# so that nm can report sizeof(StaticTask_t), sizeof(StaticSemaphore_t), ... for
# this build.  Those numbers are what the per-instance _storage members of
# lnFreeRTOS_pp.h have to match, and what the Rust constants of the in-tree
# kernel (rs_rtos: STATIC_QUEUE_BYTES, TICK_RATE_HZ) are generated from - the
# generated file is only rewritten when a number changes, and the consumer makes
# this target a prerequisite of its cargo build (swindle/rs/CMakeLists.txt).
# ---------------------------------------------------------------------------
if(CMAKE_NM)
  add_library(freertos_struct_probe OBJECT ${ESPRIT_ROOT}/cmake/lnFreeRTOS_structs_probe.c)
  target_link_libraries(freertos_struct_probe PUBLIC esprit_dev)
  add_custom_target(
    freertos_struct_header ALL
    COMMAND
      ${CMAKE_COMMAND} -DNM=${CMAKE_NM} -DOBJ=$<TARGET_OBJECTS:freertos_struct_probe>
      -DRUST_OUT=${ESPRIT_ROOT}/rust/rs_rtos/src/kernel/esprit/generated.rs
      -DOUT=${CMAKE_BINARY_DIR}/freertos_struct.h "-DWHO=${LN_ARCH}/${LN_MCU} ${LN_BOARD_NAME}" -P
      ${ESPRIT_ROOT}/cmake/lnFreeRTOS_structs.cmake
    DEPENDS freertos_struct_probe
    COMMENT "Measuring FreeRTOS objects -> ${CMAKE_BINARY_DIR}/freertos_struct.h"
    VERBATIM)
else()
  message(STATUS "No nm found, freertos_struct.h will not be generated")
endif()
#
