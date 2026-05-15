# -----------------------------------------------------------------------------
# CMakeLists for rust_demo
# -----------------------------------------------------------------------------

cmake_minimum_required(VERSION 3.20)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
message(STATUS "======================")
message(STATUS "Starting rust_demo")
message(STATUS "======================")
# ____________________ Options ____________________

set(USE_CLANG
    ON
    CACHE INTERNAL "")
set(LN_ENABLE_RUST
    ON
    CACHE INTERNAL "")
set(LN_ENABLE_I2C
    FALSE
    CACHE INTERNAL "")
set(LN_ENABLE_SPI
    FALSE
    CACHE INTERNAL "")

#
set(LN_USB_NB_CDC
    0
    CACHE INTERNAL "")
set(LN_USB_NB_HID
    0
    CACHE INTERNAL "")
set(LN_USB_DFU_RUNTIME
    1
    CACHE INTERNAL "")

# INCLUDE(./build_default.cmake)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(lnADCTest NONE) # the NONE is very important !

enable_language(C CXX ASM) # this is important too!

#

include_directories(${CMAKE_SOURCE_DIR})
include_directories(${CMAKE_SOURCE_DIR}/INCLUDE)
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

set(EXTENSION "_${LN_MCU}")

include_directories(src)

# _______________________ FPU _______________________
if("${LN_ARCH}" STREQUAL "ARM")
  if("${LN_MCU}" STREQUAL "M3")
    set(FP_LIB qfpm3)
  endif()
endif()

# _______________________ Rust vs C++ _______________________
set(START_CODE ${CMAKE_CURRENT_SOURCE_DIR}/src/cpp/rust_start.cpp)
# SET(CPP_CODE    ${CMAKE_CURRENT_SOURCE_DIR}/src/rs/cpp/lnI2CTask.cpp )
#

generate_gd32_firmware(rust_demo ${START_CODE})
hash_gd32_firmware(rust_demo)
add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/src/)
target_link_libraries(rust_demo PRIVATE lnarduino_c_bindings)
target_link_libraries(rust_demo PRIVATE rsPowerSupply)

# Summary
message(STATUS "Configuration:")
message(STATUS "\tBuild folder ${CMAKE_BINARY_DIR}")
message(STATUS "\tUsing ${LN_MCU} based MCU at ${LN_MCU_SPEED} Hz")
message(STATUS "\tUsing ${LN_MCU_FLASH_SIZE} kB of flash (small = ${USE_SMALLFOOTPRINT})")
message(STATUS "\tUSB control enabled : ${LN_ENABLE_USBD}")

#
set(EXTENSION ${EXTENSION}_RUST)
set_target_properties(rust_demo PROPERTIES OUTPUT_NAME rust_demo${EXTENSION})

message(STATUS "\tFinal name is ${EXTENSION}")
