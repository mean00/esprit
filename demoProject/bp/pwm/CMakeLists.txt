cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(lnPwm C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

generate_gd32_firmware(simple_pwm simple_pwm.cpp)
generate_gd32_firmware(shifting_fq shifting_fq.cpp)
generate_gd32_firmware(single_shot single_shot.cpp)
