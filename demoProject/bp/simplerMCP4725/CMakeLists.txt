cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(MCP4725 C CXX ASM)

set(LN_ENABLE_I2C
    ON
    CACHE INTERNAL "")

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

use_library(simplerMCP4725)
generate_gd32_firmware(mcp4725 hello.cpp)
