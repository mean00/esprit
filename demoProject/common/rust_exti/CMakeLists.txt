SET(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_POLICY_DEFAULT_CMP0077 NEW)

cmake_minimum_required(VERSION 3.13)
MESSAGE(STATUS "======================")
MESSAGE(STATUS "Starting pin test")
MESSAGE(STATUS "======================")

SET(LN_ENABLE_USBD FALSE CACHE INTERNAL "")
SET(LN_ENABLE_I2C  FALSE CACHE INTERNAL "")
SET(LN_ENABLE_SPI  FALSE CACHE INTERNAL "")
SET(LN_ENABLE_RUST  TRUE CACHE INTERNAL "")
SET(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
SET(ESPRIT_ROOT  ${CMAKE_SOURCE_DIR}/esprit/)
include(common_CMakeLists )

PROJECT(pin_rust NONE) # the NONE is very important !

enable_language(C CXX ASM) # this is important too!

add_subdirectory(${ESPRIT_ROOT})
add_subdirectory(rust)
include(${ESPRIT_ROOT}/setup.cmake)

USE_LIBRARY(ln-rtt)

GENERATE_GD32_FIRMWARE(pin src/demo.cpp)
target_link_libraries(pin PRIVATE rust_demo)

