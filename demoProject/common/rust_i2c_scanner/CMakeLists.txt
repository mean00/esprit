set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_POLICY_DEFAULT_CMP0077 NEW)

cmake_minimum_required(VERSION 3.13)
message(STATUS "======================")
message(STATUS "Starting pin test")
message(STATUS "======================")

set(LN_ENABLE_USBD
    FALSE
    CACHE INTERNAL "")
set(LN_ENABLE_I2C
    TRUE
    CACHE INTERNAL "")
set(LN_ENABLE_SPI
    FALSE
    CACHE INTERNAL "")
set(LN_ENABLE_RUST
    TRUE
    CACHE INTERNAL "")
set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
set(ESPRIT_ROOT ${CMAKE_SOURCE_DIR}/esprit/)
include(common_CMakeLists)
set(LN_ENABLE_I2C
    TRUE
    CACHE INTERNAL "")

project(pin_rust NONE) # the NONE is very important !

enable_language(C CXX ASM) # this is important too!

add_subdirectory(${ESPRIT_ROOT})
add_subdirectory(rust)
include(${ESPRIT_ROOT}/setup.cmake)

use_library(ln-rtt)

generate_gd32_firmware(pin src/demo.cpp)
target_link_libraries(pin PRIVATE rust_demo)
