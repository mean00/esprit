cmake_minimum_required(VERSION 3.20)
set(LN_ENABLE_UART
    TRUE
    CACHE INTERNAL "")

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(lnPwm C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

generate_gd32_firmware(hello hello.cpp)
