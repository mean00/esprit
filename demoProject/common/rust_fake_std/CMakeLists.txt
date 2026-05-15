set(CMAKE_POLICY_DEFAULT_CMP0077 NEW)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

cmake_minimum_required(VERSION 3.20)
message(STATUS "======================")
message(STATUS "Starting rust fake_std test suite ")
message(STATUS "======================")
#

# SET(LN_LTO "-flto"  CACHE INTERNAL "") SET(LN_ENABLE_USBD  OFF CACHE INTERNAL "")
set(LN_ENABLE_I2C
    FALSE
    CACHE INTERNAL "")
set(LN_ENABLE_SPI
    FALSE
    CACHE INTERNAL "")
set(ESPRIT_ROOT ${CMAKE_SOURCE_DIR}/esprit)
set(LN_ENABLE_RUST ON CACHED "")

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

set(CMAKE_TOOLCHAIN_FILE ${ESPRIT_ROOT}/esprit.cmake)
set(CMAKE_MODULE_PATH
    ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/cmake ${ESPRIT_ROOT}/cmake
    CACHE INTERNAL "")

project(lnRUST_FAKE_STD NONE) # the NONE is very important !

enable_language(C CXX ASM) # this is important too!

add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)
use_library(ln-rtt)
generate_gd32_firmware(lnDEMO src/demo.cpp)

add_subdirectory(rust)
target_link_libraries(lnDEMO PRIVATE rust_demo)
