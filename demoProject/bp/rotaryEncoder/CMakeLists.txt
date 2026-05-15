#
cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(RotaryEncoder C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

set(Rotary ${ESPRIT_ROOT}/libraries/RotaryEncoder)
include_directories(${Rotary})
add_subdirectory(${Rotary})

generate_gd32_firmware(hello hello.cpp)
target_link_libraries(hello PRIVATE RotaryEncoder)
