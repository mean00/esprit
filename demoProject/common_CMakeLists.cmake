cmake_minimum_required(VERSION 3.20)

set(ESPRIT_ROOT ${CMAKE_SOURCE_DIR}/esprit/)
find_program(CCACHE_FOUND ccache)
if(CCACHE_FOUND)
  set(CMAKE_C_COMPILER_LAUNCHER ${CCACHE_FOUND})
  set(CMAKE_CXX_COMPILER_LAUNCHER ${CCACHE_FOUND})
endif()
include(common_platformConfig)
include(common_mcuSelect)
set(CMAKE_TOOLCHAIN_FILE ${ESPRIT_ROOT}/esprit.cmake)
set(CMAKE_MODULE_PATH
    ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/cmake ${ESPRIT_ROOT}/cmake
    CACHE INTERNAL "")
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
