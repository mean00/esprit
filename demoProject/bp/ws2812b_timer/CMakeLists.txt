cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(Longan_WS12812B C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

use_library(WS2812B)
generate_gd32_firmware(ws2812 hello.cpp)
