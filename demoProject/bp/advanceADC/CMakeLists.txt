cmake_minimum_required(VERSION 3.20)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/../..)
include(common_CMakeLists)

project(lnAdvanceADC C CXX ASM)

include_directories(${CMAKE_SOURCE_DIR})
add_subdirectory(${ESPRIT_ROOT})

include(${ESPRIT_ROOT}/setup.cmake)

set(NVM ${ESPRIT_ROOT}/libraries/NeVerMind)
include_directories(${NVM})
include_directories(${NVM}/gd32)

set(NVM_SRC ${NVM}/nvmCore.cpp ${NVM}/gd32/nvm_gd32.cpp)

generate_gd32_firmware(hello hello.cpp ${NVM_SRC})
