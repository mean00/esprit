set(CMAKE_POLICY_DEFAULT_CMP0077 NEW)

cmake_minimum_required(VERSION 3.20)
message(STATUS "======================")
message(STATUS "Starting rust CDC test")
message(STATUS "======================")

set(LN_ENABLE_USBD
    TRUE
    CACHE INTERNAL "")
set(LN_ENABLE_I2C
    FALSE
    CACHE INTERNAL "")
set(LN_ENABLE_SPI
    FALSE
    CACHE INTERNAL "")
set(LN_ENABLE_RUST
    TRUE
    CACHE INTERNAL "")
set(LN_USB_NB_CDC 1) # 1 CDC interface
set(LN_USB_NB_HID 0) # No HID

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(rust_cdc NONE) # the NONE is very important !

enable_language(C CXX ASM) # this is important too!

include_directories(${CMAKE_CURRENT_SOURCE_DIR})
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src)

add_subdirectory(${ESPRIT_ROOT})
add_subdirectory(rust)
include(${ESPRIT_ROOT}/setup.cmake)

use_library(tinyUsb)
use_library(ln-rtt)

include(${ESPRIT_ROOT}/libraries/tinyUsb/tiny.cmake)

generate_gd32_firmware(rust_cdc src/demo.cpp)
target_link_libraries(rust_cdc PRIVATE rust_demo)
