cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(pico_w5500_echo_server C CXX ASM)
include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)
use_library(w5500)
use_library(ln-rtt)
generate_gd32_firmware(echo_server app_echo_sr.cpp)
target_sources(echo_server PRIVATE ${ESPRIT_ROOT}/src/lnSocketRunner.cpp)
