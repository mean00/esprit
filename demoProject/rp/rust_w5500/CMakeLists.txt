cmake_minimum_required(VERSION 3.20)

set(LN_ENABLE_RUST
    TRUE
    CACHE INTERNAL "")

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(pico_w5500_echo_server NONE)

enable_language(C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})
add_subdirectory(rust)

include(${ESPRIT_ROOT}/setup.cmake)
use_library(w5500)
use_library(ln-rtt)

generate_gd32_firmware(echo_server app_echo_sr.cpp)
target_link_libraries(echo_server PRIVATE rust_demo)
