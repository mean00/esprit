cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(Hello_gd32_freetos C CXX ASM)
include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

rp_pio_generate(${CMAKE_SOURCE_DIR}/ws.pio ${CMAKE_BINARY_DIR}/ws.h)
generate_gd32_firmware(pio_ws2812 main_pio.cpp pio_ws2812.cpp ${CMAKE_BINARY_DIR}/ws.h)
